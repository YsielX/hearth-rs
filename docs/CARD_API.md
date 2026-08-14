# Lua Card API (version 1)

[English](CARD_API.md) | [简体中文](zhCN/CARD_API.md) | [繁體中文](zhTW/CARD_API.md)

Every `.lua` file returns one table. The loader sandbox does not expose `io`, `os`, `package`, `require`, arbitrary native modules, or Lua randomness.

## Card metadata

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `api_version` | integer | yes | Must be `1` |
| `module_type` | string | no | Cards default to `card`; standalone Hero Powers use `hero_power`; keyword modules use `keyword` |
| `id` | string | yes | Globally unique official card ID |
| `name` | string | yes | English fallback display name |
| `text` | string | no | English fallback card text |
| `set` | string | official cards | HearthstoneJSON set code |
| `type` | string | card modules | `hero`, `minion`, `spell`, `weapon`, or `location`; implicit for Hero Power modules |
| `collectible` | boolean | no | Main cards default true; embedded tokens default false |
| `class` | string | no | Defaults to `neutral` |
| `rarity` | string | no | Printed rarity, normalized to lowercase for generation filters |
| `spell_school` | string | no | Printed spell school, normalized to lowercase for generation filters |
| `tags` | string[] | no | Tribes or pack-defined pool tags |
| `cost` | integer | yes | Base Mana Cost |
| `attack` | integer | minion/weapon | Base Attack |
| `health` | integer | minion/weapon/location | Health or Durability |
| `armor` | integer | hero | Armor gained when the Hero card is played |
| `hero_power` | string | hero | Official ID of the replacement Hero Power module |
| `keywords` | string[] | no | IDs under `data/keywords/`; validated at load time |
| `keyword_params` | table<string, integer> | no | Numeric keyword configuration |
| `deck_allowances` | table[] | no | Generic cross-class deck permissions; required by Tourist |
| `target_mode` | string | no | `optional`, `required`, or `required_if_available` |
| `secret` | boolean | legacy only | Compatibility field; new cards reference keyword `secret` |

English is the canonical fallback language. Official display catalogs live at `data/locales/enUS.json`, `zhCN.json`, and `zhTW.json`, keyed by official card ID. Missing supported locale entries fail tests.

`target_mode` behavior:

- `required`: the card cannot be used without selecting a legal target;
- `required_if_available`: select when candidates exist, otherwise resolve with `nil`;
- `optional`: selection may be omitted even when candidates exist.

Non-optional cards must define `targets` or `location_targets`. The legacy `requires_target = true` is accepted as `required`.

Selectors are validated when the player declares the action. The engine remembers the selected `EntityId`; later triggers, controller changes, and aura changes do not rerun the selector.

## Hero Power and Hero modules

Each Hero Power lives in its own file under `data/hero_powers/` and declares `module_type = "hero_power"`. Its `type = "hero_power"` and `collectible = false` metadata are supplied by the loader:

```lua
return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_08bp",
    name = "Fireblast",
    text = "<b>Hero Power</b>\nDeal $1 damage.",
    set = "LEGACY",
    class = "mage",
    cost = 2,
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
    on_play = function(ctx, self, target) ctx:damage(target, 1) end,
}
```

A collectible Hero card remains a card module with `type = "hero"`, `health`, `armor`, and `hero_power`. Playing it preserves current Health and damage, gains its Armor, replaces the Hero entity and Hero Power, and then resolves Lua lifecycle hooks. `hero_replaced` and `hero_power_replaced` are published as ordinary events.

## Dynamic localization

Card names/text come from the locale catalogs. A Lua-generated prompt uses the runtime locale:

```lua
local prompt = ctx:localize(
    "Discover a spell",
    "发现一张法术牌",
    "發現一張法術牌"
)
```

`ctx.locale` is the selected locale code. Locale is read-only and must not affect gameplay rules, candidate pools, randomness, or authoritative state.

## Keyword modules and contracts

A keyword file returns `module_type = "keyword"`:

