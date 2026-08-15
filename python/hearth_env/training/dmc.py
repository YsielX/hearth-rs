from __future__ import annotations

import random
from pathlib import Path
from typing import Any

import torch

from hearth_env import HearthEnv

from .catalog import CardCatalog
from .checkpoint import load_checkpoint, save_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .decks import DeckPool
from .league import CheckpointLeague
from .learn import train_batch
from .model import HearthQNetwork
from .policies import HeuristicPolicy, ModelPolicy
from .rollout import ParallelCollector, RolloutJob, play_episode
from .tensorize import Tensorizer
from .trajectory import ReplayBuffer, write_episodes


def _epsilon(config: TrainConfig, iteration: int) -> float:
    progress = min(iteration / max(config.epsilon_decay_iterations, 1), 1.0)
    return config.epsilon_start + progress * (config.epsilon_end - config.epsilon_start)


def train_dmc(
    data_path: str | Path,
    catalog: CardCatalog,
    deck_pool: DeckPool,
    run_dir: str | Path,
    train_config: TrainConfig,
    *,
    initial_checkpoint: str | Path | None = None,
    model_config: ModelConfig | None = None,
    specialist_probability: float = 0.1,
) -> HearthQNetwork:
    """Train action values against complete-game Monte Carlo returns.

    Actors may use current, historical, or heuristic policies. Since the same
    network represents both players, all non-truncated actions are valid
    off-policy state/action/terminal-return examples.
    """

    run_dir = Path(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    device = resolve_device(train_config.device)
    rng = random.Random(train_config.seed)
    torch.manual_seed(train_config.seed)
    if initial_checkpoint:
        model, payload = load_checkpoint(initial_checkpoint, catalog, device=device)
        start_step = int(payload.get("step", 0))
        migration = payload.get("migration", {})
        print(
            "checkpoint migration "
            f"retained={migration.get('retained_cards', 0)} new={migration.get('new_cards', 0)}"
        )
    else:
        model = HearthQNetwork(
            catalog, model_config or ModelConfig(card_hash_dim=catalog.hash_dim)
        ).to(device)
        start_step = 0
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=train_config.learning_rate,
        weight_decay=train_config.weight_decay,
    )
    if (
        initial_checkpoint
        and payload.get("optimizer")
        and not payload["migration"]["new_cards"]
    ):
        optimizer.load_state_dict(payload["optimizer"])
    tensorizer = Tensorizer(catalog, model.config)
    replay = ReplayBuffer(train_config.replay_capacity, train_config.seed)
    league = CheckpointLeague(run_dir / "league", seed=train_config.seed)
    actor_path = run_dir / "actor.pt"
    latest_path = run_dir / "latest.pt"
    base_config = deck_pool.sample_match()
    env = None
    collector = None
    if train_config.workers > 0:
        collector = ParallelCollector(
            data_path,
            base_config,
            workers=train_config.workers,
            max_steps=train_config.max_steps,
            history_limit=train_config.history_limit,
            card_hash_dim=catalog.hash_dim,
        )
    else:
        env = HearthEnv(
            data_path,
            base_config,
            max_steps=train_config.max_steps,
            history_limit=train_config.history_limit,
        )
    learner_step = start_step
    try:
        for iteration in range(train_config.dmc_iterations):
            epsilon = _epsilon(train_config, iteration)
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
                current = {
                    "kind": "model",
                    "checkpoint": str(actor_path),
                    "epsilon": epsilon,
                }
                if rng.random() < specialist_probability:
                    opponent: dict[str, Any] = {"kind": "heuristic", "noise": 0.15}
                else:
                    opponent = {
                        "kind": "model",
                        "checkpoint": str(league.sample(actor_path)),
                        "epsilon": epsilon,
                    }
                policies = (
                    (current, opponent) if current_seat == 0 else (opponent, current)
                )
                jobs.append(RolloutJob(deck_pool.sample_match(), seed, policies))

            if collector is not None:
                episodes = collector.collect(jobs)
            else:
                assert env is not None
                current_policy = ModelPolicy(
                    model,
                    tensorizer,
                    device=device,
                    epsilon=epsilon,
                    seed=rng.randrange(2**63),
                )
                model_cache: dict[Path, ModelPolicy] = {}
                episodes = []
                for job in jobs:
                    policies = []
                    for seat, spec in enumerate(job.policies):
                        if spec["kind"] == "heuristic":
                            policies.append(
                                HeuristicPolicy(
                                    job.seed ^ seat, spec.get("noise", 0.08)
                                )
                            )
                            continue
                        path = Path(spec["checkpoint"])
                        if path == actor_path:
                            policies.append(current_policy)
                        else:
                            if path not in model_cache:
                                old_model, _ = load_checkpoint(
                                    path, catalog, device=device
                                )
                                model_cache[path] = ModelPolicy(
                                    old_model,
                                    tensorizer,
                                    device=device,
                                    epsilon=epsilon,
                                    seed=job.seed,
                                )
                            policies.append(model_cache[path])
                    episodes.append(
                        play_episode(env, policies, job.match_config, job.seed)
                    )

            completed = 0
            new_samples = 0
            for episode in episodes:
                if not episode["truncated"]:
                    completed += 1
                    new_samples += replay.extend_episode(episode)
            write_episodes(
                run_dir / "rollouts" / f"iteration-{iteration:06d}.jsonl.gz", episodes
            )

            losses: list[float] = []
            if len(replay) >= max(train_config.replay_warmup, train_config.batch_size):
                for _ in range(train_config.updates_per_iteration):
                    metrics = train_batch(
                        model,
                        optimizer,
                        tensorizer,
                        replay.sample(train_config.batch_size),
                        train_config,
                        device,
                        behavior_clone=False,
                    )
                    losses.append(metrics.loss)
                    learner_step += 1
            mean_loss = sum(losses) / max(len(losses), 1)
            print(
                f"dmc iteration={iteration + 1} completed={completed}/{len(episodes)} "
                f"new_samples={new_samples} replay={len(replay)} epsilon={epsilon:.3f} "
                f"updates={len(losses)} loss={mean_loss:.5f}"
            )
            if (iteration + 1) % train_config.checkpoint_every == 0 or iteration == 0:
                save_checkpoint(
                    latest_path,
                    model,
                    catalog,
                    optimizer=optimizer,
                    step=learner_step,
                    phase="dmc",
                    metrics={"loss": mean_loss, "replay_size": float(len(replay))},
                )
            if (iteration + 1) % train_config.league_snapshot_every == 0:
                save_checkpoint(
                    league.directory / f"snapshot-{iteration + 1:06d}.pt",
                    model,
                    catalog,
                    step=learner_step,
                    phase="league",
                )
    finally:
        if collector is not None:
            collector.close()
    save_checkpoint(
        latest_path, model, catalog, optimizer=optimizer, step=learner_step, phase="dmc"
    )
    return model
