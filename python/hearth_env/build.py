"""Detect a stale editable native extension before generating training data."""

from functools import lru_cache
from pathlib import Path


@lru_cache(maxsize=1)
def source_fingerprint() -> str | None:
    root = Path(__file__).resolve().parents[2]
    if not (root / "crates/hearth-env-py/build.rs").exists():
        return None  # Installed wheel, without a source checkout.
    files = [
        root / "Cargo.lock",
        root / "Cargo.toml",
        root / "crates/hearth-env-py/build.rs",
    ]
    for name in ("hearth-core", "hearth-bot", "hearth-script", "hearth-env", "hearth-env-py"):
        directory = root / "crates" / name
        files.append(directory / "Cargo.toml")
        files.extend((directory / "src").rglob("*.rs"))
    value = 14695981039346656037
    for path in sorted(files):
        for byte in (
            path.relative_to(root).as_posix().encode()
            + b"\0"
            + path.read_bytes()
            + b"\0"
        ):
            value = ((value ^ byte) * 1099511628211) & ((1 << 64) - 1)
    return f"{value:016x}"
