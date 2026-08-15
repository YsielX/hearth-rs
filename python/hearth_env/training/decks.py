from __future__ import annotations

import json
import random
from collections import Counter
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

SET_ORDER = (
    "BASIC",
    "CORE",
    "LEGACY",
    "EXPERT1",
    "NAXX",
    "GVG",
    "BRM",
    "TGT",
    "LOE",
    "OG",
    "KARA",
    "GANGS",
    "UNGORO",
    "ICECROWN",
)


@dataclass(frozen=True)
class Deck:
    name: str
    card_class: str
    cards: tuple[str, ...]
    hero_power: str
    unrestricted: bool = False
    archetype: str = ""
    strategy: str = "unknown"
    bc_eligible: bool = True
    source: str | None = None
    adapted: bool = False
    era_cutoff: str | None = None

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
            archetype=str(value.get("archetype", "")),
            strategy=str(value.get("strategy", "unknown")),
            bc_eligible=bool(value.get("bc_eligible", True)),
            source=value.get("source"),
            adapted=bool(value.get("adapted", False)),
            era_cutoff=value.get("era_cutoff"),
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
        self._curated_by_class = {
            card_class: [deck for deck in self.curated if deck.card_class == card_class]
            for card_class in sorted({deck.card_class for deck in self.curated})
        }
        self.allowed_sets = self._common_era_sets()
        self._pools = self._class_pools()

    def _common_era_sets(self) -> frozenset[str] | None:
        """Infer a shared historical card pool when every deck declares one."""
        cutoffs = [deck.era_cutoff for deck in self.curated]
        if not all(cutoffs):
            return None
        try:
            latest = max(SET_ORDER.index(cutoff) for cutoff in cutoffs if cutoff)
        except ValueError:
            return None
        return frozenset(SET_ORDER[: latest + 1])

    def _latest_known_cutoff(self) -> str | None:
        cutoffs = [
            deck.era_cutoff for deck in self.curated if deck.era_cutoff in SET_ORDER
        ]
        return max(cutoffs, key=SET_ORDER.index, default=None)

    def _class_pools(self) -> dict[str, list[str]]:
        result: dict[str, list[str]] = {}
        deckable = {"hero", "minion", "spell", "weapon", "location"}
        represented_classes = {deck.card_class for deck in self.curated}
        for card_id, entry in self.catalog.entries.items():
            definition = entry["definition"]
            if not definition.get("collectible", False):
                continue
            if (
                self.allowed_sets is not None
                and definition.get("set") not in self.allowed_sets
            ):
                continue
            if str(definition.get("kind", "")).lower() not in deckable:
                continue
            classes = definition.get("classes") or [definition.get("class", "neutral")]
            for card_class in represented_classes:
                if "neutral" in classes or card_class in classes:
                    result.setdefault(card_class, []).append(card_id)
        for cards in result.values():
            cards.sort()
        return result

    def _perturb(self, deck: Deck, fraction: float = 0.2) -> Deck:
        cards = list(deck.cards)
        pool = self._pools.get(deck.card_class, list(deck.cards))
        counts = Counter(cards)
        replacements = max(1, round(len(cards) * fraction))
        for index in self.rng.sample(range(len(cards)), min(replacements, len(cards))):
            previous = cards[index]
            counts[previous] -= 1
            candidates = []
            for card_id in pool:
                rarity = str(
                    self.catalog.entries[card_id]["definition"].get("rarity", "")
                ).lower()
                maximum = 1 if rarity == "legendary" else 2
                if counts[card_id] < maximum:
                    candidates.append(card_id)
            replacement = self.rng.choice(candidates) if candidates else previous
            cards[index] = replacement
            counts[replacement] += 1
        return Deck(
            name=f"{deck.name} (perturbed)",
            card_class=deck.card_class,
            cards=tuple(cards),
            hero_power=deck.hero_power,
            unrestricted=deck.unrestricted,
            archetype=deck.archetype,
            strategy=deck.strategy,
            bc_eligible=deck.bc_eligible,
            source=deck.source,
            adapted=True,
            era_cutoff=deck.era_cutoff,
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
            name=f"random-{card_class}",
            card_class=card_class,
            cards=cards,
            hero_power=DEFAULT_HERO_POWERS[card_class],
            era_cutoff=self._latest_known_cutoff(),
        )

    def sample(self) -> Deck:
        roll = self.rng.random()
        card_class = self.rng.choice(sorted(self._curated_by_class))
        deck = self.rng.choice(self._curated_by_class[card_class])
        if roll < self.curated_probability:
            return deck
        if roll < self.curated_probability + self.perturb_probability:
            return self._perturb(deck)
        return self._random()

    def sample_match(self) -> dict[str, Any]:
        return match_config(self.sample(), self.sample())
