from __future__ import annotations

import math
import random
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import torch
from torch.nn import functional as F

from hearth_env import HearthEnv

from .catalog import CardCatalog
from .checkpoint import load_checkpoint, save_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .decks import DeckPool
from .health import EpisodeHealth, health_gate
from .league import CheckpointLeague
from .model import HearthQNetwork
from .policies import HeuristicPolicy, ModelPolicy
from .rollout import ParallelCollector, RolloutJob, play_episode
from .tensorize import Tensorizer, collate, move_batch
from .trajectory import write_episodes


@dataclass
class PPOExperience:
    decision: dict[str, Any]
    self_deck: list[str]
    action_index: int
    old_log_probability: float
    old_value: float
    advantage: float
    return_value: float


@dataclass
class PPOMetrics:
    loss: float
    policy_loss: float
    value_loss: float
    entropy: float
    approximate_kl: float
    clip_fraction: float
    reference_kl: float
    updates: int


def state_potential(decision: dict[str, Any]) -> float:
    """A small actor-relative material/health potential in approximately [-1, 1]."""

    observation = decision["observation"]
    score = 0.0
    for entity in observation.get("entities", []):
        controller = entity.get("controller")
        sign = (
            1.0
            if controller == "self_player"
            else -1.0
            if controller == "opponent"
            else 0.0
        )
        if not sign:
            continue
        area = entity.get("area")
        attack = max(float(entity.get("attack", 0)), 0.0)
        remaining = max(
            float(entity.get("max_health", 0)) - float(entity.get("damage", 0)),
            0.0,
        )
        if area == "hero":
            effective_health = remaining + max(float(entity.get("armor", 0)), 0.0)
            score += sign * effective_health * 0.35
        elif area == "board":
            score += sign * (attack + remaining) * 0.65
        elif area == "weapon":
            score += sign * attack * max(remaining, 1.0) * 0.3
    own = observation.get("self_player", {})
    opponent = observation.get("opponent", {})
    score += 0.15 * (
        float(own.get("hand_size", 0)) - float(opponent.get("hand_size", 0))
    )
    return math.tanh(score / 20.0)


def _policy_evaluations(
    model: HearthQNetwork,
    tensorizer: Tensorizer,
    items: list[tuple[dict[str, Any], list[str], int]],
    *,
    device: str,
    batch_size: int,
) -> list[tuple[float, float]]:
    output: list[tuple[float, float]] = []
    model.eval()
    with torch.no_grad():
        for start in range(0, len(items), batch_size):
            chunk = items[start : start + batch_size]
            batch = move_batch(
                collate(
                    [tensorizer.encode(decision, deck) for decision, deck, _ in chunk]
                ),
                device,
            )
            logits, values = model.policy_value(batch)
            log_probabilities = torch.log_softmax(logits, dim=1)
            actions = torch.tensor(
                [action for _, _, action in chunk], dtype=torch.long, device=device
            )
            selected = log_probabilities.gather(1, actions[:, None]).squeeze(1)
            output.extend(
                zip(
                    selected.detach().cpu().tolist(),
                    values.detach().cpu().tolist(),
                    strict=True,
                )
            )
    return output


