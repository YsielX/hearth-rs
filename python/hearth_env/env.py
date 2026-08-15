"""Small Python facade over the Rust environment transport.

This module deliberately has no dependency on Gymnasium, Torch, or a learning
algorithm. Adapters for those frameworks can consume the returned dictionaries.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from copy import deepcopy
from pathlib import Path
from typing import Any

from ._native import HearthEnv as _NativeHearthEnv


class HearthEnv:
    """Turn-based, two-player environment with indexed legal actions."""

    def __init__(
        self,
        data_path: str | Path,
        match_config: Mapping[str, Any],
        *,
        seed: int = 0,
        max_steps: int = 1000,
        history_limit: int | None = None,
    ) -> None:
        self._native = _NativeHearthEnv(
            str(data_path),
            json.dumps(dict(match_config), separators=(",", ":")),
            seed,
            max_steps,
            history_limit,
        )
        self._match_config = deepcopy(dict(match_config))
        raw = self._native.decision_json()
        self._decision: dict[str, Any] | None = json.loads(raw) if raw else None

    @property
    def decision(self) -> dict[str, Any] | None:
        return self._decision

    @property
    def pack_hash(self) -> str:
        return self._native.pack_hash()

    @property
    def card_ids(self) -> list[str]:
        return self._native.card_ids()

    @property
    def card_catalog(self) -> list[dict[str, Any]]:
        """Operator-only definitions and Lua source used by feature builders."""

        return json.loads(self._native.card_catalog_json())

    @property
    def match_config(self) -> dict[str, Any]:
        return deepcopy(self._match_config)

    def reset(
        self,
        *,
        seed: int,
        match_config: Mapping[str, Any] | None = None,
    ) -> dict[str, Any]:
        if match_config is None:
            raw = self._native.reset_json(seed)
        else:
            next_config = deepcopy(dict(match_config))
            raw = self._native.reset_match_json(
                json.dumps(next_config, separators=(",", ":")), seed
            )
            self._match_config = next_config
        self._decision = json.loads(raw)
        return self._decision

    def step(self, action_index: int) -> dict[str, Any]:
        if self._decision is None:
            raise RuntimeError("the episode has ended; call reset()")
        transition = json.loads(
            self._native.step_json(self._decision["id"], action_index)
        )
        self._decision = transition["next"]
        return transition

    def replay(self) -> dict[str, Any]:
        """Return operator-only reproducibility data, never a policy observation."""

        return json.loads(self._native.replay_json())
