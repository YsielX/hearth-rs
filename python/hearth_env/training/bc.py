from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path

import torch

from .catalog import CardCatalog
from .checkpoint import save_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .learn import train_stream
from .model import HearthQNetwork
from .tensorize import Tensorizer
from .trajectory import stream_samples


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
        lr=train_config.learning_rate,
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
