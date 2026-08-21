from __future__ import annotations

import hashlib
import json
import random
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from .catalog import CardCatalog
from .decks import Deck, DeckPool


def _source_cards(path: Path) -> Counter[str]:
    value = json.loads(path.read_text(encoding="utf-8"))
    result: Counter[str] = Counter()
    for card in value.get("source_cards", []):
        result[str(card["name"]).strip().casefold()] += int(card["count"])
    if not result:
        result.update(str(card_id) for card_id in value["cards"])
    return result


def _replacement_distance(one: Counter[str], two: Counter[str]) -> int:
    common = sum((one & two).values())
    return max(sum(one.values()), sum(two.values())) - common


def _clusters(paths: list[Path]) -> list[list[Path]]:
    parents = list(range(len(paths)))

    def find(index: int) -> int:
        while parents[index] != index:
            parents[index] = parents[parents[index]]
            index = parents[index]
        return index

    def union(left: int, right: int) -> None:
        left_root, right_root = find(left), find(right)
        if left_root != right_root:
            parents[right_root] = left_root

    cards = [_source_cards(path) for path in paths]
    for left in range(len(paths)):
        for right in range(left + 1, len(paths)):
            if _replacement_distance(cards[left], cards[right]) <= 4:
                union(left, right)
    grouped: dict[int, list[Path]] = defaultdict(list)
    for index, path in enumerate(paths):
        grouped[find(index)].append(path)
    return [sorted(group) for group in grouped.values()]


def _deck_record(deck: Deck, path: Path | None = None) -> dict[str, Any]:
    value: dict[str, Any] = {
        "name": deck.name,
        "class": deck.card_class,
        "cards": list(deck.cards),
        "hero_power": deck.hero_power,
        "unrestricted": deck.unrestricted,
        "archetype": deck.archetype,
    }
    if path is not None:
        value["path"] = str(path)
    return value


def write_deck_split(
    paths: list[str | Path],
    output_dir: str | Path,
    catalog: CardCatalog,
    *,
    seed: int,
) -> dict[str, Any]:
    """Cluster near-duplicate source lists, then split whole clusters by class."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    normalized = sorted(Path(path).resolve() for path in paths)
    by_class: dict[str, list[Path]] = defaultdict(list)
    for path in normalized:
        by_class[Deck.from_file(path).card_class].append(path)

    rng = random.Random(seed)
    assignments: dict[str, list[dict[str, Any]]] = {
        "train": [],
        "validation": [],
        "test": [],
    }
    cluster_records: list[dict[str, Any]] = []
    for card_class, class_paths in sorted(by_class.items()):
        clusters = _clusters(class_paths)
        rng.shuffle(clusters)
        count = len(clusters)
        validation_count = max(1, round(count * 0.15)) if count >= 3 else 0
        test_count = max(1, round(count * 0.15)) if count >= 3 else 0
        if validation_count + test_count >= count:
            validation_count, test_count = 1, 1
        train_count = count - validation_count - test_count
        splits = (
            ["train"] * train_count
            + ["validation"] * validation_count
            + ["test"] * test_count
        )
        for class_index, (cluster, split) in enumerate(zip(clusters, splits, strict=True)):
            digest = hashlib.sha256(
                "\0".join(str(path) for path in cluster).encode()
            ).hexdigest()[:12]
            cluster_id = f"{card_class}-{class_index:03d}-{digest}"
            members = []
            for path in cluster:
                record = {
                    "path": str(path),
                    "name": Deck.from_file(path).name,
                    "class": card_class,
                    "cluster": cluster_id,
                }
                assignments[split].append(record)
                members.append(str(path))
            cluster_records.append(
                {
                    "id": cluster_id,
                    "class": card_class,
                    "split": split,
                    "members": members,
                }
            )

    manifest = {
        "format_version": 1,
        "seed": seed,
        "distance": "30 minus multiset intersection; connected at <=4 replacements",
        "clusters": cluster_records,
        "splits": assignments,
    }
    (output_dir / "deck_split_manifest.json").write_text(
        json.dumps(manifest, indent=2), encoding="utf-8"
    )

    train_decks = [Deck.from_file(item["path"]) for item in assignments["train"]]
    validation_decks = [
        Deck.from_file(item["path"]) for item in assignments["validation"]
    ]
    (output_dir / "eval_seen.json").write_text(
        json.dumps([_deck_record(deck) for deck in train_decks], indent=2),
        encoding="utf-8",
    )
    (output_dir / "eval_unseen.json").write_text(
        json.dumps([_deck_record(deck) for deck in validation_decks], indent=2),
        encoding="utf-8",
    )
    pool = DeckPool(catalog, train_decks, seed=seed)
    perturbed = [pool.perturb(deck, 0.2) for deck in train_decks]
    (output_dir / "eval_perturbed.json").write_text(
        json.dumps([_deck_record(deck) for deck in perturbed], indent=2),
        encoding="utf-8",
    )
    return manifest
