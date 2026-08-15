from __future__ import annotations

import hashlib
from collections.abc import Sequence
from typing import Any

import torch

from .catalog import CardCatalog
from .config import ModelConfig

ACTION_KINDS = {
    name: index
    for index, name in enumerate(
        [
            "mulligan",
            "play_card",
            "play_card_at",
            "trade_card",
            "use_card_action",
            "attack",
            "use_hero_power",
            "use_location",
            "end_turn",
            "concede",
            "choose",
        ]
    )
}
AREAS = {
    name: index
    for index, name in enumerate(
        ["hero", "hero_power", "weapon", "board", "hand", "secret", "public_objective"]
    )
}
TENSOR_SCHEMA_VERSION = 1


def _stable_bucket(value: str | None, size: int) -> int:
    if not value:
        return 0
    return 1 + int.from_bytes(
        hashlib.blake2b(value.encode(), digest_size=4).digest(), "little"
    ) % (size - 1)


def _relative(value: str | None) -> float:
    if value == "self_player":
        return 1.0
    if value == "opponent":
        return -1.0
    return 0.0


class Tensorizer:
    def __init__(self, catalog: CardCatalog, config: ModelConfig) -> None:
        self.catalog = catalog
        self.config = config

    def encode(
        self,
        decision: dict[str, Any],
        self_deck: Sequence[str],
    ) -> dict[str, torch.Tensor]:
        obs = decision["observation"]
        entities = obs.get("entities", [])[: self.config.max_entities]
        ref_to_position = {entity["entity"]: i for i, entity in enumerate(entities)}

        entity_cards = torch.zeros(self.config.max_entities, dtype=torch.long)
        entity_state = torch.zeros(
            self.config.max_entities, self.config.entity_state_dim, dtype=torch.float32
        )
        entity_mask = torch.zeros(self.config.max_entities, dtype=torch.bool)
        for i, entity in enumerate(entities):
            entity_cards[i] = self.catalog.index(entity.get("card_id"))
            values = [
                float(entity.get("attack", 0)) / 20,
                float(entity.get("max_health", 0)) / 20,
                float(entity.get("damage", 0)) / 20,
                float(entity.get("armor", 0)) / 20,
                float(entity.get("cost", 0)) / 10,
                float(entity.get("spell_damage", 0)) / 10,
                float(entity.get("position", 0)) / 10,
                float(entity.get("attacks_this_turn", 0)) / 4,
                float(entity.get("location_cooldown", 0)) / 4,
                float(bool(entity.get("exhausted"))),
                float(bool(entity.get("frozen"))),
                float(bool(entity.get("silenced"))),
                _relative(entity.get("owner")),
                _relative(entity.get("controller")),
                float(_stable_bucket(str(entity.get("kind", "")), 16)) / 15,
                float(AREAS.get(entity.get("area", ""), 0)) / max(len(AREAS) - 1, 1),
                min(len(entity.get("keywords", [])), 10) / 10,
            ]
            limit = min(len(values), self.config.entity_state_dim)
            entity_state[i, :limit] = torch.tensor(values[:limit])
            for keyword in entity.get("keywords", []):
                slot = 17 + _stable_bucket(keyword, self.config.entity_state_dim - 17)
                if slot < self.config.entity_state_dim:
                    entity_state[i, slot] = 1.0
            entity_mask[i] = True

        global_values: list[float] = [
            min(float(obs.get("turn", 0)), 60) / 60,
            _relative(obs.get("active_player")),
            float(obs.get("phase") == "mulligan"),
            float(obs.get("phase") == "choice"),
            float(obs.get("phase") == "main"),
        ]
        for key in ("self_player", "opponent"):
            player = obs[key]
            sign = 1.0 if key == "self_player" else -1.0
            global_values.extend(
                [
                    sign * float(player.get("deck_size", 0)) / 40,
                    sign * float(player.get("hand_size", 0)) / 10,
                    sign * float(player.get("mana", 0)) / 10,
                    sign * float(player.get("max_mana", 0)) / 10,
                    sign * float(player.get("temporary_mana", 0)) / 10,
                    sign * float(player.get("overload_pending", 0)) / 10,
                    sign * float(player.get("overloaded_mana", 0)) / 10,
                    sign * min(float(player.get("fatigue", 0)), 20) / 20,
                    sign * float(bool(player.get("hero_power_used"))),
                    sign * float(player.get("hero_power_uses_this_turn", 0)) / 4,
                    sign * min(float(player.get("cards_played_this_turn", 0)), 20) / 20,
                    sign * len(player.get("board", [])) / 7,
                    sign * float(player.get("secrets_count", 0)) / 5,
                    sign * float(_stable_bucket(player.get("class"), 32)) / 31,
                ]
            )
        global_state = torch.zeros(self.config.global_dim)
        global_state[: min(len(global_values), self.config.global_dim)] = torch.tensor(
            global_values[: self.config.global_dim]
        )

        deck_cards = torch.zeros(self.config.max_deck_cards, dtype=torch.long)
        deck_mask = torch.zeros(self.config.max_deck_cards, dtype=torch.bool)
        for i, card_id in enumerate(self_deck[: self.config.max_deck_cards]):
            deck_cards[i] = self.catalog.index(card_id)
            deck_mask[i] = True

        history = obs.get("history", {}).get("events", [])[-self.config.max_history :]
        history_kinds = torch.zeros(self.config.max_history, dtype=torch.long)
        history_cards = torch.zeros(
            self.config.max_history, self.config.max_history_entities, dtype=torch.long
        )
        history_card_mask = torch.zeros(
            self.config.max_history, self.config.max_history_entities, dtype=torch.bool
        )
        history_numeric = torch.zeros(
            self.config.max_history, self.config.history_numeric_dim
        )
        history_mask = torch.zeros(self.config.max_history, dtype=torch.bool)
        for i, record in enumerate(history):
            event = record.get("event", {})
            event_entities = event.get("entities", [])
            history_kinds[i] = _stable_bucket(event.get("kind"), 256)
            card_ids = [entity.get("card_id") for entity in event_entities]
            card_ids.extend([event.get("from_card_id"), event.get("to_card_id")])
            for j, card_id in enumerate(
                [card_id for card_id in card_ids if card_id][
                    : self.config.max_history_entities
                ]
            ):
                history_cards[i, j] = self.catalog.index(card_id)
                history_card_mask[i, j] = True
            values = [
                min(float(record.get("turn", 0)), 60) / 60,
                min(float(record.get("cursor", 0)), 512) / 512,
                i / max(len(history) - 1, 1),
                _relative(event.get("player")),
                _relative(event.get("from_player")),
                _relative(event.get("to_player")),
                float(event.get("amount", 0) or 0) / 20,
                float(event.get("cost", 0) or 0) / 10,
                float(event.get("pending", 0) or 0) / 10,
                float(event.get("locked", 0) or 0) / 10,
                float(event.get("temporary", 0) or 0) / 10,
                float(event.get("position", 0) or 0) / 10,
                float(event.get("option_count", 0) or 0) / 10,
                float(event.get("choice_index", 0) or 0) / 10,
            ]
            limit = min(len(values), self.config.history_numeric_dim)
            history_numeric[i, :limit] = torch.tensor(values[:limit])
            history_mask[i] = True
        if not history:
            # TransformerEncoder cannot consume a row that is entirely padding.
            history_mask[0] = True

        actions = decision.get("actions", [])
        count = len(actions)
        action_kinds = torch.zeros(count, dtype=torch.long)
        action_sources = torch.full(
            (count, self.config.max_action_sources), -1, dtype=torch.long
        )
        action_source_mask = torch.zeros(
            count, self.config.max_action_sources, dtype=torch.bool
        )
        action_targets = torch.full((count,), -1, dtype=torch.long)
        action_choice_cards = torch.full((count,), self.catalog.UNK, dtype=torch.long)
        action_numeric = torch.zeros(count, self.config.action_numeric_dim)
        choice_options = (obs.get("pending_choice") or {}).get("options", [])
        for i, action in enumerate(actions):
            action_kinds[i] = ACTION_KINDS.get(action.get("kind", ""), 0)
            for j, source in enumerate(
                action.get("sources", [])[: self.config.max_action_sources]
            ):
                if source in ref_to_position:
                    action_sources[i, j] = ref_to_position[source]
                    action_source_mask[i, j] = True
            target = action.get("target")
            if target in ref_to_position:
                action_targets[i] = ref_to_position[target]
            choice_index = action.get("choice_index")
            if choice_index is not None and choice_index < len(choice_options):
                option = choice_options[choice_index]
                value = option.get("value", {})
                action_choice_cards[i] = self.catalog.index(value.get("card_id"))
            else:
                option = {}
            values = [
                float(action.get("mana_cost", 0)) / 10,
                float(action.get("board_position", 0) or 0) / 7,
                float(choice_index or 0) / 10,
                len(action.get("sources", [])) / max(self.config.max_action_sources, 1),
                float(target is not None),
                float(action.get("card_action") is not None),
                float(_stable_bucket(action.get("card_action"), 32)) / 31,
                float(_stable_bucket(option.get("label"), 128)) / 127,
                float(
                    _stable_bucket((obs.get("pending_choice") or {}).get("prompt"), 128)
                )
                / 127,
            ]
            limit = min(len(values), self.config.action_numeric_dim)
            action_numeric[i, :limit] = torch.tensor(values[:limit])

        return {
            "global_state": global_state,
            "entity_cards": entity_cards,
            "entity_state": entity_state,
            "entity_mask": entity_mask,
            "deck_cards": deck_cards,
            "deck_mask": deck_mask,
            "history_kinds": history_kinds,
            "history_cards": history_cards,
            "history_card_mask": history_card_mask,
            "history_numeric": history_numeric,
            "history_mask": history_mask,
            "action_kinds": action_kinds,
            "action_sources": action_sources,
            "action_source_mask": action_source_mask,
            "action_targets": action_targets,
            "action_choice_cards": action_choice_cards,
            "action_numeric": action_numeric,
        }


