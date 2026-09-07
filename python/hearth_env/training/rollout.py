from __future__ import annotations

import multiprocessing as mp
import os
import json
import traceback
from collections.abc import Iterator, Sequence
from concurrent.futures import FIRST_COMPLETED, ProcessPoolExecutor, wait
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from typing_extensions import Self

from hearth_env import HearthEnv

from .catalog import CardCatalog
from .checkpoint import load_checkpoint
from .policies import HeuristicPolicy, ModelPolicy, Policy, RandomPolicy
from .tensorize import Tensorizer


@dataclass(frozen=True)
class RolloutJob:
    match_config: dict[str, Any]
    seed: int
    policies: tuple[dict[str, Any], dict[str, Any]]
    capture_replay: bool = False


def play_episode(
    env: HearthEnv,
    policies: Sequence[Policy],
    match_config: dict[str, Any],
    seed: int,
    *,
    capture_replay: bool = False,
) -> dict[str, Any]:
    for policy in policies:
        if isinstance(policy, HeuristicPolicy):
            policy.env = env
    steps: list[dict[str, Any]] = []
    decision: dict[str, Any] | None = None
    action_index: int | None = None
    try:
        decision = env.reset(seed=seed, match_config=match_config)
        while decision is not None:
            seat = int(decision["actor_seat"])
            action_index = policies[seat].choose(decision, match_config["decks"][seat])
            step = {"decision": decision, "action_index": action_index}
            behavior = getattr(policies[seat], "last_behavior", None)
            if behavior is not None:
                step["behavior"] = dict(behavior)
            steps.append(step)
            transition = env.step(action_index)
            decision = transition["next"]
    except Exception as error:
        try:
            replay = env.replay()
        except Exception as replay_error:
            replay = {"capture_error": repr(replay_error)}
        return {
            "format_version": 1,
            "observation_schema_version": env.observation_schema_version,
            "engine_build": env.engine_build,
            "pack_hash": env.pack_hash,
            "seed": seed,
            "match_config": match_config,
            "steps": steps,
            "rewards": [0.0, 0.0],
            "outcome": None,
            "terminated": False,
            "truncated": True,
            "error": {
                "type": type(error).__name__,
                "message": str(error),
                "traceback": traceback.format_exc(),
                "last_observation": decision,
                "last_action_index": action_index,
                "replay": replay,
            },
        }
    episode = {
        "format_version": 1,
        "observation_schema_version": env.observation_schema_version,
        "engine_build": env.engine_build,
        "pack_hash": env.pack_hash,
        "seed": seed,
        "match_config": match_config,
        "steps": steps,
        "rewards": transition["rewards"],
        "outcome": transition.get("outcome"),
        "terminated": transition["terminated"],
        "truncated": transition["truncated"],
    }
    if capture_replay:
        episode["replay"] = env.replay()
    return episode


_WORKER_ENV: HearthEnv | None = None
_WORKER_CATALOG: CardCatalog | None = None
_WORKER_MODELS: dict[tuple[str, int], tuple[Any, Tensorizer]] = {}


def _worker_init(
    data_path: str,
    base_match_config: dict[str, Any],
    max_steps: int,
    history_limit: int | None,
    torch_threads: int,
    card_hash_dim: int,
) -> None:
    global _WORKER_ENV, _WORKER_CATALOG
    if torch_threads > 0:
        try:
            import torch

            torch.set_num_threads(torch_threads)
        except ImportError:
            pass
    _WORKER_ENV = HearthEnv(
        data_path,
        base_match_config,
        seed=os.getpid(),
        max_steps=max_steps,
        history_limit=history_limit,
    )
    _WORKER_CATALOG = CardCatalog.build(
        _WORKER_ENV.card_catalog, _WORKER_ENV.pack_hash, hash_dim=card_hash_dim
    )


def _policy(spec: dict[str, Any], seed: int) -> Policy:
    seed = int(spec.get("policy_seed", seed))
    kind = spec.get("kind", "heuristic")
    if kind == "heuristic":
        return HeuristicPolicy()
    if kind == "random":
        return RandomPolicy(seed)
    if kind != "model":
        raise ValueError(f"unknown policy kind {kind}")
    if _WORKER_CATALOG is None:
        raise RuntimeError("rollout worker is not initialized")
    path = str(spec["checkpoint"])
    modified = Path(path).stat().st_mtime_ns
    key = (path, modified)
    cached = _WORKER_MODELS.get(key)
    if cached is None:
        model, _ = load_checkpoint(path, _WORKER_CATALOG, device="cpu")
        tensorizer = Tensorizer(_WORKER_CATALOG, model.config)
        cached = (model, tensorizer)
        if len(_WORKER_MODELS) >= 4:
            _WORKER_MODELS.pop(next(iter(_WORKER_MODELS)))
        _WORKER_MODELS[key] = cached
    model, tensorizer = cached
    return ModelPolicy(
        model,
        tensorizer,
        device="cpu",
        epsilon=float(spec.get("epsilon", 0.0)),
        seed=seed,
        sample=bool(spec.get("sample", False)),
        temperature=float(spec.get("temperature", 1.0)),
    )


