from __future__ import annotations

import argparse
import json
import random
import time
from collections.abc import Iterable, Iterator
from pathlib import Path
from typing import Any

from hearth_env import HearthEnv

from .bc import evaluate_behavior_clone, train_behavior_clone
from .catalog import CardCatalog
from .checkpoint import load_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .decks import Deck, DeckPool, match_config
from .diagnostics import attack_target_diagnostic
from .dmc import train_dmc
from .evaluate import paired_evaluate
from .health import EpisodeHealth
from .interactive import play_interactive_match
from .manifests import write_deck_split
from .model import HearthQNetwork, parameter_count
from .policies import HeuristicPolicy, ModelPolicy, RandomPolicy
from .ppo import train_ppo
from .rollout import ParallelCollector, RolloutJob, play_episode
from .tensorize import Tensorizer
from .trajectory import write_episodes

ROOT = Path(__file__).parents[3]


def _default_deck_paths() -> list[str]:
    frozen_throne = sorted((ROOT / "decks/frozen_throne").glob("*.json"))
    return [str(path) for path in frozen_throne] + [
        str(ROOT / "decks/quest_rogue.json")
    ]


def _decks(paths: list[str]) -> list[Deck]:
    return [Deck.from_file(path) for path in paths]


def _bc_decks(decks: list[Deck]) -> list[Deck]:
    eligible = [deck for deck in decks if deck.bc_eligible]
    if not eligible:
        raise ValueError("no decks are marked bc_eligible for heuristic collection")
    return eligible


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
        bc_learning_rate=args.bc_learning_rate,
        dmc_learning_rate=args.dmc_learning_rate,
        ppo_learning_rate=args.ppo_learning_rate,
        batch_size=args.batch_size,
        workers=args.workers,
        max_steps=args.max_steps,
        history_limit=args.history_limit,
        bc_epochs=getattr(args, "epochs", 3),
        dmc_iterations=getattr(args, "iterations", 1000),
        ppo_iterations=getattr(args, "iterations", 1000),
        episodes_per_iteration=getattr(args, "episodes_per_iteration", 64),
        updates_per_iteration=getattr(args, "updates_per_iteration", 128),
        replay_warmup=getattr(args, "replay_warmup", 2000),
        replay_capacity=getattr(args, "replay_capacity", 500_000),
        epsilon_start=getattr(args, "epsilon_start", 0.25),
        epsilon_end=getattr(args, "epsilon_end", 0.05),
        epsilon_decay_iterations=getattr(args, "epsilon_decay_iterations", 500),
        checkpoint_every=getattr(args, "checkpoint_every", 10),
        league_snapshot_every=getattr(args, "league_snapshot_every", 25),
        ppo_epochs=getattr(args, "ppo_epochs", 4),
        ppo_clip=getattr(args, "ppo_clip", 0.2),
        value_clip=getattr(args, "value_clip", 0.2),
        value_coefficient=getattr(args, "value_coefficient", 0.5),
        entropy_coefficient=getattr(args, "entropy_coefficient", 0.01),
        gamma=getattr(args, "gamma", 0.995),
        gae_lambda=getattr(args, "gae_lambda", 0.95),
        shaping_coefficient=getattr(args, "shaping_coefficient", 0.05),
        reference_kl_coefficient=getattr(args, "reference_kl_coefficient", 0.02),
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
    demonstrations = _bc_decks(decks)
    print(f"deck pool: {len(decks)} total, {len(demonstrations)} heuristic-compatible")
    pool = DeckPool(
        catalog,
        demonstrations,
        seed=args.seed,
        curated_probability=args.curated_probability,
        perturb_probability=args.perturb_probability,
    )
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
            failure_dir=Path(args.output).parent / "failures",
            max_failures=0,
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


def _validate_history(episode: dict[str, Any]) -> None:
    for step in episode.get("steps", []):
        history = step["decision"]["observation"]["history"]
        events = history["events"]
        if history["start_cursor"] + len(events) != history["next_cursor"]:
            raise ValueError("public history window cursor bounds are inconsistent")
        cursors = [int(record["cursor"]) for record in events]
        if cursors and cursors != list(range(cursors[0], cursors[0] + len(cursors))):
            raise ValueError("public history cursors are not contiguous")


