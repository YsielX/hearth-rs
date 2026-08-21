from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path

import torch

from .catalog import CardCatalog
from .checkpoint import save_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .learn import _sample_batch, train_stream
from .model import HearthQNetwork
from .tensorize import Tensorizer
from .trajectory import stream_samples


def evaluate_behavior_clone(
    catalog: CardCatalog,
    shards: Sequence[str | Path],
    model: HearthQNetwork,
    *,
    device: str = "cpu",
    batch_size: int = 128,
) -> dict[str, float | int]:
    """Measure imitation quality on held-out behavior-cloning samples."""

    resolved_device = resolve_device(device)
    model.to(resolved_device)
    model.eval()
    tensorizer = Tensorizer(catalog, model.config)
    pending = []
    samples = 0
    weighted_loss = 0.0
    total_weight = 0.0
    top1 = 0
    top3 = 0

    def evaluate_pending() -> None:
        nonlocal samples, weighted_loss, total_weight, top1, top3
        if not pending:
            return
        batch, actions, _, weights = _sample_batch(
            pending, tensorizer, resolved_device
        )
        q_values = model(batch)
        losses = torch.nn.functional.cross_entropy(
            q_values, actions, reduction="none"
        )
        weighted_loss += float((losses * weights).sum().item())
        total_weight += float(weights.sum().item())
        top1 += int((q_values.argmax(1) == actions).sum().item())
        topk = q_values.topk(min(3, q_values.shape[1]), dim=1).indices
        top3 += int((topk == actions.unsqueeze(1)).any(dim=1).sum().item())
        samples += len(pending)
        pending.clear()

    with torch.inference_mode():
        for sample in stream_samples(shards, behavior_clone=True, seed=0):
            pending.append(sample)
            if len(pending) == batch_size:
                evaluate_pending()
        evaluate_pending()
    if samples == 0:
        raise ValueError("behavior-cloning validation input contains no decisions")
    return {
        "samples": samples,
        "loss": weighted_loss / max(total_weight, 1.0),
        "top1_accuracy": top1 / samples,
        "top3_accuracy": top3 / samples,
    }


def train_behavior_clone(
    catalog: CardCatalog,
    shards: Sequence[str | Path],
    output: str | Path,
    train_config: TrainConfig,
    *,
    model_config: ModelConfig | None = None,
    initial_model: HearthQNetwork | None = None,
) -> HearthQNetwork:
    device = resolve_device(train_config.device)
    torch.manual_seed(train_config.seed)
    model = initial_model or HearthQNetwork(
        catalog, model_config or ModelConfig(card_hash_dim=catalog.hash_dim)
    )
    model.to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=train_config.bc_learning_rate,
        weight_decay=train_config.weight_decay,
    )
    tensorizer = Tensorizer(catalog, model.config)
    step = 0
    for epoch in range(train_config.bc_epochs):
        metrics = train_stream(
            model,
            optimizer,
            tensorizer,
            stream_samples(
                shards,
                behavior_clone=True,
                seed=train_config.seed + epoch,
            ),
            train_config,
            device,
            behavior_clone=True,
        )
        step += len(metrics)
        mean_loss = sum(item.loss for item in metrics) / max(len(metrics), 1)
        mean_accuracy = sum(item.accuracy or 0.0 for item in metrics) / max(
            len(metrics), 1
        )
        print(
            f"bc epoch={epoch + 1} batches={len(metrics)} loss={mean_loss:.5f} accuracy={mean_accuracy:.3f}"
        )
        save_checkpoint(
            output,
            model,
            catalog,
            optimizer=optimizer,
            step=step,
            phase="bc",
            metrics={"loss": mean_loss, "accuracy": mean_accuracy},
        )
    return model
