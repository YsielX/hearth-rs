# Card data provenance

[English](README.md) | [简体中文](zhCN/README.md) | [繁體中文](zhTW/README.md)

`selected.enUS.json` is the source snapshot for the 248 implemented official definitions as of 2026-08-14. It is derived from:

```text
https://api.hearthstonejson.com/v1/latest/enUS/cards.json
```

`selected.zhCN.json` preserves the corresponding Simplified Chinese source records. HearthstoneJSON provides the original data. This repository retains only implemented data/tokens and relevant fields such as ID, dbfId, name, text, set, type, class, stats, tribes, spell schools, and mechanics instead of committing the complete multi-megabyte dataset.

Every Lua definition under `data/sets/` and `data/hero_powers/` must resolve to an official ID in the selected snapshot. End-to-end tests compare metadata one by one.

`data/locales/enUS.json`, `zhCN.json`, and `zhTW.json` come from the same client-data version and retain `id`, `name`, and `text`. Tests require every implemented ID to have a non-empty name in all three catalogs.

The canonical Lua fallback is English. After refreshing locale catalogs and the selected zhCN records, run:

```bash
python3 scripts/sync_lua_fallbacks.py
```

The script synchronizes each Lua `name`/`text` fallback with `enUS.json`, normalizes keyword display names to English, and regenerates `selected.enUS.json` while preserving non-display metadata.

HearthstoneJSON distinguishes `cards.json` from collectible-only `cards.collectible.json`. Tokens and other non-collectible definitions require the complete `cards.json` source.
