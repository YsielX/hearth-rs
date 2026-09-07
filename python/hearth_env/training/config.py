from __future__ import annotations

from dataclasses import asdict, dataclass
import math
from typing import Any


@dataclass(frozen=True)
class ModelConfig:
    architecture_version: int = 3
    text_vocabulary: tuple[str, ...] = ()
    text_dim: int = 48
    max_card_tokens: int = 256
    max_memory: int = 512
    max_facts: int = 96
    hidden_dim: int = 128
    card_hash_dim: int = 256
    entity_state_dim: int = 32
    global_dim: int = 40
    history_numeric_dim: int = 14
    action_numeric_dim: int = 9
    max_entities: int = 64
    max_entity_cards: int = 16
    max_history: int = 96
    max_history_entities: int = 16
    max_deck_cards: int = 40
    max_action_sources: int = 10
    max_action_cards: int = 4
    transformer_layers: int = 2
    attention_heads: int = 4
    dropout: float = 0.1

    def to_dict(self) -> dict[str, Any]:
        result = asdict(self)
        return result

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> ModelConfig:
        value = dict(value)
        value.setdefault("architecture_version", 2)
        value["text_vocabulary"] = tuple(value.get("text_vocabulary", ()))
        return cls(**value)


@dataclass
class TrainConfig:
    excluded_cards: tuple[str, ...] = ()
    device: str = "auto"
    seed: int = 0
    bc_learning_rate: float = 3e-4
    dmc_learning_rate: float = 1e-5
    ppo_learning_rate: float = 3e-5
    weight_decay: float = 1e-4
    batch_size: int = 128
    grad_clip: float = 1.0
    amp: bool = True
    bc_epochs: int = 3
    dmc_iterations: int = 1000
    ppo_iterations: int = 1000
    episodes_per_iteration: int = 64
    updates_per_iteration: int = 128
    replay_capacity: int = 500_000
    replay_warmup: int = 2_000
    epsilon_start: float = 0.25
    epsilon_end: float = 0.05
    epsilon_decay_iterations: int = 500
    workers: int = 0
    checkpoint_every: int = 10
    league_snapshot_every: int = 25
    max_steps: int = 1000
    history_limit: int | None = 96
    bc_regularization_start: float = 0.2
    bc_regularization_end: float = 0.05
    ppo_epochs: int = 4
    ppo_temperature: float = 1.0
    ppo_clip: float = 0.2
    value_clip: float = 0.2
    value_coefficient: float = 0.5
    entropy_coefficient: float = 0.01
    gamma: float = 1.0
    gae_lambda: float = 0.98
    reference_kl_coefficient: float = 0.0
    ppo_bc_coefficient: float = 0.0

    def __post_init__(self) -> None:
        if not math.isfinite(self.ppo_temperature) or self.ppo_temperature <= 0:
            raise ValueError("PPO temperature must be positive and finite")

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def resolve_device(requested: str) -> str:
    if requested != "auto":
        return requested
    import torch

    if torch.cuda.is_available():
        return "cuda"
    if getattr(torch.backends, "mps", None) and torch.backends.mps.is_available():
        return "mps"
    return "cpu"
