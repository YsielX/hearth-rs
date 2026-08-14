# Architecture

[English](ARCHITECTURE.md) | [简体中文](zhCN/ARCHITECTURE.md) | [繁體中文](zhTW/ARCHITECTURE.md)

## Design constraints

1. Rust is the sole authority for mutable game state.
2. Rust code does not branch on card IDs or keyword IDs to implement card rules.
3. Lua reads snapshots and emits effect descriptions; Rust validates and commits them atomically.
4. A seed, card-pack fingerprint, and command sequence must reproduce the same state.
5. A new card that composes existing rules and primitives requires only a Lua file.
6. English is the fallback language; runtime display localization is explicit and deterministic.

## Layers

```text
CLI / future protocol
        │ PlayerCommand / legal_actions
        ▼
Authoritative Rust Game state machine
  ├─ entities, zones, mana, turns, combat, damage, death checkpoints
  ├─ PendingEvent / ResolutionItem queue
  ├─ deterministic RNG, rollback, replay, snapshot
  └─ generic named rule queries and atomic EffectSpec execution
        │ read-only GameState + hook arguments
        ▼
LuaCardRuntime
  ├─ card modules: targets, lifecycle hooks, triggers, auras, actions
  ├─ Hero Power modules: one independently loaded module per power
  ├─ keyword modules: rules, hooks, triggers, actions, contracts
  └─ ctx read APIs + EffectSpec output buffer + locale selection
        │
        ▼
data/sets/**/*.lua + data/hero_powers/**/*.lua + data/keywords/*.lua + data/libraries/*.lua + data/locales/*.json
```

## Lua module types

A card module returns a table whose default `module_type` is `card`. It contains immutable official metadata and card-specific hooks.

A Hero Power module declares `module_type = "hero_power"`. The loader supplies the non-collectible `hero_power` type, while the module owns its cost, targets, `on_play`, triggers, tokens, and keyword references. Hero cards remain card modules with `type = "hero"`, `armor`, and a validated `hero_power` ID.

A shared Lua library declares `module_type = "library"`, `api_version = 1`, and an `id`. It is exposed as `cardlib[id]`, participates in the pack hash, and is not registered as a card. Libraries compose generic context operations; they do not add card-specific Rust effects.

A keyword module declares `module_type = "keyword"`:

```lua
return {
    api_version = 1,
    module_type = "keyword",
    id = "taunt",
    name = "Taunt",
    rules = {
        attack_priority = function(ctx, self, current, attacker)
            return math.max(current, 1)
        end,
    },
}
```

The loader rejects unknown keyword references. Card and keyword sources both contribute to the card-pack hash, so a rules change invalidates incompatible replays.

Keyword modules can use:

- `rules` for read-only rule folding;
- `hooks.on_play` and `hooks.on_location_use` for lifecycle behavior;
- `triggers` for before/after event listeners;
- `actions` for named player actions in a zone;
- `required_card_hooks`, `required_card_actions`, and `required_card_fields` for load-time contracts;
- `requires_param = true` for an integer in `keyword_params`.

For example, Battlecry owns timing and calls the card's required `on_battlecry`; Overload reads its numeric parameter and emits a generic mana-debt effect; Forge declares a 2-Mana hand action and requires `action_effects.forge` from the card.

## Rule folding, not keyword branches

When Rust needs a decision, it asks for a generic rule name and folds every active module in stable order:

| Rule | Initial value | Purpose |
| --- | ---: | --- |
| `attack_priority` | `0` | Higher-priority defenders mask lower-priority defenders |
| `can_be_attacked` | `true` | Whether a character may be attacked |
| `can_be_targeted` | `true` | Whether any targeted effect may select the entity |
| `can_be_targeted_by_enemy` | `true` | Enemy-specific target protection |
| `can_attack_while_exhausted` | `false` | Rush-style conditional attacks |
| `ready_on_summon` | `false` | Charge-style summon readiness |
| `max_attacks` | `1` | Attacks allowed each turn |
| `can_trade` | `false` | Expose the Trade player action |
| `can_play` | `true` | Whether a hand entity can be played |
| `can_attack` | `true` | Whether a character can attack |
| `enters_secret_zone` | `false` | Persistent Secret/Quest placement |
| `starts_in_opening_hand` | `false` | Forced opening-hand placement |
| `hero_power_is_passive` | `false` | Disable active Hero Power use |
| `can_magnetize` | `false` | Expose adjacent Mech merge positions |
| `base_spell_damage` | `0` | Printed Spell Damage layer |

