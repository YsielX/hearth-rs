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
├── hearth-app/            # Shared match sessions, deck services, and presentation text
├── hearth-cli/            # `play` and `fuzz` commands
├── hearth-client-bevy/    # Bevy 0.19 native graphical client
├── hearth-bot/            # Non-cheating deterministic baseline Bot
└── hearth-fuzz/           # State-machine fuzzing library (no binary)
decks/demo.json            # Mixed-class mechanics showcase
decks/quest_rogue.json     # Dog's 2017 Caverns Quest Rogue
decks/frozen_throne/       # 354 sourced 2017 decks plus runnable adaptations
```

Rust owns state that scripts must not mutate directly: entity identity, zone containers, mana payment, combat and damage commits, death checkpoints, the resolution queue, pending input, deterministic randomness, transaction rollback, snapshots, and replay.

Lua owns card and keyword semantics: target selection, Battlecries, Deathrattles, Secrets, Discover pools, triggers, effects, and rule modifiers. The engine does not implement keyword behavior with branches such as `if keyword == "taunt"`.

`hearth-app` is the UI-independent application layer used by both local clients. Its controller-neutral `MatchSession` owns runtime/deck construction, sideboards, seeded opening order, replay/snapshot access, and public projections. The managed `GameSession` adds hotseat and Bot policy. Shared localized event/action text and timeout policy also live there; terminal I/O and Bevy ECS/rendering remain frontend-specific.

## Official cards

The repository currently contains 1,999 official collectible/token/Hero/Hero Power definitions across 46 sets. This includes the complete 30-card Curse of Naxxramas, 123-card Goblins vs Gnomes, 31-card Blackrock Mountain, 132-card Grand Tournament, 45-card League of Explorers, 134-card Whispers of the Old Gods, 45-card One Night in Karazhan, 132-card Mean Streets of Gadgetzan, 135-card Journey to Un'Goro, 135-card Knights of the Frozen Throne, and 26-card Path of Arthas sets, all 11 canonical base Heroes and class Hero Powers, and at least one implemented official card for every tracked Constructed keyword. It also implements a playable historical Caverns Quest Rogue list. This is a representative rules corpus, not the complete Hearthstone card pool.

Official IDs, stats, sets, names, and card text come from HearthstoneJSON client data. The selected English source snapshot is [data/hearthstonejson/selected.enUS.json](data/hearthstonejson/selected.enUS.json); translated display catalogs are under [data/locales](data/locales). See the [data provenance notes](data/hearthstonejson/README.md).

## Localization

English is the default everywhere:

- `LuaCardRuntime::load_dir` uses English fallback `name` and `text` values embedded in Lua;
- the CLI defaults to `enUS` when `--locale` is omitted;
- README and `docs/` use English as their canonical language.

The CLI accepts `enUS`, `zhCN`, and `zhTW`. It localizes card names/text, command help, state labels, event messages, errors, and Lua choice prompts. A deck name is user-authored metadata and is always displayed exactly as its single `name` value.

```bash
cargo run -p hearth-cli -- play --locale enUS
cargo run -p hearth-cli -- play --locale zhCN
cargo run -p hearth-cli -- play --locale zhTW
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
        cardlib.effects.damage(ctx, target, 3)
    end,
}
```

Restart the process and the recursive loader discovers it. As long as the effect can be expressed with existing rules, events, choices, and effect primitives, no Rust change is needed. See the [Lua card API](docs/CARD_API.md).

## Running a game

Rust 1.95 or newer is required. Lua 5.4 is built through `mlua`'s `vendored` feature.

```bash
cargo run -p hearth-cli -- play \
  --deck-one decks/demo.json \
  --deck-two decks/demo.json \
  --seed 42
