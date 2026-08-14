#!/usr/bin/env python3
"""Synchronize Lua card fallback name/text literals with the enUS catalog.

The fallback embedded in each Lua definition is intentionally English. Runtime
locale catalogs remain authoritative for enUS, zhCN, and zhTW display text.
"""

import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def quoted(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def main() -> None:
    en = {item["id"]: item for item in json.loads((ROOT / "data/locales/enUS.json").read_text())}
    zh = {item["id"]: item for item in json.loads((ROOT / "data/locales/zhCN.json").read_text())}
    replacements: dict[str, str] = {}
    conflicts: set[str] = set()
    for card_id, localized in zh.items():
        english = en[card_id]
        for field in ("name", "text"):
            old = quoted(localized.get(field, ""))
            new = quoted(english.get(field, ""))
            if old == new:
                continue
            previous = replacements.get(old)
            if previous is not None and previous != new:
                conflicts.add(old)
            else:
                replacements[old] = new
    for conflict in conflicts:
        replacements.pop(conflict, None)

    changed = 0
    card_paths = list((ROOT / "data/sets").rglob("*.lua"))
    card_paths.extend((ROOT / "data/hero_powers").rglob("*.lua"))
    for path in sorted(card_paths):
        source = path.read_text()
        updated = source
        for old, new in replacements.items():
            updated = updated.replace(old, new)
        if updated != source:
            path.write_text(updated)
            changed += 1
    keyword_names = {
        path.stem: " ".join(word.capitalize() for word in path.stem.split("_"))
        for path in (ROOT / "data/keywords").glob("*.lua")
    }
    keyword_names.update(
        {
            "casts_when_drawn": "Casts When Drawn",
            "choose_one": "Choose One",
            "choose_multiple": "Choose Multiple",
            "conditional_charge": "Conditional Charge",
            "divine_shield": "Divine Shield",
            "honorable_kill": "Honorable Kill",
            "mega_windfury": "Mega-Windfury",
            "spell_damage": "Spell Damage",
            "start_of_game": "Start of Game",
            "summoned_when_drawn": "Summoned When Drawn",
        }
    )
    keyword_changed = 0
    for path in sorted((ROOT / "data/keywords").glob("*.lua")):
        source = path.read_text()
        updated, count = re.subn(
            r'(\bname\s*=\s*)"[^"]*"',
            lambda match: match.group(1) + quoted(keyword_names[path.stem]),
            source,
            count=1,
        )
        if count and updated != source:
            path.write_text(updated)
            keyword_changed += 1
    selected_path = ROOT / "data/hearthstonejson/selected.zhCN.json"
    selected = json.loads(selected_path.read_text())
    for record in selected:
        localized = en[record["id"]]
        record["name"] = localized["name"]
        record["text"] = localized.get("text", "")
    (ROOT / "data/hearthstonejson/selected.enUS.json").write_text(
        json.dumps(selected, ensure_ascii=False, indent=2) + "\n"
    )
    print(
        f"updated {changed} card files and {keyword_changed} keyword files; "
        f"skipped {len(conflicts)} ambiguous literals"
    )


if __name__ == "__main__":
    main()
