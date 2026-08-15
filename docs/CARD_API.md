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
ctx:minions_summoned(player)
ctx:minions_died(player)
ctx:minions_died_this_turn(player)
ctx:minion_death_records(player) -- { card_id, turn, had_deathrattle, keywords }
ctx:discarded_cards(player)      -- original entity IDs in discard-event order
ctx:discarded_card_ids(player)   -- frozen definition IDs in discard-event order
ctx:starting_deck(player)        -- frozen opening card-ID multiset
ctx:cards_added_to_hand(player)  -- post-start card IDs in event order
ctx:overload_queued_total(player) -- lifetime overloaded Crystal count
ctx:hero_was_healed_this_turn(player)
ctx:weapons_played(player)
ctx:locations_played(player)
ctx:last_spell_cast(player)
ctx:hero_power_uses(player)

ctx:hand(player)
ctx:deck(player)                 -- top to bottom
ctx:board(player)                -- minions and Locations, left to right
ctx:secrets(player)
ctx:graveyard(player)
ctx:characters()
ctx:minions()
ctx:enemy_characters(entity)
ctx:enemy_minions(entity)
ctx:friendly_minions(entity)
ctx:adjacent_minions(entity)
ctx:board_position(entity)

ctx:card_ids()
ctx:collectible_cards()
ctx:card_definition(card_id)
ctx:get_data(entity, key)
ctx:get_player_data(player, key)
```

Entity snapshots include identity, definition, owner/controller, zone/type, Attack, current/max Health, damage, Armor, Cost, Spell Damage, keywords, Silence, Freeze, attacks used this turn, `attack_at_death`, `started_in_deck`, Location cooldown, enchantments, attached scripts/Deathrattles, hand-entry/play context, and script data relevant to rules. `starting_deck` remains frozen when cards move or transform; `cards_added_to_hand` records successful draws, generated cards, and zone moves into Hand.

Player snapshots include class, hero, Hero Power, weapon, player keywords, mana fields, overload fields, played/summoned histories and counts, fatigue, zone sizes, current-turn Hero Power state, and lifetime Hero Power uses.

Card definition snapshots returned by `ctx:card_definition` include `classes` for multi-class cards, plus `rarity` and `spell_school` when declared. `card_created` events include both the created `entity` and the creating effect's `source`, allowing generated-card mechanics to identify only their own output.

Returned arrays/tables are snapshots. Lua cannot mutate Rust containers. Scripts are trusted server rules and may query hidden zones; UI clients do not receive those values automatically.

## Effect output

These functions append validated effects; they do not mutate state during the Lua call:

```lua
ctx:damage(target, amount)
ctx:damage_ignoring_spell_damage(target, amount)
ctx:damage_all(targets, amount)
ctx:heal(target, amount)
ctx:gain_armor(player, amount)

ctx:overload(player, amount)
ctx:unlock_mana(player, amount)
ctx:clear_overload(player)
ctx:gain_temporary_mana(player, amount)
ctx:gain_mana_crystals(player, amount, filled)
ctx:fill_mana_crystals(player, amount)
ctx:refresh_mana_crystals(player)
ctx:destroy_mana_crystals(player, amount)
ctx:spend_mana(player, amount)

ctx:draw(player, count)
ctx:draw_entity(player, deck_entity)
ctx:give_card(player, card_id)
ctx:give_card_at(player, card_id, position)
ctx:create_card(player, card_id, spec_or_nil)
ctx:give_copy(player, entity)
ctx:give_copy_with_stats(player, entity, attack, health, cost_or_nil)
ctx:give_base_copy(player, entity)
ctx:give_base_copy_with_stats(player, entity, attack, health, cost_or_nil)
ctx:shuffle_card_into_deck(player, card_id)
ctx:discard(player, entity)
ctx:cast_spell(player, card_id, options_or_nil)
ctx:cast_existing_spell(card, options_or_nil)

ctx:summon(player, card_id)
ctx:summon_at(player, card_id, position)
ctx:summon_with_stats(player, card_id, attack, health, keywords_or_nil)
ctx:summon_with_base_stats(player, card_id, attack, health, keywords_or_nil)
ctx:summon_copy(player, entity)
ctx:summon_copy_at(player, entity, position)
ctx:summon_copy_with_stats(player, entity, attack, health)
ctx:summon_fresh_copy(entity, position_or_nil, health, without_keywords)
ctx:summon_fresh_copy_with_stats(entity, position_or_nil, attack, health, without_keywords)
ctx:summon_from_hand(card)
ctx:summon_existing(player, graveyard_entity)
ctx:summon_existing_at(player, graveyard_entity, position)
ctx:recruit(player, deck_entity)
ctx:recruit_at(player, deck_entity, position)