def build_ppo_experiences(
    episodes: list[tuple[dict[str, Any], set[int]]],
    model: HearthQNetwork,
    tensorizer: Tensorizer,
    config: TrainConfig,
    *,
    device: str,
) -> list[PPOExperience]:
    """Turn on-policy episodes into actor-relative GAE training examples."""

    flat: list[tuple[dict[str, Any], list[str], int]] = []
    keys: list[tuple[int, int]] = []
    for episode_index, (episode, controlled_seats) in enumerate(episodes):
        if episode.get("truncated") or episode.get("error"):
            continue
        decks = episode["match_config"]["decks"]
        for step_index, step in enumerate(episode.get("steps", [])):
            seat = int(step["decision"]["actor_seat"])
            if seat not in controlled_seats:
                continue
            action = int(step["action_index"])
            if action < 0 or action >= len(step["decision"].get("actions", [])):
                raise ValueError(f"invalid PPO action index {action}")
            flat.append((step["decision"], list(decks[seat]), action))
            keys.append((episode_index, step_index))
    if not flat:
        return []

    evaluations = _policy_evaluations(
        model,
        tensorizer,
        flat,
        device=device,
        batch_size=config.batch_size,
    )
    by_step = dict(zip(keys, evaluations, strict=True))
    experiences: list[PPOExperience] = []
    for episode_index, (episode, controlled_seats) in enumerate(episodes):
        if episode.get("truncated") or episode.get("error"):
            continue
        decks = episode["match_config"]["decks"]
        rewards = episode.get("rewards", [0.0, 0.0])
        for seat in sorted(controlled_seats):
            sequence = [
                (step_index, step)
                for step_index, step in enumerate(episode.get("steps", []))
                if int(step["decision"]["actor_seat"]) == seat
            ]
            if not sequence:
                continue
            advantages = [0.0] * len(sequence)
            returns = [0.0] * len(sequence)
            next_advantage = 0.0
            next_value = 0.0
            next_potential = 0.0
            for position in range(len(sequence) - 1, -1, -1):
                step_index, step = sequence[position]
                _, value = by_step[(episode_index, step_index)]
                potential = state_potential(step["decision"])
                shaped_reward = config.shaping_coefficient * (
                    config.gamma * next_potential - potential
                )
                if position == len(sequence) - 1:
                    shaped_reward += float(rewards[seat])
                delta = shaped_reward + config.gamma * next_value - value
                advantage = delta + config.gamma * config.gae_lambda * next_advantage
                advantages[position] = advantage
                returns[position] = advantage + value
                next_advantage = advantage
                next_value = value
                next_potential = potential
            for position, (step_index, step) in enumerate(sequence):
                old_log_probability, old_value = by_step[(episode_index, step_index)]
                experiences.append(
                    PPOExperience(
                        decision=step["decision"],
                        self_deck=list(decks[seat]),
                        action_index=int(step["action_index"]),
                        old_log_probability=old_log_probability,
                        old_value=old_value,
                        advantage=advantages[position],
                        return_value=returns[position],
                    )
                )
    return experiences


