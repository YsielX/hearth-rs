from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import torch

from .catalog import CARD_FEATURE_SCHEMA_VERSION, CardCatalog
from .config import ModelConfig
from .model import HearthQNetwork
from .tensorize import TENSOR_SCHEMA_VERSION

CHECKPOINT_VERSION = 1


def save_checkpoint(
    path: str | Path,
    model: HearthQNetwork,
    catalog: CardCatalog,
    *,
    optimizer: torch.optim.Optimizer | None = None,
    step: int = 0,
    phase: str = "unknown",
    metrics: dict[str, float] | None = None,
    extra_state: dict[str, Any] | None = None,
) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    payload: dict[str, Any] = {
        "checkpoint_version": CHECKPOINT_VERSION,
        "observation_schema_version": 3,
        "tensor_schema_version": TENSOR_SCHEMA_VERSION,
        "model_config": model.config.to_dict(),
        "catalog": catalog.manifest(),
        "model": model.state_dict(),
        "step": step,
        "phase": phase,
        "metrics": metrics or {},
    }
    if optimizer is not None:
        payload["optimizer"] = optimizer.state_dict()
    if extra_state:
        overlap = set(payload).intersection(extra_state)
        if overlap:
            raise ValueError(f"extra checkpoint state uses reserved keys: {sorted(overlap)}")
        payload.update(extra_state)
    temporary = path.with_suffix(path.suffix + f".tmp-{os.getpid()}")
    torch.save(payload, temporary)
    os.replace(temporary, path)


def load_checkpoint(
    path: str | Path,
    catalog: CardCatalog,
    *,
    device: str = "cpu",
    strict_pack: bool = False,
) -> tuple[HearthQNetwork, dict[str, Any]]:
    payload = torch.load(path, map_location=device, weights_only=False)
    if payload.get("checkpoint_version") != CHECKPOINT_VERSION:
        raise ValueError("unsupported checkpoint version")
    if payload.get("tensor_schema_version") != TENSOR_SCHEMA_VERSION:
        raise ValueError("tensor schema differs from checkpoint")
    old_manifest = payload["catalog"]
    if old_manifest.get("feature_schema_version") != CARD_FEATURE_SCHEMA_VERSION:
        raise ValueError("card feature schema differs from checkpoint")
    if strict_pack and old_manifest["pack_hash"] != catalog.pack_hash:
        raise ValueError(
            f"card pack differs: checkpoint={old_manifest['pack_hash']} current={catalog.pack_hash}"
        )
    config = ModelConfig.from_dict(payload["model_config"])
    if config.card_hash_dim != catalog.hash_dim:
        raise ValueError("card hash dimension differs from checkpoint")
    model = HearthQNetwork(catalog, config)
    saved = payload["model"]
    old_embedding = saved.pop("card_id_embedding.weight")
    incompatible = model.load_state_dict(saved, strict=False)
    unexpected = [
        key for key in incompatible.unexpected_keys if key != "card_feature_table"
    ]
    missing = [
        key
        for key in incompatible.missing_keys
        if key != "card_id_embedding.weight" and not key.startswith("value_head.")
    ]
    if unexpected or missing:
        raise ValueError(
            f"incompatible checkpoint keys: missing={missing}, unexpected={unexpected}"
        )
    old_indices = {card_id: i for i, card_id in enumerate(old_manifest["card_ids"])}
    new_indices = catalog.id_to_index
    with torch.no_grad():
        for card_id in set(new_indices) - set(old_indices):
            model.card_id_embedding.weight[new_indices[card_id]].zero_()
        for card_id, old_index in old_indices.items():
            new_index = new_indices.get(card_id)
            if new_index is not None and old_index < old_embedding.shape[0]:
                model.card_id_embedding.weight[new_index].copy_(
                    old_embedding[old_index]
                )
    model.to(device)
    special = {"<pad>", "<unk>"}
    payload["migration"] = {
        "old_pack_hash": old_manifest["pack_hash"],
        "new_pack_hash": catalog.pack_hash,
        "retained_cards": len((set(old_indices) & set(new_indices)) - special),
        "new_cards": len((set(new_indices) - set(old_indices)) - special),
    }
    return model, payload
