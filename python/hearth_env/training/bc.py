from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path
from collections.abc import Iterator

import torch

from .catalog import CardCatalog
from .checkpoint import save_checkpoint
from .config import ModelConfig, TrainConfig, resolve_device
from .distribution import policy_logits
from .learn import _sample_batch, train_stream, imitation_loss, imitation_logits
from .model import HearthQNetwork
from .tensorize import Tensorizer
from .trajectory import stream_samples
from hearth_env._native import HearthEnv as NativeEnv


def mix_replay(primary, replay_factory, fraction: float) -> Iterator:
    """Consume every primary sample and cycle replay at an explicit fraction."""
    if not 0 <= fraction < 1:
        raise ValueError("replay fraction must be in [0, 1)")
    replay = iter(replay_factory()) if fraction else iter(())
    credit = 0.0
    for sample in primary:
        yield sample
        credit += fraction / (1 - fraction)
        while credit >= 1 - 1e-9:
            item = next(replay, None)
            if item is None:
                replay = iter(replay_factory())
                item = next(replay, None)
                if item is None:
                    raise ValueError("replay input contains no decisions")
            yield item
            credit -= 1


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
    by_mechanism: dict[str, dict[str, int]] = {}

    def evaluate_pending() -> None:
        nonlocal samples, weighted_loss, total_weight, top1, top3
        if not pending:
            return
        batch, actions, _, weights = _sample_batch(pending, tensorizer, resolved_device)
        q_values = model(batch)
        q_values = policy_logits(q_values, batch)
        q_values = imitation_logits(q_values, pending)
        losses, correct = imitation_loss(q_values, pending)
        weighted_loss += float((losses * weights).sum().item())
        total_weight += float(weights.sum().item())
        top1 += int(correct.sum().item())
        for sample, success in zip(pending, correct.tolist(), strict=True):
            group = by_mechanism.setdefault(
                sample.mechanism, {"samples": 0, "correct": 0}
            )
            group["samples"] += 1
            group["correct"] += int(success)

        topk = q_values.topk(min(3, q_values.shape[1]), dim=1).indices
        top3 += sum(
            any(
                index in (sample.acceptable_actions or (sample.action_index,))
                for index in indices
            )
            for sample, indices in zip(pending, topk.tolist(), strict=True)
        )
        samples += len(pending)
        pending.clear()

    with torch.inference_mode():
        for sample in stream_samples(
            shards,
            behavior_clone=True,
            seed=0,
            expected_pack_hash=catalog.pack_hash,
            expected_engine_build=NativeEnv.engine_build(),
            for_evaluation=True,
        ):
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
        "by_mechanism": {
            name: {**values, "top1_accuracy": values["correct"] / values["samples"]}
            for name, values in sorted(by_mechanism.items())
        },
    }


def build_bc_optimizer(model, config):
    return torch.optim.AdamW(
        [p for p in model.parameters() if p.requires_grad],
        lr=config.bc_learning_rate,
        weight_decay=config.weight_decay,
    )


def train_behavior_clone(
    catalog: CardCatalog,
    shards: Sequence[str | Path],
    output: str | Path,
    train_config: TrainConfig,
    *,
    model_config: ModelConfig | None = None,
    initial_model: HearthQNetwork | None = None,
    replay_shards: Sequence[str | Path] = (),
    replay_fraction: float = 0.3,
    validation_shards: Sequence[str | Path] = (),
) -> HearthQNetwork:
    if train_config.bc_epochs < 1:
        raise ValueError("BC requires at least one epoch")
    device = resolve_device(train_config.device)
    torch.manual_seed(train_config.seed)
    model = initial_model or HearthQNetwork(
        catalog, model_config or ModelConfig(card_hash_dim=catalog.hash_dim)
    )
    model.to(device)
    optimizer = build_bc_optimizer(model, train_config)
    tensorizer = Tensorizer(catalog, model.config)
    step = 0
    best_loss = float("inf")
    for epoch in range(train_config.bc_epochs):

        def samples(paths):
            return stream_samples(
                paths,
                behavior_clone=True,
                seed=train_config.seed + epoch,
                expected_pack_hash=catalog.pack_hash,
                expected_engine_build=NativeEnv.engine_build(),
                excluded_cards=train_config.excluded_cards,
            )

        training_samples = samples(shards)
        if replay_shards:
            training_samples = mix_replay(
                training_samples, lambda: samples(replay_shards), replay_fraction
            )
        metrics = train_stream(
            model,
            optimizer,
            tensorizer,
            training_samples,
            train_config,
            device,
            behavior_clone=True,
        )
        step += len(metrics)
        if not metrics:
            raise ValueError("BC training input contains no decisions")
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
            extra_state={
                "train_config": train_config.to_dict(),
                "data_sources": {
                    "primary": [str(p) for p in shards],
                    "replay": [str(p) for p in replay_shards],
                    "replay_fraction": replay_fraction if replay_shards else 0.0,
                },
            },
        )
        save_checkpoint(
            Path(output).with_stem(Path(output).stem + f".epoch-{epoch + 1:03d}"),
            model,
            catalog,
            step=step,
            phase="bc",
            metrics={"loss": mean_loss, "accuracy": mean_accuracy},
        )
        if validation_shards:
            validation = evaluate_behavior_clone(
                catalog,
                validation_shards,
                model,
                device=device,
                batch_size=train_config.batch_size,
            )
            print(
                f"bc validation epoch={epoch + 1} loss={validation['loss']:.5f} accuracy={validation['top1_accuracy']:.3f}"
            )
            if validation["loss"] < best_loss:
                best_loss = validation["loss"]
                best_path = Path(output).with_stem(Path(output).stem + ".best")
                save_checkpoint(
                    best_path,
                    model,
                    catalog,
                    step=step,
                    phase="bc",
                    metrics=validation,
                )
    return model
