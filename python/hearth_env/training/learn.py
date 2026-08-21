from __future__ import annotations

from collections.abc import Iterable, Sequence
from contextlib import nullcontext
from dataclasses import dataclass

import torch
from torch.nn import functional as F

from .config import TrainConfig
from .model import HearthQNetwork, selected_q
from .tensorize import Tensorizer, collate, move_batch
from .trajectory import TrainingSample


@dataclass
class LossMetrics:
    loss: float
    accuracy: float | None = None
    dmc_loss: float | None = None
    bc_loss: float | None = None


def _sample_batch(
    samples: Sequence[TrainingSample], tensorizer: Tensorizer, device: str
) -> tuple[dict[str, torch.Tensor], torch.Tensor, torch.Tensor, torch.Tensor]:
    batch = move_batch(
        collate(
            [tensorizer.encode(sample.decision, sample.self_deck) for sample in samples]
        ),
        device,
    )
    actions = torch.tensor([sample.action_index for sample in samples], device=device)
    targets = torch.tensor(
        [sample.target for sample in samples], dtype=torch.float32, device=device
    )
    weights = torch.tensor(
        [sample.weight for sample in samples], dtype=torch.float32, device=device
    )
    return batch, actions, targets, weights


def train_batch(
    model: HearthQNetwork,
    optimizer: torch.optim.Optimizer,
    tensorizer: Tensorizer,
    samples: Sequence[TrainingSample],
    config: TrainConfig,
    device: str,
    *,
    behavior_clone: bool,
) -> LossMetrics:
    model.train()
    batch, actions, targets, weights = _sample_batch(samples, tensorizer, device)
    amp_enabled = config.amp and device.startswith("cuda")
    context = (
        torch.autocast(device_type="cuda", enabled=True)
        if amp_enabled
        else nullcontext()
    )
    optimizer.zero_grad(set_to_none=True)
    with context:
        q_values = model(batch)
        if behavior_clone:
            per_item = F.cross_entropy(q_values, actions, reduction="none")
            accuracy = float((q_values.argmax(1) == actions).float().mean().item())
        else:
            per_item = F.smooth_l1_loss(
                selected_q(q_values, actions), targets, reduction="none"
            )
            accuracy = None
        loss = (per_item * weights).sum() / weights.sum().clamp_min(1.0)
    loss.backward()
    torch.nn.utils.clip_grad_norm_(model.parameters(), config.grad_clip)
    optimizer.step()
    return LossMetrics(float(loss.detach().item()), accuracy)


def train_mixed_batch(
    model: HearthQNetwork,
    optimizer: torch.optim.Optimizer,
    tensorizer: Tensorizer,
    dmc_samples: Sequence[TrainingSample],
    bc_samples: Sequence[TrainingSample],
    config: TrainConfig,
    device: str,
    *,
    bc_weight: float,
) -> LossMetrics:
    """Apply DMC regression and a weighted BC anchor in one optimizer step."""

    if not dmc_samples:
        raise ValueError("a mixed DMC batch needs at least one return sample")
    model.train()
    dmc_batch, dmc_actions, dmc_targets, dmc_weights = _sample_batch(
        dmc_samples, tensorizer, device
    )
    bc_encoded = _sample_batch(bc_samples, tensorizer, device) if bc_samples else None
    amp_enabled = config.amp and device.startswith("cuda")
    context = (
        torch.autocast(device_type="cuda", enabled=True)
        if amp_enabled
        else nullcontext()
    )
    optimizer.zero_grad(set_to_none=True)
    with context:
        dmc_values = model(dmc_batch)
        dmc_per_item = F.smooth_l1_loss(
            selected_q(dmc_values, dmc_actions), dmc_targets, reduction="none"
        )
        dmc_loss = (dmc_per_item * dmc_weights).sum() / dmc_weights.sum().clamp_min(1.0)
        bc_loss = torch.zeros((), device=device)
        accuracy = None
        if bc_encoded is not None:
            bc_batch, bc_actions, _, bc_weights = bc_encoded
            bc_values = model(bc_batch)
            bc_per_item = F.cross_entropy(bc_values, bc_actions, reduction="none")
            bc_loss = (bc_per_item * bc_weights).sum() / bc_weights.sum().clamp_min(1.0)
            accuracy = float((bc_values.argmax(1) == bc_actions).float().mean().item())
        loss = dmc_loss + bc_weight * bc_loss
    loss.backward()
    torch.nn.utils.clip_grad_norm_(model.parameters(), config.grad_clip)
    optimizer.step()
    return LossMetrics(
        float(loss.detach().item()),
        accuracy,
        float(dmc_loss.detach().item()),
        float(bc_loss.detach().item()) if bc_encoded is not None else None,
    )


def train_stream(
    model: HearthQNetwork,
    optimizer: torch.optim.Optimizer,
    tensorizer: Tensorizer,
    samples: Iterable[TrainingSample],
    config: TrainConfig,
    device: str,
    *,
    behavior_clone: bool,
) -> list[LossMetrics]:
    pending: list[TrainingSample] = []
    metrics: list[LossMetrics] = []
    for sample in samples:
        pending.append(sample)
        if len(pending) == config.batch_size:
            metrics.append(
                train_batch(
                    model,
                    optimizer,
                    tensorizer,
                    pending,
                    config,
                    device,
                    behavior_clone=behavior_clone,
                )
            )
            pending.clear()
    if pending:
        metrics.append(
            train_batch(
                model,
                optimizer,
                tensorizer,
                pending,
                config,
                device,
                behavior_clone=behavior_clone,
            )
        )
    return metrics
