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
    steps: list[dict[str, Any]] = []
    decision: dict[str, Any] | None = None
    action_index: int | None = None
    try:
        decision = env.reset(seed=seed, match_config=match_config)
        while decision is not None:
            seat = int(decision["actor_seat"])
            action_index = policies[seat].choose(
                decision, match_config["decks"][seat]
            )
            steps.append({"decision": decision, "action_index": action_index})
            transition = env.step(action_index)
            decision = transition["next"]
    except Exception as error:
        try:
            replay = env.replay()
        except Exception as replay_error:
            replay = {"capture_error": repr(replay_error)}
        return {
            "format_version": 1,
            "observation_schema_version": 3,
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
        "observation_schema_version": 3,
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
    kind = spec.get("kind", "heuristic")
    if kind == "heuristic":
        return HeuristicPolicy(seed, float(spec.get("noise", 0.08)))
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
        return list(self.iter_collect(jobs, progress_every=progress_every))

    def iter_collect(
        self, jobs: Sequence[RolloutJob], *, progress_every: int = 0
    ) -> Iterator[dict[str, Any]]:
        """Yield episodes in job order with bounded in-flight results."""

        pending_jobs = iter(enumerate(jobs))
        futures: dict[Any, tuple[int, RolloutJob]] = {}
        ready: dict[int, dict[str, Any]] = {}
        in_flight_limit = self.workers * 2

        def submit_next() -> bool:
            try:
                index, job = next(pending_jobs)
            except StopIteration:
                return False
            futures[self.executor.submit(_worker_play, job)] = (index, job)
            return True

        for _ in range(min(len(jobs), in_flight_limit)):
            submit_next()

        completed = 0
        next_index = 0
        while futures or ready:
            if next_index in ready:
                episode = ready.pop(next_index)
                next_index += 1
                completed += 1
                while len(futures) + len(ready) < in_flight_limit and submit_next():
                    pass
                if progress_every > 0 and (
                    completed % progress_every == 0 or completed == len(jobs)
                ):
                    print(f"rollout progress={completed}/{len(jobs)}", flush=True)
                if episode.get("error"):
                    self.failures += 1
                    if self.failure_dir is not None:
                        self.failure_dir.mkdir(parents=True, exist_ok=True)
                        path = self.failure_dir / (
                            f"failure-{episode['seed']}-{next_index - 1:06d}.json"
                        )
                        path.write_text(
                            json.dumps(episode, indent=2), encoding="utf-8"
                        )
                    if self.failures > self.max_failures:
                        raise RuntimeError(
                            f"rollout failure threshold exceeded: {self.failures} > "
                            f"{self.max_failures}; reproduction saved under "
                            f"{self.failure_dir}"
                        )
                yield episode
                continue

            done, _ = wait(futures, return_when=FIRST_COMPLETED)
            for future in done:
                index, job = futures.pop(future)
                try:
                    ready[index] = future.result()
                except Exception as error:
                    classes = job.match_config.get("classes", ["?", "?"])
                    raise RuntimeError(
                        f"rollout failed: seed={job.seed}, classes={classes}, "
                        f"job_index={index}"
                    ) from error
            while len(futures) + len(ready) < in_flight_limit and submit_next():
                pass

    def close(self) -> None:
        self.executor.shutdown(wait=True, cancel_futures=True)

    def __enter__(self) -> Self:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()
