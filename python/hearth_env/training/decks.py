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
    protected_cards: tuple[str, ...] = ()

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
            protected_cards=tuple(value.get("protected_cards", ())),
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
        card_pool: str = "era",
        excluded_cards: Sequence[str] = (),
    ) -> None:
        if not curated:
            raise ValueError("at least one curated deck is required")
        if not 0.0 <= curated_probability <= 1.0:
            raise ValueError("curated_probability must be between 0 and 1")
        if not 0.0 <= perturb_probability <= 1.0:
            raise ValueError("perturb_probability must be between 0 and 1")
        if curated_probability + perturb_probability > 1.0:
            raise ValueError(
                "curated_probability + perturb_probability must not exceed 1"
            )
        self.catalog = catalog
        self.curated = list(curated)
        self.rng = random.Random(seed)
        self.curated_probability = curated_probability
        self.perturb_probability = perturb_probability
        if card_pool not in {"era", "all"}:
            raise ValueError("card_pool must be era or all")
        self.card_pool = card_pool
        self.excluded_cards = frozenset(excluded_cards)
        if any(self.excluded_cards.intersection(deck.cards) for deck in curated):
            raise ValueError("curated training deck contains held-out cards")
        self._curated_by_class = {
            card_class: [deck for deck in self.curated if deck.card_class == card_class]
            for card_class in sorted({deck.card_class for deck in self.curated})
        }
        self.allowed_sets = self._common_era_sets() if card_pool == "era" else None
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
        represented_classes = (
            set(DEFAULT_HERO_POWERS)
            if self.card_pool == "all"
            else {deck.card_class for deck in self.curated}
        )
        for card_id, entry in self.catalog.entries.items():
            definition = entry["definition"]
            if card_id in self.excluded_cards or definition.get("set") == "HERO_SKINS":
                continue
            # These require an explicit constructed list, not an arbitrary
            # 30-card sample. The coverage report must account for them.
            if definition.get("sideboard_size", 0) or definition.get(
                "deck_size"
            ) not in (None, 0, 30):
                continue
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

    def _runes_fit(self, cards: Sequence[str]) -> bool:
        required = {name: 0 for name in ("blood", "frost", "unholy")}
        for card in cards:
            for name, value in (
                self.catalog.entries[card]["definition"].get("rune_cost") or {}
            ).items():
                required[name] = max(required.get(name, 0), int(value))
        return sum(required.values()) <= 3

    def _perturb(self, deck: Deck, fraction: float = 0.1) -> Deck:
        cards = list(deck.cards)
        pool = self._pools.get(deck.card_class, list(deck.cards))
        counts = Counter(cards)
        replacements = max(1, round(len(cards) * fraction))
        highlander = len(counts) == len(cards)
        replaceable = [
            i
            for i, card in enumerate(cards)
            if card not in deck.protected_cards
            and str(self.catalog.entries[card]["definition"].get("rarity", "")).lower()
            != "legendary"
        ]
        for index in self.rng.sample(replaceable, min(replacements, len(replaceable))):
            previous = cards[index]
            original = self.catalog.entries[previous]["definition"]
            counts[previous] -= 1
            candidates = []
            for card_id in pool:
                rarity = str(
                    self.catalog.entries[card_id]["definition"].get("rarity", "")
                ).lower()
                replacement_definition = self.catalog.entries[card_id]["definition"]
                maximum = 1 if rarity == "legendary" or highlander else 2
                fits_role = (
                    replacement_definition.get("kind") == original.get("kind")
                    and abs(
                        replacement_definition.get("cost", 0) - original.get("cost", 0)
                    )
                    <= 1
                )
                if (
                    counts[card_id] < maximum
                    and fits_role
                    and self._runes_fit(cards[:index] + [card_id] + cards[index + 1 :])
                ):
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
            protected_cards=deck.protected_cards,
        )

    def perturb(self, deck: Deck, fraction: float = 0.2) -> Deck:
        """Return a deterministic (for this pool seed) class-legal perturbation."""

        return self._perturb(deck, fraction)

    def _random(self) -> Deck:
        card_class = self.rng.choice(sorted(self._pools))
        pool = self._pools[card_class]
        cards: list[str] = []
        counts: Counter[str] = Counter()
        for low, high, count in ((0, 2, 10), (3, 4, 10), (5, 6, 6), (7, 100, 4)):
            for _ in range(count):
                candidates = [
                    card
                    for card in pool
                    if counts[card]
                    < (
                        1
                        if self.catalog.entries[card]["definition"].get("rarity")
                        == "legendary"
                        else 2
                    )
                    and self._runes_fit([*cards, card])
                ]
                if not candidates:
                    raise ValueError(
                        f"not enough legal cards to construct {card_class}"
                    )
                curved = [
                    card
                    for card in candidates
                    if low
                    <= self.catalog.entries[card]["definition"].get("cost", 0)
                    <= high
                ]
                card = self.rng.choice(curved or candidates)
                cards.append(card)
                counts[card] += 1
        return Deck(
            name=f"random-{card_class}",
            card_class=card_class,
            cards=tuple(cards),
            hero_power=DEFAULT_HERO_POWERS[card_class],
            era_cutoff=self._latest_known_cutoff() if self.card_pool == "era" else None,
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