```lua
return {
    api_version = 1,
    module_type = "keyword",
    id = "windfury",
    name = "Windfury",
    weapon_inherits_to_hero = true,
    rules = {
        max_attacks = function(ctx, self, current, other)
            return math.max(current, 2)
        end,
    },
}
```

Rule functions have signature `(ctx, self, current, other) -> value`. They are read-only; attempting to emit an effect is an error.

Keyword modules may also declare card-style `triggers`, lifecycle `hooks`, and named `actions`. Contracts prevent display-only keywords:

```lua
required_card_hooks = { "on_battlecry" }
required_card_actions = { "forge" }
required_card_fields = { "deck_allowances" }
requires_param = true
```

The loader rejects a referencing card that lacks the required hook, action effect, field, or `keyword_params[keyword_id]`.

Example numeric configuration:

```lua
keywords = { "overload" },
keyword_params = { overload = 2 },
```

The keyword reads it with `ctx:keyword_param(self, "overload")`.

## Card functions

```lua
targets(ctx, self) -> entity_id[]
on_play(ctx, self, target_or_nil)
on_battlecry(ctx, self, target_or_nil)
on_combo(ctx, self, target_or_nil)
on_finale(ctx, self)
location_targets(ctx, self) -> entity_id[]
on_location_use(ctx, self, target_or_nil)
```

Other named hooks are allowed when referenced by a keyword contract, trigger, action, choice, random continuation, or explicit `continue_with*` effect.

`self` and targets are stable integer `EntityId` values, not mutable Rust objects.

## Embedded tokens

A card can define official non-collectible derivatives in `tokens`:

```lua
return {
    api_version = 1,
    id = "FP1_002",
    name = "Haunted Creeper",
    set = "NAXX",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 2,
    keywords = { "deathrattle" },

    tokens = {
        {
            id = "FP1_002t",
            name = "Spectral Spider",
            set = "NAXX",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
        },
    },
}
```

Tokens inherit `api_version`, default to `collectible = false`, enter the global catalog, and participate in ID collision checks and the card-pack hash.

## Named card actions

Keyword modules and cards can expose replayable actions from a zone:

```lua
-- keyword module
actions = {
    forge = { active_zones = { "hand" }, cost = 2 },
}

-- card module
action_effects = {
    forge = function(ctx, self, spent, target)
        ctx:modify(self, {
            stat = "cost", operation = "add", value = -2,
        })
    end,
}
```

Cards may additionally declare `card_actions`, `action_targets`, and `action_target_modes`. Conditions are read-only. Effects receive the actual Mana spent and optional target. Forge, Prepare, and Titan abilities use this interface; Rust does not identify those keywords.

## Read-only context

```lua
ctx:turn()
ctx:active_player()
ctx:controller(entity)
ctx:opponent(player)
ctx:player(player)
ctx:entity(entity)
ctx:keyword_param(entity, keyword_id)

ctx:cards_played_this_turn(player)
ctx:cards_played_last_turn(player)
ctx:combo_active(entity)
ctx:outcast_active(entity)
ctx:entered_hand_this_turn(entity)

ctx:cards_played(player)
ctx:spells_cast(player)
ctx:minions_played(player)
ctx:weapons_played(player)
ctx:locations_played(player)
ctx:last_spell_cast(player)

ctx:hand(player)
ctx:deck(player)                 -- top to bottom
ctx:board(player)                -- minions and Locations, left to right
ctx:secrets(player)
ctx:graveyard(player)
ctx:characters()
ctx:minions()
ctx:enemy_characters(entity)
ctx:friendly_minions(entity)
ctx:adjacent_minions(entity)
ctx:board_position(entity)

ctx:card_ids()
ctx:collectible_cards()
ctx:card_definition(card_id)
ctx:get_data(entity, key)
ctx:get_player_data(player, key)
```

