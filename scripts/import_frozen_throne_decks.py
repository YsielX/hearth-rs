"""Import sourced 2017 Frozen Throne decks and materialize runnable adaptations.

The source lists are kept verbatim in each generated JSON file.  A small
explicit substitution table is used only when this repository does not yet
implement a Basic/Classic card from the published list.
"""

from __future__ import annotations

import argparse
import html
import json
import re
import time
import unicodedata
import urllib.request
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlparse

KFT_INDEX = (
    "https://www.hearthstonetopdecks.com/knights-of-the-frozen-throne-deck-lists/"
)
HCT_INDEX = (
    "https://www.hearthstonetopdecks.com/hct-americas-summer-playoffs-2017-deck-lists/"
)
FROZEN_THRONE_SETS = {
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
}

HERO_POWERS = {
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
class ImportSpec:
    slug: str
    index: str
    title_fragment: str
    archetype: str
    strategy: str
    bc_eligible: bool = False


SPECS = [
    ImportSpec(
        "druid_aggro_token",
        KFT_INDEX,
        "Aggro Token Druid",
        "aggro_token",
        "aggro",
        True,
    ),
    ImportSpec(
        "druid_jade", KFT_INDEX, "Infesting Plague Jade Druid", "jade", "control"
    ),
    ImportSpec(
        "druid_hadronox_taunt",
        KFT_INDEX,
        "Hadronox Taunt Ramp Druid",
        "hadronox_taunt",
        "control",
    ),
    ImportSpec("druid_quest", HCT_INDEX, "Monsanto’s Quest Druid", "quest", "combo"),
    ImportSpec(
        "hunter_midrange",
        HCT_INDEX,
        "Izzy’s Midrange Hunter",
        "midrange",
        "midrange",
        True,
    ),
    ImportSpec(
        "hunter_secret",
        KFT_INDEX,
        "Putricide Secret Hunter",
        "secret",
        "midrange",
        True,
    ),
    ImportSpec(
        "hunter_nzoth",
        KFT_INDEX,
        "N’Zoth Deathstalker Hunter",
        "nzoth_deathrattle",
        "control",
    ),
    ImportSpec(
        "hunter_beast", KFT_INDEX, "Penguin Beast Hunter", "beast", "aggro", True
    ),
    ImportSpec(
        "mage_control", KFT_INDEX, "Death Knight Control Mage", "control", "control"
    ),
    ImportSpec(
        "mage_exodia_quest", KFT_INDEX, "Exodia Quest Mage", "exodia_quest", "combo"
    ),
    ImportSpec(
        "mage_elemental",
        KFT_INDEX,
        "Jaina Elemental Mage",
        "elemental",
        "midrange",
        True,
    ),
    ImportSpec(
        "mage_secret",
        KFT_INDEX,
        "Glacial Mysteries Secret Mage",
        "secret",
        "tempo",
        True,
    ),
    ImportSpec(
        "paladin_murloc",
        KFT_INDEX,
        "Midrange Murloc Paladin",
        "murloc",
        "midrange",
        True,
    ),
    ImportSpec(
        "paladin_control",
        KFT_INDEX,
        "Vlps’ KFT Death Knight Control Paladin",
        "control",
        "control",
    ),
    ImportSpec(
        "paladin_divine_shield",
        KFT_INDEX,
        "Divine Shield Paladin",
        "divine_shield",
        "midrange",
        True,
    ),
    ImportSpec(
        "paladin_nzoth",
        KFT_INDEX,
        "N’Zoth Deathrattle Paladin",
        "nzoth_deathrattle",
        "control",
    ),
    ImportSpec(
        "priest_highlander",
        KFT_INDEX,
        "Shadowreaper Kazakus Highlander Priest",
        "highlander",
        "control",
    ),
    ImportSpec("priest_big", HCT_INDEX, "Killinallday’s Big Priest", "big", "control"),
    ImportSpec(
        "priest_silence", HCT_INDEX, "Monsanto’s Silence Priest", "silence", "combo"
    ),
    ImportSpec(
        "rogue_giants_miracle",
        KFT_INDEX,
        "Giants Death Knight Miracle Rogue",
        "miracle_giants",
        "combo",
    ),
    ImportSpec(
        "rogue_jade_nzoth",
        KFT_INDEX,
        "N’Zoth Jade Valeera Rogue",
        "jade_nzoth",
        "control",
    ),
    ImportSpec(
        "shaman_evolve",
        KFT_INDEX,
        "Deathseer Evolve Shaman",
        "evolve",
        "midrange",
        True,
    ),
    ImportSpec(
        "shaman_jade_midrange",
        HCT_INDEX,
        "Freohr’s Midrange Jade Shaman",
        "jade_midrange",
        "midrange",
        True,
    ),
    ImportSpec(
        "shaman_freeze", KFT_INDEX, "Moorabi Freeze Shaman", "freeze", "control"
    ),
    ImportSpec(
        "shaman_control", KFT_INDEX, "Snowfury Concede Shaman", "control", "control"
    ),
    ImportSpec(
        "warlock_zoo", KFT_INDEX, "Kripp’s KFT Warlock Zoo", "zoo", "aggro", True
    ),
    ImportSpec(
        "warlock_discard_quest",
        KFT_INDEX,
        "Death Knight Discard Questlock",
        "discard_quest",
        "combo",
    ),
    ImportSpec(
        "warlock_demon_control",
        KFT_INDEX,
        "Death Knight Demon Control Warlock",
        "demon_control",
        "control",
    ),
    ImportSpec(
        "warlock_highlander",
        KFT_INDEX,
        "Kazakus Highlander Warlock",
        "highlander",
        "control",
    ),
    ImportSpec(
        "warlock_handlock", HCT_INDEX, "Gallon’s Handlock", "handlock", "control"
    ),
    ImportSpec(
        "warrior_pirate",
        HCT_INDEX,
        "Guiyze’s Aggro Pirate Warrior",
        "pirate",
        "aggro",
        True,
    ),
    ImportSpec(
        "warrior_quest_taunt",
        KFT_INDEX,
        "Quest Taunt Warrior",
        "quest_taunt",
        "control",
    ),
    ImportSpec(
        "warrior_blood", KFT_INDEX, "Death Knight Blood Warriors", "blood", "combo"
    ),
    ImportSpec(
        "warrior_rotface_tempo",
        KFT_INDEX,
        "Rotface Tempo Warrior",
        "rotface_tempo",
        "midrange",
        True,
    ),
    ImportSpec(
        "warrior_nzoth", HCT_INDEX, "Fibonacci’s N’zoth Warrior", "nzoth", "control"
    ),
]


# Ordered fallbacks. Capacity and class legality are checked before selection.
SUBSTITUTIONS = {
    "powerofthewild": ["Mark of the Lotus", "Evolving Spores"],
    "savageroar": ["Evolving Spores", "Living Mana"],
    "bloodsailcorsair": ["Small-Time Buccaneer", "Southsea Deckhand"],
    "wrath": ["Living Roots", "Mulch"],
    "swipe": ["Starfall", "Poison Seeds"],
    "nourish": ["Lunar Visions", "Grove Tender"],
    "spellbreaker": ["Wailing Soul", "Eater of Secrets"],
    "theblackknight": ["Big Game Hunter", "Gluttonous Ooze"],
    "stranglethorntiger": ["Sabretooth Stalker", "Giant Wasp"],
    "tracking": ["Stitched Tracker", "Jeweled Macaw"],
    "deadlyshot": ["Toxic Arrow", "Grievous Bite"],
    "hungrycrab": ["Golakka Crawler", "Huge Toad"],
    "eaglehornbow": ["Glaivezooka", "Quick Shot"],
    "killcommand": ["Quick Shot", "Grievous Bite"],
    "unleashthehounds": ["Grievous Bite", "Ball of Spiders"],
    "houndmaster": ["Menagerie Magician", "Trogg Beastrager"],
    "savannahhighmane": ["Infested Wolf", "Nesting Roc"],
    "freezingtrap": ["Bear Trap", "Cat Trick"],
    "snipe": ["Dart Trap", "Venomstrike Trap"],
    "animalcompanion": ["Rat Pack", "Call Pet"],
    "secretkeeper": ["Cloaked Huntress", "Avian Watcher"],
    "timberwolf": ["Dire Wolf Alpha", "Alleycat"],
    "scavenginghyena": ["Trogg Beastrager", "Crackling Razormaw"],
    "arcaneintellect": ["Cabalist's Tome", "Novice Engineer", "Loot Hoarder"],
    "frostnova": ["Volcanic Potion", "Shatter"],
    "icebarrier": ["Counterspell", "Potion of Polymorph", "Frozen Clone"],
    "iceblock": ["Counterspell", "Potion of Polymorph", "Frozen Clone"],
    "spellbender": ["Potion of Polymorph", "Mana Bind"],
    "blizzard": ["Meteor", "Flamestrike"],
    "doomsayer": ["Explosive Sheep", "Volcanic Potion", "Validated Doomsayer"],
    "barongeddon": ["Meteor", "Baron Rivendare"],
    "sorcerersapprentice": ["Cult Sorcerer", "Arcane Anomaly"],
    "archmageantonidas": ["Rhonin", "Medivh, the Guardian"],
    "coldlightoracle": [
        "Novice Engineer",
        "Loot Hoarder",
        "Polluted Hoarder",
        "Cult Master",
        "Azure Drake",
    ],
    "alexstrasza": ["Medivh, the Guardian", "Nefarian"],
    "manawyrm": ["Babbling Book", "Arcane Anomaly"],
    "waterelemental": ["Fire Plume Phoenix", "Tar Creeper"],
    "kirintormage": ["Kabal Lackey", "Cloaked Huntress"],
    "mirrorentity": ["Potion of Polymorph", "Frozen Clone"],
    "blessingofkings": ["Seal of Champions", "Spikeridged Steed"],
    "tirionfordring": [
        "Wickerflame Burnbristle",
        "Ragnaros, Lightlord",
        "Sneed's Old Shredder",
    ],
    "murloctidecaller": ["Vilefin Inquisitor", "Murloc Tinyfin"],
    "murlocwarleader": ["Gentle Megasaur", "Rockpool Hunter"],
    "equality": ["Enter the Coliseum", "Eadric the Pure"],
    "aldorpeacekeeper": ["Keeper of Uldaman", "Aldor Peacekeeper"],
    "consecration": ["Avenging Wrath", "Light's Sorrow"],
    "truesilverchampion": ["Rallying Blade", "Coghammer"],
    "wildpyromancer": ["Tainted Zealot", "Explosive Sheep"],
    "acolyteofpain": ["Loot Hoarder", "Novice Engineer"],
    "northshirecleric": ["Crystalline Oracle", "Museum Curator"],
    "powerwordshield": ["Kabal Talonpriest", "Power Word: Glory"],
    "shadowworddeath": ["Entomb", "Lightbomb"],
    "shadowwordpain": ["Potion of Madness", "Entomb", "Excavated Evil"],
    "holynova": ["Excavated Evil", "Dragonfire Potion"],
    "cabalshadowpriest": ["Kabal Talonpriest", "Museum Curator"],
    "cairnebloodhoof": ["Infested Tauren", "Sneed's Old Shredder"],
    "thoughtsteal": ["Curious Glimmerroot", "Crystalline Oracle"],
    "ysera": ["Nefarian", "Chromaggus"],
    "circleofhealing": ["Binding Heal", "Flash Heal"],
    "divinespirit": ["Power Word: Glory", "Kabal Talonpriest"],
    "innerfire": ["Confuse", "Kooky Chemist"],
    "ancientwatcher": ["Eerie Statue", "Validated Doomsayer"],
    "vanish": ["Sabotage", "Dark Iron Skulker"],
    "gadgetzanauctioneer": [
        "Mimic Pod",
        "Red Mana Wyrm",
        "Edwin VanCleef",
        "Violet Teacher",
        "Burgly Bully",
    ],
    "ancestralspirit": ["Ancestral Knowledge", "Reincarnate"],
    "farsight": ["Ancestral Knowledge", "Loot Hoarder", "Novice Engineer"],
    "hex": ["Devolve", "Big Game Hunter", "Kooky Chemist"],
    "lightningstorm": ["Maelstrom Portal", "Elemental Destruction"],
    "manatidetotem": ["Loot Hoarder", "Novice Engineer"],
    "flametonguetotem": ["Dire Wolf Alpha", "Primalfin Totem"],
    "bloodlust": ["Everyfin is Awesome", "Doppelgangster"],
    "frostshock": ["Glacial Shard", "Cryostasis"],
    "alakirthewindlord": ["Windfury Harpy", "The Mistcaller"],
    "flameimp": ["Flame Juggler", "Fire Fly"],
    "voidwalker": ["Possessed Villager", "Mistress of Mixtures"],
    "abusivesergeant": ["Acherus Veteran", "Lance Carrier"],
    "bloodknight": ["Ravenous Pterrordax", "Darkspeaker"],
    "defenderofargus": ["Faceless Shambler", "Sunborne Val'kyr"],
    "mortalcoil": ["Darkbomb", "Drain Soul"],
    "soulfire": ["Darkbomb", "Fist of Jaraxxus"],
    "siphonsoul": ["Blastcrystal Potion", "Unwilling Sacrifice"],
    "doomguard": ["Lakkari Felhound", "Fearsome Doomguard"],
    "hellfire": ["Felfire Potion", "Demonwrath"],
    "shadowbolt": ["Darkbomb", "Demonwrath", "Blastcrystal Potion"],
    "twistingnether": ["DOOM!", "Felfire Potion"],
    "twilightdrake": ["Midnight Drake", "Twilight Summoner"],
    "shadowflame": ["Felfire Potion", "Demonwrath"],
    "lordjaraxxus": ["Bloodreaver Gul'dan", "Krul the Unshackled"],
    "earthenringfarseer": [
        "Friendly Bartender",
        "Mistress of Mixtures",
        "Refreshment Vendor",
        "Antique Healbot",
        "Cult Apothecary",
    ],
    "mindcontroltech": ["Dirty Rat", "Kooky Chemist"],
    "mountaingiant": ["Frost Giant", "Arcane Giant"],
    "dreadinfernal": ["Despicable Dreadlord", "Fearsome Doomguard"],
    "acidicswampooze": ["Gluttonous Ooze", "Toxic Sewer Ooze"],
    "execute": ["Crush", "Bouncing Blade"],
    "slam": ["Blood To Ichor", "Revenge"],
    "whirlwind": ["Ravaging Ghoul", "Revenge"],
    "armorsmith": ["Alley Armorsmith", "Shield Block"],
    "battlerage": ["Shield Block", "Forge of Souls"],
    "commandingshout": ["Sudden Genesis", "Blood Warriors"],
    "deadmanshand": ["Dead Man's Hand"],
    "frothingberserker": ["Grim Patron", "Val'kyr Soulclaimer"],
    "korkronelite": ["Argent Horserider", "Naga Corsair"],
    "grommashhellscream": ["Rotface", "Varian Wrynn"],
    "brawl": ["Sleep with the Fishes", "Bouncing Blade"],
    "upgrade": ["Bloodsail Cultist", "Orgrimmar Aspirant"],
    "heroicstrike": ["Bash", "Blood To Ichor"],
    "arcanitereaper": ["Fool's Bane", "Death's Bite"],
    "bloodsailraider": ["Bloodsail Cultist", "Naga Corsair"],
    "southseacaptain": ["Phantom Freebooter", "Skycap'n Kragg"],
    "dreadcorsair": ["Blackwater Pirate", "Salty Dog"],
    "captaingreenskin": ["Skycap'n Kragg", "Blingtron 3000"],
    "shieldslam": ["Bash", "Crush", "Bouncing Blade"],
    "prophetvelen": ["Lyra the Sunshard", "Confessor Paletress"],
    "holysmite": ["Potion of Madness", "Shadowbomber"],
    "gnomishinventor": ["Novice Engineer", "Loot Hoarder", "Polluted Hoarder"],
    "crazedalchemist": ["Kooky Chemist"],
    "mindblast": ["Shadowbomber", "Embrace Darkness"],
    "divinefavor": ["Small-Time Recruits", "Solemn Vigil"],
    "naturalize": ["Mulch", "Poison Seeds"],
    "flamestrike": [
        "Firelands Portal",
        "Meteor",
        "Volcanic Potion",
        "Flame Lance",
        "Forbidden Flame",
        "Fireball",
    ],
    "auchenaisoulpriest": [
        "Priest of the Feast",
        "Kabal Songstealer",
        "Shifting Shade",
    ],
    "leeroyjenkins": [
        "Argent Horserider",
        "Skycap'n Kragg",
        "Gnomeregan Infantry",
        "Charged Devilsaur",
    ],
    "coldlightseer": ["Primalfin Lookout", "Gentle Megasaur"],
    "pyroblast": [
        "Firelands Portal",
        "Meteor",
        "Flame Lance",
        "Forbidden Flame",
        "Fireball",
        "Roaring Torch",
    ],
    "druidoftheclaw": ["Druid of the Saber", "Shellshifter"],
    "harrisonjones": ["Gluttonous Ooze", "Big Game Hunter"],
    "questingadventurer": ["Edwin VanCleef", "Red Mana Wyrm"],
    "ancientofwar": ["Ancient of Blossoms", "Nesting Roc"],
    "baneofdoom": ["Blastcrystal Potion", "Kara Kazham!"],
    "blessingofmight": ["Seal of Champions", "Spikeridged Steed"],
    "bluegillwarrior": ["Bilefin Tidehunter", "Murloc Tinyfin"],
    "chillwindyeti": ["Infested Tauren", "Nesting Roc"],
    "coldblood": ["Tinker's Sharpsword Oil", "Shadow Sensei"],
    "deathwing": ["Deathwing, Dragonlord", "Nefarian"],
    "fireelemental": ["Fireguard Destroyer", "Thing from Below"],
    "frostelemental": ["Glacial Shard", "Frozen Crusher"],
    "innerrage": ["Blood To Ichor", "Revenge"],
    "seagiant": ["Frost Giant", "Arcane Giant"],
    "snaketrap": ["Cat Trick", "Bear Trap", "Venomstrike Trap", "Dart Trap"],
    "voodoodoctor": ["Mistress of Mixtures", "Friendly Bartender"],
    "blessingofwisdom": ["Solemn Vigil", "Small-Time Recruits"],
    "bootybaybodyguard": ["Infested Tauren", "Nesting Roc"],
    "crueltaskmaster": ["Blood To Ichor", "Ravaging Ghoul"],
    "explosiveshot": ["Grievous Bite", "Powershot"],
    "madbomber": ["Flame Juggler", "Huge Toad"],
    "massdispel": ["Purify", "Excavated Evil"],
    "mindvision": ["Crystalline Oracle", "Curious Glimmerroot"],
    "nozdormu": ["Chromaggus", "Nefarian"],
    "ragnarosthefirelord": ["Nefarian", "Chromaggus"],
    "stampedingkodo": ["Big Game Hunter", "Kooky Chemist"],
    "sylvanaswindrunner": ["Sneed's Old Shredder", "Infested Tauren"],
}


def infer_archetype(title: str) -> str:
    value = normalize(title)
    patterns = (
        ("exodia", "exodia_quest"),
        ("quest", "quest"),
        ("kazakus", "highlander"),
        ("highlander", "highlander"),
        ("pirate", "pirate"),
        ("murloc", "murloc"),
        ("evolve", "evolve"),
        ("silence", "silence"),
        ("hadronox", "hadronox_taunt"),
        ("nzoth", "nzoth_deathrattle"),
        ("discard", "discard"),
        ("handlock", "handlock"),
        ("zoo", "zoo"),
        ("miracle", "miracle"),
        ("freeze", "freeze"),
        ("jade", "jade"),
        ("secret", "secret"),
        ("elemental", "elemental"),
        ("token", "token"),
        ("taunt", "taunt"),
        ("control", "control"),
        ("aggro", "aggro"),
        ("midrange", "midrange"),
        ("tempo", "tempo"),
        ("ramp", "ramp"),
        ("beast", "beast"),
    )
    return next(
        (archetype for marker, archetype in patterns if marker in value), "other"
    )


def infer_strategy(title: str) -> str:
    value = normalize(title)
    if any(marker in value for marker in ("quest", "exodia", "miracle", "silence")):
        return "combo"
    if any(
        marker in value
        for marker in (
            "control",
            "kazakus",
            "highlander",
            "freeze",
            "nzoth",
            "hadronox",
            "ramp",
        )
    ):
        return "control"
    if any(marker in value for marker in ("aggro", "zoo", "pirate", "token")):
        return "aggro"
    if any(marker in value for marker in ("tempo", "secret")):
        return "tempo"
    return "midrange"


def simple_bot_candidate(title: str) -> bool:
    strategy = infer_strategy(title)
    value = normalize(title)
    direct_markers = (
        "aggro",
        "zoo",
        "pirate",
        "token",
        "tempo",
        "secret",
        "midrange",
        "murloc",
        "evolve",
        "elemental",
        "beast",
        "divineshield",
    )
    excluded = ("quest", "exodia", "miracle", "silence", "discard", "jade")
    return (
        strategy in {"aggro", "tempo", "midrange"}
        and any(marker in value for marker in direct_markers)
        and not any(marker in value for marker in excluded)
    )


def slug_from_url(url: str) -> str:
    slug = unquote(urlparse(url).path.rstrip("/").split("/")[-1])
    slug = unicodedata.normalize("NFKD", slug).encode("ascii", "ignore").decode()
    return re.sub(r"[^a-z0-9]+", "_", slug.lower()).strip("_")


def normalize(value: str) -> str:
    value = unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode()
    return re.sub(r"[^a-z0-9]", "", value.lower())


def fetch(url: str) -> str:
    request = urllib.request.Request(
        url, headers={"User-Agent": "hearth-rs deck importer"}
    )
    for attempt in range(3):
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return response.read().decode("utf-8", "replace")
        except (OSError, TimeoutError):
            if attempt == 2:
                raise
            time.sleep(0.5 * (attempt + 1))
    raise AssertionError("unreachable")


def index_links(page: str) -> list[tuple[str, str]]:
    links = []
    for url, label in re.findall(
        r'<a[^>]+href="([^"]+/decks/[^"]+)"[^>]*>(.*?)</a>', page, re.DOTALL
    ):
        label = html.unescape(re.sub(r"<[^>]+>", "", label)).strip()
        if label and url not in {candidate for candidate, _ in links}:
            links.append((url, label))
    return links


def page_cards(page: str) -> list[tuple[str, int]]:
    cards = []
    for name, count in re.findall(
        r'<span class="card-name">(.*?)</span>.*?'
        r'<span class="card-count">(\d+)</span>',
        page,
        re.DOTALL,
    ):
        cards.append((html.unescape(re.sub(r"<[^>]+>", "", name)).strip(), int(count)))
    return cards


def card_class(page: str) -> str:
    match = re.search(r"Class:.*?>([A-Za-z]+)</a>", page, re.DOTALL)
    if not match:
        raise ValueError("deck page has no class")
    return match.group(1).lower()


def legal_for(card: dict[str, Any], card_class_name: str) -> bool:
    classes = [str(value).lower() for value in card.get("classes", [])]
    single = str(card.get("cardClass", "neutral")).lower()
    class_legal = (
        "neutral" in classes or card_class_name in classes
        if classes
        else single in {"neutral", card_class_name}
    )
    return (
        bool(card.get("collectible", False))
        and card.get("set") in FROZEN_THRONE_SETS
        and class_legal
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=Path("decks/frozen_throne"))
    parser.add_argument("--workers", type=int, default=8)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    local_cards = json.loads(
        (root / "data/hearthstonejson/selected.enUS.json").read_text(encoding="utf-8")
    )
    local_by_name = {
        normalize(card["name"]): card
        for card in local_cards
        if card.get("collectible", False) and card.get("set") in FROZEN_THRONE_SETS
    }

    indexes = {url: index_links(fetch(url)) for url in {spec.index for spec in SPECS}}
    overrides: dict[str, ImportSpec] = {}
    for spec in SPECS:
        matches = [
            (url, title)
            for url, title in indexes[spec.index]
            if normalize(spec.title_fragment) in normalize(title)
        ]
        if len(matches) != 1:
            raise ValueError(f"{spec.slug}: expected one source link, found {matches}")
        overrides[matches[0][0]] = spec

    sources: list[tuple[ImportSpec, str, str]] = []
    seen_urls: set[str] = set()
    for index_url in (KFT_INDEX, HCT_INDEX):
        for source_url, source_title in indexes[index_url]:
            if source_url in seen_urls:
                continue
            seen_urls.add(source_url)
            spec = overrides.get(source_url)
            if spec is None:
                spec = ImportSpec(
                    slug=slug_from_url(source_url),
                    index=index_url,
                    title_fragment=source_title,
                    archetype=infer_archetype(source_title),
                    strategy=infer_strategy(source_title),
                    bc_eligible=simple_bot_candidate(source_title),
                )
            sources.append((spec, source_url, source_title))

    slugs = [spec.slug for spec, _, _ in sources]
    if len(slugs) != len(set(slugs)):
        duplicates = [slug for slug, count in Counter(slugs).items() if count > 1]
        raise ValueError(f"duplicate output slugs: {duplicates}")

    urls = [source_url for _, source_url, _ in sources]
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        pages = dict(zip(urls, executor.map(fetch, urls), strict=True))

    output = root / args.output
    output.mkdir(parents=True, exist_ok=True)
    written: set[Path] = set()
    skipped: list[tuple[str, str]] = []

    for spec, source_url, source_title in sources:
        page = pages[source_url]
        source_cards = page_cards(page)
        if sum(count for _, count in source_cards) != 30:
            skipped.append((source_url, "source page does not contain 30 cards"))
            continue
        try:
            deck_class = card_class(page)
        except ValueError as error:
            skipped.append((source_url, str(error)))
            continue
        counts: Counter[str] = Counter()
        playable_cards: list[str] = []
        replacements: list[dict[str, Any]] = []

        for source_name, source_count in source_cards:
            source_key = normalize(source_name)
            local = local_by_name.get(source_key)
            if local is not None and legal_for(local, deck_class):
                playable_cards.extend([local["id"]] * source_count)
                counts[local["id"]] += source_count
                continue

            options = SUBSTITUTIONS.get(source_key, [])
            for _ in range(source_count):
                selected = None
                for replacement_name in options:
                    candidate = local_by_name.get(normalize(replacement_name))
                    if candidate is None or not legal_for(candidate, deck_class):
                        continue
                    maximum = (
                        1
                        if str(candidate.get("rarity", "")).lower() == "legendary"
                        else 2
                    )
                    if counts[candidate["id"]] < maximum:
                        selected = candidate
                        break
                if selected is None:
                    raise ValueError(
                        f"{spec.slug}: no runnable replacement for {source_name!r}; "
                        f"tried {options}"
                    )
                playable_cards.append(selected["id"])
                counts[selected["id"]] += 1
                replacements.append(
                    {
                        "source": source_name,
                        "replacement": selected["name"],
                        "replacement_id": selected["id"],
                    }
                )

        original = [
            {"name": source_name, "count": count} for source_name, count in source_cards
        ]

        adapted = bool(replacements)
        # The simple heuristic bot is useful for direct tempo-oriented decks,
        # but not for plans whose value depends on a quest or combo sequence.
        bc_eligible = spec.bc_eligible

        value = {
            "name": (
                f"KFT 2017 runtime-adapted: {source_title}"
                if adapted
                else f"KFT 2017: {source_title}"
            ),
            "class": deck_class,
            "hero_power": HERO_POWERS[deck_class],
            "format": "wild_through_icecrown",
            "source_format": (
                "wild_through_icecrown"
                if "wild" in normalize(source_title)
                else "year_of_the_mammoth_standard"
            ),
            "era_cutoff": "ICECROWN",
            "archetype": spec.archetype,
            "strategy": spec.strategy,
            "bc_eligible": bc_eligible,
            "source": source_url,
            "source_collection": (
                "kft_deck_index"
                if spec.index == KFT_INDEX
                else "hct_americas_summer_playoffs_2017"
            ),
            "source_name": source_title,
            "source_cards": original,
            "adapted": adapted,
            "adaptation_ratio": len(replacements) / 30,
            "substitutions": replacements,
            "cards": playable_cards,
        }
        destination = output / f"{spec.slug}.json"
        destination.write_text(
            json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        written.add(destination)

    for stale in output.glob("*.json"):
        if stale not in written:
            stale.unlink()
    print(
        f"wrote {len(written)} runnable decks from {len(sources)} source pages; "
        f"skipped {len(skipped)} malformed pages"
    )
    for source_url, reason in skipped:
        print(f"skip: {source_url}: {reason}")


if __name__ == "__main__":
    main()
