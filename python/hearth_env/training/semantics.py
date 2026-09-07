"""Versioned semantic vocabularies. IDs are stable; append, never reorder.

No card name, ID, expansion, rarity or Lua identifier is a semantic feature.
New cards reuse effect words and explicit numbers instead of a fresh ID vector.
"""

from __future__ import annotations

import html
import math
import re
from typing import Any

KEYWORDS = tuple(
    """
adapt battlecry battlecry_repeater cannot_be_attacked_by_charged_devilsaur
cannot_be_attacked_by_fools_bane cannot_be_attacked_by_icehowl casts_when_drawn
charge choose_multiple choose_one colossal combo conditional_charge corrupt
corrupting_mist_curse costs_health_instead_of_mana counter cthun_buffs cthun_taunt
death_knight_corpses deathrattle deathrattle_repeater discover divine_shield dormant
dragon_consort_discount dredge echo elusive end_of_turn_repeater excavate fabled
finale fools_bane_unlimited_attacks forge freeze frenzy frost_plague_surcharge
frozen_solid gigantify healing_becomes_damage herald hero_power_can_target_minions
hero_power_disabled hero_power_next_turn_surcharge hero_power_twice_per_turn
hero_power_unlimited honorable_kill imbue immune infuse inspire invoke kindred
lifesteal magnetic manathirst mega_windfury miniaturize mograine
next_hero_power_discount next_murloc_costs_health next_secret_cost_one_this_turn
next_spell_cost_zero_this_turn next_spell_costs_health no_corpse outcast overheal
overkill overload passive poisonous power_word_glory prepare primus_frost_runes
quest questline quickdraw radiant_elemental_minimum_cost randomize_targets
raza_hero_power_zero reborn recruit rewind rush secret shatter sidequest silence
spell_damage spellburst starship start_of_game stealth summoned_when_drawn taunt
temporary titan tourist tradeable twinspell unending_plagues weapon_durability_immune
windfury
""".split()
)
KEYWORD_INDEX = {name: i for i, name in enumerate(KEYWORDS)}
KINDS = ("hero", "hero_power", "minion", "spell", "weapon", "location", "enchantment")
CLASSES = (
    "neutral",
    "death_knight",
    "demon_hunter",
    "druid",
    "hunter",
    "mage",
    "paladin",
    "priest",
    "rogue",
    "shaman",
    "warlock",
    "warrior",
)
TAGS = (
    "beast",
    "demon",
    "dragon",
    "elemental",
    "mech",
    "murloc",
    "naga",
    "pirate",
    "quilboar",
    "totem",
    "undead",
    "all",
)
SCHOOLS = ("arcane", "fel", "fire", "frost", "holy", "nature", "shadow")
EVENT_KINDS = tuple(
    """
game_started turn_started card_drawn card_burned card_created fatigue card_played
spell_cast spell_targeted minion_played weapon_played location_played card_countered
card_discarded card_traded trade_draw minion_summoned magnetized weapon_equipped
weapon_destroyed location_used location_destroyed hero_power_used hero_power_replaced
hero_replaced secret_played secret_revealed zone_changed controller_changed transformed
attack damaged damage_prevented healed armor_gained overload_queued mana_locked
mana_unlocked overload_cleared temporary_mana_gained temporary_mana_expired
mana_crystals_gained mana_crystals_destroyed mana_spent player_resource_gained
player_resource_spent keyword_disabled frozen entity_died turn_ended conceded
game_ended choice_requested choice_made
""".split()
)
EVENT_INDEX = {name: i + 1 for i, name in enumerate(EVENT_KINDS)}
ROLES = tuple(
    "card source spell target minion weapon location hero_power old new secret entity attachment attacker defender collateral".split()
)
ROLE_INDEX = {name: i + 1 for i, name in enumerate(ROLES)}
HISTORY_GROUPS = (
    "cards_played",
    "spells_cast",
    "minions_played",
    "weapons_played",
    "locations_played",
    "discarded_cards",
    "minions_died",
)
ZONES = (
    "deck",
    "hand",
    "board",
    "graveyard",
    "secret",
    "set_aside",
    "removed",
    "hero",
    "hero_power",
    "weapon",
)
STATIC_DIM = (
    12 + len(KINDS) + len(CLASSES) + len(TAGS) + len(SCHOOLS) + len(KEYWORDS) * 2
)


def keyword_features(keywords: list[str]) -> list[float]:
    unknown = set(keywords) - set(KEYWORD_INDEX)
    if unknown:
        raise ValueError(
            f"unknown gameplay keywords; extend semantic schema: {sorted(unknown)}"
        )
    present = set(keywords)
    return [float(name in present) for name in KEYWORDS]


def static_features(definition: dict[str, Any]) -> list[float]:
    classes = definition.get("classes") or [definition.get("class", "neutral")]
    tags = {str(tag).lower() for tag in definition.get("tags", [])}
    runes = definition.get("rune_cost") or {}
    parameters = definition.get("keyword_params") or {}
    return [
        float(definition.get("cost", 0)) / 10,
        float(definition.get("attack", 0)) / 20,
        float(definition.get("health", 0)) / 20,
        float(definition.get("armor", 0)) / 20,
        float(definition.get("secret", False)),
        float(definition.get("target_mode") == "required"),
        float(definition.get("target_mode") == "required_if_available"),
        float(definition.get("target_mode") == "optional"),
        float(runes.get("blood", 0)) / 3,
        float(runes.get("frost", 0)) / 3,
        float(runes.get("unholy", 0)) / 3,
        float(definition.get("sideboard_size", 0)) / 10,
        *[float(definition.get("kind") == name) for name in KINDS],
        *[float(name in classes) for name in CLASSES],
        *[float(name in tags) for name in TAGS],
        *[float(definition.get("spell_school") == name) for name in SCHOOLS],
        *keyword_features(definition.get("keywords", [])),
        *[float(parameters.get(name, 0)) / 10 for name in KEYWORDS],
    ]


def effect_tokens(text: str) -> list[tuple[str, tuple[float, float]]]:
    source = re.sub(r"<[^>]+>|\[x\]", " ", html.unescape(text)).lower()
    output = []
    for token in re.findall(r"\d+(?:\.\d+)?|[a-z]+(?:'[a-z]+)?|[^\w\s]", source):
        if re.fullmatch(r"\d+(?:\.\d+)?", token):
            number = float(token)
            output.append(("<number>", (number / 20, math.log1p(number) / 5)))
        else:
            output.append((token, (0.0, 0.0)))
    return output


def vocabulary(entries: dict[str, dict[str, Any]]) -> tuple[str, ...]:
    words = {
        word
        for entry in entries.values()
        for word, _ in effect_tokens(str(entry["definition"].get("text", "")))
    }
    return (
        "<pad>",
        "<unk>",
        "<number>",
        *sorted(words - {"<pad>", "<unk>", "<number>"}),
    )
