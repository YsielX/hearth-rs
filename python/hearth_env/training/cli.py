from __future__ import annotations

import argparse
import hashlib
import json
import random
import time
from collections.abc import Iterable, Iterator
from pathlib import Path
from typing import Any

import torch

from hearth_env import HearthEnv

from .bc import evaluate_behavior_clone, train_behavior_clone
from .catalog import CardCatalog
from .checkpoint import load_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .decks import Deck, DeckPool, match_config
from .diagnostics import attack_target_diagnostic
from .dmc import train_dmc
from .evaluate import paired_evaluate, paired_configurations
from .health import EpisodeHealth
from .interactive import play_interactive_match
from .manifests import write_deck_split, load_split_paths
from .model import HearthQNetwork, parameter_count
from .policies import HeuristicPolicy, ModelPolicy, RandomPolicy
from .ppo import train_ppo
from .rollout import ParallelCollector, RolloutJob, play_episode
from .tensorize import Tensorizer
from .trajectory import write_episodes, contains_excluded_cards

ROOT = Path(__file__).parents[3]


def _default_deck_paths() -> list[str]:
    frozen_throne = sorted((ROOT / "decks/frozen_throne").glob("*.json"))
    return (
        [str(path) for path in frozen_throne]
        + [str(ROOT / "decks/quest_rogue.json")]
        + [str(path) for path in sorted((ROOT / "decks/training").glob("*.json"))]
    )


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
        excluded_cards=tuple(getattr(args, "excluded_cards", ())),
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
        dmc_iterations=getattr(args, "iterations", 1000)
        if args.command == "train-dmc"
        else 1000,
        ppo_iterations=getattr(args, "iterations", 1000)
        if args.command != "train-dmc"
        else 1000,
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
        ppo_temperature=getattr(args, "ppo_temperature", 1.0),
        ppo_clip=getattr(args, "ppo_clip", 0.2),
        value_clip=getattr(args, "value_clip", 0.2),
        value_coefficient=getattr(args, "value_coefficient", 0.5),
        entropy_coefficient=getattr(args, "entropy_coefficient", 0.01),
        gamma=getattr(args, "gamma", 1.0),
        gae_lambda=getattr(args, "gae_lambda", 0.98),
        reference_kl_coefficient=getattr(args, "reference_kl_coefficient", 0.0),
        ppo_bc_coefficient=getattr(args, "ppo_bc_coefficient", 0.0),
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
    demonstrations = decks if args.teacher_checkpoint else _bc_decks(decks)
    print(f"deck pool: {len(decks)} total, {len(demonstrations)} demonstration decks")
    teacher_spec = (
        {"kind": "model", "checkpoint": args.teacher_checkpoint}
        if args.teacher_checkpoint
        else {"kind": "heuristic"}
    )
    pool = DeckPool(
        catalog,
        demonstrations,
        seed=args.seed,
        curated_probability=args.curated_probability,
        perturb_probability=args.perturb_probability,
        card_pool=args.card_pool,
        excluded_cards=getattr(args, "excluded_cards", ()),
    )
    jobs = [
        RolloutJob(
            pool.sample_match(),
            args.seed + index,
            (teacher_spec,) * 2,
        )
        for index in range(args.episodes)
    ]
    started = time.monotonic()
    decisions = 0

    def counted(episodes: Iterable[dict[str, Any]]) -> Iterator[dict[str, Any]]:
        nonlocal decisions
        for episode in episodes:
            if episode.get("error"):
                raise RuntimeError(
                    f"demonstration environment error: {episode['error']}"
                )
            if contains_excluded_cards(episode, frozenset(args.excluded_cards)):
                continue
            episode["source"] = (
                "model_demonstration"
                if args.teacher_checkpoint
                else "heuristic_demonstration"
            )
            episode["teacher_checkpoint"] = args.teacher_checkpoint
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
        teacher = None
        if args.teacher_checkpoint:
            teacher, _ = load_checkpoint(args.teacher_checkpoint, catalog, device="cpu")
        episodes = (
            play_episode(
                env,
                [
                    ModelPolicy(
                        teacher,
                        Tensorizer(catalog, teacher.config),
                        device="cpu",
                        seed=job.seed,
                    )
                    if teacher is not None
                    else HeuristicPolicy(),
                    ModelPolicy(
                        teacher,
                        Tensorizer(catalog, teacher.config),
                        device="cpu",
                        seed=job.seed ^ 1,
                    )
                    if teacher is not None
                    else HeuristicPolicy(),
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
        raise ValueError(
            "authoritative replay differs after deterministic re-execution"
        )


def command_split_decks(args: argparse.Namespace) -> None:
    if not args.include_complex:
        args.deck = [
            path for path in args.deck if Deck.from_file(path).strategy != "combo"
        ]
    decks = _decks(args.deck)
    _, catalog = _env_and_catalog(args, decks)
    manifest = write_deck_split(args.deck, args.output_dir, catalog, seed=args.seed)
    split_counts = {name: len(records) for name, records in manifest["splits"].items()}
    cluster_counts = {
        name: sum(1 for cluster in manifest["clusters"] if cluster["split"] == name)
        for name in manifest["splits"]
    }
    print(json.dumps({"decks": split_counts, "clusters": cluster_counts}, indent=2))


def command_stability(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    pool = DeckPool(catalog, decks, seed=args.seed, card_pool=args.card_pool)
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
            policies = (
                [
                    HeuristicPolicy(),
                    HeuristicPolicy(),
                ]
                if job.policies[0]["kind"] == "heuristic"
                else [
                    RandomPolicy(job.seed),
                    RandomPolicy(job.seed ^ 1),
                ]
            )
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
        replay_shards=args.replay_input,
        replay_fraction=args.replay_fraction,
        validation_shards=args.validation_input,
    )


def command_evaluate_bc(args: argparse.Namespace) -> None:
    decks = _decks(args.deck)
    env, catalog = _env_and_catalog(args, decks)
    device = resolve_device(args.device)
    model, payload = load_checkpoint(args.checkpoint, catalog, device=device)
    print(
        json.dumps(
            {
                "checkpoint": str(args.checkpoint),
                "checkpoint_sha256": hashlib.sha256(
                    Path(args.checkpoint).read_bytes()
                ).hexdigest(),
                "pack_hash": env.pack_hash,
                "engine_build": env.engine_build,
                "inputs": [
                    {
                        "path": str(path),
                        "sha256": hashlib.sha256(Path(path).read_bytes()).hexdigest(),
                    }
                    for path in args.input
                ],
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
        card_pool=args.card_pool,
        excluded_cards=getattr(args, "excluded_cards", ()),
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
        card_pool=args.card_pool,
        excluded_cards=getattr(args, "excluded_cards", ()),
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
        bc_shards=args.bc_input,
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
        card_pool=args.card_pool,
        excluded_cards=getattr(args, "excluded_cards", ()),
    )
    swap_decks = not args.mirror
    if args.match_list:
        frozen = json.loads(Path(args.match_list).read_text(encoding="utf-8"))
        if (
            frozen["pack_hash"] != env.pack_hash
            or frozen["engine_build"] != env.engine_build
        ):
            raise ValueError(
                "evaluation match list requires its frozen card pack and engine"
            )
        matches, args.seed = frozen["matches"], frozen["seed"]
        swap_decks = bool(frozen.get("swap_decks", True))
        if args.mirror and swap_decks:
            raise ValueError("--mirror conflicts with the frozen four-game pairing")
    else:
        if args.matches < len(decks) and args.balanced:
            raise ValueError("balanced evaluation needs at least one match per deck")
        if args.balanced:
            random.Random(args.seed).shuffle(decks)
        matches = (
            [
                match_config(
                    decks[i % len(decks)], decks[(i + 1 + i // len(decks)) % len(decks)]
                )
                for i in range(args.matches)
            ]
            if args.balanced
            else [pool.sample_match() for _ in range(args.matches)]
        )
        if args.mirror:
            for config in matches:
                for key in ("decks", "hero_powers", "classes", "sideboards"):
                    if key in config:
                        config[key] = [config[key][0], config[key][0]]
    frozen_matches = {
        "pack_hash": env.pack_hash,
        "engine_build": env.engine_build,
        "seed": args.seed,
        "matches": matches,
        "swap_decks": swap_decks,
    }
    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.with_suffix(".matches.json").write_text(
            json.dumps(frozen_matches, indent=2), encoding="utf-8"
        )
    if args.opponent_checkpoint:
        opponent_model, _ = load_checkpoint(
            args.opponent_checkpoint, catalog, device=device
        )
        opponent_tensorizer = Tensorizer(catalog, opponent_model.config)
        opponent_factory = lambda seed: ModelPolicy(
            opponent_model, opponent_tensorizer, device=device, seed=seed
        )
    else:
        opponent_factory = lambda seed: HeuristicPolicy()
    episodes = None
    if args.workers > 0:
        jobs = []
        opponent_spec = (
            {"kind": "model", "checkpoint": args.opponent_checkpoint}
            if args.opponent_checkpoint
            else {"kind": "heuristic"}
        )
        for index, base in enumerate(matches):
            game_seed = args.seed + index
            for config, seat in paired_configurations(base, swap_decks=swap_decks):
                specs = [
                    {**opponent_spec, "policy_seed": game_seed},
                    {**opponent_spec, "policy_seed": game_seed ^ 0xA5A5},
                ]
                specs[seat] = {
                    "kind": "model",
                    "checkpoint": args.checkpoint,
                    "policy_seed": game_seed ^ 0x5A5A,
                }
                jobs.append(RolloutJob(config, game_seed, tuple(specs)))
        with ParallelCollector(
            args.data,
            matches[0],
            workers=args.workers,
            max_steps=args.max_steps,
            history_limit=args.history_limit,
            card_hash_dim=catalog.hash_dim,
            failure_dir=Path(args.output).with_suffix(".failures")
            if args.output
            else None,
        ) as collector:
            episodes = collector.collect(jobs, progress_every=max(len(jobs) // 10, 1))
    result = paired_evaluate(
        env,
        lambda seed: ModelPolicy(model, tensorizer, device=device, seed=seed),
        opponent_factory,
        matches,
        seed=args.seed,
        episodes=episodes,
        swap_decks=swap_decks,
    )
    report = {
        "pack_hash": env.pack_hash,
        "engine_build": env.engine_build,
        "checkpoint": str(args.checkpoint),
        "opponent_checkpoint": args.opponent_checkpoint,
        "checkpoint_step": payload.get("step", 0),
        "swap_decks": swap_decks,
        **result.summary(),
        "by_matchup": {
            name: value.summary() for name, value in sorted(result.by_matchup.items())
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
        card_pool=args.card_pool,
        excluded_cards=getattr(args, "excluded_cards", ()),
    )
    jobs: list[RolloutJob] = []
    for index in range(args.matches):
        model_seat = index % 2
        model = {"kind": "model", "checkpoint": args.checkpoint}
        opponent = {"kind": "heuristic"}
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
            model_policy = ModelPolicy(model, tensorizer, device=device, seed=job.seed)
            opponent_policy = HeuristicPolicy()
            policies = (
                [model_policy, opponent_policy]
                if index % 2 == 0
                else [opponent_policy, model_policy]
            )
            episodes.append(play_episode(env, policies, job.match_config, job.seed))
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
    decks = [human_deck, ai_deck] if human_seat == 0 else [ai_deck, human_deck]
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
        env, [policy, HeuristicPolicy()], env.match_config, args.seed
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


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="hearth-train")
    root.add_argument("--data", default="data")
    root.add_argument("--deck", action="append", default=[])
    root.add_argument(
        "--deck-manifest", help="Frozen train/validation/test deck manifest"
    )
    root.add_argument("--split", choices=("train", "validation", "test"))
    root.add_argument(
        "--card-split", help="Held-out simple card manifest (applies to training)"
    )
    root.add_argument("--seed", type=int, default=0)
    root.add_argument("--max-steps", type=int, default=1000)
    root.add_argument("--history-limit", type=int, default=96)
    root.add_argument("--card-hash-dim", type=int, default=256)
    root.add_argument("--card-pool", choices=("all", "era"), default="all")
    root.add_argument("--device", default="auto")
    root.add_argument("--workers", type=int, default=0)
    root.add_argument("--torch-threads", type=int, default=8)
    root.add_argument("--hidden-dim", type=int, default=128)
    root.add_argument("--layers", type=int, default=2)
    root.add_argument("--batch-size", type=int, default=128)
    root.add_argument("--bc-learning-rate", type=float, default=3e-4)
    root.add_argument("--dmc-learning-rate", type=float, default=1e-5)
    root.add_argument("--ppo-learning-rate", type=float, default=3e-5)
    commands = root.add_subparsers(dest="command", required=True)

    catalog = commands.add_parser("catalog")
    catalog.set_defaults(function=command_catalog)

    def coverage_report(args):
        from .coverage import decision_coverage
        from .trajectory import read_episodes

        report = decision_coverage(read_episodes(args.input))
        path = Path(args.output)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(
            json.dumps(
                {key: value for key, value in report.items() if key != "cards"},
                indent=2,
            )
        )

    coverage = commands.add_parser("coverage")
    coverage.add_argument("--input", action="append", required=True)
    coverage.add_argument("--output", required=True)
    coverage.set_defaults(function=coverage_report)

    collect = commands.add_parser("collect-bc")
    collect.add_argument("--episodes", type=int, default=1000)
    collect.add_argument(
        "--teacher-checkpoint",
        help="Visible-information model demonstrator; never a PPO behavior policy",
    )
    collect.add_argument("--curated-probability", type=float, default=1.0)
    collect.add_argument("--perturb-probability", type=float, default=0.0)
    collect.add_argument("--output", required=True)
    collect.set_defaults(function=command_collect_bc)

    split = commands.add_parser("split-decks")
    split.add_argument("--output-dir", required=True)
    split.add_argument(
        "--include-complex",
        action="store_true",
        help="Also include combo/OTK decks in the split",
    )
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
    bc.add_argument("--replay-input", action="append", default=[])
    bc.add_argument("--replay-fraction", type=float, default=0.3)
    bc.add_argument("--validation-input", action="append", default=[])
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
    ppo.add_argument(
        "--ppo-temperature",
        type=float,
        default=1.0,
        help="Positive finite temperature shared by online actors and PPO likelihoods; greedy evaluation and auxiliary BC stay untempered",
    )
    ppo.add_argument("--ppo-clip", type=float, default=0.2)
    ppo.add_argument("--value-clip", type=float, default=0.2)
    ppo.add_argument("--value-coefficient", type=float, default=0.5)
    ppo.add_argument("--entropy-coefficient", type=float, default=0.01)
    ppo.add_argument("--gamma", type=float, default=1.0)
    ppo.add_argument("--gae-lambda", type=float, default=0.98)
    ppo.add_argument("--reference-kl-coefficient", type=float, default=0.0)
    ppo.add_argument("--bc-input", action="append", default=[])
    ppo.add_argument("--ppo-bc-coefficient", type=float, default=0.0)
    ppo.add_argument("--checkpoint-every", type=int, default=10)
    ppo.add_argument("--league-snapshot-every", type=int, default=25)
    ppo.add_argument("--specialist-probability", type=float, default=0.1)
    ppo.add_argument("--curated-probability", type=float, default=0.8)
    ppo.add_argument("--perturb-probability", type=float, default=0.15)
    ppo.set_defaults(function=command_train_ppo)

    evaluate = commands.add_parser("evaluate")
    evaluate.add_argument(
        "--mirror",
        action="store_true",
        help="Use identical decks on both sides and two games per seed, swapping model seats",
    )
    evaluate.add_argument("--checkpoint", required=True)
    evaluate.add_argument("--opponent-checkpoint")
    evaluate.add_argument("--matches", type=int, default=100)
    evaluate.add_argument("--output")
    evaluate.add_argument(
        "--match-list", help="Replay a saved .matches.json evaluation schedule"
    )
    evaluate.add_argument(
        "--balanced", action="store_true", help="Cycle through every supplied deck"
    )
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
    play_model.add_argument(
        "--locale", choices=("enUS", "zhCN", "zhTW"), default="zhCN"
    )
    play_model.set_defaults(function=command_play_model)

    smoke = commands.add_parser("smoke")
    smoke.set_defaults(function=command_smoke)

    return root


def main() -> None:
    args = parser().parse_args()
    if args.torch_threads < 1:
        raise ValueError("--torch-threads must be positive")
    torch.set_num_threads(args.torch_threads)
    args.excluded_cards = []
    if args.card_split:
        split = json.loads(Path(args.card_split).read_text(encoding="utf-8"))
        if args.command in {
            "train-bc",
            "train-ppo",
            "train-dmc",
            "collect-bc",
        }:
            args.excluded_cards = split["held_out_cards"]
    if args.deck_manifest:
        if args.deck or not args.split:
            raise ValueError(
                "--deck-manifest requires --split and cannot be mixed with --deck"
            )
        if (
            args.command in {"train-bc", "train-ppo", "train-dmc", "collect-bc"}
            and args.split != "train"
        ):
            raise ValueError("training commands require the train split")
        args.deck = load_split_paths(args.deck_manifest, args.split)
    elif args.split:
        raise ValueError("--split requires --deck-manifest")
    if not args.deck:
        args.deck = _default_deck_paths()
    if args.excluded_cards:
        args.deck = [
            path
            for path in args.deck
            if not set(Deck.from_file(path).cards).intersection(args.excluded_cards)
        ]
        if not args.deck:
            raise ValueError("no training decks remain after held-out card exclusion")
    args.function(args)


if __name__ == "__main__":
    main()
