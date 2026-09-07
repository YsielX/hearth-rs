"""Additional public information for the semantic architecture (tensor v3)."""

from __future__ import annotations

from collections import Counter
import math
from typing import Any

import torch

from .semantics import (
    CLASSES,
    EVENT_INDEX,
    HISTORY_GROUPS,
    KEYWORDS,
    KINDS,
    ROLE_INDEX,
    ZONES,
    keyword_features,
)


def augment(
    tensorizer: Any,
    decision: dict[str, Any],
    self_deck: Any,
    out: dict[str, torch.Tensor],
) -> None:
    cfg, catalog = tensorizer.config, tensorizer.catalog
    obs = decision["observation"]
    entities = obs.get("entities", [])
    if len(entities) > cfg.max_entities or len(self_deck) > cfg.max_deck_cards:
        raise ValueError(
            "visible entity/deck capacity exceeded; increase model configuration"
        )
    positions = {entity["entity"]: i for i, entity in enumerate(entities)}
    out["entity_keywords"] = torch.zeros(cfg.max_entities, len(KEYWORDS))
    out["entity_kinds"] = torch.zeros(cfg.max_entities, len(KINDS))
    out["player_classes"] = torch.zeros(2, len(CLASSES))
    for i, entity in enumerate(entities):
        if len(entity.get("public_cards", [])) > cfg.max_entity_cards:
            raise ValueError("public card component capacity exceeded")
        out["entity_keywords"][i] = torch.tensor(
            keyword_features(entity.get("keywords", []))
        )
        out["entity_kinds"][i] = torch.tensor(
            [float(entity.get("kind") == kind) for kind in KINDS]
        )
    # Historical architecture used hashed keywords and categorical scalar IDs.
    out["entity_state"][:, 14] = 0
    out["entity_state"][:, 17:] = 0
    out["global_state"][18] = 0
    out["global_state"][32] = 0

    memory: list[tuple[int, int, float]] = []
    facts: list[tuple[str, float, float, int]] = []
    for seat, key in enumerate(("self_player", "opponent")):
        player = obs[key]
        sign = 1.0 if seat == 0 else -1.0
        out["player_classes"][seat] = torch.tensor(
            [float(player.get("class") == name) for name in CLASSES]
        )
        for group, name in enumerate(HISTORY_GROUPS):
            for card_id, count in sorted(
                Counter(player.get("history", {}).get(name, [])).items()
            ):
                memory.append(
                    (catalog.index(card_id), seat * len(HISTORY_GROUPS) + group, count)
                )
        for name in ("resources", "resources_spent", "public_counters"):
            for label, value in sorted(player.get(name, {}).items()):
                facts.append((f"{name}:{label}", float(value), sign, -1))
        for label in sorted(player.get("public_statuses", [])):
            facts.append((f"status:{label}", 1.0, sign, -1))
    for i, entity in enumerate(entities):
        for key, value in sorted(entity.get("public_counters", {}).items()):
            facts.append((f"entity:{key}", float(value), 0.0, i))
    if len(memory) > cfg.max_memory or len(facts) > cfg.max_facts:
        raise ValueError(
            "public memory/fact capacity exceeded; no silent history truncation"
        )
    out["memory_cards"] = torch.zeros(cfg.max_memory, dtype=torch.long)
    out["memory_groups"] = torch.zeros(cfg.max_memory, dtype=torch.long)
    out["memory_counts"] = torch.zeros(cfg.max_memory)
    out["memory_mask"] = torch.zeros(cfg.max_memory, dtype=torch.bool)
    for i, (card, group, count) in enumerate(memory):
        out["memory_cards"][i] = card
        out["memory_groups"][i] = group
        out["memory_counts"][i] = count
        out["memory_mask"][i] = True
    out["fact_bytes"] = torch.zeros(cfg.max_facts, 96, dtype=torch.long)
    out["fact_values"] = torch.zeros(cfg.max_facts, 3)
    out["fact_entities"] = torch.full((cfg.max_facts,), -1, dtype=torch.long)
    out["fact_mask"] = torch.zeros(cfg.max_facts, dtype=torch.bool)
    for i, (label, value, seat, entity) in enumerate(facts):
        encoded = list(label.encode("utf-8"))
        if len(encoded) > 96:
            raise ValueError(f"public fact label too long: {label}")
        out["fact_bytes"][i, : len(encoded)] = torch.tensor(
            [byte + 1 for byte in encoded]
        )
        out["fact_values"][i] = torch.tensor(
            [value / 20, math.copysign(math.log1p(abs(value)), value) / 5, seat]
        )
        out["fact_entities"][i] = entity
        out["fact_mask"][i] = True

    out["history_roles"] = torch.zeros(
        cfg.max_history, cfg.max_history_entities, dtype=torch.long
    )
    out["history_refs"] = torch.full(
        (cfg.max_history, cfg.max_history_entities), -1, dtype=torch.long
    )
    out["history_entity_values"] = torch.zeros(
        cfg.max_history, cfg.max_history_entities, 1
    )
    out["history_keywords"] = torch.zeros(cfg.max_history, len(KEYWORDS))
    out["history_zones"] = torch.zeros(cfg.max_history, 2, dtype=torch.long)
    out["history_cards"].zero_()
    out["history_card_mask"].zero_()
    for i, record in enumerate(
        obs.get("history", {}).get("events", [])[-cfg.max_history :]
    ):
        event = record.get("event", {})
        kind = event.get("kind")
        if kind not in EVENT_INDEX:
            raise ValueError(f"unknown public event kind: {kind}")
        out["history_kinds"][i] = EVENT_INDEX[kind]
        participants = list(event.get("entities", []))
        for key, role in (("from_card_id", "old"), ("to_card_id", "new")):
            card = event.get(key)
            if card and not any(
                p.get("card_id") == card and p.get("role") == role for p in participants
            ):
                participants.append({"card_id": card, "role": role})
        if len(participants) > cfg.max_history_entities:
            raise ValueError(
                "event participant capacity exceeded; increase max_history_entities"
            )
        for j, participant in enumerate(participants):
            out["history_cards"][i, j] = catalog.index(participant.get("card_id"))
            out["history_card_mask"][i, j] = True
            out["history_roles"][i, j] = ROLE_INDEX.get(participant.get("role"), 0)
            out["history_refs"][i, j] = positions.get(participant.get("entity"), -1)
            out["history_entity_values"][i, j, 0] = (
                float(participant.get("value") or 0) / 20
            )
        if event.get("keyword"):
            out["history_keywords"][i] = torch.tensor(
                keyword_features([event["keyword"]])
            )
        for j, name in enumerate(("from_zone", "to_zone")):
            if event.get(name) in ZONES:
                out["history_zones"][i, j] = ZONES.index(event[name]) + 1
    # Array index / localized labels must not teach option-position shortcuts.
    out["action_numeric"][:, 2] = 0
    out["action_numeric"][:, 6:] = 0
    for action in decision.get("actions", []):
        if len(action.get("sources", [])) > cfg.max_action_sources:
            raise ValueError("action source capacity exceeded")
