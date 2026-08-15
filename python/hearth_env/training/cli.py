from __future__ import annotations

import argparse
import json
import time
from collections.abc import Iterable, Iterator
from pathlib import Path
from typing import Any

from hearth_env import HearthEnv

from .bc import train_behavior_clone
from .catalog import CardCatalog
from .checkpoint import load_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .decks import Deck, DeckPool, match_config
from .dmc import train_dmc
from .evaluate import paired_evaluate
from .model import HearthQNetwork, parameter_count
from .policies import HeuristicPolicy, ModelPolicy
from .rollout import ParallelCollector, RolloutJob, play_episode
from .tensorize import Tensorizer
from .trajectory import write_episodes


def _decks(paths: list[str]) -> list[Deck]:
    return [Deck.from_file(path) for path in paths]


def _env_and_catalog(
    args: argparse.Namespace, decks: list[Deck]
) -> tuple[HearthEnv, CardCatalog]:
    initial = match_config(decks[0], decks[min(1, len(decks) - 1)])
    env = HearthEnv(
        args.data,
        initial,
        seed=args.seed,
        max_steps=getattr(args, "max_steps", 1000),
        history_limit=getattr(args, "history_limit", 96),
    )
    catalog = CardCatalog.build(
        env.card_catalog, env.pack_hash, hash_dim=args.card_hash_dim
    )
    return env, catalog


def _train_config(args: argparse.Namespace) -> TrainConfig:
    return TrainConfig(
        device=args.device,
        seed=args.seed,
        learning_rate=args.learning_rate,
        batch_size=args.batch_size,
        workers=args.workers,
        max_steps=args.max_steps,
        history_limit=args.history_limit,
        bc_epochs=getattr(args, "epochs", 3),
        dmc_iterations=getattr(args, "iterations", 1000),
        episodes_per_iteration=getattr(args, "episodes_per_iteration", 64),
        updates_per_iteration=getattr(args, "updates_per_iteration", 128),
        replay_warmup=getattr(args, "replay_warmup", 2000),
        checkpoint_every=getattr(args, "checkpoint_every", 10),
        league_snapshot_every=getattr(args, "league_snapshot_every", 25),
    )