def collate(samples: Sequence[dict[str, torch.Tensor]]) -> dict[str, torch.Tensor]:
    if not samples:
        raise ValueError("cannot collate an empty batch")
    max_actions = max(sample["action_kinds"].shape[0] for sample in samples)
    output: dict[str, torch.Tensor] = {}
    action_keys = {
        "action_kinds",
        "action_sources",
        "action_source_mask",
        "action_targets",
        "action_choice_cards",
        "action_numeric",
    }
    for key in samples[0]:
        if key not in action_keys:
            output[key] = torch.stack([sample[key] for sample in samples])
            continue
        first = samples[0][key]
        shape = (len(samples), max_actions, *first.shape[1:])
        fill = -1 if key in {"action_sources", "action_targets"} else 0
        padded = torch.full(shape, fill, dtype=first.dtype)
        for i, sample in enumerate(samples):
            length = sample[key].shape[0]
            padded[i, :length] = sample[key]
        output[key] = padded
    output["action_mask"] = torch.zeros(len(samples), max_actions, dtype=torch.bool)
    for i, sample in enumerate(samples):
        output["action_mask"][i, : sample["action_kinds"].shape[0]] = True
    return output


def move_batch(batch: dict[str, torch.Tensor], device: str) -> dict[str, torch.Tensor]:
    return {key: value.to(device, non_blocking=True) for key, value in batch.items()}
