#!/usr/bin/env python3
"""Refresh the checked-in HearthstoneJSON subset for implemented Lua modules."""

import json
import re
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
API = "https://api.hearthstonejson.com/v1/latest/{locale}/cards.json"
CARD_ROOTS = (ROOT / "data" / "sets", ROOT / "data" / "hero_powers")


def implemented_ids() -> list[str]:
    ids: set[str] = set()
    pattern = re.compile(r'\bid\s*=\s*"([^"]+)"')
    for card_root in CARD_ROOTS:
        for path in card_root.rglob("*.lua"):
            ids.update(pattern.findall(path.read_text()))
    return sorted(ids)


def download(locale: str) -> dict[str, dict]:
    request = urllib.request.Request(
        API.format(locale=locale), headers={"User-Agent": "hearth-rs card importer"}
    )
    with urllib.request.urlopen(request) as response:
        records = json.load(response)
    return {record["id"]: record for record in records}


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def main() -> None:
    ids = implemented_ids()
    catalogs = {locale: download(locale) for locale in ("enUS", "zhCN", "zhTW")}
    for locale, catalog in catalogs.items():
        missing = [card_id for card_id in ids if card_id not in catalog]
        if missing:
            raise SystemExit(f"{locale} is missing official IDs: {', '.join(missing)}")
        localized = [
            {
                "id": card_id,
                "name": catalog[card_id]["name"],
                "text": catalog[card_id].get("text", ""),
            }
            for card_id in ids
        ]
        write_json(ROOT / "data" / "locales" / f"{locale}.json", localized)

    write_json(
        ROOT / "data" / "hearthstonejson" / "selected.enUS.json",
        [catalogs["enUS"][card_id] for card_id in ids],
    )
    write_json(
        ROOT / "data" / "hearthstonejson" / "selected.zhCN.json",
        [catalogs["zhCN"][card_id] for card_id in ids],
    )
    print(f"refreshed {len(ids)} implemented official definitions")


if __name__ == "__main__":
    main()