def _worker_play(job: RolloutJob) -> dict[str, Any]:
    if _WORKER_ENV is None:
        raise RuntimeError("rollout worker is not initialized")
    policies = [
        _policy(spec, job.seed ^ (seat << 32)) for seat, spec in enumerate(job.policies)
    ]
    episode = play_episode(
        _WORKER_ENV,
        policies,
        job.match_config,
        job.seed,
        capture_replay=job.capture_replay,
    )
    if episode.get("error"):
        episode["error"]["policies"] = job.policies
    return episode


class ParallelCollector:
    """Persistent OS workers; each worker owns and reuses one Lua runtime."""

    def __init__(
        self,
        data_path: str | Path,
        base_match_config: dict[str, Any],
        *,
        workers: int,
        max_steps: int = 1000,
        history_limit: int | None = 96,
        torch_threads: int = 1,
        card_hash_dim: int = 256,
        failure_dir: str | Path | None = None,
        max_failures: int = 0,
    ) -> None:
        if workers < 1:
            raise ValueError("workers must be positive")
        self.workers = workers
        self.failure_dir = Path(failure_dir) if failure_dir else None
        self.max_failures = max_failures
        self.failures = 0
        self.executor = ProcessPoolExecutor(
            max_workers=workers,
            mp_context=mp.get_context("spawn"),
            initializer=_worker_init,
            initargs=(
                str(data_path),
                base_match_config,
                max_steps,
                history_limit,
                torch_threads,
                card_hash_dim,
            ),
        )

    def collect(
        self, jobs: Sequence[RolloutJob], *, progress_every: int = 0
    ) -> list[dict[str, Any]]:
        # Restore job order only after collection; slow jobs never prevent
        # replacement work from being submitted to free workers.
        ordered: list[dict[str, Any] | None] = [None] * len(jobs)
        for index, episode in self.iter_indexed(jobs, progress_every=progress_every):
            ordered[index] = episode
        if any(episode is None for episode in ordered):
            raise RuntimeError("collector lost a rollout result")
        return [episode for episode in ordered if episode is not None]

    def iter_collect(
        self, jobs: Sequence[RolloutJob], *, progress_every: int = 0
    ) -> Iterator[dict[str, Any]]:
        """Stream completed episodes immediately; order is unspecified."""
        for _, episode in self.iter_indexed(jobs, progress_every=progress_every):
            yield episode

    def iter_indexed(
        self, jobs: Sequence[RolloutJob], *, progress_every: int = 0
    ) -> Iterator[tuple[int, dict[str, Any]]]:
        pending = iter(enumerate(jobs))
        futures: dict[Any, tuple[int, RolloutJob]] = {}

        def submit() -> None:
            try:
                index, job = next(pending)
            except StopIteration:
                return
            futures[self.executor.submit(_worker_play, job)] = (index, job)

        for _ in range(min(len(jobs), self.workers * 2)):
            submit()
        completed = 0
        while futures:
            done, _ = wait(futures, return_when=FIRST_COMPLETED)
            for future in done:
                index, job = futures.pop(future)
                try:
                    episode = future.result()
                except Exception as error:
                    raise RuntimeError(
                        f"rollout failed: seed={job.seed}, job_index={index}"
                    ) from error
                submit()
                completed += 1
                if progress_every > 0 and (
                    completed % progress_every == 0 or completed == len(jobs)
                ):
                    print(f"rollout progress={completed}/{len(jobs)}", flush=True)
                if episode.get("error"):
                    self.failures += 1
                    if self.failure_dir is not None:
                        self.failure_dir.mkdir(parents=True, exist_ok=True)
                        path = (
                            self.failure_dir
                            / f"failure-{episode['seed']}-{index:06d}.json"
                        )
                        path.write_text(json.dumps(episode, indent=2), encoding="utf-8")
                    if self.failures > self.max_failures:
                        raise RuntimeError(
                            f"rollout failure threshold exceeded: {self.failures}; reproduction under {self.failure_dir}"
                        )
                yield index, episode

    def close(self) -> None:
        self.executor.shutdown(wait=True, cancel_futures=True)

    def __enter__(self) -> Self:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()
