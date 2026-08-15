from __future__ import annotations

import random
from pathlib import Path


class CheckpointLeague:
    def __init__(
        self, directory: str | Path, *, seed: int = 0, current_probability: float = 0.7
    ) -> None:
        self.directory = Path(directory)
        self.directory.mkdir(parents=True, exist_ok=True)
        self.rng = random.Random(seed)
        self.current_probability = current_probability

    def snapshots(self) -> list[Path]:
        return sorted(self.directory.glob("snapshot-*.pt"))

    def sample(self, current: str | Path) -> Path:
        snapshots = self.snapshots()
        if not snapshots or self.rng.random() < self.current_probability:
            return Path(current)
        # Bias toward recent opponents without forgetting early strategies.
        weights = list(range(1, len(snapshots) + 1))
        return self.rng.choices(snapshots, weights=weights, k=1)[0]