def _strict_replay(env: HearthEnv, episode: dict[str, Any]) -> None:
    decision = env.reset(
        seed=int(episode["seed"]), match_config=episode["match_config"]
    )
    for step in episode["steps"]:
        captured = step["decision"]
        if (
            decision.get("actor_seat") != captured.get("actor_seat")
            or decision.get("observation") != captured.get("observation")
            or decision.get("actions") != captured.get("actions")
        ):
            raise ValueError("replayed public decision differs from captured decision")
        transition = env.step(int(step["action_index"]))
        decision = transition["next"]
    if env.replay() != episode["replay"]:
        raise ValueError("authoritative replay differs after deterministic re-execution")


def command_split_decks(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    manifest = write_deck_split(args.deck, args.output_dir, catalog, seed=args.seed)
    split_counts = {
        name: len(records) for name, records in manifest["splits"].items()
    }
    cluster_counts = {
        name: sum(1 for cluster in manifest["clusters"] if cluster["split"] == name)
        for name in manifest["splits"]
    }
    print(json.dumps({"decks": split_counts, "clusters": cluster_counts}, indent=2))


def command_stability(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(catalog, decks, seed=args.seed)
    rng = random.Random(args.seed)
    replay_indices = set(rng.sample(range(args.episodes), min(100, args.episodes)))
    jobs: list[RolloutJob] = []
    for index in range(args.episodes):
        primary = decks[index % len(decks)]
        if (index // len(decks)) % 2:
            primary = pool.perturb(primary, 0.2)
        opponent = pool.sample()
        config = (
            match_config(primary, opponent)
            if index % 2 == 0
            else match_config(opponent, primary)
        )
        kind = "heuristic" if index % 2 == 0 else "random"
        policies = ({"kind": kind}, {"kind": kind})
        jobs.append(
            RolloutJob(
                config,
                args.seed + index,
                policies,
                capture_replay=index in replay_indices,
            )
        )

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    health = EpisodeHealth()
    replayed = 0
    started = time.monotonic()
    if args.workers > 0:
        with ParallelCollector(
            args.data,
            jobs[0].match_config,
            workers=args.workers,
            max_steps=args.max_steps,
            history_limit=args.history_limit,
            card_hash_dim=catalog.hash_dim,
            failure_dir=output_dir / "failures",
            max_failures=0,
        ) as collector:
            episodes: Iterable[dict[str, Any]] = collector.iter_collect(
                jobs, progress_every=max(len(jobs) // 20, 1)
            )
            for episode in episodes:
                _validate_history(episode)
                health.add(episode)
                if "replay" in episode:
                    _strict_replay(env, episode)
                    replayed += 1
    else:
        for job in jobs:
            policies = [
                HeuristicPolicy(job.seed),
                HeuristicPolicy(job.seed ^ 1),
            ] if job.policies[0]["kind"] == "heuristic" else [
                RandomPolicy(job.seed),
                RandomPolicy(job.seed ^ 1),
            ]
            episode = play_episode(
                env,
                policies,
                job.match_config,
                job.seed,
                capture_replay=job.capture_replay,
            )
            _validate_history(episode)
            health.add(episode)
            if "replay" in episode:
                _strict_replay(env, episode)
                replayed += 1
    summary = health.summary()
    report = {
        **summary,
        "strict_replays": replayed,
        "elapsed_seconds": time.monotonic() - started,
    }
    (output_dir / "report.json").write_text(
        json.dumps(report, indent=2), encoding="utf-8"
    )
    print(json.dumps(report, indent=2))
    if health.errors or replayed != min(100, args.episodes):
        raise RuntimeError("stability run had errors or incomplete replay verification")
    if summary["truncation_rate"] >= 0.001:
        raise RuntimeError("stability truncation rate did not meet the <0.1% gate")


def command_train_bc(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    initial_model = None
    if args.init:
        initial_model, _ = load_checkpoint(args.init, catalog, device="cpu")
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
        initial_model=initial_model,
    )


def command_evaluate_bc(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    device = resolve_device(args.device)
    model, payload = load_checkpoint(args.checkpoint, catalog, device=device)
    print(
        json.dumps(
            {
                "checkpoint_step": payload.get("step", 0),
                **evaluate_behavior_clone(
                    catalog,
                    args.input,
                    model,
                    device=device,
                    batch_size=args.batch_size,
                ),
            },
            indent=2,
        )
    )


def command_train_dmc(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(
        catalog,
        decks,
        seed=args.seed,
        curated_probability=args.curated_probability,
        perturb_probability=args.perturb_probability,
    )
    train_dmc(
        args.data,
        catalog,
        pool,
        args.run_dir,
        _train_config(args),
        initial_checkpoint=args.init,
        resume_checkpoint=args.resume,
        bc_shards=args.bc_input,
        model_config=ModelConfig(
            hidden_dim=args.hidden_dim,
            card_hash_dim=args.card_hash_dim,
            transformer_layers=args.layers,
        ),
        specialist_probability=args.specialist_probability,
    )


def command_train_ppo(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(
        catalog,
        decks,
        seed=args.seed,
        curated_probability=args.curated_probability,
        perturb_probability=args.perturb_probability,
    )
    train_ppo(
        args.data,
        catalog,
        pool,
        args.run_dir,
        _train_config(args),
        initial_checkpoint=args.init,
        resume_checkpoint=args.resume,
        model_config=ModelConfig(
            hidden_dim=args.hidden_dim,
            card_hash_dim=args.card_hash_dim,
            transformer_layers=args.layers,
        ),
        specialist_probability=args.specialist_probability,
        reference_checkpoint=args.reference,
    )


def command_evaluate(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    device = resolve_device(args.device)
    model, payload = load_checkpoint(args.checkpoint, catalog, device=device)
    tensorizer = Tensorizer(catalog, model.config)
    pool = DeckPool(
        catalog,
        decks,
        seed=args.seed,
        curated_probability=args.curated_probability,
        perturb_probability=args.perturb_probability,
    )
    matches = [pool.sample_match() for _ in range(args.matches)]
    if args.opponent_checkpoint:
        opponent_model, _ = load_checkpoint(
            args.opponent_checkpoint, catalog, device=device
        )
        opponent_tensorizer = Tensorizer(catalog, opponent_model.config)
        opponent_factory = lambda seed: ModelPolicy(
            opponent_model, opponent_tensorizer, device=device, seed=seed
        )
    else:
        opponent_factory = lambda seed: HeuristicPolicy(seed)
    result = paired_evaluate(
        env,
        lambda seed: ModelPolicy(model, tensorizer, device=device, seed=seed),
        opponent_factory,
        matches,
        seed=args.seed,
    )
    report = {
        "checkpoint_step": payload.get("step", 0),
        **result.summary(),
        "by_matchup": {
            name: value.summary()
            for name, value in sorted(result.by_matchup.items())
        },
    }
    rendered = json.dumps(report, indent=2)
    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(rendered + "\n", encoding="utf-8")
        print(json.dumps({**result.summary(), "output": str(output)}, indent=2))
    else:
        print(rendered)


def command_diagnose_attacks(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(
        catalog,
        decks,
        seed=args.seed,
        curated_probability=args.curated_probability,
        perturb_probability=args.perturb_probability,
    )
    jobs: list[RolloutJob] = []
    for index in range(args.matches):
        model_seat = index % 2
        model = {"kind": "model", "checkpoint": args.checkpoint}
        opponent = {"kind": "heuristic", "noise": 0.0}
        policies = (model, opponent) if model_seat == 0 else (opponent, model)
        jobs.append(RolloutJob(pool.sample_match(), args.seed + index, policies))
    if args.workers > 0:
        with ParallelCollector(
            args.data,
            jobs[0].match_config,
            workers=args.workers,
            max_steps=args.max_steps,
            history_limit=args.history_limit,
            card_hash_dim=catalog.hash_dim,
            failure_dir=Path(args.output).parent / "failures" if args.output else None,
            max_failures=0,
        ) as collector:
            episodes = collector.collect(jobs)
    else:
        device = resolve_device(args.device)
        model, _ = load_checkpoint(args.checkpoint, catalog, device=device)
        tensorizer = Tensorizer(catalog, model.config)
        episodes = []
        for index, job in enumerate(jobs):
            model_policy = ModelPolicy(
                model, tensorizer, device=device, seed=job.seed
            )
            opponent_policy = HeuristicPolicy(job.seed ^ 1, noise=0.0)
            policies = (
                [model_policy, opponent_policy]
                if index % 2 == 0
                else [opponent_policy, model_policy]
            )
            episodes.append(
                play_episode(env, policies, job.match_config, job.seed)
            )
    annotated = [(episode, {index % 2}) for index, episode in enumerate(episodes)]
    report = {
        "checkpoint": args.checkpoint,
        "episodes": len(episodes),
        **attack_target_diagnostic(annotated),
    }
    rendered = json.dumps(report, indent=2)
    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)


def command_play_model(args: argparse.Namespace) -> None:
    human_deck = Deck.from_file(args.human_deck)
    ai_deck = Deck.from_file(args.ai_deck)
    human_seat = args.human_seat - 1
    decks = (
        [human_deck, ai_deck] if human_seat == 0 else [ai_deck, human_deck]
    )
    env, catalog = _env_and_catalog(args, decks)
    device = resolve_device(args.device)
    model, _ = load_checkpoint(args.checkpoint, catalog, device=device)
    tensorizer = Tensorizer(catalog, model.config)
    print(f"你的牌组：{human_deck.name}")
    print(f"AI 牌组：{ai_deck.name}")
    print(f"你是玩家 {args.human_seat}（{'先手' if human_seat == 0 else '后手'}）")
    play_interactive_match(
        env,
        model,
        tensorizer,
        env.match_config["decks"],
        device=device,
        seed=args.seed,
        human_seat=human_seat,
        locale=args.locale,
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
    """Run the BC warm start followed by league-based PPO self-play."""

    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(catalog, decks, seed=args.seed)
    demonstration_decks = _bc_decks(decks)
    demonstration_pool = DeckPool(catalog, demonstration_decks, seed=args.seed)
    print(
        f"deck pool: {len(decks)} total, "
        f"{len(demonstration_decks)} heuristic-compatible"
    )
    run_dir = Path(args.run_dir)
    demonstrations = run_dir / "bc" / "heuristic.jsonl.gz"
    bc_checkpoint = run_dir / "bc" / "model.pt"
    jobs = [
        RolloutJob(
            demonstration_pool.sample_match(),
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
            failure_dir=run_dir / "failures",
            max_failures=0,
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
    train_ppo(
        args.data,
        catalog,
        pool,
        run_dir / "ppo",
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
    root.add_argument("--bc-learning-rate", type=float, default=3e-4)
    root.add_argument("--dmc-learning-rate", type=float, default=1e-5)
    root.add_argument("--ppo-learning-rate", type=float, default=3e-5)
    commands = root.add_subparsers(dest="command", required=True)

    catalog = commands.add_parser("catalog")
    catalog.set_defaults(function=command_catalog)

    collect = commands.add_parser("collect-bc")
    collect.add_argument("--episodes", type=int, default=1000)
    collect.add_argument("--noise", type=float, default=0.08)
    collect.add_argument("--curated-probability", type=float, default=1.0)
    collect.add_argument("--perturb-probability", type=float, default=0.0)
    collect.add_argument("--output", required=True)
    collect.set_defaults(function=command_collect_bc)

    split = commands.add_parser("split-decks")
    split.add_argument("--output-dir", required=True)
    split.set_defaults(function=command_split_decks)

    stability = commands.add_parser("stability")
    stability.add_argument("--episodes", type=int, default=5000)
    stability.add_argument("--output-dir", required=True)
    stability.set_defaults(function=command_stability)

    bc = commands.add_parser("train-bc")
    bc.add_argument("--input", action="append", required=True)
    bc.add_argument("--output", required=True)
    bc.add_argument("--epochs", type=int, default=3)
    bc.add_argument("--init")
    bc.set_defaults(function=command_train_bc)

    evaluate_bc = commands.add_parser("evaluate-bc")
    evaluate_bc.add_argument("--input", action="append", required=True)
    evaluate_bc.add_argument("--checkpoint", required=True)
    evaluate_bc.set_defaults(function=command_evaluate_bc)

    dmc = commands.add_parser("train-dmc")
    checkpoint = dmc.add_mutually_exclusive_group()
    checkpoint.add_argument("--init")
    checkpoint.add_argument("--resume")
    dmc.add_argument("--bc-input", action="append", default=[])
    dmc.add_argument("--run-dir", required=True)
    dmc.add_argument("--iterations", type=int, default=1000)
    dmc.add_argument("--episodes-per-iteration", type=int, default=64)
    dmc.add_argument("--updates-per-iteration", type=int, default=128)
    dmc.add_argument("--replay-warmup", type=int, default=2000)
    dmc.add_argument("--replay-capacity", type=int, default=500_000)
    dmc.add_argument("--epsilon-start", type=float, default=0.25)
    dmc.add_argument("--epsilon-end", type=float, default=0.05)
    dmc.add_argument("--epsilon-decay-iterations", type=int, default=500)
    dmc.add_argument("--checkpoint-every", type=int, default=10)
    dmc.add_argument("--league-snapshot-every", type=int, default=25)
    dmc.add_argument("--specialist-probability", type=float, default=0.1)
    dmc.add_argument("--curated-probability", type=float, default=1.0)
    dmc.add_argument("--perturb-probability", type=float, default=0.0)
    dmc.set_defaults(function=command_train_dmc)

    ppo = commands.add_parser("train-ppo")
    checkpoint = ppo.add_mutually_exclusive_group()
    checkpoint.add_argument("--init")
    checkpoint.add_argument("--resume")
    ppo.add_argument("--reference")
    ppo.add_argument("--run-dir", required=True)
    ppo.add_argument("--iterations", type=int, default=1000)
    ppo.add_argument("--episodes-per-iteration", type=int, default=64)
    ppo.add_argument("--ppo-epochs", type=int, default=4)
    ppo.add_argument("--ppo-clip", type=float, default=0.2)
    ppo.add_argument("--value-clip", type=float, default=0.2)
    ppo.add_argument("--value-coefficient", type=float, default=0.5)
    ppo.add_argument("--entropy-coefficient", type=float, default=0.01)
    ppo.add_argument("--gamma", type=float, default=0.995)
    ppo.add_argument("--gae-lambda", type=float, default=0.95)
    ppo.add_argument("--shaping-coefficient", type=float, default=0.05)
    ppo.add_argument("--reference-kl-coefficient", type=float, default=0.02)
    ppo.add_argument("--checkpoint-every", type=int, default=10)
    ppo.add_argument("--league-snapshot-every", type=int, default=25)
    ppo.add_argument("--specialist-probability", type=float, default=0.1)
    ppo.add_argument("--curated-probability", type=float, default=1.0)
    ppo.add_argument("--perturb-probability", type=float, default=0.0)
    ppo.set_defaults(function=command_train_ppo)

    evaluate = commands.add_parser("evaluate")
    evaluate.add_argument("--checkpoint", required=True)
    evaluate.add_argument("--opponent-checkpoint")
    evaluate.add_argument("--matches", type=int, default=100)
    evaluate.add_argument("--output")
    evaluate.add_argument("--curated-probability", type=float, default=1.0)
    evaluate.add_argument("--perturb-probability", type=float, default=0.0)
    evaluate.set_defaults(function=command_evaluate)

    diagnose = commands.add_parser("diagnose-attacks")
    diagnose.add_argument("--checkpoint", required=True)
    diagnose.add_argument("--matches", type=int, default=500)
    diagnose.add_argument("--output")
    diagnose.add_argument("--curated-probability", type=float, default=1.0)
    diagnose.add_argument("--perturb-probability", type=float, default=0.0)
    diagnose.set_defaults(function=command_diagnose_attacks)

    play_model = commands.add_parser("play-model")
    play_model.add_argument("--checkpoint", required=True)
    play_model.add_argument("--human-deck", required=True)
    play_model.add_argument("--ai-deck", required=True)
    play_model.add_argument("--human-seat", type=int, choices=(1, 2), default=1)
    play_model.add_argument("--locale", choices=("enUS", "zhCN", "zhTW"), default="zhCN")
    play_model.set_defaults(function=command_play_model)

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
    pipeline.add_argument("--ppo-epochs", type=int, default=4)
    pipeline.add_argument("--ppo-clip", type=float, default=0.2)
    pipeline.add_argument("--value-clip", type=float, default=0.2)
    pipeline.add_argument("--value-coefficient", type=float, default=0.5)
    pipeline.add_argument("--entropy-coefficient", type=float, default=0.01)
    pipeline.add_argument("--gamma", type=float, default=0.995)
    pipeline.add_argument("--gae-lambda", type=float, default=0.95)
    pipeline.add_argument("--shaping-coefficient", type=float, default=0.05)
    pipeline.add_argument("--reference-kl-coefficient", type=float, default=0.02)
    pipeline.set_defaults(function=command_pipeline)
    return root


def main() -> None:
    args = parser().parse_args()
    if not args.deck:
        args.deck = _default_deck_paths()
    args.function(args)


if __name__ == "__main__":
    main()