ctx:equip_weapon(player, card_id)
ctx:lose_weapon_durability(weapon, amount)
ctx:replace_hero(player, hero_card_id)
ctx:replace_hero_power(player, card_id)
ctx:refresh_hero_power(player)
ctx:exchange_zone_contents(first_player, second_player, "deck" | "hand" | "graveyard")
ctx:move(entity, destination)
ctx:move_to_hand(player, entity)
ctx:shuffle_entity_into_deck(player, entity)
ctx:shuffle_copy_into_deck(player, entity)
ctx:change_controller(entity, player)
ctx:change_controller_until_end_of_turn(entity, player)
ctx:transform(entity, card_id)
ctx:transform_all(entities, card_id)
ctx:transform_batch({ { entity, card_id }, ... })
ctx:transform_into_copy(entity, template, attack_or_nil, health_or_nil)
ctx:transform_preserving_scripts(entity, card_id)
ctx:destroy(entity)
ctx:destroy_all(entities)
ctx:damage_batch({ { entity, amount }, ... })
ctx:damage_batch_ignoring_spell_damage({ { entity, amount }, ... })
ctx:damage_from(source, entity, amount)
ctx:add_attack_collateral(event_id, entities, amount)
ctx:force_attack(attacker, defender)
ctx:take_extra_turn(player)
ctx:win_game(player)
ctx:set_health(entity, amount)
ctx:heal_all(entities, amount)
ctx:trigger_hook(entity, hook)
ctx:attach_hook(entity, hook, card_id)
ctx:attach_script(entity, card_id)
ctx:board_position(entity)

ctx:buff(entity, attack_delta, health_delta)
ctx:buff_until_end_of_turn(entity, attack_delta, health_delta)
ctx:modify(entity, modifier_table)
ctx:modify_all(entities, modifier_table)
ctx:remove_enchantments_from(entity, source)
ctx:grant_keyword(entity, keyword_id)
ctx:grant_keyword_until_end_of_turn(entity, keyword_id)
ctx:grant_keyword_until_next_turn(entity, keyword_id)
ctx:disable_keyword(entity, keyword_id)
ctx:grant_player_keyword(player, keyword_id)
ctx:disable_player_keyword(player, keyword_id)
ctx:set_player_class(player, class_id)
ctx:silence(entity)
ctx:freeze(entity)

ctx:reveal_secret(secret)
ctx:cancel_event(event)
ctx:set_event_amount(event, amount)
ctx:set_attack_defender(event_id, defender)
ctx:set_damage_target(event_id, target)
ctx:replace_trade_draw(event_id, replacement_entity)
ctx:set_data(entity, key, value)
ctx:set_player_data(player, key, value)
ctx:increment_player_data(player, key, delta)

