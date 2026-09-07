from __future__ import annotations

import random
import hashlib
from pathlib import Path


class CheckpointLeague:
    def __init__(
        self, directory: str | Path, *, seed: int = 0, current_probability: float = 0.7
    ) -> None:
        self.directory = Path(directory)
        self.directory.mkdir(parents=True, exist_ok=True)
        self.rng = random.Random(seed)
        self.current_probability = current_probability
        self._frozen_snapshots: list[Path] | None = None

    def snapshots(self) -> list[Path]:
        return (
            list(self._frozen_snapshots)
            if self._frozen_snapshots is not None
            else sorted(self.directory.glob("snapshot-*.pt"))
        )

    def freeze(self, state: list[dict[str, str]] | None = None) -> None:
        if state is None:
            self._frozen_snapshots = self.snapshots()
        else:
            paths = []
            for item in state:
                path = self.directory / item["name"]
                if (
                    not path.is_file()
                    or hashlib.sha256(path.read_bytes()).hexdigest() != item["sha256"]
                ):
                    raise ValueError(
                        f"frozen league opponent missing or changed: {path}"
                    )
                paths.append(path)
            self._frozen_snapshots = paths

    def register(self, path: Path) -> None:
        if self._frozen_snapshots is not None and path not in self._frozen_snapshots:
            self._frozen_snapshots.append(path)
            self._frozen_snapshots.sort()

    def state(self) -> list[dict[str, str]]:
        return [
            {"name": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
            for path in self.snapshots()
        ]

    def sample(self, current: str | Path) -> Path:
        snapshots = self.snapshots()
        if not snapshots or self.rng.random() < self.current_probability:
            return Path(current)
        # Bias toward recent opponents without forgetting early strategies.
        weights = list(range(1, len(snapshots) + 1))
        return self.rng.choices(snapshots, weights=weights, k=1)[0]
