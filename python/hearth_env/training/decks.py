from __future__ import annotations

import json
import random
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .catalog import CardCatalog

DEFAULT_HERO_POWERS = {
    "death_knight": "HERO_11bp",
    "demon_hunter": "HERO_10bp",
    "druid": "HERO_06bp",
    "hunter": "HERO_05bp",
    "mage": "HERO_08bp",
    "paladin": "HERO_04bp",
    "priest": "HERO_09bp",
    "rogue": "HERO_03bp",
    "shaman": "HERO_02bp",
    "warlock": "HERO_07bp",
    "warrior": "HERO_01bp",
}


@dataclass(frozen=True)
class Deck:
    name: str
    card_class: str
    cards: tuple[str, ...]
    hero_power: str
    unrestricted: bool = False

    @classmethod
    def from_file(cls, path: str | Path) -> Deck:
        with open(path, encoding="utf-8") as source:
            value = json.load(source)
        card_class = value.get("class", "mage")
        return cls(
            name=value.get("name", Path(path).stem),
            card_class=card_class,
            cards=tuple(value["cards"]),
            hero_power=value.get(
                "hero_power", DEFAULT_HERO_POWERS.get(card_class, "HERO_08bp")
            ),
            unrestricted=bool(value.get("unrestricted", False)),
        )


def match_config(one: Deck, two: Deck) -> dict[str, Any]:
    return {
        "decks": [list(one.cards), list(two.cards)],
        "hero_powers": [one.hero_power, two.hero_power],
        "classes": [one.card_class, two.card_class],
        "unrestricted": one.unrestricted or two.unrestricted,
    }


class DeckPool:
    """Samples curated, perturbed, and broad random class-legal decks."""

    def __init__(
        self,
        catalog: CardCatalog,
        curated: Sequence[Deck],
        *,
        seed: int = 0,
        curated_probability: float = 0.5,
        perturb_probability: float = 0.35,
    ) -> None:
        if not curated:
            raise ValueError("at least one curated deck is required")
        self.catalog = catalog
        self.curated = list(curated)
        self.rng = random.Random(seed)
        self.curated_probability = curated_probability
        self.perturb_probability = perturb_probability
        self._pools = self._class_pools()

    def _class_pools(self) -> dict[str, list[str]]:
        result: dict[str, list[str]] = {}
        deckable = {"hero", "minion", "spell", "weapon", "location"}
        for card_id, entry in self.catalog.entries.items():
            definition = entry["definition"]
            if not definition.get("collectible", False):
                continue
            if str(definition.get("kind", "")).lower() not in deckable:
                continue
            classes = definition.get("classes") or [definition.get("class", "neutral")]
            for card_class in DEFAULT_HERO_POWERS:
                if "neutral" in classes or card_class in classes:
                    result.setdefault(card_class, []).append(card_id)
        for cards in result.values():
            cards.sort()
        return result

    def _perturb(self, deck: Deck, fraction: float = 0.2) -> Deck:
        cards = list(deck.cards)
        pool = self._pools.get(deck.card_class, list(deck.cards))
        replacements = max(1, round(len(cards) * fraction))
        for index in self.rng.sample(range(len(cards)), min(replacements, len(cards))):
            cards[index] = self.rng.choice(pool)
        return Deck(
            f"{deck.name} (perturbed)",
            deck.card_class,
            tuple(cards),
            deck.hero_power,
            deck.unrestricted,
        )

    def _random(self) -> Deck:
        card_class = self.rng.choice(sorted(self._pools))
        pool = self._pools[card_class]
        cards = tuple(
            self.rng.sample(pool, 30)
            if len(pool) >= 30
            else (self.rng.choice(pool) for _ in range(30))
        )
        return Deck(
            f"random-{card_class}",
            card_class,
            cards,
            DEFAULT_HERO_POWERS[card_class],
        )

    def sample(self) -> Deck:
        roll = self.rng.random()
        deck = self.rng.choice(self.curated)
        if roll < self.curated_probability:
            return deck
        if roll < self.curated_probability + self.perturb_probability:
            return self._perturb(deck)
        return self._random()

    def sample_match(self) -> dict[str, Any]:
        return match_config(self.sample(), self.sample())