```

### Bevy graphical client

Run the native Bevy 0.19 client with:

```bash
cargo run -p hearth-client-bevy
```

The Settings screen switches language and default turn timing immediately, and now includes
windowed/borderless-fullscreen modes plus 80%, 100%, and 120% UI scaling. Press `F11` anywhere to
toggle borderless fullscreen. Settings are atomically stored in
`$XDG_CONFIG_HOME/hearth-rs/client.json` (or `~/.config/hearth-rs/client.json`) and restored on the
next launch; existing version-1/version-2 settings migrate without losing prior choices and default
the new AI difficulty to Normal. `--fullscreen`, `--windowed`, `--ui-scale 80|100|120`, and
`--bot-difficulty easy|normal|hard` override saved values, just as explicit `--locale` and
`--turn-seconds` do.

The deck selector and collection editor include a **Deck Code** screen. It exports a complete
constructed deck as a Hearthstone Deckstring and imports either a bare code or the multiline text
copied by the official client. Importing creates an unsaved custom-deck draft for review before
writing to `decks/custom/`. The importer validates format, Hero, implemented card dbfIds, class
legality, deck size, and copy limits. Sideboards are encoded with the official Deckstring extension
when their owning card supports them; cards not implemented by this repository are rejected with
an explicit error. E.T.C., Band Manager can build a three-card band in the editor, Discover one
member in play, and consumes that member for the rest of the game. Prince Renathal expands the
same editor and Deckstring path to 40 cards and establishes 40 starting Health before start-of-game
effects; replay and snapshot reconstruction derive both rules from the submitted deck.
Death Knight cards declare structured Blood, Frost, and Unholy rune costs in Lua. The application,
Deckstring path, authoritative game constructor, and E.T.C. sideboards all enforce the official
three-slot rule. The Bevy editor displays the deck's minimum commitment (for example, `BB-`) and
dynamically hides cards that would make the component-wise rune requirements exceed three slots.
Friendly minion deaths grant the Death Knight's public Corpse resource in simultaneous-death
batches; Reborn deaths count again, while marked tokens such as Risen Ghoul do not leave a Corpse.
Lua exposes exact, up-to, and atomic spend-and-continue operations. Body Bagger, Army of the Dead,
and Defrost exercise gain, capped spending, token exceptions, and conditional follow-up effects;
CLI, Bevy, public views, replay/snapshot state, and RL observations carry the same resource.

The TITANS Plague package implements Helya, Distressed Kvaldir, Down with the Ship, Tomb Traitor,
Staff of the Primus, Chained Guardian, all three draw-cast Plagues, and the Undead Peasant token.
Helya immediately reshuffles each successfully
drawn Plague, while burned Plagues do not cast; Frost Plagues stack their next-card surcharge and
cap the resulting cost at 10. Every initial and unending reshuffle contributes to Chained
Guardian's live hand discount. The persistent unending status is public in views, RL observations,
snapshots/replays, and the Bevy battlefield HUD.
Eulogizer implements both corpse-spending and its Forged corpse-gaining form; Northern Navigation
discovers and draws an actual spell entity from the deck before checking its official spell school;
Frozen Over locks only the opponent's two direct draws through the end of their next turn.
The Primus implements all three targeted or untargeted Titan abilities, discovers from the matching
printed Rune pool, and carries its stacked next-spell discount and Spell Damage through replay.
The Fall of Ulduar Death Knight mini-set package adds Runes of Darkness, Sickly Grimewalker, and
Sinister Soulcage, including conditional Corpse spending and copying the fully buffed minion state.
The complete eleven-card Path of Arthas Frost package covers simultaneous enemy-wide spell damage,
bounded Mana refresh, Freeze, consumable next-spell discounts, repeated random shots, hand-school
counting and copying, capped Corpse spending, and post-combat weapon triggers that survive final
durability.
The Path of Arthas Blood package adds Hematurge, Vicious Bloodworm, Blood Tap, Blood Boil,
Darkfallen Neophyte, Asphyxiate, and Nerubian Swarmguard. It covers hand-entity targeting,
conditional Corpse-powered hand buffs, filtered Rune Discover, tied highest-Attack removal, exact
state copies, and independently stacked Lifesteal infections that Silence can remove.
The remaining Path of Arthas Unholy and rune-free cards complete the 26-card set. Plague Strike,
Dark Transformation, Tomb Guardians, Unholy Frenzy, The Scourge, Corpse Bride, Malignant Horror,
and Frostmourne cover conditional summon-after-damage, tribe-filtered transformation, Corpse-paid
Reborn, ordered forced combat and fresh resummoning, replayable random board filling, scaling
corpseless tokens, end-turn state copies, and final-durability kill tracking.
The complete 13-card March of the Lich King Death Knight package adds Corpse Explosion's
wave-by-wave Corpse spending, persistent Mograine end-turn damage, infected Deathrattles,
hand-tracked spell casting, targeted Locations, and Frost Queen Sindragosa's Colossal wings and
Freeze destruction. Its remaining cards cover combat kill tracking, Corpse-powered hand and board
effects, filtered Rune Discover, deck-minion consumption, friendly Undead death triggers, and
whole-board destruction.

The deck selector can switch between the built-in AI and **Local Two Player**. Practice AI has
Easy, Normal, and Hard settings: Easy is deliberately naive, Normal uses lethal/Mana planning,
and Hard also mulligans expensive cards and prioritizes advantageous trades. Every policy sees
only the current player projection and legal-action list, remains deterministic, and survives
autosave/resume. Hot-seat matches
use separate Player 1/Player 2 decks and show a full-screen privacy handoff before the opening
mulligan, every turn, and any other input-player change. Hands and card previews remain hidden,
and the turn timer is paused, until the next player confirms they are ready. Use
`--hotseat` to select this mode from the command line; it can be combined with `--quick-start`.

Each new graphical match deterministically randomizes the first player from its seed. The first
player mulligans three cards; the second mulligans four and receives The Coin after both choices.
The client identifies first/second status during mulligan, hot-seat handoff, and play. Replay and
snapshot proofs store the opening order, while older files default compatibly to Player 1 first.

Each constructed class now starts with its canonical base Hero identity—from Garrosh through The
Lich King—while keeping the selected deck's Hero Power authoritative. Hero names and previews use
the active locale. Base portraits remain collectible metadata but are correctly excluded from the
main-deck pool; playable Hero cards remain deckable.

Long games follow the official constructed limit: after player one's 45th turn (game turn 89)
finishes resolving, an undecided match ends in a draw. Turn 90 is never started, so it cannot draw
a card or trigger start-of-turn effects. The result banner identifies this turn-limit draw in all
three supported languages.

Against the built-in AI, the opponent's legal actions play back one at a time with a short visible
delay instead of resolving the entire turn synchronously. The action panel shows when the opponent
is thinking, combat feedback completes between decisions, and every AI step is atomically
autosaved. Pausing or restarting during an AI turn resumes safely from the last completed action.

Press **Pause** or `Esc` during an active match to open the in-match menu. While it is open—or while
visiting Settings from it—the turn timer, AI playback, targeting guide, and combat presentation stop
advancing. The menu can resume the match, round-trip through Settings, save and return to the main
menu, or concede after a separate confirmation. Conceding always applies to the local player, even
if the AI currently owns input, and is recorded in the deterministic replay.

The match controls also expose the six familiar hero emotes: Thanks, Well Played, Greetings, Wow,
Oops, and Threaten. Localized speech bubbles close automatically and a short cooldown prevents
spam. The built-in AI gives a delayed deterministic reply; **Squelch Opponent** suppresses those
replies for the rest of that match. In hot-seat mode, squelch is private to each current viewer.

Unfinished matches are atomically autosaved after every legal action. Use **Pause** in a match and
**Continue** on the main menu, or restart the client and continue from the same verified state.
The private checkpoint defaults to `$XDG_STATE_HOME/hearth-rs/active-match.json` (or
`~/.local/state/hearth-rs/active-match.json`), is created with owner-only permissions on Unix, and
is removed when the match ends. It contains both hidden zones and is never exposed as a normal UI
export. Use `--resume PATH` for an isolated checkpoint or `--no-resume` to disable disk persistence;
hot-seat restoration always returns through the privacy handoff screen, which also has a safe
pause-to-menu action. **Abandon Saved Match** requires a second confirmation before deleting it.

The match view uses original embedded artwork for its painterly tavern board and identity-free
opponent card backs. Both assets are compiled into the executable, retain solid-color fallbacks
while textures load, and contain no extracted official game art or trademarks. Their generation
prompts and provenance are recorded in the [UI asset notes](crates/hearth-client-bevy/assets/ui/README.md).

Combat presentation consumes only the viewer-safe public event stream. Visible characters pulse
for attacks, impacts, healing, summoning, shields, and transformations; damage, healing, Armor,
blocked damage, Freeze, and destroyed entities receive short localized floating feedback. Attacks
and targeted effects also travel along color-coded source-to-target paths; non-board sources such
as Hero Powers resolve safely to their controller's hero. Lethal targets that have already left the
board fall back to a centered battlefield cue. Before an action is dispatched, targeted click and
drag interactions show a gold aiming arrow that turns red and snaps to a legal visible target; the
guide is cleared on action completion, scene changes, and hot-seat privacy handoffs. Selecting or
dragging a playable Minion or Location reveals every legal insertion gap on the friendly board.
The gap can be clicked or used as an exact drop target; if a Battlecry still needs a target, the
chosen placement remains active while the player aims at that target.

Run the historical Quest Rogue mirror:

```bash
cargo run -p hearth-cli -- play \
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