ctx:continue_with(hook)
ctx:continue_with_entity(hook, entity)
ctx:continue_with_card(hook, card_id)
ctx:continue_with_number(hook, number)
ctx:continue_with_value(hook, serializable_value)
```

The current hook entity is automatically recorded as the effect source.

`replace_hero` requires a Hero definition with a valid `hero_power`. The new hero starts at its defined full Health while preserving Armor, frozen state, and attacks used this turn; both replacement events are published. Player keywords are persistent, serializable Lua mechanics hosted by the current hero, so they survive minion silence, transformation, death, and hero replacement.

`move` destinations are `hand`, `secret`, `deck_top`, `deck_bottom`, `deck_random`, `graveyard`, and `removed`. Moving to `secret` validates that the entity is a Secret and that the destination zone has room. `shuffle_entity_into_deck` transfers the original entity to the specified player's deck with deterministic Rust RNG, including ownership/controller transfer and hidden-zone reset. Hidden-zone resets clear board-only state. A full hand sends generated/returned cards to the appropriate burn or graveyard path.

`transform` may replace card kinds while an entity is in the hidden Hand or Deck zones and preserves its identity and zone position. `transform_all` applies one definition atomically; `transform_batch` applies per-entity definitions in one atomic group. `transform_into_copy` copies an entity's complete state before optional silenciable final Attack/Health values. `transform_preserving_scripts` additionally retains `attached_cards` and script data for effects whose behavior must survive their own transformation; attach a reusable module first with `attach_script`. `attach_hook` adds an ordered, stackable card script to any named Lua hook; Silence removes existing hook attachments from minions.

`cast_spell` creates a spell from its definition; `cast_existing_spell` casts an existing entity from a hidden or terminal zone. Both accept `{ target = entity, skip_if_invalid = true, random_target = true, choice_policy = "random" }`. Target randomization and automatic choices are explicit policies instead of hidden script-data flags. Repeated random casting belongs in Lua; `cardlib.random_spell` composes authoritative `random_value` and `cast_spell` operations.

`create_card` accepts `destination`, optional hand `position`, `attack`, `health`, `cost`, `spell_damage`, `keywords`, and `attached_scripts`. Composition formulas belong in Lua; `cardlib.fusion.create_minion` is the reusable fusion implementation. Files with `module_type = "library"` are exposed under `cardlib[id]`, validated and included in the deterministic pack hash, but are not registered as cards.

`damage_ignoring_spell_damage` follows the normal damage/event pipeline but does not add the source controller's Spell Damage. `spend_mana` atomically spends up to the player's current Mana (temporary Mana first) and publishes `mana_spent` for the actual positive amount. `increment_player_data` atomically adds a signed delta to a player script-data key, publishes `player_script_data_changed` with `old/new/delta`, and avoids lost updates from one snapshot. Death records store whether the minion's base definition has Deathrattle: Silence does not clear this flag, and attached Deathrattles do not set it.

`give_copy` preserves the source entity's persistent state for forward or same-zone copies; `give_copy_with_stats` also applies final Attack/Health and optional Cost setters. The `give_base_copy*` variants instantiate the printed definition without copied enchantments, for backward-zone copies such as Battlefield-to-Hand effects.

`draw_entity` removes the specified original entity from that player's deck and uses the normal cancellable CardDrawn/CardBurned pipeline. `summon_existing` moves an original Graveyard or Removed minion through the full cancellable summon pipeline and restores it if the summon is cancelled or the board fills; `summon_existing_at` additionally preserves a remembered board position. `move_to_hand` transfers an original entity to another player's hand; `shuffle_copy_into_deck` preserves the copied entity state. `summon_with_stats` applies silenciable final-set stats; `summon_with_base_stats` replaces printed base stats, so Silence does not revert scaling tokens such as Jade Golems. `summon_fresh_copy_with_stats` creates an unenchanted template copy with final Attack/Health. `lose_weapon_durability` reduces an equipped weapon and uses the normal cancellable `weapon_destroyed` lifecycle at zero. `add_attack_collateral` adds simultaneous combat damage to a pending attack.

`damage_batch` atomically commits different damage amounts against a frozen target set; its spell-damage-immune variant skips Spell Damage. `modify_all` applies one stat specification to a frozen group; `modify_batch` accepts per-entity specifications, including a `modifiers` array when each stat needs a different operation. Both support `reset_damage = true`. `force_attack` starts a full attack event without requiring a ready attacker, while `take_extra_turn` queues a replayable extra turn for the specified player. `grant_keyword_until_next_turn` expires at the start of that minion controller's next turn and survives loss of its source.

`refresh_mana_crystals` fills only the player's existing unlocked permanent crystals. It preserves temporary Mana and both current and pending Overload. `change_controller_until_end_of_turn` records a reversible board-minion control change: Silence immediately returns the minion, transformation makes the current controller permanent, and end of turn returns it or destroys it when the original board is full.

`modify` supports:

```lua
ctx:modify(target, {
    stat = "attack",             -- attack / health / cost / spell_damage
    operation = "set",           -- set / add / pre_final_add / multiply / final_set
    value = 5,
    duration = "end_of_turn",    -- permanent (default) / end_of_turn
    silenciable = true,
})
```

Without a `final_set`, permanent stats use stable `SET → ADD/PRE_FINAL_ADD → MULTIPLY` grouping. With a `final_set`, the latest setter becomes the value and only later ordinary Set/Add/Multiply modifiers apply; `pre_final_add` always remains below that setter. Live Aura Set/Add layers apply last.

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
card_discarded, card_traded, trade_draw, spell_targeted, spell_cast,
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

`card_drawn` and `card_burned` expose the drawn card as `entity` and the causal effect entity as `source`. Natural turn and opening-hand draws use `source = nil`; script draws, Hero Power draws, and trade replacement draws retain their actual source.

`card_played` exposes `cost`, the effective card cost captured when the play command was committed. This remains the played cost even if the card leaves a cost aura or its effect spends additional Mana later.

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

`spell_damage` auras may target minions or heroes. A player's spell bonus is the sum on their board minions and hero, which allows symmetric player-level effects without card-specific engine logic.

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

Normal `Game::new*` construction and CLI decks allow their class, Neutral cards,
multi-class cards that include their class, and matching allowances. Mechanics-only
tests may explicitly use `Game::new_unrestricted*`; a CLI sandbox deck may set
`"unrestricted": true` in its deck JSON.

## Script rules

- Never store match state in module globals; use entity/player script data.
- Never use Lua randomness; use the seeded `ctx:random_*` and `ctx:discover_*` APIs.
- Never depend on hash-table or filesystem iteration order.
- Keep locale selection out of gameplay decisions.
- Use named continuations when later logic must observe state after earlier effects commit.
- A hook has an approximately 200,000-instruction budget; one command may resolve at most 10,000 effects.
