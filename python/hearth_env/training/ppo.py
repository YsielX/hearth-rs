from __future__ import annotations

import math
import hashlib
import random
from dataclasses import asdict, dataclass, field
from pathlib import Path
import json
import time
from typing import Any

import torch
from torch.nn import functional as F

from hearth_env import HearthEnv

from .catalog import CardCatalog
from .checkpoint import load_checkpoint, save_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .distribution import policy_logits
from .decks import DeckPool
from .health import EpisodeHealth, health_gate
from .league import CheckpointLeague
from .learn import _sample_batch, imitation_loss
from .model import HearthQNetwork
from .policies import HeuristicPolicy, ModelPolicy
from .rollout import ParallelCollector, RolloutJob, play_episode
from .tensorize import Tensorizer, collate, move_batch, tensor_schema_version
from .trajectory import (
    write_episodes,
    contains_excluded_cards,
    stream_samples,
    TrainingSample,
)
from hearth_env._native import HearthEnv as NativeEnv


@dataclass
class PPOExperience:
    decision: dict[str, Any]
    self_deck: list[str]
    action_index: int
    old_log_probability: float
    old_value: float
    advantage: float
    return_value: float
    # In-memory only, tied to the exact encoder that produced the tensors.
    # Checkpoints and replay files continue to contain raw public decisions.
    encoding: tuple[Tensorizer, dict[str, torch.Tensor]] | None = field(
        default=None, repr=False, compare=False
    )


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
    auxiliary_bc_loss: float = 0.0
    collected_experiences: int = 0
    update_experiences: int = 0


def load_auxiliary_samples(shards, *, limit=20000, **stream_options):
    """Keep the existing shuffled prefix, but validate every input episode.

    Stopping the iterator at the memory limit would hide incompatible or
    evaluation-only episodes in the unread tail. Consuming the tail does not
    change the retained prefix or any learner/matchmaking random state.
    """
    if limit < 1:
        raise ValueError("auxiliary sample limit must be positive")
    retained = []
    for sample in stream_samples(shards, behavior_clone=True, **stream_options):
        if len(retained) < limit:
            retained.append(sample)
    return retained