## Player controllers and hidden information

Each player slot independently accepts `interactive`, `bot`, or `fuzzer`:

```bash
cargo run -p hearth-cli --release -- play \
  --deck-one decks/quest_rogue.json \
  --deck-two decks/quest_rogue.json \
  --player-one interactive \
  --player-two bot
```

Controllers receive a player-facing projection and authoritative legal-action metadata, never raw `GameState`. The projection excludes deck order (including the viewer's own deck), opponent hand and ordinary Secret identities, script data, hidden aura sources, RNG state, and replay data. Public Quests, Questlines, and Sidequests remain visible as in the official game. CLI event output redacts opponent draws, generated hand cards, Secret names, hidden choices, and hidden random samples. Two-human hot-seat games use a terminal-clearing handoff screen. Authoritative replay/snapshot export is disabled during ordinary play; `--debug-state` explicitly enables this debugging capability.

The baseline [`hearth-bot`](crates/hearth-bot/README.md) prioritizes board lethal, plans currently legal plays to minimize unspent Mana, takes favorable trades, then attacks face. Taunt and other attack restrictions remain authoritative because the Bot selects only from engine-enumerated legal attacks.

## Reinforcement learning

The framework-neutral `hearth-env` adapter exposes player-safe observations,
structured choices, indexed legal actions, and public-event history. The optional
Python package adds card/Lua-aware encoders, behavior cloning, PPO actor-critic
self-play, parallel rollout workers, checkpoint leagues, evaluation, and
new-card checkpoint migration. See the [Chinese training guide](docs/RL_TRAINING.md).

## Verification

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

End-to-end tests compare every Lua definition with the English source snapshot, require all three official locales, lock the 68-keyword catalog, exercise rules and card actions, and verify replay/snapshot determinism.

## State-machine fuzzing

The deterministic state-machine fuzzer is implemented in the [`hearth-fuzz` library](crates/hearth-fuzz/README.md) and exposed only through the `hearth-cli` subcommand, while normal `cargo test` runs do not start fuzz campaigns. It generates class-legal decks, dispatches sampled legal actions, validates state after every step, and compares the final state with replay:

```bash
cargo run -p hearth-cli --release -- fuzz --seeds 100 --steps 180
```

## Scope

This remains a rules prototype, not a complete Hearthstone server. The keyword layer covers Constructed, but the card library is a representative official subset. Battlegrounds and Mercenaries mode-specific keywords are outside this CLI ruleset. When a future mechanic cannot be expressed through existing generic boundaries, extend a reusable rule or atomic effect—never branch on a specific card or keyword ID.