Entity snapshots include identity, definition, owner/controller, zone/type, Attack, current/max Health, damage, Armor, Cost, Spell Damage, keywords, Silence, Freeze, Location cooldown, enchantments, hand-entry/play context, and script data relevant to rules.

Player snapshots include class, hero, Hero Power, weapon, mana fields, overload fields, play histories/counts, fatigue, zone sizes, and Hero Power use.

Card definition snapshots returned by `ctx:card_definition` include `rarity` and `spell_school` when declared. `card_created` events include both the created `entity` and the creating effect's `source`, allowing generated-card mechanics to identify only their own output.

Returned arrays/tables are snapshots. Lua cannot mutate Rust containers. Scripts are trusted server rules and may query hidden zones; UI clients do not receive those values automatically.

## Effect output

These functions append validated effects; they do not mutate state during the Lua call:

```lua
ctx:damage(target, amount)
ctx:damage_all(targets, amount)
ctx:heal(target, amount)
ctx:gain_armor(player, amount)

ctx:overload(player, amount)
ctx:unlock_mana(player, amount)
ctx:clear_overload(player)
ctx:gain_temporary_mana(player, amount)
ctx:gain_mana_crystals(player, amount, filled)
ctx:destroy_mana_crystals(player, amount)

ctx:draw(player, count)
ctx:give_card(player, card_id)
ctx:give_card_at(player, card_id, position)
ctx:shuffle_card_into_deck(player, card_id)
ctx:discard(player, entity)
ctx:cast_spell(player, card_id, target_or_nil)
ctx:cast_drawn(card)

ctx:summon(player, card_id)
ctx:summon_at(player, card_id, position)
ctx:summon_copy(player, entity)
ctx:summon_copy_at(player, entity, position)
ctx:summon_fresh_copy(entity, position_or_nil, health, without_keywords)
ctx:summon_from_hand(card)
ctx:recruit(player, deck_entity)
ctx:recruit_at(player, deck_entity, position)

ctx:equip_weapon(player, card_id)
ctx:replace_hero_power(player, card_id)
ctx:refresh_hero_power(player)
ctx:give_merged_minion(player, template_card_id, first_card_id, second_card_id)
ctx:move(entity, destination)
ctx:change_controller(entity, player)
ctx:transform(entity, card_id)
ctx:destroy(entity)

ctx:buff(entity, attack_delta, health_delta)
ctx:buff_until_end_of_turn(entity, attack_delta, health_delta)
ctx:modify(entity, modifier_table)
ctx:remove_enchantments_from(entity, source)
ctx:grant_keyword(entity, keyword_id)
ctx:disable_keyword(entity, keyword_id)
ctx:silence(entity)
ctx:freeze(entity)

ctx:reveal_secret(secret)
ctx:cancel_event(event)
ctx:set_event_amount(event, amount)
ctx:replace_trade_draw(event_id, replacement_entity)
ctx:set_data(entity, key, value)
ctx:set_player_data(player, key, value)

ctx:continue_with(hook)
ctx:continue_with_entity(hook, entity)
ctx:continue_with_card(hook, card_id)
ctx:continue_with_number(hook, number)
ctx:continue_with_value(hook, serializable_value)
```

The current hook entity is automatically recorded as the effect source.

`move` destinations are `hand`, `deck_top`, `deck_bottom`, `deck_random`, `graveyard`, and `removed`. Hidden-zone resets clear board-only state. A full hand sends generated/returned cards to the appropriate burn or graveyard path.

`modify` supports:

```lua
ctx:modify(target, {
    stat = "attack",             -- attack / health / cost / spell_damage
    operation = "set",           -- set / add / multiply
    value = 5,
    duration = "end_of_turn",    -- permanent (default) / end_of_turn
    silenciable = true,
})
```

Stat layers use stable `SET → ADD → MULTIPLY → Aura` ordering.

## Choices and deterministic randomness

