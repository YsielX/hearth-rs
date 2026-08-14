# Constructed Keyword Coverage

[English](KEYWORDS.md) | [简体中文](zhCN/KEYWORDS.md) | [繁體中文](zhTW/KEYWORDS.md)

Audit date: 2026-08-14.

## Scope

The catalog starts with the Hearthstone Wiki Constructed ability list and cross-checks recent additions against Blizzard announcements:

- [Ability and keyword index](https://hearthstone.wiki.gg/wiki/Ability)
- [Escape from Violet Hold: Prepare](https://hearthstone.blizzard.com/en-us/news/24276664)
- [Cataclysm: Herald, Shatter, and returning Colossal](https://hearthstone.blizzard.com/en-gb/news/24250357/cataclysm-is-now-live)
- [Across the Timeways: Rewind and Fabled](https://hearthstone.blizzard.com/en-us/news/24226328/)
- [Into the Emerald Dream: Imbue](https://hearthstone.blizzard.com/en-us/news/24179067/step-into-the-emerald-dream-hearthstone-s-next-expansion)

Result: **68 functional Constructed keywords and 68 Lua modules**. The repository also has `conditional_charge.lua`, an internal reusable rule for Southsea Deckhand; it is not counted as an official keyword. `keyword_catalog_matches_the_constructed_hearthstone_glossary` locks the exact set in tests.

Excluded from the count:

- Battlegrounds, Mercenaries, and other mode-specific abilities;
- resource or generated-pool names such as Corpse, Dark Gift, Jade Golem, Lackey, and Spare Part;
- explanatory terms such as Bonus Effect that do not own independent match timing.

Cards express those concepts with player data, dynamic pools, and generic effects rather than empty combat keywords.

## Evergreen keywords (27/27)

| Lua ID | Keyword | Shared implementation |
| --- | --- | --- |
| `battlecry` | Battlecry | Play timing and required `on_battlecry` payload |
| `casts_when_drawn` | Casts When Drawn | Move and cast the drawn entity, then replace the draw |
| `charge` | Charge | Ready-on-summon rule |
| `counter` | Counter | Generic before-event cancellation; Secret Lua owns its trigger |
| `deathrattle` | Deathrattle | Death position and serializable `on_deathrattle` continuation |
| `discover` | Discover | Card Lua builds the pool; Rust RNG samples and resumes deterministically |
| `divine_shield` | Divine Shield | Disable itself and cancel positive incoming damage |
| `dormant` | Dormant | Block attacks and targeted selection; card script owns wake condition |
| `elusive` | Elusive | Block targeted spells and Hero Powers from either player |
| `freeze` | Freeze | Card emits Freeze; core owns authoritative thaw timing |
| `immune` | Immune | Block enemy targeting/attacks and cancel damage |
| `lifesteal` | Lifesteal | Heal for positive actual damage, including weapon inheritance |
| `mega_windfury` | Mega-Windfury | Four attacks per turn, including weapon inheritance |
| `passive` | Passive | Prevent active Hero Power use |
| `poisonous` | Poisonous | Destroy a minion after positive actual damage |
| `reborn` | Reborn | Summon a fresh 1-Health copy without Reborn at the death position |
| `rush` | Rush | On the summon turn, attacks may target minions only |
| `secret` | Secret | Lua `enters_secret_zone` rule plus card triggers |
| `silence` | Silence | Card selects the target; generic effect removes silenciable layers/scripts |
| `spell_damage` | Spell Damage | Parameterized base Spell Damage and generic stat layering |
| `start_of_game` | Start of Game | `game_started` before opening hands plus required payload |
| `stealth` | Stealth | Target protection; removed after attack or positive dealt damage |
| `summoned_when_drawn` | Summoned When Drawn | Summon the same drawn entity and replace the draw |
| `taunt` | Taunt | Generic attack-priority rule |
| `temporary` | Temporary | Move to Removed at controller turn end; not a discard |
| `tradeable` | Tradeable | 1-Mana Trade action, deterministic deck insertion, replay support |
| `windfury` | Windfury | Two attacks per turn, including weapon inheritance |

## Evergreen class keywords (6/6)

| Lua ID | Keyword | Shared implementation |
| --- | --- | --- |
| `choose_one` | Choose One | Lifecycle and required `on_choose_one` payload |
| `choose_multiple` | Choose Multiple | Lifecycle and required multi-choice payload |
| `combo` | Combo | Frozen pre-play cards-played count |
| `outcast` | Outcast | Frozen pre-play left/right hand position |
| `overheal` | Overheal | Healing event and excess amount passed to card payload |
| `overload` | Overload | Parameterized debt, next-turn lock, unlock, and clear events |

## Expansion keywords (35/35)

| Lua ID | Keyword | Shared implementation |
| --- | --- | --- |
| `adapt` | Adapt | Shared play entry; card payload defines choice/effect |
| `colossal` | Colossal | Component callback after summon from any source |
| `corrupt` | Corrupt | Hand listener for a higher-Cost play and one-shot transform |
| `dredge` | Dredge | Shared entry; payload uses deck-entity choice and move-to-top primitives |
| `echo` | Echo | Shared entry; payload creates a Temporary same-turn copy |
| `excavate` | Excavate | Player-level four-tier cycle passed to reward payload |
| `fabled` | Fabled | Pre-opening companion callback from the deck |
| `finale` | Finale | Fires only when payment leaves zero Mana |
| `forge` | Forge | Generic 2-Mana hand action; card defines `action_effects.forge` |
| `frenzy` | Frenzy | One-shot trigger after surviving damage |
| `gigantify` | Gigantify | Shared entry; card payload creates its official giant token |
| `herald` | Herald | Parameterized progress, 2/4 upgrade tiers, structured Soldier payload |
| `honorable_kill` | Honorable Kill | Exact damage to zero, including weapon sources |
| `imbue` | Imbue | Player-level permanent count and Hero Power payload |
| `infuse` | Infuse | Friendly minion deaths while in hand; one-shot parameter threshold |
| `inspire` | Inspire | Trigger after a friendly successful Hero Power use |
| `invoke` | Invoke | Player-level count passed to the invocation payload |
| `kindred` | Kindred | Compare tribe tags with cards played last turn |
| `magnetic` | Magnetic | Adjacent Mech placement, stat/keyword/script merge, Silence behavior |
| `manathirst` | Manathirst | Required maximum-Mana threshold parameter |
| `miniaturize` | Miniaturize | Shared entry; payload creates the official 1/1 Mini token |
| `overkill` | Overkill | Damage leaves Health below zero, including weapon sources |
| `prepare` | Prepare | Spend all Mana, reduce by spent + 1, unplayable that turn |
| `quest` | Quest | Forced opening hand and persistent Quest zone |
| `questline` | Questline | Forced opening, persistent zone, staged payloads |
| `quickdraw` | Quickdraw | Fires only on the turn the entity entered hand |
| `recruit` | Recruit | Move the original deck entity with reservation/cancel/position events |
| `rewind` | Rewind | Shared entry; Lua owns reroll state and acceptance timing |
| `shatter` | Shatter | Trigger on draw/create for left/right fragment generation |
| `sidequest` | Sidequest | Persistent Quest zone without forced opening hand |
| `spellburst` | Spellburst | One-shot trigger after a friendly successful spell cast |
| `starship` | Starship | Piece-death payload and generic launch action |
| `titan` | Titan | Three one-shot abilities, once per turn, Freeze and attack restrictions |
| `tourist` | Tourist | Lua deck allowance for class/set, excluding destination Tourists |
| `twinspell` | Twinspell | Shared entry; payload creates the official non-Twinspell copy |

## What “implemented” means

A keyword is not always one fixed numeric effect. Taunt, Divine Shield, and Magnetic are fully executed by their modules. Battlecry, Discover, Adapt, and Miniaturize own shared timing, but their target pools, values, options, or official token IDs belong to the individual card text.

Modules use `required_card_hooks`, `required_card_actions`, `required_card_fields`, and `requires_param` to enforce those payloads at load time. A missing hook, action, field, or parameter rejects the card pack; it never degrades into a display-only string.

Therefore a new card using these 68 keywords still requires only Lua. Rust changes are reserved for a genuinely new reusable rule, event, choice, or atomic effect boundary.