def train_ppo_epochs(
    model: HearthQNetwork,
    optimizer: torch.optim.Optimizer,
    tensorizer: Tensorizer,
    experiences: list[PPOExperience],
    config: TrainConfig,
    *,
    device: str,
    rng: random.Random,
    target_kl: float = 0.03,
    reference_model: HearthQNetwork | None = None,
) -> PPOMetrics:
    if not experiences:
        raise ValueError("PPO update needs at least one experience")
    advantages = torch.tensor([item.advantage for item in experiences])
    advantage_mean = float(advantages.mean().item())
    advantage_std = float(advantages.std(unbiased=False).item())
    normalized = [
        (item.advantage - advantage_mean) / max(advantage_std, 1e-8)
        for item in experiences
    ]
    totals = {
        "loss": 0.0,
        "policy": 0.0,
        "value": 0.0,
        "entropy": 0.0,
        "kl": 0.0,
        "clip": 0.0,
        "reference_kl": 0.0,
    }
    updates = 0
    indices = list(range(len(experiences)))
    stop = False
    # Dropout changes likelihood ratios even without a parameter update. Eval
    # mode keeps the PPO behavior/current policy comparison well-defined.
    model.eval()
    for _ in range(config.ppo_epochs):
        rng.shuffle(indices)
        for start in range(0, len(indices), config.batch_size):
            selected_indices = indices[start : start + config.batch_size]
            selected = [experiences[index] for index in selected_indices]
            batch = move_batch(
                collate(
                    [
                        tensorizer.encode(item.decision, item.self_deck)
                        for item in selected
                    ]
                ),
                device,
            )
            actions = torch.tensor(
                [item.action_index for item in selected], device=device
            )
            old_log = torch.tensor(
                [item.old_log_probability for item in selected], device=device
            )
            old_value = torch.tensor(
                [item.old_value for item in selected], device=device
            )
            returns = torch.tensor(
                [item.return_value for item in selected], device=device
            )
            batch_advantages = torch.tensor(
                [normalized[index] for index in selected_indices], device=device
            )

            logits, values = model.policy_value(batch)
            log_all = torch.log_softmax(logits, dim=1)
            log_probability = log_all.gather(1, actions[:, None]).squeeze(1)
            probability = torch.softmax(logits, dim=1)
            entropy = -(probability * log_all).sum(dim=1).mean()
            ratio = torch.exp(log_probability - old_log)
            unclipped = ratio * batch_advantages
            clipped = ratio.clamp(1.0 - config.ppo_clip, 1.0 + config.ppo_clip) * batch_advantages
            policy_loss = -torch.minimum(unclipped, clipped).mean()

            value_delta = (values - old_value).clamp(
                -config.value_clip, config.value_clip
            )
            clipped_values = old_value + value_delta
            value_loss = 0.5 * torch.maximum(
                F.mse_loss(values, returns, reduction="none"),
                F.mse_loss(clipped_values, returns, reduction="none"),
            ).mean()
            reference_kl = torch.zeros((), device=device)
            if reference_model is not None and config.reference_kl_coefficient > 0:
                with torch.no_grad():
                    reference_logits = reference_model(batch)
                    reference_log = torch.log_softmax(reference_logits, dim=1)
                    reference_probability = torch.softmax(reference_logits, dim=1)
                reference_kl = (
                    reference_probability * (reference_log - log_all)
                ).sum(dim=1).mean().clamp_min(0.0)
            loss = (
                policy_loss
                + config.value_coefficient * value_loss
                - config.entropy_coefficient * entropy
                + config.reference_kl_coefficient * reference_kl
            )
            optimizer.zero_grad(set_to_none=True)
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), config.grad_clip)
            optimizer.step()

            with torch.no_grad():
                log_ratio = log_probability - old_log
                approximate_kl = ((torch.exp(log_ratio) - 1.0) - log_ratio).mean()
                clip_fraction = (
                    (torch.abs(ratio - 1.0) > config.ppo_clip).float().mean()
                )
            totals["loss"] += float(loss.item())
            totals["policy"] += float(policy_loss.item())
            totals["value"] += float(value_loss.item())
            totals["entropy"] += float(entropy.item())
            totals["kl"] += float(approximate_kl.item())
            totals["clip"] += float(clip_fraction.item())
            totals["reference_kl"] += float(reference_kl.item())
            updates += 1
            if float(approximate_kl.item()) > target_kl:
                stop = True
                break
        if stop:
            break
    denominator = max(updates, 1)
    return PPOMetrics(
        loss=totals["loss"] / denominator,
        policy_loss=totals["policy"] / denominator,
        value_loss=totals["value"] / denominator,
        entropy=totals["entropy"] / denominator,
        approximate_kl=totals["kl"] / denominator,
        clip_fraction=totals["clip"] / denominator,
        reference_kl=totals["reference_kl"] / denominator,
        updates=updates,
    )


