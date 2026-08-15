from __future__ import annotations

import hashlib
import math
import re
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from functools import cached_property
from itertools import pairwise
from typing import Any

_TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|\d+(?:\.\d+)?|==|~=|<=|>=|\.\.|[-+*/%#<>]")
CARD_FEATURE_SCHEMA_VERSION = 1


def _hash_token(namespace: str, token: str, size: int) -> tuple[int, float]:
    digest = hashlib.blake2b(f"{namespace}\0{token}".encode(), digest_size=8).digest()
    value = int.from_bytes(digest, "little")
    return value % size, 1.0 if value & (1 << 63) else -1.0


def _hashed_bag(parts: Iterable[tuple[str, str]], size: int) -> list[float]:
    output = [0.0] * size
    count = 0
    for namespace, source in parts:
        tokens = [token.lower() for token in _TOKEN.findall(source)]
        for token in tokens:
            bucket, sign = _hash_token(namespace, token, size)
            output[bucket] += sign
            count += 1
        for left, right in pairwise(tokens):
            bucket, sign = _hash_token(namespace + ":bigram", left + "\0" + right, size)
            output[bucket] += 0.5 * sign
    if count:
        scale = 1.0 / math.sqrt(count)
        output = [value * scale for value in output]
    return output


@dataclass(frozen=True)
class CardCatalog:
    """Append-only IDs plus portable semantic features for every card.

    Index 0 is padding and index 1 is unknown. Stable IDs let an old checkpoint
    be expanded when a card pack gains cards, while unknown cards still receive
    useful definition/Lua features.
    """

    entries: dict[str, dict[str, Any]]
    card_ids: tuple[str, ...]
    pack_hash: str
    hash_dim: int = 256

    PAD = 0
    UNK = 1

    @classmethod
    def build(
        cls,
        entries: Sequence[dict[str, Any]],
        pack_hash: str,
        *,
        hash_dim: int = 256,
        prior_card_ids: Sequence[str] = (),
    ) -> CardCatalog:
        by_id = {entry["definition"]["id"]: dict(entry) for entry in entries}
        ordered: list[str] = []
        seen: set[str] = set()
        for card_id in prior_card_ids:
            if (
                card_id not in {"<pad>", "<unk>"}
                and card_id in by_id
                and card_id not in seen
            ):
                ordered.append(card_id)
                seen.add(card_id)
        for card_id in sorted(by_id):
            if card_id not in seen:
                ordered.append(card_id)
                seen.add(card_id)
        return cls(by_id, ("<pad>", "<unk>", *ordered), pack_hash, hash_dim)

    @cached_property
    def id_to_index(self) -> dict[str, int]:
        return {card_id: index for index, card_id in enumerate(self.card_ids)}

    @property
    def feature_dim(self) -> int:
        return 16 + self.hash_dim

    def index(self, card_id: str | None) -> int:
        if not card_id:
            return self.UNK
        return self.id_to_index.get(card_id, self.UNK)

    def feature(self, card_id: str) -> list[float]:
        entry = self.entries.get(card_id)
        if entry is None:
            return [0.0] * self.feature_dim
        definition = entry["definition"]
        kind = str(definition.get("kind", "")).lower()
        explicit = [
            float(definition.get("cost", 0)) / 10.0,
            float(definition.get("attack", 0)) / 20.0,
            float(definition.get("health", 0)) / 20.0,
            float(definition.get("armor", 0)) / 20.0,
            float(bool(definition.get("collectible", False))),
            float(bool(definition.get("secret", False))),
            float(kind == "minion"),
            float(kind == "spell"),
            float(kind == "weapon"),
            float(kind == "location"),
            float(kind == "hero"),
            float(kind in {"hero_power", "heropower"}),
            min(len(definition.get("keywords", [])), 10) / 10.0,
            min(len(definition.get("tags", [])), 10) / 10.0,
            min(len(definition.get("deck_allowances", [])), 4) / 4.0,
            1.0,
        ]
        structured = " ".join(
            [
                str(definition.get("name", "")),
                str(definition.get("text", "")),
                str(definition.get("class", "")),
                *map(str, definition.get("classes", [])),
                str(definition.get("set", "")),
                str(definition.get("rarity", "")),
                str(definition.get("spell_school", "")),
                *map(str, definition.get("keywords", [])),
                *map(str, definition.get("tags", [])),
                *map(str, definition.get("keyword_params", {}).keys()),
            ]
        )
        hashed = _hashed_bag(
            (("definition", structured), ("lua", str(entry.get("lua_source", "")))),
            self.hash_dim,
        )
        return explicit + hashed

    def feature_matrix(self) -> list[list[float]]:
        zeros = [0.0] * self.feature_dim
        return [zeros, zeros, *(self.feature(card_id) for card_id in self.card_ids[2:])]

    def manifest(self) -> dict[str, Any]:
        return {
            "feature_schema_version": CARD_FEATURE_SCHEMA_VERSION,
            "pack_hash": self.pack_hash,
            "hash_dim": self.hash_dim,
            "card_ids": list(self.card_ids),
        }
