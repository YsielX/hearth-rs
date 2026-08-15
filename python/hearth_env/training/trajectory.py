from __future__ import annotations

import gzip
import json
import random
from collections import deque
from collections.abc import Iterable, Iterator, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass
class TrainingSample:
    decision: dict[str, Any]
    self_deck: list[str]
    action_index: int
    target: float
    weight: float = 1.0


def write_episodes(path: str | Path, episodes: Iterable[dict[str, Any]]) -> int:
    """Append newline-delimited episodes to a gzip shard."""

    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with gzip.open(path, "at", encoding="utf-8") as output:
        for episode in episodes:
            output.write(json.dumps(episode, separators=(",", ":")))
            output.write("\n")
            count += 1
    return count


def read_episodes(paths: Sequence[str | Path]) -> Iterator[dict[str, Any]]:
    for value in paths:
        path = Path(value)
        opener = gzip.open if path.suffix == ".gz" else open
        with opener(path, "rt", encoding="utf-8") as source:
            for line in source:
                if line.strip():
                    yield json.loads(line)


def episode_samples(
    episode: dict[str, Any],
    *,
    behavior_clone: bool = False,
) -> Iterator[TrainingSample]:
    if episode.get("truncated") and not behavior_clone:
        return
    decks = episode["match_config"]["decks"]
    rewards = episode.get("rewards", [0.0, 0.0])
    for step in episode.get("steps", []):
        seat = int(step["decision"]["actor_seat"])
        yield TrainingSample(
            decision=step["decision"],
            self_deck=list(decks[seat]),
            action_index=int(step["action_index"]),
            target=float(rewards[seat]),
            weight=float(step.get("weight", 1.0)),
        )


def stream_samples(
    paths: Sequence[str | Path],
    *,
    behavior_clone: bool = False,
    shuffle_buffer: int = 4096,
    seed: int = 0,
) -> Iterator[TrainingSample]:
    rng = random.Random(seed)
    buffer: list[TrainingSample] = []
    for episode in read_episodes(paths):
        for sample in episode_samples(episode, behavior_clone=behavior_clone):
            buffer.append(sample)
            if len(buffer) >= shuffle_buffer:
                index = rng.randrange(len(buffer))
                yield buffer.pop(index)
    rng.shuffle(buffer)
    yield from buffer


class ReplayBuffer:
    def __init__(self, capacity: int, seed: int = 0) -> None:
        self._items: deque[TrainingSample] = deque(maxlen=capacity)
        self._rng = random.Random(seed)

    def extend_episode(self, episode: dict[str, Any]) -> int:
        before = len(self._items)
        self._items.extend(episode_samples(episode))
        return len(self._items) - before

    def extend(self, values: Iterable[TrainingSample]) -> None:
        self._items.extend(values)

    def sample(self, count: int) -> list[TrainingSample]:
        if count > len(self._items):
            raise ValueError(
                f"requested {count} samples from replay of size {len(self._items)}"
            )
        return self._rng.sample(list(self._items), count)

    def __len__(self) -> int:
        return len(self._items)