def train_ppo(
    data_path: str | Path,
    catalog: CardCatalog,
    deck_pool: DeckPool,
    run_dir: str | Path,
    train_config: TrainConfig,
    *,
    initial_checkpoint: str | Path | None = None,
    resume_checkpoint: str | Path | None = None,
    model_config: ModelConfig | None = None,
    specialist_probability: float = 0.1,
    reference_checkpoint: str | Path | None = None,
) -> HearthQNetwork:
    if initial_checkpoint and resume_checkpoint:
        raise ValueError("--init and --resume are mutually exclusive")
    run_dir = Path(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    device = resolve_device(train_config.device)
    rng = random.Random(train_config.seed)
    torch.manual_seed(train_config.seed)
    checkpoint = resume_checkpoint or initial_checkpoint
    payload: dict[str, Any] = {}
    if checkpoint:
        model, payload = load_checkpoint(checkpoint, catalog, device=device)
    else:
        model = HearthQNetwork(
            catalog, model_config or ModelConfig(card_hash_dim=catalog.hash_dim)
        ).to(device)
    start_iteration = int(payload.get("ppo_iteration", 0)) if resume_checkpoint else 0
    learner_step = int(payload.get("step", 0)) if resume_checkpoint else 0
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=train_config.ppo_learning_rate,
        weight_decay=train_config.weight_decay,
    )
    if resume_checkpoint:
        if payload.get("phase") != "ppo":
            raise ValueError("--resume requires a PPO checkpoint")
        if not payload.get("optimizer"):
            raise ValueError("resume checkpoint does not contain optimizer state")
        optimizer.load_state_dict(payload["optimizer"])
    tensorizer = Tensorizer(catalog, model.config)
    reference_path = reference_checkpoint or payload.get("reference_checkpoint")
    if reference_path is None and initial_checkpoint is not None:
        reference_path = initial_checkpoint
    reference_model = None
    if reference_path is not None and train_config.reference_kl_coefficient > 0:
        reference_model, _ = load_checkpoint(reference_path, catalog, device=device)
        reference_model.eval()
        for parameter in reference_model.parameters():
            parameter.requires_grad_(False)
    league = CheckpointLeague(run_dir / "league", seed=train_config.seed)
    actor_path = run_dir / "actor.pt"
    latest_path = run_dir / "latest.pt"
    base_config = deck_pool.sample_match()
    env: HearthEnv | None = None
    collector: ParallelCollector | None = None
    if train_config.workers > 0:
        collector = ParallelCollector(
            data_path,
            base_config,
            workers=train_config.workers,
            max_steps=train_config.max_steps,
            history_limit=train_config.history_limit,
            card_hash_dim=catalog.hash_dim,
            failure_dir=run_dir / "failures",
            max_failures=0,
        )
    else:
        env = HearthEnv(
            data_path,
            base_config,
            max_steps=train_config.max_steps,
            history_limit=train_config.history_limit,
        )
    final_metrics: dict[str, float] = {}
    try:
        for iteration in range(start_iteration, train_config.ppo_iterations):
            save_checkpoint(actor_path, model, catalog, step=learner_step, phase="actor")
            jobs: list[RolloutJob] = []
            for episode_index in range(train_config.episodes_per_iteration):
                seed = (
                    train_config.seed
                    + iteration * train_config.episodes_per_iteration
                    + episode_index
                )
                current_seat = rng.randrange(2)
                current: dict[str, Any] = {
                    "kind": "model",
                    "checkpoint": str(actor_path),
                    "sample": True,
                    "temperature": 1.0,
                    "train": True,
                }
                if rng.random() < specialist_probability:
                    opponent: dict[str, Any] = {
                        "kind": "heuristic",
                        "noise": 0.15,
                        "train": False,
                    }
                else:
                    opponent = {
                        "kind": "model",
                        "checkpoint": str(league.sample(actor_path)),
                        "train": False,
                    }
                policies = (
                    (current, opponent) if current_seat == 0 else (opponent, current)
                )
                jobs.append(RolloutJob(deck_pool.sample_match(), seed, policies))

            if collector is not None:
                raw_episodes = collector.collect(jobs)
            else:
                assert env is not None
                raw_episodes = []
                model_cache: dict[Path, ModelPolicy] = {}
                for job in jobs:
                    policies = []
                    for seat, spec in enumerate(job.policies):
                        if spec["kind"] == "heuristic":
                            policies.append(HeuristicPolicy(job.seed ^ seat, 0.15))
                            continue
                        path = Path(spec["checkpoint"])
                        if path == actor_path:
                            actor_model = model
                            actor_tensorizer = tensorizer
                        else:
                            if path not in model_cache:
                                old_model, _ = load_checkpoint(path, catalog, device=device)
                                model_cache[path] = ModelPolicy(
                                    old_model,
                                    Tensorizer(catalog, old_model.config),
                                    device=device,
                                    seed=job.seed,
                                )
                            policies.append(model_cache[path])
                            continue
                        policies.append(
                            ModelPolicy(
                                actor_model,
                                actor_tensorizer,
                                device=device,
                                seed=job.seed ^ seat,
                                sample=bool(spec.get("sample", False)),
                            )
                        )
                    raw_episodes.append(
                        play_episode(env, policies, job.match_config, job.seed)
                    )

            annotated: list[tuple[dict[str, Any], set[int]]] = []
            health = EpisodeHealth()
            for job, episode in zip(jobs, raw_episodes, strict=True):
                controlled = {
                    seat
                    for seat, spec in enumerate(job.policies)
                    if bool(spec.get("train", False))
                }
                health.add(episode, controlled_seats=controlled)
                annotated.append((episode, controlled))
            write_episodes(
                run_dir / "rollouts" / f"iteration-{iteration:06d}.jsonl.gz",
                raw_episodes,
                append=False,
            )
            experiences = build_ppo_experiences(
                annotated, model, tensorizer, train_config, device=device
            )
            metrics = train_ppo_epochs(
                model,
                optimizer,
                tensorizer,
                experiences,
                train_config,
                device=device,
                rng=rng,
                reference_model=reference_model,
            )
            learner_step += metrics.updates
            summary = health.summary()
            final_metrics = {
                "loss": metrics.loss,
                "policy_loss": metrics.policy_loss,
                "value_loss": metrics.value_loss,
                "entropy": metrics.entropy,
                "approximate_kl": metrics.approximate_kl,
                "clip_fraction": metrics.clip_fraction,
                "reference_kl": metrics.reference_kl,
                "experiences": float(len(experiences)),
                "avoidable_end_turn_rate": float(summary["avoidable_end_turn_rate"]),
                "truncation_rate": float(summary["truncation_rate"]),
                "nonlethal_face_with_killable_minion_rate": float(
                    summary["nonlethal_face_with_killable_minion_rate"]
                ),
            }
            print(
                f"ppo iteration={iteration + 1} episodes={len(raw_episodes)} "
                f"experiences={len(experiences)} updates={metrics.updates} "
                f"loss={metrics.loss:.5f} kl={metrics.approximate_kl:.5f} "
                f"ref_kl={metrics.reference_kl:.5f} entropy={metrics.entropy:.3f} "
                f"avoidable_end={summary['avoidable_end_turn_rate']:.3%} "
                f"trade_skip={summary['nonlethal_face_with_killable_minion_rate']:.3%}"
            )
            if (iteration + 1) % train_config.checkpoint_every == 0 or iteration == 0:
                save_checkpoint(
                    latest_path,
                    model,
                    catalog,
                    optimizer=optimizer,
                    step=learner_step,
                    phase="ppo",
                    metrics=final_metrics,
                    extra_state={
                        "ppo_iteration": iteration + 1,
                        "reference_checkpoint": str(reference_path)
                        if reference_path is not None
                        else None,
                    },
                )
            if (iteration + 1) % train_config.league_snapshot_every == 0:
                failures = health_gate(health)
                if failures:
                    print("league promotion rejected: " + "; ".join(failures))
                else:
                    save_checkpoint(
                        league.directory / f"snapshot-{iteration + 1:06d}.pt",
                        model,
                        catalog,
                        step=learner_step,
                        phase="league",
                        metrics=final_metrics,
                        extra_state={
                            "ppo_iteration": iteration + 1,
                            "reference_checkpoint": str(reference_path)
                            if reference_path is not None
                            else None,
                        },
                    )
    finally:
        if collector is not None:
            collector.close()
    save_checkpoint(
        latest_path,
        model,
        catalog,
        optimizer=optimizer,
        step=learner_step,
        phase="ppo",
        metrics=final_metrics,
        extra_state={
            "ppo_iteration": train_config.ppo_iterations,
            "reference_checkpoint": str(reference_path)
            if reference_path is not None
            else None,
        },
    )
    return model
