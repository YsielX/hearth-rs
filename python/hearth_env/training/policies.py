from __future__ import annotations

import random
from collections.abc import Sequence
from typing import Any, Protocol

import torch

from .model import HearthQNetwork
from .distribution import policy_logits
from .tensorize import Tensorizer, collate, move_batch


class Policy(Protocol):
    def choose(self, decision: dict[str, Any], self_deck: Sequence[str]) -> int: ...


class RandomPolicy:
    def __init__(self, seed: int = 0) -> None:
        self.rng = random.Random(seed)

    def choose(self, decision: dict[str, Any], self_deck: Sequence[str]) -> int:
        return self.rng.choice([action["index"] for action in decision["actions"]])


class HeuristicPolicy:
    """Transport adapter for the Rust bot, bound by the rollout environment."""

    def __init__(self):
        self.env = None

    def choose(self, decision, self_deck):
        if self.env is None:
            raise RuntimeError("Rust bot must be bound to its rollout environment")
        return self.env.heuristic_action(decision["id"])


class ModelPolicy:
    def __init__(
        self,
        model: HearthQNetwork,
        tensorizer: Tensorizer,
        *,
        device: str,
        epsilon: float = 0.0,
        seed: int = 0,
        sample: bool = False,
        temperature: float = 1.0,
    ) -> None:
        self.model = model
        self.tensorizer = tensorizer
        self.device = device
        self.epsilon = epsilon
        self.rng = random.Random(seed)
        self.sample = sample
        self.temperature = temperature
        self.last_behavior: dict[str, float] | None = None

    @torch.no_grad()
    def choose(self, decision: dict[str, Any], self_deck: Sequence[str]) -> int:
        self.last_behavior = None
        if self.rng.random() < self.epsilon:
            return RandomPolicy(self.rng.randrange(2**63)).choose(decision, self_deck)
        was_training = self.model.training
        self.model.eval()
        batch = move_batch(
            collate([self.tensorizer.encode(decision, self_deck)]), self.device
        )
        logits, state_value = self.model.policy_value(batch)
        values = policy_logits(
            logits,
            batch,
            temperature=self.temperature if self.sample else 1.0,
        )[0]
        if self.sample:
            if self.temperature <= 0:
                raise ValueError("sampling temperature must be positive")
            probabilities = torch.softmax(values, dim=0)
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
            self.last_behavior = {
                "log_probability": float(values.log_softmax(0)[action].item()),
                "value": float(state_value[0].item()),
                "temperature": self.temperature,
                "action_support": "all_legal",
            }
        else:
            action = int(values.argmax().item())
        if was_training:
            self.model.train()
        return action