def _policy_evaluations(
    model: HearthQNetwork,
    tensorizer: Tensorizer,
    items: list[tuple[dict[str, Any], list[str], int]],
    *,
    device: str,
    batch_size: int,
    encoded: list[dict[str, torch.Tensor]] | None = None,
    temperature: float = 1.0,
) -> list[tuple[float, float]]:
    output: list[tuple[float, float]] = []
    model.eval()
    with torch.no_grad():
        for start in range(0, len(items), batch_size):
            chunk = items[start : start + batch_size]
            batch = move_batch(
                collate(
                    encoded[start : start + len(chunk)]
                    if encoded is not None
                    else [
                        tensorizer.encode(decision, deck) for decision, deck, _ in chunk
                    ]
                ),
                device,
            )
            logits, values = model.policy_value(batch)
            logits = policy_logits(logits, batch, temperature=temperature)
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
    excluded = frozenset(config.excluded_cards)
    keys: list[tuple[int, int]] = []
    for episode_index, (episode, controlled_seats) in enumerate(episodes):
        if (
            episode.get("truncated")
            or episode.get("error")
            or contains_excluded_cards(episode, excluded)
        ):
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

    encoded = [tensorizer.encode(decision, deck) for decision, deck, _ in flat]
    evaluations = _policy_evaluations(
        model,
        tensorizer,
        flat,
        device=device,
        batch_size=config.batch_size,
        encoded=encoded,
        temperature=config.ppo_temperature,
    )
    by_step = dict(zip(keys, evaluations, strict=True))
    encoded_by_step = dict(zip(keys, encoded, strict=True))
    for key in keys:
        episode_index, step_index = key
        behavior = episodes[episode_index][0]["steps"][step_index].get("behavior")
        if behavior is None:
            raise ValueError(
                "PPO training decisions require recorded behavior probabilities and values"
            )
        if (
            behavior.get("temperature", 1.0) != config.ppo_temperature
            or behavior.get("action_support") != "all_legal"
        ):
            raise ValueError("PPO behavior support/temperature differs from learner")
        expected_log, expected_value = by_step[key]
        observed_log, observed_value = (
            float(behavior["log_probability"]),
            float(behavior["value"]),
        )
        if not math.isfinite(observed_log) or not math.isfinite(observed_value):
            raise ValueError("nonfinite PPO behavior probability or value")
        if (
            abs(expected_log - observed_log) > 2e-4
            or abs(expected_value - observed_value) > 2e-4
        ):
            raise ValueError(
                "stale/off-policy rollout: behavior differs from frozen actor"
            )
        by_step[key] = observed_log, observed_value
    experiences: list[PPOExperience] = []
    for episode_index, (episode, controlled_seats) in enumerate(episodes):
        if (
            episode.get("truncated")
            or episode.get("error")
            or contains_excluded_cards(episode, excluded)
        ):
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
            for position in range(len(sequence) - 1, -1, -1):
                step_index, step = sequence[position]
                _, value = by_step[(episode_index, step_index)]
                reward = 0.0
                if position == len(sequence) - 1:
                    reward += float(rewards[seat])
                delta = reward + config.gamma * next_value - value
                advantage = delta + config.gamma * config.gae_lambda * next_advantage
                advantages[position] = advantage
                returns[position] = advantage + value
                next_advantage = advantage
                next_value = value
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
                        encoding=(
                            tensorizer,
                            encoded_by_step[(episode_index, step_index)],
                        ),
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
    auxiliary_samples: list[TrainingSample] | None = None,
) -> PPOMetrics:
    if not experiences:
        raise ValueError("PPO update needs at least one experience")
    collected_count = len(experiences)
    encoded = [
        item.encoding[1]
        if item.encoding is not None and item.encoding[0] is tensorizer
        else tensorizer.encode(item.decision, item.self_deck)
        for item in experiences
    ]
    advantages = torch.tensor(
        [item.advantage for item in experiences], dtype=torch.float32
    )
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
        "auxiliary_bc": 0.0,
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
                collate([encoded[index] for index in selected_indices]),
                device,
            )
            actions = torch.tensor(
                [item.action_index for item in selected], device=device
            )
            old_log = torch.tensor(
                [item.old_log_probability for item in selected],
                device=device,
                dtype=torch.float32,
            )
            old_value = torch.tensor(
                [item.old_value for item in selected],
                device=device,
                dtype=torch.float32,
            )
            returns = torch.tensor(
                [item.return_value for item in selected],
                device=device,
                dtype=torch.float32,
            )
            batch_advantages = torch.tensor(
                [normalized[index] for index in selected_indices], device=device
            )

            logits, values = model.policy_value(batch)
            logits = policy_logits(logits, batch, temperature=config.ppo_temperature)
            log_all = torch.log_softmax(logits, dim=1)
            log_probability = log_all.gather(1, actions[:, None]).squeeze(1)
            probability = torch.softmax(logits, dim=1)
            entropy = -(probability * log_all).sum(dim=1).mean()
            ratio = torch.exp(log_probability - old_log)
            unclipped = ratio * batch_advantages
            clipped = (
                ratio.clamp(1.0 - config.ppo_clip, 1.0 + config.ppo_clip)
                * batch_advantages
            )
            policy_loss = -torch.minimum(unclipped, clipped).mean()

            value_delta = (values - old_value).clamp(
                -config.value_clip, config.value_clip
            )
            clipped_values = old_value + value_delta
            value_loss = (
                0.5
                * torch.maximum(
                    F.mse_loss(values, returns, reduction="none"),
                    F.mse_loss(clipped_values, returns, reduction="none"),
                ).mean()
            )
            reference_kl = torch.zeros((), device=device)
            if reference_model is not None and config.reference_kl_coefficient > 0:
                with torch.no_grad():
                    reference_logits = reference_model(batch)
                    reference_logits = policy_logits(
                        reference_logits, batch, temperature=config.ppo_temperature
                    )
                    reference_log = torch.log_softmax(reference_logits, dim=1)
                    reference_probability = torch.softmax(reference_logits, dim=1)
                reference_kl = (
                    (reference_probability * (reference_log - log_all))
                    .sum(dim=1)
                    .mean()
                    .clamp_min(0.0)
                )
            loss = (
                policy_loss
                + config.value_coefficient * value_loss
                - config.entropy_coefficient * entropy
                + config.reference_kl_coefficient * reference_kl
            )
            auxiliary_bc = torch.zeros((), device=device)
            if auxiliary_samples and config.ppo_bc_coefficient > 0:
                supervised = rng.choices(auxiliary_samples, k=len(selected))
                aux_batch, _, _, weights = _sample_batch(supervised, tensorizer, device)
                aux_logits = policy_logits(model(aux_batch), aux_batch)
                losses, _ = imitation_loss(aux_logits, supervised)
                auxiliary_bc = (losses * weights).sum() / weights.sum().clamp_min(1)
                loss = loss + config.ppo_bc_coefficient * auxiliary_bc
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
            totals["auxiliary_bc"] += float(auxiliary_bc.item())
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
        auxiliary_bc_loss=totals["auxiliary_bc"] / denominator,
        collected_experiences=collected_count,
        update_experiences=len(experiences),
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
    bc_shards: list[str | Path] | None = None,
) -> HearthQNetwork:
    if initial_checkpoint and resume_checkpoint:
        raise ValueError("--init and --resume are mutually exclusive")
    if bool(bc_shards) != (train_config.ppo_bc_coefficient > 0):
        raise ValueError(
            "PPO auxiliary BC requires both data and a positive coefficient"
        )
    auxiliary_samples = (
        load_auxiliary_samples(
            bc_shards,
            seed=train_config.seed,
            expected_pack_hash=catalog.pack_hash,
            expected_engine_build=NativeEnv.engine_build(),
            excluded_cards=train_config.excluded_cards,
        )
        if bc_shards
        else []
    )
    if bc_shards and not auxiliary_samples:
        raise ValueError("PPO auxiliary BC input contains no samples")
    run_dir = Path(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    if not resume_checkpoint and (run_dir / "latest.pt").exists():
        raise ValueError(
            "run directory already contains a model; use --resume or a new directory"
        )
    device = resolve_device(train_config.device)
    rng = random.Random(train_config.seed)
    torch.manual_seed(train_config.seed)
    checkpoint = resume_checkpoint or initial_checkpoint
    payload: dict[str, Any] = {}
    if checkpoint:
        model, payload = load_checkpoint(
            checkpoint, catalog, device=device, strict_pack=bool(resume_checkpoint)
        )
    else:
        model = HearthQNetwork(
            catalog, model_config or ModelConfig(card_hash_dim=catalog.hash_dim)
        ).to(device)
    start_iteration = int(payload.get("ppo_iteration", 0)) if resume_checkpoint else 0
    if train_config.ppo_iterations <= start_iteration:
        raise ValueError("target PPO iterations must exceed the checkpoint iteration")
    learner_step = int(payload.get("step", 0)) if resume_checkpoint else 0
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=train_config.ppo_learning_rate,
        weight_decay=train_config.weight_decay,
    )
    if resume_checkpoint:
        if payload["catalog"]["card_ids"] != list(catalog.card_ids):
            raise ValueError(
                "resume requires identical card embedding layout; use --init"
            )
        expected_schema = tensor_schema_version(model.config)
        if payload.get("tensor_schema_version") != expected_schema:
            raise ValueError("resume cannot migrate tensor schemas; use --init")
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
    if resume_checkpoint and "league_state" not in payload:
        raise ValueError(
            "checkpoint lacks frozen league state; use --init in a new run"
        )
    league.freeze(payload.get("league_state") if resume_checkpoint else None)
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
    run_manifest = {
        "format_version": 4,
        "auxiliary_data": [
            {
                "path": str(Path(path).resolve()),
                "sha256": hashlib.sha256(Path(path).read_bytes()).hexdigest(),
            }
            for path in bc_shards or []
        ],
        "reference_sha256": hashlib.sha256(
            Path(reference_path).read_bytes()
        ).hexdigest()
        if reference_model is not None
        else None,
        "train_config": train_config.to_dict(),
        "model_config": model.config.to_dict(),
        "catalog": catalog.manifest(),
        "decks": [
            dict(
                asdict(d),
                cards=list(d.cards),
                protected_cards=list(d.protected_cards),
            )
            for d in deck_pool.curated
        ],
        "deck_sampling": {
            "curated_probability": deck_pool.curated_probability,
            "perturb_probability": deck_pool.perturb_probability,
            "card_pool": deck_pool.card_pool,
            "excluded_cards": sorted(deck_pool.excluded_cards),
        },
        "specialist_probability": specialist_probability,
    }
    if resume_checkpoint:
        saved_manifest = payload.get("run_manifest")
        if saved_manifest is None or "random_state" not in payload:
            raise ValueError(
                "checkpoint lacks exact-resume state; start a new run with --init"
            )
        if saved_manifest.get("format_version", 0) < 4:
            # Earlier manifests omitted sampling-affecting Deck metadata.
            # Its historical values cannot be recovered from current defaults.
            raise ValueError(
                "resume checkpoint lacks complete deck metadata; use --init in a new run"
            )
        manifest_defaults = {"auxiliary_data": []}
        for key in (
            "model_config",
            "catalog",
            "decks",
            "deck_sampling",
            "specialist_probability",
            "reference_sha256",
            "auxiliary_data",
        ):
            if saved_manifest.get(key, manifest_defaults.get(key)) != run_manifest[key]:
                raise ValueError(f"resume configuration differs: {key}")
        mutable = {"ppo_iterations", "workers", "device", "checkpoint_every"}
        unrelated = {
            "bc_learning_rate",
            "dmc_learning_rate",
            "bc_epochs",
            "dmc_iterations",
            "updates_per_iteration",
            "replay_capacity",
            "replay_warmup",
            "epsilon_start",
            "epsilon_end",
            "epsilon_decay_iterations",
            "bc_regularization_start",
            "bc_regularization_end",
        }
        # Old checkpoints used temperature 1. Missing metadata cannot authorize
        # resuming with a different behavior distribution.
        saved_train_config = {"ppo_temperature": 1.0, **saved_manifest["train_config"]}
        for key, value in saved_train_config.items():
            if (
                key not in mutable | unrelated
                and train_config.to_dict().get(key) != value
            ):
                raise ValueError(f"resume training setting differs: {key}")
        state = payload["random_state"]
        rng.setstate(state["matchmaking"])
        deck_pool.rng.setstate(state["decks"])
        league.rng.setstate(state["league"])
        torch.set_rng_state(state["torch"].cpu())
        if torch.cuda.is_available() and state.get("cuda") is not None:
            torch.cuda.set_rng_state_all(state["cuda"])
    (run_dir / "run_manifest.json").write_text(
        json.dumps(run_manifest, indent=2), encoding="utf-8"
    )

    def training_state(iteration: int) -> dict[str, Any]:
        return {
            "ppo_iteration": iteration,
            "reference_checkpoint": str(reference_path) if reference_path else None,
            "run_manifest": run_manifest,
            "league_state": league.state(),
            "random_state": {
                "matchmaking": rng.getstate(),
                "decks": deck_pool.rng.getstate(),
                "league": league.rng.getstate(),
                "torch": torch.get_rng_state(),
                "cuda": torch.cuda.get_rng_state_all()
                if torch.cuda.is_available()
                else None,
            },
        }

    try:
        for iteration in range(start_iteration, train_config.ppo_iterations):
            started = time.monotonic()
            save_checkpoint(
                actor_path, model, catalog, step=learner_step, phase="actor"
            )
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
                    "temperature": train_config.ppo_temperature,
                    "train": True,
                }
                if rng.random() < specialist_probability:
                    opponent: dict[str, Any] = {
                        "kind": "heuristic",
                        "train": False,
                    }
                else:
                    opponent = {
                        "kind": "model",
                        "checkpoint": str(league.sample(actor_path)),
                        "train": False,
                        "sample": rng.random() < 0.5,
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
                            policies.append(HeuristicPolicy())
                            continue
                        path = Path(spec["checkpoint"])
                        if path == actor_path:
                            actor_model = model
                            actor_tensorizer = tensorizer
                        else:
                            if path not in model_cache:
                                old_model, _ = load_checkpoint(
                                    path, catalog, device=device
                                )
                                model_cache[path] = ModelPolicy(
                                    old_model,
                                    Tensorizer(catalog, old_model.config),
                                    device=device,
                                    seed=job.seed,
                                )
                            cached = model_cache[path]
                            policies.append(
                                ModelPolicy(
                                    cached.model,
                                    cached.tensorizer,
                                    device=device,
                                    seed=job.seed ^ (seat << 32),
                                    sample=bool(spec.get("sample", False)),
                                    temperature=float(spec.get("temperature", 1.0)),
                                )
                            )
                            continue
                        policies.append(
                            ModelPolicy(
                                actor_model,
                                actor_tensorizer,
                                device=device,
                                seed=job.seed ^ (seat << 32),
                                sample=bool(spec.get("sample", False)),
                                temperature=float(spec.get("temperature", 1.0)),
                            )
                        )
                    raw_episodes.append(
                        play_episode(env, policies, job.match_config, job.seed)
                    )

            annotated: list[tuple[dict[str, Any], set[int]]] = []
            rollout_seconds = time.monotonic() - started
            health = EpisodeHealth()
            for job, episode in zip(jobs, raw_episodes, strict=True):
                controlled = {
                    seat
                    for seat, spec in enumerate(job.policies)
                    if bool(spec.get("train", False))
                }
                episode["training_seats"] = sorted(controlled)
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
                auxiliary_samples=auxiliary_samples,
            )
            learner_step += metrics.updates
            summary = health.summary()
            final_metrics = {
                "held_out_episodes_excluded": sum(
                    contains_excluded_cards(
                        episode, frozenset(train_config.excluded_cards)
                    )
                    for episode in raw_episodes
                ),
                "rollout_seconds": rollout_seconds,
                "iteration_seconds": time.monotonic() - started,
                "loss": metrics.loss,
                "policy_loss": metrics.policy_loss,
                "value_loss": metrics.value_loss,
                "entropy": metrics.entropy,
                "approximate_kl": metrics.approximate_kl,
                "clip_fraction": metrics.clip_fraction,
                "reference_kl": metrics.reference_kl,
                "auxiliary_bc_loss": metrics.auxiliary_bc_loss,
                "ppo_collected_experiences": metrics.collected_experiences,
                "ppo_update_experiences": metrics.update_experiences,
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
                    league.register(
                        league.directory / f"snapshot-{iteration + 1:06d}.pt"
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
                    extra_state=training_state(iteration + 1),
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
        extra_state=training_state(train_config.ppo_iterations),
    )
    return model
