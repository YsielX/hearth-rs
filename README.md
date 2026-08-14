# hearth-rs

[English](README.md) | [简体中文](README.zhCN.md) | [繁體中文](README.zhTW.md)

A command-line Hearthstone rules prototype with an authoritative Rust engine and a Lua card/keyword layer.

The central design goal is simple: adding a card should require only a Lua file—no Rust card ID registration and no Rust branch for a card name or keyword.

## Repository layout

```text
data/
├── hero_powers/           # One standalone Lua module per Hero Power
├── hearthstonejson/       # Source metadata snapshots for implemented definitions
├── keywords/              # Reusable Lua keyword modules
├── locales/               # Official enUS / zhCN / zhTW names and card text
└── sets/                  # Official cards grouped by HearthstoneJSON set
crates/
├── hearth-core/           # State machine, zones, event queue, RNG, replay
├── hearth-script/         # Lua sandbox, module loader, rules/effect bridge
└── hearth-cli/            # Two-player hot-seat CLI
decks/demo.json            # Mixed-class mechanics showcase
decks/quest_rogue.json     # Dog's 2017 Caverns Quest Rogue
```

Rust owns state that scripts must not mutate directly: entity identity, zone containers, mana payment, combat and damage commits, death checkpoints, the resolution queue, pending input, deterministic randomness, transaction rollback, snapshots, and replay.

Lua owns card and keyword semantics: target selection, Battlecries, Deathrattles, Secrets, Discover pools, triggers, effects, and rule modifiers. The engine does not implement keyword behavior with branches such as `if keyword == "taunt"`.

## Official cards

The repository currently contains 150 official collectible/token/Hero Power definitions across 43 sets. This includes all 11 basic class Hero Powers and all nine Knights of the Frozen Throne Hero cards with their replacement Hero Powers. It also implements a playable historical Caverns Quest Rogue list. This is a representative rules corpus, not the complete Hearthstone card pool.

Official IDs, stats, sets, names, and card text come from HearthstoneJSON client data. The selected English source snapshot is [data/hearthstonejson/selected.enUS.json](data/hearthstonejson/selected.enUS.json); translated display catalogs are under [data/locales](data/locales). See the [data provenance notes](data/hearthstonejson/README.md).

## Localization

English is the default everywhere:

- `LuaCardRuntime::load_dir` uses English fallback `name` and `text` values embedded in Lua;
- the CLI defaults to `enUS` when `--locale` is omitted;
- README and `docs/` use English as their canonical language.

The CLI accepts `enUS`, `zhCN`, and `zhTW`. It localizes card names/text, command help, state labels, event messages, errors, and Lua choice prompts. A deck name is user-authored metadata and is always displayed exactly as its single `name` value.

```bash
cargo run -p hearth-cli -- --locale enUS
cargo run -p hearth-cli -- --locale zhCN
cargo run -p hearth-cli -- --locale zhTW
```

Lua card definitions may localize a dynamic prompt without storing locale state:

```lua
local prompt = ctx:localize(
    "Discover a spell",
    "发现一张法术牌",
    "發現一張法術牌"
)
ctx:discover_cards(player, prompt, candidates, 3, "on_discovered")
```

The maintenance script [scripts/sync_lua_fallbacks.py](scripts/sync_lua_fallbacks.py) synchronizes Lua fallback names/text and the selected English metadata snapshot from the locale catalogs. Use [scripts/import_official_card_data.py](scripts/import_official_card_data.py) to refresh the checked-in subset for every implemented card and Hero Power from HearthstoneJSON.

## Keywords are Lua modules

A card references reusable keyword modules:

```lua
return {
    api_version = 1,
    id = "GVG_085",
    name = "Annoy-o-Tron",
    text = "<b>Taunt</b>\n<b>Divine Shield</b>",
    set = "GVG",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 2,
    keywords = { "taunt", "divine_shield" },
}
```

`taunt.lua` contributes the generic `attack_priority` rule. `divine_shield.lua` listens to `damaged/before`, disables itself, and cancels the pending damage. Neither behavior is keyed by keyword ID in Rust.

All 68 functional Constructed keywords in the current audit have Lua modules. Rule keywords directly fold rules or listen to events. Effect words own their shared timing and require card-specific payload hooks, parameters, or actions at load time. See the [keyword coverage matrix](docs/KEYWORDS.md).

Player-facing keyword actions are also generic. Tradeable folds `can_trade`; Forge, Prepare, and Titan abilities expose named card actions. The CLI uses:

```text
trade <entity-id>
action <entity-id> <action-id> [target-id]
```

## Adding a card

Add a Lua file under `data/sets/<set>/`. For example:

```lua
return {
    api_version = 1,
    id = "MY_SET_001",
    name = "Example Spell",
    text = "Deal 3 damage.",
    set = "MY_SET",
    type = "spell",
    cost = 2,
    target_mode = "required",

    targets = function(ctx, self)
        return ctx:enemy_characters(self)
    end,

    on_play = function(ctx, self, target)
        ctx:damage(target, 3)
    end,
}
```

Restart the process and the recursive loader discovers it. As long as the effect can be expressed with existing rules, events, choices, and effect primitives, no Rust change is needed. See the [Lua card API](docs/CARD_API.md).

## Running a game

Rust 1.88 or newer is required. Lua 5.4 is built through `mlua`'s `vendored` feature.

```bash
cargo run -p hearth-cli -- \
  --deck-one decks/demo.json \
  --deck-two decks/demo.json \
  --seed 42
```

Run the historical Quest Rogue mirror:

```bash
cargo run -p hearth-cli -- \
  --deck-one decks/quest_rogue.json \
  --deck-two decks/quest_rogue.json \
  --locale enUS \
  --seed 42
```

Each player enters `keep` or `mulligan ...` to finish the opening mulligan. Common commands:

```text
state                         show the board and player state
hand                          show the active player's hand
cards                         list the loaded card pack
legal                         list legal commands
targets <card>                list legal targets
play <card> [target]          play a card
playat <card> <pos> [target]  play a minion at a board position
trade <card>                  Trade a Tradeable card for 1 Mana
action <card> <id> [target]   Forge, Prepare, or use a Titan ability
attack <attacker> <target>    attack
power [target]                use the Hero Power
location <location> [target]  use a Location
choose <index>                answer a pending choice
end                           end the turn
save <file>                   save a replay
snapshot <file>               save a state snapshot
```

Normal deck files enforce class/Neutral legality. Tourist cards declare generic `deck_allowances` in Lua. `demo.json` explicitly sets `unrestricted: true` because it intentionally mixes classes to showcase mechanics.

## Verification

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

End-to-end tests compare every Lua definition with the English source snapshot, require all three official locales, lock the 68-keyword catalog, exercise rules and card actions, perform randomized legal-action walks, and verify replay/snapshot determinism.

## Scope

This remains a rules prototype, not a complete Hearthstone server. The keyword layer covers Constructed, but the card library is a representative official subset. Battlegrounds and Mercenaries mode-specific keywords are outside this CLI ruleset. When a future mechanic cannot be expressed through existing generic boundaries, extend a reusable rule or atomic effect—never branch on a specific card or keyword ID.