Rust knows these interfaces, not the keyword IDs that provide them. Weapon modules participate in hero combat rules only when `weapon_inherits_to_hero = true`.

## Events and resolution

Responsive actions create a pending event:

```text
create before event
  → collect APNAP card and keyword triggers
  → execute emitted EffectSpec items
  → commit or cancel the pending event
  → append the committed event to the log
  → publish after triggers
  → run a death checkpoint
```

Lua cannot mutate `GameState`. `ctx:damage(target, 3)` only appends an `EffectSpec::Damage`. A player command executes against a temporary transaction; a script error, illegal target, or failed invariant rolls back the entire command.

Grouped damage collects every `damaged/before`, commits all uncancelled damage against one stable state, publishes the after group, then performs one death checkpoint.

Named continuations and pending choices are serializable. The engine stores a source entity, hook name, typed payload, options, and remaining queue—not a Lua closure or coroutine.

## Targeting

Lua selectors (`targets`, `location_targets`, or action target selectors) produce candidates. Rust then applies generic entity-existence and keyword rule filters.

`target_mode` controls declaration:

- `required`: a legal target must exist and be selected;
- `required_if_available`: select when candidates exist, otherwise resolve with `nil`;
- `optional`: target selection may be omitted.

The chosen stable `EntityId` is frozen at declaration time. Pre-play triggers, controller changes, and aura updates do not rerun the selector. Atomic effects still verify the entity's current zone when they resolve.

## Keyword-owned behavior

Examples of rules implemented without keyword-specific Rust branches:

- Divine Shield cancels `damaged/before` and disables itself.
- Immune blocks attacks/targets and cancels incoming damage.
- Stealth blocks enemy attacks/targets and disables itself after attacking or dealing damage.
- Poisonous destroys a minion after positive actual damage.
- Lifesteal heals after positive actual damage.
- Deathrattle listens from the graveyard and continues to `on_deathrattle` with the remembered position.
- Reborn summons a fresh 1-Health copy without Reborn.
- Spellburst listens for a friendly successful spell cast and consumes itself before continuing.
- Overload reads a Lua parameter and emits generic mana-debt effects.
- Spell Damage contributes a generic stat layer.
- Secret, Quest, Questline, and Sidequest provide persistent-zone rules.
- Forge, Prepare, Titan, and Tradeable expose generic replayable actions.
- Magnetic merges printed stats, enchantments, keywords, attached scripts, and Deathrattles; Silence removes the silenciable merge layer.

Effect words such as Discover, Adapt, Shatter, and Miniaturize still need card-specific pools, values, or official token IDs. Their keyword modules own shared timing and enforce the required Lua payload contract. They are not string markers interpreted by Rust.

## State and data

`CardDefinition` is immutable Lua-authored metadata. Each runtime instance is a Rust `Entity` containing owner/controller, zone, base and current stats, damage, armor, enchantments, disabled keywords, frozen timing, attack counters, attached Magnetic card IDs, and serializable `script_data`.

Per-player counters use `PlayerState.script_data`; card instance counters use `Entity.script_data`. Lua module globals must never store match state.

History arrays store card definition IDs at the authoritative event time. Transforming an entity later does not rewrite history.

Returning an entity to a hidden zone clears board damage, Silence, enchantments, frozen state, attachments, and instance script data according to the generic zone-reset policy.

## Localization

Lua fallback `name` and `text` values are English. Official display text is merged from `data/locales/<locale>.json` by card ID. `LuaCardRuntime::load_dir` defaults to English; `load_dir_with_locale` selects an explicit display locale.

Dynamic Lua prompts use:

```lua
ctx:localize("English", "简体中文", "繁體中文")
```

The selected locale is immutable for a runtime and does not enter gameplay decisions. Locale catalogs and the selected locale suffix participate in compatibility fingerprints where appropriate.

## Sandbox and determinism

- `dofile`, `loadfile`, `require`, `package`, `io`, `os`, and `debug` are unavailable.
- `math.random` and `math.randomseed` are unavailable.
- Lua memory is limited to 16 MiB.
- A hook has an approximately 200,000-instruction budget.
- One player command may resolve at most 10,000 effects.
- Random sampling uses Rust's seeded RNG and is logged.
- Replay records initial decks, classes, Hero Powers, seed, card-pack hash, and successful commands.
- Snapshot embeds replay and verifies the reconstructed authoritative state.

## Extension rule

First compose existing `ctx` APIs, named rules, triggers, choices, and actions. If a new mechanic cannot be expressed, add a reusable named rule or atomic `EffectSpec`. Never add `if card_id == ...` or `if keyword == ...` business logic to Rust.