```lua
ctx:choose_entities(player, prompt, entities, "resume_hook")
ctx:choose_cards(player, prompt, card_ids, "resume_hook")
ctx:choose_options(player, prompt, options, "resume_hook")

ctx:discover_cards(player, prompt, candidates, count, "resume_hook")
ctx:discover_entities(player, prompt, candidates, count, "resume_hook")

ctx:random_entity(entities, "resume_hook")
ctx:random_value(values, "resume_hook")
```

Discover does not invent a pool. Card Lua filters immutable definitions by class, type, set, tags, or any other printed property, then passes the candidate IDs to deterministic sampling.

Pending options, the source, hook name, typed payload, RNG counter, and remaining queue serialize into snapshots/replays. Resume hooks are module functions by name; closures and coroutines are not stored.

Choice values may be `nil`, boolean, signed integer, UTF-8 string, dense arrays, string-key objects, or recursive combinations within documented depth/node/string limits.

## Triggers and events

```lua
triggers = {
    {
        event = "damaged",
        timing = "before",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.target == self and event.amount > 0
        end,
        effect = function(ctx, self, event)
            ctx:cancel_event(event)
        end,
    },
}
```

Without `active_zones`, a trigger is active on the board. Valid public zones include `hero`, `hero_power`, `deck`, `hand`, `board`, `weapon`, `secret`, and `graveyard`.

Current event names include:

```text
game_started, turn_started, turn_ended,
card_drawn, card_burned, card_created, card_played, card_countered,
card_discarded, card_traded, trade_draw, spell_cast,
minion_played, minion_summoned, magnetized,
weapon_played, weapon_equipped, weapon_destroyed,
location_played, location_used, location_destroyed,
hero_power_used, hero_power_replaced,
secret_played, secret_revealed,
zone_changed, controller_changed, transformed,
attack, damaged, damage_prevented, healed, entity_died,
armor_gained, overload_queued, mana_locked, mana_unlocked,
overload_cleared, temporary_mana_gained, temporary_mana_expired,
mana_crystals_gained, mana_crystals_destroyed, mana_spent,
keyword_disabled, frozen, fatigue,
choice_requested, choice_made, random_choice_made,
random_cards_sampled, random_entities_sampled,
conceded, game_ended
```

Before events may be cancelled while pending. `set_event_amount` applies to supported numeric pending events. Cancellation semantics are generic and event-specific: for example, countered hand cards still consume payment, cancelled draws restore the reserved top card, and cancelled effect summons move their reserved token to Removed.

Listeners use APNAP order, then stable entity timestamps. Multiple triggers in one module retain Lua array order. Death checkpoints remove lethal minions in stable entry order, remember removal positions, publish the whole death batch, then execute Deathrattles and Reborn effects.

## Auras

```lua
auras = {
    {
        active_zones = { "board" },
        attack = 1,
        health = 1,
        cost = 0,
        spell_damage = 0,
        keywords = { "taunt" },
        targets = function(ctx, self)
            return ctx:friendly_minions(self)
        end,
    },
}
```

Numeric aura fields may be integers or read-only `(ctx, self) -> integer` functions. Aura selectors and dynamic values cannot emit effects. Recalculation removes old aura layers, collects every source against one stable no-aura snapshot, aggregates targets, applies the result, then performs invariant/death checks.

## Deck allowances

Tourist cards declare generic construction permissions:

```lua
keywords = { "tourist" },
deck_allowances = {
    {
        class = "druid",
        set = "ISLAND_VACATION",
        excluded_keywords = { "tourist" },
    },
},
```

Normal CLI decks allow their class, Neutral cards, and matching allowances. A mechanics-only sandbox deck may explicitly set `"unrestricted": true` in its deck JSON.

## Script rules

- Never store match state in module globals; use entity/player script data.
- Never use Lua randomness; use the seeded `ctx:random_*` and `ctx:discover_*` APIs.
- Never depend on hash-table or filesystem iteration order.
- Keep locale selection out of gameplay decisions.
- Use named continuations when later logic must observe state after earlier effects commit.
- A hook has an approximately 200,000-instruction budget; one command may resolve at most 10,000 effects.