def command_catalog(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    source_bytes = sum(
        len(entry.get("lua_source", "").encode()) for entry in catalog.entries.values()
    )
    unique_sources = {entry.get("lua_path") for entry in catalog.entries.values()}
    print(
        json.dumps(
            {
                "pack_hash": catalog.pack_hash,
                "cards": len(catalog.entries),
                "unique_lua_units": len(unique_sources),
                "associated_lua_bytes": source_bytes,
                "semantic_feature_dim": catalog.feature_dim,
            },
            indent=2,
        )
    )


def command_collect_bc(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(catalog, decks, seed=args.seed)
    jobs = [
        RolloutJob(
            pool.sample_match(),
            args.seed + index,
            ({"kind": "heuristic", "noise": args.noise},) * 2,
        )
        for index in range(args.episodes)
    ]
    started = time.monotonic()
    decisions = 0

    def counted(episodes: Iterable[dict[str, Any]]) -> Iterator[dict[str, Any]]:
        nonlocal decisions
        for episode in episodes:
            decisions += len(episode["steps"])
            yield episode

    if args.workers > 0:
        with ParallelCollector(
            args.data,
            jobs[0].match_config,
            workers=args.workers,
            max_steps=args.max_steps,
            history_limit=args.history_limit,
            card_hash_dim=catalog.hash_dim,
        ) as collector:
            written = write_episodes(
                args.output,
                counted(
                    collector.iter_collect(jobs, progress_every=max(len(jobs) // 20, 1))
                ),
            )
    else:
        episodes = (
            play_episode(
                env,
                [
                    HeuristicPolicy(job.seed, args.noise),
                    HeuristicPolicy(job.seed ^ 1, args.noise),
                ],
                job.match_config,
                job.seed,
            )
            for job in jobs
        )
        written = write_episodes(args.output, counted(episodes))
    elapsed = time.monotonic() - started
    print(
        f"wrote {written} episodes / {decisions} decisions to {args.output} "
        f"in {elapsed:.1f}s ({decisions / max(elapsed, 1e-6):.1f} decisions/s)"
    )


def command_train_bc(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    train_behavior_clone(
        catalog,
        args.input,
        args.output,
        _train_config(args),
        model_config=ModelConfig(
            hidden_dim=args.hidden_dim,
            card_hash_dim=args.card_hash_dim,
            transformer_layers=args.layers,
        ),
    )


def command_train_dmc(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(catalog, decks, seed=args.seed)
    train_dmc(
        args.data,
        catalog,
        pool,
        args.run_dir,
        _train_config(args),
        initial_checkpoint=args.init,
        model_config=ModelConfig(
            hidden_dim=args.hidden_dim,
            card_hash_dim=args.card_hash_dim,
            transformer_layers=args.layers,
        ),
        specialist_probability=args.specialist_probability,
    )


def command_evaluate(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    device = resolve_device(args.device)
    model, payload = load_checkpoint(args.checkpoint, catalog, device=device)
    tensorizer = Tensorizer(catalog, model.config)
    pool = DeckPool(catalog, decks, seed=args.seed)
    matches = [pool.sample_match() for _ in range(args.matches)]
    result = paired_evaluate(
        env,
        lambda seed: ModelPolicy(model, tensorizer, device=device, seed=seed),
        lambda seed: HeuristicPolicy(seed),
        matches,
        seed=args.seed,
    )
    print(
        json.dumps(
            {
                "checkpoint_step": payload.get("step", 0),
                **result.summary(),
                "by_matchup": {
                    name: value.summary()
                    for name, value in sorted(result.by_matchup.items())
                },
            },
            indent=2,
        )
    )


def command_smoke(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    config = ModelConfig(
        hidden_dim=args.hidden_dim,
        card_hash_dim=args.card_hash_dim,
        transformer_layers=args.layers,
    )
    device = resolve_device(args.device)
    model = HearthQNetwork(catalog, config).to(device)
    tensorizer = Tensorizer(catalog, config)
    policy = ModelPolicy(model, tensorizer, device=device, epsilon=0.2, seed=args.seed)
    episode = play_episode(
        env, [policy, HeuristicPolicy(args.seed)], env.match_config, args.seed
    )
    print(
        json.dumps(
            {
                "device": device,
                "parameters": parameter_count(model),
                "cards": len(catalog.card_ids) - 2,
                "decisions": len(episode["steps"]),
                "terminated": episode["terminated"],
                "truncated": episode["truncated"],
                "rewards": episode["rewards"],
            },
            indent=2,
        )
    )


def command_pipeline(args: argparse.Namespace) -> None:
    """Run the BC warm start followed by league-based DMC self-play."""

    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(catalog, decks, seed=args.seed)
    run_dir = Path(args.run_dir)
    demonstrations = run_dir / "bc" / "heuristic.jsonl.gz"
    bc_checkpoint = run_dir / "bc" / "model.pt"
    jobs = [
        RolloutJob(
            pool.sample_match(),
            args.seed + index,
            ({"kind": "heuristic", "noise": args.noise},) * 2,
        )
        for index in range(args.bc_episodes)
    ]
    if args.workers > 0:
        with ParallelCollector(
            args.data,
            jobs[0].match_config,
            workers=args.workers,
            max_steps=args.max_steps,
            history_limit=args.history_limit,
            card_hash_dim=catalog.hash_dim,
        ) as collector:
            write_episodes(
                demonstrations,
                collector.iter_collect(jobs, progress_every=max(len(jobs) // 20, 1)),
            )
    else:
        episodes = (
            play_episode(
                env,
                [
                    HeuristicPolicy(job.seed, args.noise),
                    HeuristicPolicy(job.seed ^ 1, args.noise),
                ],
                job.match_config,
                job.seed,
            )
            for job in jobs
        )
        write_episodes(demonstrations, episodes)
    config = _train_config(args)
    config.bc_epochs = args.bc_epochs
    model_config = ModelConfig(
        hidden_dim=args.hidden_dim,
        card_hash_dim=args.card_hash_dim,
        transformer_layers=args.layers,
    )
    train_behavior_clone(
        catalog,
        [demonstrations],
        bc_checkpoint,
        config,
        model_config=model_config,
    )
    train_dmc(
        args.data,
        catalog,
        pool,
        run_dir / "dmc",
        config,
        initial_checkpoint=bc_checkpoint,
        specialist_probability=args.specialist_probability,
    )


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="hearth-train")
    root.add_argument("--data", default="data")
    root.add_argument("--deck", action="append", default=[])
    root.add_argument("--seed", type=int, default=0)
    root.add_argument("--max-steps", type=int, default=1000)
    root.add_argument("--history-limit", type=int, default=96)
    root.add_argument("--card-hash-dim", type=int, default=256)
    root.add_argument("--device", default="auto")
    root.add_argument("--workers", type=int, default=0)
    root.add_argument("--hidden-dim", type=int, default=128)
    root.add_argument("--layers", type=int, default=2)
    root.add_argument("--batch-size", type=int, default=128)
    root.add_argument("--learning-rate", type=float, default=3e-4)
    commands = root.add_subparsers(dest="command", required=True)

    catalog = commands.add_parser("catalog")
    catalog.set_defaults(function=command_catalog)

    collect = commands.add_parser("collect-bc")
    collect.add_argument("--episodes", type=int, default=1000)
    collect.add_argument("--noise", type=float, default=0.08)
    collect.add_argument("--output", required=True)
    collect.set_defaults(function=command_collect_bc)

    bc = commands.add_parser("train-bc")
    bc.add_argument("--input", action="append", required=True)
    bc.add_argument("--output", required=True)
    bc.add_argument("--epochs", type=int, default=3)
    bc.set_defaults(function=command_train_bc)

    dmc = commands.add_parser("train-dmc")
    dmc.add_argument("--init")
    dmc.add_argument("--run-dir", required=True)
    dmc.add_argument("--iterations", type=int, default=1000)
    dmc.add_argument("--episodes-per-iteration", type=int, default=64)
    dmc.add_argument("--updates-per-iteration", type=int, default=128)
    dmc.add_argument("--replay-warmup", type=int, default=2000)
    dmc.add_argument("--checkpoint-every", type=int, default=10)
    dmc.add_argument("--league-snapshot-every", type=int, default=25)
    dmc.add_argument("--specialist-probability", type=float, default=0.1)
    dmc.set_defaults(function=command_train_dmc)

    evaluate = commands.add_parser("evaluate")
    evaluate.add_argument("--checkpoint", required=True)
    evaluate.add_argument("--matches", type=int, default=100)
    evaluate.set_defaults(function=command_evaluate)

    smoke = commands.add_parser("smoke")
    smoke.set_defaults(function=command_smoke)

    pipeline = commands.add_parser("pipeline")
    pipeline.add_argument("--run-dir", required=True)
    pipeline.add_argument("--bc-episodes", type=int, default=10_000)
    pipeline.add_argument("--bc-epochs", type=int, default=3)
    pipeline.add_argument("--noise", type=float, default=0.08)
    pipeline.add_argument("--iterations", type=int, default=1000)
    pipeline.add_argument("--episodes-per-iteration", type=int, default=64)
    pipeline.add_argument("--updates-per-iteration", type=int, default=128)
    pipeline.add_argument("--replay-warmup", type=int, default=2000)
    pipeline.add_argument("--checkpoint-every", type=int, default=10)
    pipeline.add_argument("--league-snapshot-every", type=int, default=25)
    pipeline.add_argument("--specialist-probability", type=float, default=0.1)
    pipeline.set_defaults(function=command_pipeline)
    return root


def main() -> None:
    args = parser().parse_args()
    if not args.deck:
        args.deck = ["decks/demo.json", "decks/quest_rogue.json"]
    args.function(args)


if __name__ == "__main__":
    main()
