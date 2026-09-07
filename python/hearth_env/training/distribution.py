"""One policy support definition for actors, PPO, BC and reference policies."""

from __future__ import annotations

import math

import torch


def policy_logits(
    logits: torch.Tensor,
    batch: dict[str, torch.Tensor],
    *,
    temperature: float = 1.0,
) -> torch.Tensor:
    if not math.isfinite(temperature) or temperature <= 0:
        raise ValueError("temperature must be positive and finite")
    support = batch["action_mask"]
    if not support.any(dim=-1).all():
        raise ValueError("policy needs at least one supported action per state")
    return (logits / temperature).masked_fill(~support, torch.finfo(logits.dtype).min)


def log_probabilities(
    logits: torch.Tensor, batch: dict[str, torch.Tensor]
) -> torch.Tensor:
    return torch.log_softmax(policy_logits(logits, batch), dim=-1)
