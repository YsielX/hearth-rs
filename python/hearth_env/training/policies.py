from __future__ import annotations

import random
from collections.abc import Sequence
from typing import Any, Protocol

import torch

from .model import HearthQNetwork
from .tensorize import Tensorizer, collate, move_batch


class Policy(Protocol):
    def choose(self, decision: dict[str, Any], self_deck: Sequence[str]) -> int: ...


class RandomPolicy:
    def __init__(self, seed: int = 0) -> None:
        self.rng = random.Random(seed)

    def choose(self, decision: dict[str, Any], self_deck: Sequence[str]) -> int:
        candidates = [
            action["index"]
            for action in decision["actions"]
            if action["kind"] != "concede"
        ] or [action["index"] for action in decision["actions"]]
        return self.rng.choice(candidates)


class HeuristicPolicy:
    """A noisy, framework-external demonstrator used only to warm-start BC."""

    def __init__(self, seed: int = 0, noise: float = 0.08) -> None:
        self.rng = random.Random(seed)
        self.noise = noise

    def choose(self, decision: dict[str, Any], self_deck: Sequence[str]) -> int:
        obs = decision["observation"]
        entities = {entity["entity"]: entity for entity in obs["entities"]}
        scored: list[tuple[float, int]] = []
        for action in decision["actions"]:
            kind = action["kind"]
            score = {
                "attack": 8.0,
                "play_card": 6.0,
                "play_card_at": 6.2,
                "use_location": 5.5,
                "use_hero_power": 4.0,
                "use_card_action": 4.0,
                "trade_card": 2.0,
                "choose": 3.0,
                "end_turn": -1.0,
                "concede": -100.0,
                "mulligan": 0.0,
            }.get(kind, 0.0)
            sources = [entities.get(ref, {}) for ref in action.get("sources", [])]
            if kind == "mulligan":
                score = sum(
                    max(float(entity.get("cost", 0)) - 3.0, 0.0) for entity in sources
                )
                score -= sum(
                    max(3.0 - float(entity.get("cost", 0)), 0.0) for entity in sources
                )
            elif kind == "attack":
                target = entities.get(action.get("target"), {})
                source = sources[0] if sources else {}
                source_attack = float(source.get("attack", 0))
                source_health = max(
                    float(source.get("max_health", 0))
                    - float(source.get("damage", 0)),
                    0.0,
                )
                if target.get("area") == "hero":
                    target_health = max(
                        float(target.get("max_health", 0))
                        - float(target.get("damage", 0))
                        + float(target.get("armor", 0)),
                        0.0,
                    )
                    if source_attack >= target_health:
                        # Never trade away a legal lethal attack.
                        score += 100.0
                    else:
                        opponent_board = [
                            entity
                            for entity in entities.values()
                            if entity.get("area") == "board"
                            and entity.get("controller") == "opponent"
                        ]
                        # Face pressure is useful when the board is empty or the
                        # opponent is already low, but no longer dominates every
                        # available minion trade.
                        score += 1.0 + max(15.0 - target_health, 0.0) / 5.0
                        if not opponent_board:
                            score += 2.0
                else:
                    remaining = float(target.get("max_health", 0)) - float(
                        target.get("damage", 0)
                    )
                    target_attack = float(target.get("attack", 0))
                    if source_attack >= remaining:
                        target_value = target_attack + max(remaining, 0.0)
                        score += 4.0 + min(target_value, 16.0) / 4.0
                        if source_health > target_attack:
                            score += 2.0
                        elif target_attack > 0:
                            source_value = source_attack + source_health
                            score -= min(source_value, 16.0) / 8.0
                    elif source_health > target_attack:
                        # Damaging a minion without killing it is only mildly
                        # useful, and only when the attacker survives.
                        score += min(source_attack / max(remaining, 1.0), 1.0)
            elif kind.startswith("play_card"):
                score += float(action.get("mana_cost", 0)) / 2.0
            score += self.rng.random() * self.noise
            scored.append((score, int(action["index"])))
        return max(scored)[1]


class ModelPolicy:
    def __init__(
        self,
        model: HearthQNetwork,
        tensorizer: Tensorizer,
        *,
        device: str,
        epsilon: float = 0.0,
        seed: int = 0,
        allow_concede: bool = False,
        sample: bool = False,
        temperature: float = 1.0,
    ) -> None:
        self.model = model
        self.tensorizer = tensorizer
        self.device = device
        self.epsilon = epsilon
        self.rng = random.Random(seed)
        self.allow_concede = allow_concede
        self.sample = sample
        self.temperature = temperature

    @torch.no_grad()
    def choose(self, decision: dict[str, Any], self_deck: Sequence[str]) -> int:
        if self.rng.random() < self.epsilon:
            return RandomPolicy(self.rng.randrange(2**63)).choose(decision, self_deck)
        was_training = self.model.training
        self.model.eval()
        batch = move_batch(
            collate([self.tensorizer.encode(decision, self_deck)]), self.device
        )
        values = self.model(batch)[0]
        if not self.allow_concede and len(decision["actions"]) > 1:
            values = values.clone()
            for position, action_observation in enumerate(decision["actions"]):
                if action_observation["kind"] == "concede":
                    values[position] = torch.finfo(values.dtype).min
        if self.sample:
            if self.temperature <= 0:
                raise ValueError("sampling temperature must be positive")
            probabilities = torch.softmax(values / self.temperature, dim=0)
            action = int(
                torch.multinomial(
                    probabilities,
                    1,
                    generator=torch.Generator(device="cpu").manual_seed(
                        self.rng.randrange(2**63)
                    )
                    if probabilities.device.type == "cpu"
                    else None,
                ).item()
            )
        else:
            action = int(values.argmax().item())
        if was_training:
            self.model.train()
        return action
