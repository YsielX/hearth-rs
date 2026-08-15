use super::*;

pub(super) fn build_context(
    lua: &Lua,
    state: &GameState,
    source: EntityId,
    effects: Rc<RefCell<Vec<EffectSpec>>>,
    catalog: Arc<BTreeMap<String, CardDefinition>>,
    locale: Locale,
) -> mlua::Result<Table> {
    let state = Arc::new(state.clone());
    let ctx = lua.create_table()?;

    ctx.set("locale", locale.code())?;
    ctx.set(
        "localize",
        lua.create_function(
            move |_, (_ctx, en_us, zh_cn, zh_tw): (Table, String, String, String)| {
                Ok(match locale {
                    Locale::EnUs => en_us,
                    Locale::ZhCn => zh_cn,
                    Locale::ZhTw => zh_tw,
                })
            },
        )?,
    )?;

    let current_turn = state.turn;
    ctx.set(
        "turn",
        lua.create_function(move |_, _ctx: Table| Ok(current_turn))?,
    )?;

    let active_player = state.active_player;
    ctx.set(
        "active_player",
        lua.create_function(move |_, _ctx: Table| Ok(active_player.0))?,
    )?;

    let snapshot = state.clone();
    let definitions = catalog.clone();
    ctx.set(
        "keyword_param",
        lua.create_function(move |_, (_ctx, entity, keyword): (Table, u64, String)| {
            let entity = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?;
            Ok(definitions
                .get(&entity.card_id)
                .and_then(|definition| definition.keyword_params.get(&keyword))
                .copied())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "controller",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            snapshot
                .entity(EntityId(entity))
                .map(|entity| entity.controller.0)
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))
        })?,
    )?;
    ctx.set(
        "opponent",
        lua.create_function(|_, (_ctx, player): (Table, u8)| {
            Ok(parse_player(player)?.opponent().0)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "player",
        lua.create_function(move |lua, (_ctx, player): (Table, u8)| {
            let player = snapshot.player(parse_player(player)?);
            let table = lua.create_table()?;
            table.set("id", player.id.0)?;
            table.set("class", player.class.as_str())?;
            table.set("hero", player.hero.0)?;
            table.set("hero_power", player.hero_power.0)?;
            table.set("weapon", player.weapon.map(|weapon| weapon.0))?;
            table.set("mana", player.mana)?;
            table.set("max_mana", player.max_mana)?;
            table.set("temporary_mana", player.temporary_mana)?;
            table.set("overload_pending", player.overload_pending)?;
            table.set("overloaded_mana", player.overloaded_mana)?;
            table.set("fatigue", player.fatigue)?;
            table.set("deck_size", player.deck.len())?;
            table.set("hand_size", player.hand.len())?;
            table.set("board_size", player.board.len())?;
            table.set("secret_count", player.secrets.len())?;
            table.set("hero_power_used", player.hero_power_used)?;
            table.set("hero_power_uses", player.hero_power_uses)?;
            table.set(
                "hero_power_uses_this_turn",
                player.hero_power_uses_this_turn,
            )?;
            let keywords = lua.create_table()?;
            for (index, keyword) in player.keywords.iter().enumerate() {
                keywords.set(index + 1, keyword.as_str())?;
            }
            table.set("keywords", keywords)?;
            table.set("cards_played_this_turn", player.cards_played_this_turn)?;
            table.set("cards_played_this_game", player.cards_played_history.len())?;
            table.set("spells_cast_this_game", player.spells_cast_history.len())?;
            table.set(
                "minions_played_this_game",
                player.minions_played_history.len(),
            )?;
            table.set(
                "minions_summoned_this_game",
                player.minions_summoned_history.len(),
            )?;
            table.set(
                "weapons_played_this_game",
                player.weapons_played_history.len(),
            )?;
            table.set(
                "weapons_destroyed_this_game",
                player.weapons_destroyed_history.len(),
            )?;
            table.set(
                "locations_played_this_game",
                player.locations_played_history.len(),
            )?;
            Ok(table)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "cards_played_this_turn",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .cards_played_this_turn)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "hero_power_uses",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot.player(parse_player(player)?).hero_power_uses)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "combo_active",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let entity = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?;
            Ok(entity.cards_played_before > 0)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "board_position",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let entity = EntityId(entity);
            let controller = snapshot
                .entity(entity)
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?
                .controller;
            Ok(snapshot
                .player(controller)
                .board
                .iter()
                .position(|candidate| *candidate == entity))
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "outcast_active",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let entity = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?;
            let hand_len_before = snapshot.player(entity.controller).hand.len() + 1;
            Ok(entity
                .hand_position_before_play
                .is_some_and(|position| position == 0 || position + 1 == hand_len_before))
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "entered_hand_this_turn",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let entity = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?;
            Ok(entity.entered_hand_turn == Some(snapshot.turn))
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "cards_played_last_turn",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .cards_played_last_turn
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "starting_deck",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot.player(parse_player(player)?).starting_deck.clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "cards_added_to_hand",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .cards_added_to_hand_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "overload_queued_total",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot.player(parse_player(player)?).overload_queued_total)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "hero_was_healed_this_turn",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot.player(parse_player(player)?).hero_last_healed_turn == Some(snapshot.turn))
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "cards_played",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .cards_played_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "spells_cast",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .spells_cast_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "minions_played",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .minions_played_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "minions_summoned",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .minions_summoned_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "minions_died",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .minions_died_history
                .iter()
                .map(|record| record.card_id.clone())
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "minions_died_this_turn",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .minions_died_history
                .iter()
                .filter(|record| record.turn == snapshot.turn)
                .map(|record| record.card_id.clone())
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "minion_death_records",
        lua.create_function(move |lua, (_ctx, player): (Table, u8)| {
            let records = lua.create_table()?;
            for (index, record) in snapshot
                .player(parse_player(player)?)
                .minions_died_history
                .iter()
                .enumerate()
            {
                let entry = lua.create_table()?;
                entry.set("card_id", record.card_id.as_str())?;
                entry.set("turn", record.turn)?;
                entry.set("had_deathrattle", record.had_deathrattle)?;
                entry.set("keywords", record.keywords.clone())?;
                records.set(index + 1, entry)?;
            }
            Ok(records)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "discarded_cards",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .discarded_cards_history
                .iter()
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "discarded_card_ids",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .discarded_card_ids_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "weapons_played",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .weapons_played_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "weapons_destroyed",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .weapons_destroyed_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "locations_played",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .locations_played_history
                .clone())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "last_spell_cast",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .spells_cast_history
                .last()
                .cloned())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "spell_cast_records",
        lua.create_function(move |lua, (_ctx, player): (Table, u8)| {
            let records = lua.create_table()?;
            for (index, record) in snapshot
                .player(parse_player(player)?)
                .spell_cast_records
                .iter()
                .enumerate()
            {
                let entry = lua.create_table()?;
                entry.set("card_id", record.card_id.as_str())?;
                entry.set("cost", record.cost)?;
                entry.set(
                    "target_was_friendly_minion",
                    record.target_was_friendly_minion,
                )?;
                records.set(index + 1, entry)?;
            }
            Ok(records)
        })?,
    )?;

    let cards = catalog.clone();
    ctx.set(
        "card_ids",
        lua.create_function(move |_, _ctx: Table| Ok(cards.keys().cloned().collect::<Vec<_>>()))?,
    )?;
    let cards = catalog.clone();
    ctx.set(
        "collectible_cards",
        lua.create_function(move |_, _ctx: Table| {
            Ok(cards
                .values()
                .filter(|card| card.collectible)
                .map(|card| card.id.clone())
                .collect::<Vec<_>>())
        })?,
    )?;
    let cards = catalog.clone();
    ctx.set(
        "card_definition",
        lua.create_function(move |lua, (_ctx, card_id): (Table, String)| {
            let card = cards.get(&card_id).ok_or_else(|| {
                mlua::Error::runtime(format!("unknown card definition {card_id}"))
            })?;
            card_definition_to_table(lua, card)
        })?,
    )?;

    let snapshot = state.clone();
    ctx.set(
        "characters",
        lua.create_function(move |_, _ctx: Table| {
            Ok(snapshot
                .entities
                .values()
                .filter(|entity| {
                    entity.zone == Zone::Hero
                        || (entity.zone == Zone::Board && entity.kind == CardKind::Minion)
                })
                .map(|entity| entity.id.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "hand",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .hand
                .iter()
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "deck",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .deck
                .iter()
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "board",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .board
                .iter()
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "secrets",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .secrets
                .iter()
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "graveyard",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .graveyard
                .iter()
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "minions",
        lua.create_function(move |_, _ctx: Table| {
            Ok(snapshot
                .entities
                .values()
                .filter(|entity| entity.zone == Zone::Board && entity.kind == CardKind::Minion)
                .map(|entity| entity.id.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "enemy_characters",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let controller = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?
                .controller;
            Ok(snapshot
                .entities
                .values()
                .filter(|candidate| {
                    candidate.controller == controller.opponent()
                        && (candidate.zone == Zone::Hero
                            || (candidate.zone == Zone::Board
                                && candidate.kind == CardKind::Minion))
                })
                .map(|candidate| candidate.id.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "enemy_minions",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let controller = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?
                .controller;
            Ok(snapshot
                .player(controller.opponent())
                .board
                .iter()
                .filter(|entity| snapshot.entities[entity].kind == CardKind::Minion)
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "friendly_minions",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let controller = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?
                .controller;
            Ok(snapshot
                .player(controller)
                .board
                .iter()
                .filter(|entity| snapshot.entities[entity].kind == CardKind::Minion)
                .map(|entity| entity.0)
                .collect::<Vec<_>>())
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "adjacent_minions",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let entity = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?;
            if entity.zone != Zone::Board || entity.kind != CardKind::Minion {
                return Ok(Vec::new());
            }
            let board = &snapshot.player(entity.controller).board;
            let position = board
                .iter()
                .position(|candidate| *candidate == entity.id)
                .ok_or_else(|| mlua::Error::runtime("board entity is absent from board list"))?;
            let mut adjacent = Vec::with_capacity(2);
            if position > 0 {
                let candidate = board[position - 1];
                if snapshot.entities[&candidate].kind == CardKind::Minion {
                    adjacent.push(candidate.0);
                }
            }
            if position + 1 < board.len() {
                let candidate = board[position + 1];
                if snapshot.entities[&candidate].kind == CardKind::Minion {
                    adjacent.push(candidate.0);
                }
            }
            Ok(adjacent)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "board_position",
        lua.create_function(move |_, (_ctx, entity): (Table, u64)| {
            let entity = snapshot
                .entity(EntityId(entity))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {entity}")))?;
            if entity.zone != Zone::Board
                || !matches!(entity.kind, CardKind::Minion | CardKind::Location)
            {
                return Ok(None);
            }
            Ok(snapshot
                .player(entity.controller)
                .board
                .iter()
                .position(|candidate| *candidate == entity.id))
        })?,
    )?;

    let snapshot = state.clone();
    ctx.set(
        "entity",
        lua.create_function(move |lua, (_ctx, id): (Table, u64)| {
            let entity = snapshot
                .entity(EntityId(id))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {id}")))?;
            entity_to_table(lua, entity)
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "get_data",
        lua.create_function(move |_, (_ctx, id, key): (Table, u64, String)| {
            let entity = snapshot
                .entity(EntityId(id))
                .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {id}")))?;
            Ok(entity.script_data.get(&key).copied().unwrap_or(0))
        })?,
    )?;
    let snapshot = state.clone();
    ctx.set(
        "get_player_data",
        lua.create_function(move |_, (_ctx, player, key): (Table, u8, String)| {
            Ok(snapshot
                .player(parse_player(player)?)
                .script_data
                .get(&key)
                .copied()
                .unwrap_or(0))
        })?,
    )?;

    let output = effects.clone();
    ctx.set(
        "damage",
        lua.create_function(move |_, (_ctx, target, amount): (Table, u64, i32)| {
            output.borrow_mut().push(EffectSpec::Damage {
                source,
                target: EntityId(target),
                amount,
                apply_spell_damage: true,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "damage_ignoring_spell_damage",
        lua.create_function(move |_, (_ctx, target, amount): (Table, u64, i32)| {
            output.borrow_mut().push(EffectSpec::Damage {
                source,
                target: EntityId(target),
                amount,
                apply_spell_damage: false,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "damage_from",
        lua.create_function(
            move |_, (_ctx, damage_source, target, amount): (Table, u64, u64, i32)| {
                output.borrow_mut().push(EffectSpec::Damage {
                    source: EntityId(damage_source),
                    target: EntityId(target),
                    amount,
                    apply_spell_damage: false,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "damage_all",
        lua.create_function(move |_, (_ctx, targets, amount): (Table, Table, i32)| {
            let targets = targets
                .sequence_values::<u64>()
                .map(|target| target.map(EntityId))
                .collect::<mlua::Result<Vec<_>>>()?;
            output.borrow_mut().push(EffectSpec::DamageGroup {
                source,
                targets,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "damage_batch",
        lua.create_function(move |_, (_ctx, hits): (Table, Table)| {
            let hits = hits
                .sequence_values::<Table>()
                .map(|hit| {
                    let hit = hit?;
                    Ok((EntityId(hit.get::<u64>(1)?), hit.get::<i32>(2)?))
                })
                .collect::<mlua::Result<Vec<_>>>()?;
            output.borrow_mut().push(EffectSpec::DamageBatch {
                source,
                hits,
                apply_spell_damage: true,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "damage_batch_ignoring_spell_damage",
        lua.create_function(move |_, (_ctx, hits): (Table, Table)| {
            let hits = hits
                .sequence_values::<Table>()
                .map(|hit| {
                    let hit = hit?;
                    Ok((EntityId(hit.get::<u64>(1)?), hit.get::<i32>(2)?))
                })
                .collect::<mlua::Result<Vec<_>>>()?;
            output.borrow_mut().push(EffectSpec::DamageBatch {
                source,
                hits,
                apply_spell_damage: false,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "damage_batch_from",
        lua.create_function(move |_, (_ctx, damage_source, hits): (Table, u64, Table)| {
            let hits = hits
                .sequence_values::<Table>()
                .map(|hit| {
                    let hit = hit?;
                    Ok((EntityId(hit.get::<u64>(1)?), hit.get::<i32>(2)?))
                })
                .collect::<mlua::Result<Vec<_>>>()?;
            output.borrow_mut().push(EffectSpec::DamageBatch {
                source: EntityId(damage_source),
                hits,
                apply_spell_damage: false,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "heal",
        lua.create_function(move |_, (_ctx, target, amount): (Table, u64, i32)| {
            output.borrow_mut().push(EffectSpec::Heal {
                source,
                target: EntityId(target),
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "heal_all",
        lua.create_function(move |_, (_ctx, targets, amount): (Table, Table, i32)| {
            let targets = targets
                .sequence_values::<u64>()
                .map(|target| target.map(EntityId))
                .collect::<mlua::Result<Vec<_>>>()?;
            output.borrow_mut().push(EffectSpec::HealGroup {
                source,
                targets,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "gain_armor",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, i32)| {
            output.borrow_mut().push(EffectSpec::GainArmor {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "lose_armor",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, i32)| {
            output.borrow_mut().push(EffectSpec::LoseArmor {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "overload",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, u8)| {
            output.borrow_mut().push(EffectSpec::Overload {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "unlock_mana",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, u8)| {
            output.borrow_mut().push(EffectSpec::UnlockMana {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "clear_overload",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            output.borrow_mut().push(EffectSpec::ClearOverload {
                source,
                player: parse_player(player)?,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "gain_temporary_mana",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, u8)| {
            output.borrow_mut().push(EffectSpec::GainTemporaryMana {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "gain_mana_crystals",
        lua.create_function(
            move |_, (_ctx, player, amount, filled): (Table, u8, u8, bool)| {
                output.borrow_mut().push(EffectSpec::GainManaCrystals {
                    source,
                    player: parse_player(player)?,
                    amount,
                    filled,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "fill_mana_crystals",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, u8)| {
            output.borrow_mut().push(EffectSpec::FillManaCrystals {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "refresh_mana_crystals",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            output.borrow_mut().push(EffectSpec::RefreshManaCrystals {
                source,
                player: parse_player(player)?,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "destroy_mana_crystals",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, u8)| {
            output.borrow_mut().push(EffectSpec::DestroyManaCrystals {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "spend_mana",
        lua.create_function(move |_, (_ctx, player, amount): (Table, u8, u8)| {
            output.borrow_mut().push(EffectSpec::SpendMana {
                source,
                player: parse_player(player)?,
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "draw",
        lua.create_function(move |_, (_ctx, player, count): (Table, u8, u8)| {
            output.borrow_mut().push(EffectSpec::Draw {
                source: Some(source),
                player: parse_player(player)?,
                count,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "draw_entity",
        lua.create_function(move |_, (_ctx, player, card): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::DrawEntity {
                source,
                player: parse_player(player)?,
                card: EntityId(card),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "give_card",
        lua.create_function(move |_, (_ctx, player, card_id): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::CreateCard {
                source,
                player: parse_player(player)?,
                card_id,
                destination: ZonePlacement::Hand,
                position: None,
                base_attack: None,
                base_health: None,
                base_cost: None,
                base_spell_damage: None,
                keywords: None,
                attached_scripts: Vec::new(),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "give_card_at",
        lua.create_function(
            move |_, (_ctx, player, card_id, position): (Table, u8, String, usize)| {
                output.borrow_mut().push(EffectSpec::CreateCard {
                    source,
                    player: parse_player(player)?,
                    card_id,
                    destination: ZonePlacement::Hand,
                    position: Some(position),
                    base_attack: None,
                    base_health: None,
                    base_cost: None,
                    base_spell_damage: None,
                    keywords: None,
                    attached_scripts: Vec::new(),
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "create_card",
        lua.create_function(
            move |lua, (_ctx, player, card_id, spec): (Table, u8, String, Option<Table>)| {
                let spec = match spec {
                    Some(spec) => spec,
                    None => lua.create_table()?,
                };
                let destination = parse_zone_placement(
                    spec.get::<Option<String>>("destination")?
                        .as_deref()
                        .unwrap_or("hand"),
                )?;
                let keywords = spec
                    .get::<Option<Table>>("keywords")?
                    .map(|values| {
                        values
                            .sequence_values::<String>()
                            .collect::<mlua::Result<Vec<_>>>()
                    })
                    .transpose()?;
                let attached_scripts = spec
                    .get::<Option<Table>>("attached_scripts")?
                    .map(|values| {
                        values
                            .sequence_values::<String>()
                            .collect::<mlua::Result<Vec<_>>>()
                    })
                    .transpose()?
                    .unwrap_or_default();
                output.borrow_mut().push(EffectSpec::CreateCard {
                    source,
                    player: parse_player(player)?,
                    card_id,
                    destination,
                    position: spec.get("position")?,
                    base_attack: spec.get("attack")?,
                    base_health: spec.get("health")?,
                    base_cost: spec.get("cost")?,
                    base_spell_damage: spec.get("spell_damage")?,
                    keywords,
                    attached_scripts,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "give_copy",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::GiveCopy {
                source,
                player: parse_player(player)?,
                target: EntityId(target),
                preserve_state: true,
                attack: None,
                health: None,
                cost: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "give_copy_with_stats",
        lua.create_function(
            move |_,
                  (_ctx, player, target, attack, health, cost): (
                Table,
                u8,
                u64,
                i32,
                i32,
                Option<i32>,
            )| {
                output.borrow_mut().push(EffectSpec::GiveCopy {
                    source,
                    player: parse_player(player)?,
                    target: EntityId(target),
                    preserve_state: true,
                    attack: Some(attack),
                    health: Some(health),
                    cost,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "give_base_copy",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::GiveCopy {
                source,
                player: parse_player(player)?,
                target: EntityId(target),
                preserve_state: false,
                attack: None,
                health: None,
                cost: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "give_base_copy_with_stats",
        lua.create_function(
            move |_,
                  (_ctx, player, target, attack, health, cost): (
                Table,
                u8,
                u64,
                i32,
                i32,
                Option<i32>,
            )| {
                output.borrow_mut().push(EffectSpec::GiveCopy {
                    source,
                    player: parse_player(player)?,
                    target: EntityId(target),
                    preserve_state: false,
                    attack: Some(attack),
                    health: Some(health),
                    cost,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "shuffle_card_into_deck",
        lua.create_function(move |_, (_ctx, player, card_id): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::CreateCard {
                source,
                player: parse_player(player)?,
                card_id,
                destination: ZonePlacement::DeckRandom,
                position: None,
                base_attack: None,
                base_health: None,
                base_cost: None,
                base_spell_damage: None,
                keywords: None,
                attached_scripts: Vec::new(),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "shuffle_copy_into_deck",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::ShuffleCopyIntoDeck {
                source,
                player: parse_player(player)?,
                target: EntityId(target),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "replace_hero_power",
        lua.create_function(move |_, (_ctx, player, card_id): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::ReplaceHeroPower {
                source,
                player: parse_player(player)?,
                card_id,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "replace_hero",
        lua.create_function(move |_, (_ctx, player, card_id): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::ReplaceHero {
                source,
                player: parse_player(player)?,
                card_id,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "grant_player_keyword",
        lua.create_function(move |_, (_ctx, player, keyword): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::GrantPlayerKeyword {
                source,
                player: parse_player(player)?,
                keyword,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "disable_player_keyword",
        lua.create_function(move |_, (_ctx, player, keyword): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::DisablePlayerKeyword {
                source,
                player: parse_player(player)?,
                keyword,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_player_class",
        lua.create_function(move |_, (_ctx, player, class): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::SetPlayerClass {
                source,
                player: parse_player(player)?,
                class,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "discard",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::Discard {
                source,
                player: parse_player(player)?,
                target: EntityId(target),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "cast_spell",
        lua.create_function(
            move |lua, (_ctx, player, card_id, spec): (Table, u8, String, Option<Table>)| {
                let spec = match spec {
                    Some(spec) => spec,
                    None => lua.create_table()?,
                };
                output.borrow_mut().push(EffectSpec::CastSpell {
                    source,
                    player: parse_player(player)?,
                    card_id,
                    target: spec.get::<Option<u64>>("target")?.map(EntityId),
                    skip_if_invalid: spec
                        .get::<Option<bool>>("skip_if_invalid")?
                        .unwrap_or(false),
                    random_target: spec.get::<Option<bool>>("random_target")?.unwrap_or(false),
                    choice_policy: parse_choice_policy(spec.get("choice_policy")?)?,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "cast_existing_spell",
        lua.create_function(
            move |lua, (_ctx, card, spec): (Table, u64, Option<Table>)| {
                let spec = match spec {
                    Some(spec) => spec,
                    None => lua.create_table()?,
                };
                output.borrow_mut().push(EffectSpec::CastExistingSpell {
                    source,
                    card: EntityId(card),
                    target: spec.get::<Option<u64>>("target")?.map(EntityId),
                    skip_if_invalid: spec
                        .get::<Option<bool>>("skip_if_invalid")?
                        .unwrap_or(false),
                    random_target: spec.get::<Option<bool>>("random_target")?.unwrap_or(false),
                    choice_policy: parse_choice_policy(spec.get("choice_policy")?)?,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "refresh_hero_power",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            output.borrow_mut().push(EffectSpec::RefreshHeroPower {
                source,
                player: parse_player(player)?,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon",
        lua.create_function(move |_, (_ctx, player, card_id): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::Summon {
                player: parse_player(player)?,
                card_id,
                position: None,
                attack: None,
                health: None,
                keywords: Vec::new(),
                base_stats: false,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_from_hand",
        lua.create_function(move |_, (_ctx, card): (Table, u64)| {
            output.borrow_mut().push(EffectSpec::SummonFromHand {
                card: EntityId(card),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_existing",
        lua.create_function(move |_, (_ctx, player, card): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::SummonExisting {
                source,
                player: parse_player(player)?,
                card: EntityId(card),
                position: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_existing_at",
        lua.create_function(
            move |_, (_ctx, player, card, position): (Table, u8, u64, usize)| {
                output.borrow_mut().push(EffectSpec::SummonExisting {
                    source,
                    player: parse_player(player)?,
                    card: EntityId(card),
                    position: Some(position),
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_at",
        lua.create_function(
            move |_, (_ctx, player, card_id, position): (Table, u8, String, usize)| {
                output.borrow_mut().push(EffectSpec::Summon {
                    player: parse_player(player)?,
                    card_id,
                    position: Some(position),
                    attack: None,
                    health: None,
                    keywords: Vec::new(),
                    base_stats: false,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_with_stats",
        lua.create_function(
            move |_,
                  (_ctx, player, card_id, attack, health, keywords): (
                Table,
                u8,
                String,
                i32,
                i32,
                Option<Table>,
            )| {
                let keywords = keywords
                    .map(|keywords| {
                        keywords
                            .sequence_values::<String>()
                            .collect::<mlua::Result<Vec<_>>>()
                    })
                    .transpose()?
                    .unwrap_or_default();
                output.borrow_mut().push(EffectSpec::Summon {
                    player: parse_player(player)?,
                    card_id,
                    position: None,
                    attack: Some(attack),
                    health: Some(health),
                    keywords,
                    base_stats: false,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_with_base_stats",
        lua.create_function(
            move |_,
                  (_ctx, player, card_id, attack, health, keywords): (
                Table,
                u8,
                String,
                i32,
                i32,
                Option<Table>,
            )| {
                let keywords = keywords
                    .map(|keywords| {
                        keywords
                            .sequence_values::<String>()
                            .collect::<mlua::Result<Vec<_>>>()
                    })
                    .transpose()?
                    .unwrap_or_default();
                output.borrow_mut().push(EffectSpec::Summon {
                    player: parse_player(player)?,
                    card_id,
                    position: None,
                    attack: Some(attack),
                    health: Some(health),
                    keywords,
                    base_stats: true,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_copy",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::SummonCopy {
                source,
                player: parse_player(player)?,
                target: EntityId(target),
                position: None,
                attack: None,
                health: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_copy_at",
        lua.create_function(
            move |_, (_ctx, player, target, position): (Table, u8, u64, usize)| {
                output.borrow_mut().push(EffectSpec::SummonCopy {
                    source,
                    player: parse_player(player)?,
                    target: EntityId(target),
                    position: Some(position),
                    attack: None,
                    health: None,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "summon_copy_with_stats",
        lua.create_function(
            move |_, (_ctx, player, target, attack, health): (Table, u8, u64, i32, i32)| {
                output.borrow_mut().push(EffectSpec::SummonCopy {
                    source,
                    player: parse_player(player)?,
                    target: EntityId(target),
                    position: None,
                    attack: Some(attack),
                    health: Some(health),
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "recruit",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::Recruit {
                source,
                player: parse_player(player)?,
                target: EntityId(target),
                position: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "recruit_at",
        lua.create_function(
            move |_, (_ctx, player, target, position): (Table, u8, u64, usize)| {
                output.borrow_mut().push(EffectSpec::Recruit {
                    source,
                    player: parse_player(player)?,
                    target: EntityId(target),
                    position: Some(position),
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "move",
        lua.create_function(
            move |_, (_ctx, target, destination): (Table, u64, String)| {
                output.borrow_mut().push(EffectSpec::MoveEntity {
                    source,
                    target: EntityId(target),
                    destination: parse_zone_placement(&destination)?,
                    destination_player: None,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "shuffle_entity_into_deck",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::MoveEntity {
                source,
                target: EntityId(target),
                destination: ZonePlacement::DeckRandom,
                destination_player: Some(parse_player(player)?),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "move_to_hand",
        lua.create_function(move |_, (_ctx, player, target): (Table, u8, u64)| {
            output.borrow_mut().push(EffectSpec::MoveEntity {
                source,
                target: EntityId(target),
                destination: ZonePlacement::Hand,
                destination_player: Some(parse_player(player)?),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "change_controller",
        lua.create_function(move |_, (_ctx, target, player): (Table, u64, u8)| {
            output.borrow_mut().push(EffectSpec::ChangeController {
                source,
                target: EntityId(target),
                player: parse_player(player)?,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "change_controller_until_end_of_turn",
        lua.create_function(move |_, (_ctx, target, player): (Table, u64, u8)| {
            output
                .borrow_mut()
                .push(EffectSpec::ChangeControllerUntilEndOfTurn {
                    source,
                    target: EntityId(target),
                    player: parse_player(player)?,
                });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "force_attack",
        lua.create_function(move |_, (_ctx, attacker, defender): (Table, u64, u64)| {
            output.borrow_mut().push(EffectSpec::ForceAttack {
                source,
                attacker: EntityId(attacker),
                defender: EntityId(defender),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "transform",
        lua.create_function(move |_, (_ctx, target, card_id): (Table, u64, String)| {
            output.borrow_mut().push(EffectSpec::Transform {
                source,
                target: EntityId(target),
                card_id,
                preserve_attached_scripts: false,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "transform_all",
        lua.create_function(
            move |_, (_ctx, targets, card_id): (Table, Vec<u64>, String)| {
                output.borrow_mut().push(EffectSpec::TransformGroup {
                    source,
                    targets: targets.into_iter().map(EntityId).collect(),
                    card_id,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "transform_batch",
        lua.create_function(move |_, (_ctx, transforms): (Table, Table)| {
            let mut parsed = Vec::new();
            for transform in transforms.sequence_values::<Table>() {
                let transform = transform?;
                parsed.push((
                    EntityId(transform.get::<u64>(1)?),
                    transform.get::<String>(2)?,
                ));
            }
            output.borrow_mut().push(EffectSpec::TransformBatch {
                source,
                transforms: parsed,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "exchange_zone_contents",
        lua.create_function(
            move |_, (_ctx, first, second, zone): (Table, u8, u8, String)| {
                output.borrow_mut().push(EffectSpec::ExchangeZoneContents {
                    source,
                    first: parse_player(first)?,
                    second: parse_player(second)?,
                    zone: parse_exchangeable_zone(&zone)?,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "transform_preserving_scripts",
        lua.create_function(move |_, (_ctx, target, card_id): (Table, u64, String)| {
            output.borrow_mut().push(EffectSpec::Transform {
                source,
                target: EntityId(target),
                card_id,
                preserve_attached_scripts: true,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "transform_into_copy",
        lua.create_function(
            move |_,
                  (_ctx, target, template, attack, health): (
                Table,
                u64,
                u64,
                Option<i32>,
                Option<i32>,
            )| {
                output.borrow_mut().push(EffectSpec::TransformIntoCopy {
                    source,
                    target: EntityId(target),
                    template: EntityId(template),
                    attack,
                    health,
                    preserve_attached_scripts: false,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "destroy",
        lua.create_function(move |_, (_ctx, target): (Table, u64)| {
            output.borrow_mut().push(EffectSpec::Destroy {
                source,
                target: EntityId(target),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_health",
        lua.create_function(move |_, (_ctx, target, health): (Table, u64, i32)| {
            output.borrow_mut().push(EffectSpec::SetHealth {
                source,
                target: EntityId(target),
                health,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "trigger_hook",
        lua.create_function(
            move |_, (_ctx, target, hook, payload): (Table, u64, String, Option<Value>)| {
                let payload = payload
                    .map(lua_to_choice_value)
                    .transpose()?
                    .filter(|value| !matches!(value, ChoiceValue::Nil));
                output.borrow_mut().push(EffectSpec::TriggerHook {
                    source,
                    target: EntityId(target),
                    hook,
                    payload,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "attach_hook",
        lua.create_function(
            move |_, (_ctx, target, hook, card_id): (Table, u64, String, String)| {
                output.borrow_mut().push(EffectSpec::AttachHook {
                    source,
                    target: EntityId(target),
                    hook,
                    card_id,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "attach_script",
        lua.create_function(move |_, (_ctx, target, card_id): (Table, u64, String)| {
            output.borrow_mut().push(EffectSpec::AttachScript {
                source,
                target: EntityId(target),
                card_id,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "buff",
        lua.create_function(
            move |_, (_ctx, target, attack, health): (Table, u64, i32, i32)| {
                output.borrow_mut().push(EffectSpec::Buff {
                    source,
                    target: EntityId(target),
                    attack,
                    health,
                    keywords: Vec::new(),
                    duration: EffectDuration::Permanent,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "destroy_all",
        lua.create_function(move |_, (_ctx, targets): (Table, Table)| {
            let targets = targets
                .sequence_values::<u64>()
                .map(|target| target.map(EntityId))
                .collect::<mlua::Result<Vec<_>>>()?;
            output
                .borrow_mut()
                .push(EffectSpec::DestroyGroup { source, targets });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "grant_keyword",
        lua.create_function(move |_, (_ctx, target, keyword): (Table, u64, String)| {
            output.borrow_mut().push(EffectSpec::Buff {
                source,
                target: EntityId(target),
                attack: 0,
                health: 0,
                keywords: vec![keyword],
                duration: EffectDuration::Permanent,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "grant_keyword_until_end_of_turn",
        lua.create_function(move |_, (_ctx, target, keyword): (Table, u64, String)| {
            output.borrow_mut().push(EffectSpec::Buff {
                source,
                target: EntityId(target),
                attack: 0,
                health: 0,
                keywords: vec![keyword],
                duration: EffectDuration::UntilEndOfTurn,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "disable_keyword",
        lua.create_function(move |_, (_ctx, target, keyword): (Table, u64, String)| {
            output.borrow_mut().push(EffectSpec::DisableKeyword {
                source,
                target: EntityId(target),
                keyword,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    let snapshot = state.clone();
    ctx.set(
        "summon_fresh_copy",
        lua.create_function(
            move |_,
                  (_ctx, target, position, health, without): (
                Table,
                u64,
                Option<usize>,
                i32,
                Table,
            )| {
                let target = EntityId(target);
                let player = snapshot
                    .entity(target)
                    .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {target}")))?
                    .controller;
                let without_keywords = without
                    .sequence_values::<String>()
                    .collect::<mlua::Result<Vec<_>>>()?;
                output.borrow_mut().push(EffectSpec::SummonFreshCopy {
                    source,
                    player,
                    target,
                    position,
                    attack: None,
                    health,
                    final_stats: false,
                    without_keywords,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    let snapshot = state.clone();
    ctx.set(
        "summon_fresh_copy_with_stats",
        lua.create_function(
            move |_,
                  (_ctx, target, position, attack, health, without): (
                Table,
                u64,
                Option<usize>,
                i32,
                i32,
                Table,
            )| {
                let target = EntityId(target);
                let player = snapshot
                    .entity(target)
                    .ok_or_else(|| mlua::Error::runtime(format!("unknown entity {target}")))?
                    .controller;
                let without_keywords = without
                    .sequence_values::<String>()
                    .collect::<mlua::Result<Vec<_>>>()?;
                output.borrow_mut().push(EffectSpec::SummonFreshCopy {
                    source,
                    player,
                    target,
                    position,
                    attack: Some(attack),
                    health,
                    final_stats: true,
                    without_keywords,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "silence",
        lua.create_function(move |_, (_ctx, target): (Table, u64)| {
            output.borrow_mut().push(EffectSpec::Silence {
                source,
                target: EntityId(target),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "freeze",
        lua.create_function(move |_, (_ctx, target): (Table, u64)| {
            output.borrow_mut().push(EffectSpec::Freeze {
                source,
                target: EntityId(target),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "reveal_secret",
        lua.create_function(move |_, (_ctx, secret): (Table, u64)| {
            output.borrow_mut().push(EffectSpec::RevealSecret {
                source,
                secret: EntityId(secret),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "cancel_event",
        lua.create_function(move |_, (_ctx, event): (Table, Table)| {
            let timing: String = event.get("timing")?;
            if timing != "before" {
                return Err(mlua::Error::runtime(
                    "cancel_event can only be used by a before trigger",
                ));
            }
            output.borrow_mut().push(EffectSpec::CancelEvent {
                source,
                event: EventId(event.get("event_id")?),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_event_amount",
        lua.create_function(move |_, (_ctx, event, amount): (Table, Table, i32)| {
            let timing: String = event.get("timing")?;
            if timing != "before" {
                return Err(mlua::Error::runtime(
                    "set_event_amount can only be used by a before trigger",
                ));
            }
            output.borrow_mut().push(EffectSpec::SetEventAmount {
                source,
                event: EventId(event.get("event_id")?),
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "multiply_event_amount",
        lua.create_function(move |_, (_ctx, event, factor): (Table, Table, i32)| {
            let timing: String = event.get("timing")?;
            if timing != "before" {
                return Err(mlua::Error::runtime(
                    "multiply_event_amount can only be used by a before trigger",
                ));
            }
            output.borrow_mut().push(EffectSpec::MultiplyEventAmount {
                source,
                event: EventId(event.get("event_id")?),
                factor,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "replace_trade_draw",
        lua.create_function(move |_, (_ctx, event, replacement): (Table, u64, u64)| {
            output.borrow_mut().push(EffectSpec::SetTradeDraw {
                source,
                event: EventId(event),
                replacement: EntityId(replacement),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "buff_until_end_of_turn",
        lua.create_function(
            move |_, (_ctx, target, attack, health): (Table, u64, i32, i32)| {
                output.borrow_mut().push(EffectSpec::Buff {
                    source,
                    target: EntityId(target),
                    attack,
                    health,
                    keywords: Vec::new(),
                    duration: EffectDuration::UntilEndOfTurn,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_attack_defender",
        lua.create_function(move |_, (_ctx, event, defender): (Table, u64, u64)| {
            output.borrow_mut().push(EffectSpec::SetAttackDefender {
                source,
                event: EventId(event),
                defender: EntityId(defender),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "add_attack_collateral",
        lua.create_function(
            move |_, (_ctx, event, targets, amount): (Table, u64, Vec<u64>, i32)| {
                output.borrow_mut().push(EffectSpec::AddAttackCollateral {
                    source,
                    event: EventId(event),
                    targets: targets.into_iter().map(EntityId).collect(),
                    amount,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_damage_target",
        lua.create_function(move |_, (_ctx, event, target): (Table, u64, u64)| {
            output.borrow_mut().push(EffectSpec::SetDamageTarget {
                source,
                event: EventId(event),
                target: EntityId(target),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_spell_target",
        lua.create_function(move |_, (_ctx, event, target): (Table, u64, u64)| {
            output.borrow_mut().push(EffectSpec::SetSpellTarget {
                source,
                event: EventId(event),
                target: EntityId(target),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "continue_with",
        lua.create_function(move |_, (_ctx, hook): (Table, String)| {
            output.borrow_mut().push(EffectSpec::Continue {
                source,
                hook,
                payload: None,
                continuation_owner: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "equip_weapon",
        lua.create_function(move |_, (_ctx, player, card_id): (Table, u8, String)| {
            output.borrow_mut().push(EffectSpec::EquipWeapon {
                source,
                player: parse_player(player)?,
                card_id,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "lose_weapon_durability",
        lua.create_function(move |_, (_ctx, weapon, amount): (Table, u64, i32)| {
            output.borrow_mut().push(EffectSpec::LoseWeaponDurability {
                source,
                weapon: EntityId(weapon),
                amount,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "continue_with_entity",
        lua.create_function(move |_, (_ctx, hook, entity): (Table, String, u64)| {
            output.borrow_mut().push(EffectSpec::Continue {
                source,
                hook,
                payload: Some(ChoiceValue::Entity(EntityId(entity))),
                continuation_owner: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "continue_with_card",
        lua.create_function(move |_, (_ctx, hook, card): (Table, String, String)| {
            output.borrow_mut().push(EffectSpec::Continue {
                source,
                hook,
                payload: Some(ChoiceValue::Card(card)),
                continuation_owner: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "continue_with_number",
        lua.create_function(move |_, (_ctx, hook, number): (Table, String, i32)| {
            output.borrow_mut().push(EffectSpec::Continue {
                source,
                hook,
                payload: Some(ChoiceValue::Number(number)),
                continuation_owner: None,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "modify",
        lua.create_function(move |_, (_ctx, target, spec): (Table, u64, Table)| {
            let stat = parse_stat(&spec.get::<String>("stat")?)?;
            let operation = parse_modifier_operation(&spec.get::<String>("operation")?)?;
            let duration = parse_duration(
                spec.get::<Option<String>>("duration")?
                    .as_deref()
                    .unwrap_or("permanent"),
            )?;
            output.borrow_mut().push(EffectSpec::ModifyStat {
                source,
                target: EntityId(target),
                modifier: StatModifier {
                    stat,
                    operation,
                    value: spec.get("value")?,
                },
                duration,
                silenciable: spec.get::<Option<bool>>("silenciable")?.unwrap_or(true),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "modify_all",
        lua.create_function(move |_, (_ctx, targets, spec): (Table, Table, Table)| {
            let targets = targets
                .sequence_values::<u64>()
                .map(|target| target.map(EntityId))
                .collect::<mlua::Result<Vec<_>>>()?;
            let duration = parse_duration(
                spec.get::<Option<String>>("duration")?
                    .as_deref()
                    .unwrap_or("permanent"),
            )?;
            let mut modifiers = Vec::new();
            for stat in ["attack", "health", "cost", "spell_damage"] {
                if let Some(value) = spec.get::<Option<i32>>(stat)? {
                    modifiers.push(StatModifier {
                        stat: parse_stat(stat)?,
                        operation: parse_modifier_operation(
                            spec.get::<Option<String>>("operation")?
                                .as_deref()
                                .unwrap_or("final_set"),
                        )?,
                        value,
                    });
                }
            }
            output.borrow_mut().push(EffectSpec::ModifyStatGroup {
                source,
                targets,
                modifiers,
                duration,
                silenciable: spec.get::<Option<bool>>("silenciable")?.unwrap_or(true),
                reset_damage: spec.get::<Option<bool>>("reset_damage")?.unwrap_or(false),
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "modify_batch",
        lua.create_function(move |_, (_ctx, entries): (Table, Table)| {
            let modifications = entries
                .sequence_values::<Table>()
                .map(|entry| {
                    let entry = entry?;
                    Ok(EntityStatModification {
                        target: EntityId(entry.get("target")?),
                        modifiers: parse_stat_modifiers(&entry)?,
                        duration: parse_duration(
                            entry
                                .get::<Option<String>>("duration")?
                                .as_deref()
                                .unwrap_or("permanent"),
                        )?,
                        silenciable: entry.get::<Option<bool>>("silenciable")?.unwrap_or(true),
                        reset_damage: entry.get::<Option<bool>>("reset_damage")?.unwrap_or(false),
                    })
                })
                .collect::<mlua::Result<Vec<_>>>()?;
            output.borrow_mut().push(EffectSpec::ModifyStatBatch {
                source,
                modifications,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "grant_keyword_until_next_turn",
        lua.create_function(move |_, (_ctx, target, keyword): (Table, u64, String)| {
            output
                .borrow_mut()
                .push(EffectSpec::GrantKeywordUntilNextTurn {
                    source,
                    target: EntityId(target),
                    keyword,
                });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "take_extra_turn",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            output.borrow_mut().push(EffectSpec::TakeExtraTurn {
                source,
                player: parse_player(player)?,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "win_game",
        lua.create_function(move |_, (_ctx, player): (Table, u8)| {
            output.borrow_mut().push(EffectSpec::WinGame {
                source,
                player: parse_player(player)?,
            });
            Ok(())
        })?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_data",
        lua.create_function(
            move |_, (_ctx, target, key, value): (Table, u64, String, i64)| {
                output.borrow_mut().push(EffectSpec::SetScriptData {
                    source,
                    target: EntityId(target),
                    key,
                    value,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "set_player_data",
        lua.create_function(
            move |_, (_ctx, player, key, value): (Table, u8, String, i64)| {
                output.borrow_mut().push(EffectSpec::SetPlayerScriptData {
                    source,
                    player: parse_player(player)?,
                    key,
                    value,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "increment_player_data",
        lua.create_function(
            move |_, (_ctx, player, key, delta): (Table, u8, String, i64)| {
                output
                    .borrow_mut()
                    .push(EffectSpec::IncrementPlayerScriptData {
                        source,
                        player: parse_player(player)?,
                        key,
                        delta,
                    });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "remove_enchantments_from",
        lua.create_function(
            move |_, (_ctx, target, enchantment_source): (Table, u64, u64)| {
                output
                    .borrow_mut()
                    .push(EffectSpec::RemoveEnchantmentsFromSource {
                        source,
                        target: EntityId(target),
                        enchantment_source: EntityId(enchantment_source),
                    });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "continue_with_value",
        lua.create_function(move |_, (_ctx, hook, value): (Table, String, Value)| {
            output.borrow_mut().push(EffectSpec::Continue {
                source,
                hook,
                payload: Some(lua_to_choice_value(value)?),
                continuation_owner: None,
            });
            Ok(())
        })?,
    )?;

    let snapshot = state.clone();
    let output = effects.clone();
    ctx.set(
        "choose_entities",
        lua.create_function(
            move |_,
                  (_ctx, player, prompt, candidates, resume_hook): (
                Table,
                u8,
                String,
                Table,
                String,
            )| {
                let mut options = Vec::new();
                for candidate in candidates.sequence_values::<u64>() {
                    let id = EntityId(candidate?);
                    let entity = snapshot.entity(id).ok_or_else(|| {
                        mlua::Error::runtime(format!("choice references unknown entity {id}"))
                    })?;
                    options.push(ChoiceOption {
                        label: entity.name.clone(),
                        value: ChoiceValue::Entity(id),
                    });
                }
                output.borrow_mut().push(EffectSpec::RequestChoice {
                    player: parse_player(player)?,
                    source,
                    prompt,
                    options,
                    resume_hook,
                    continuation_owner: None,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "choose_cards",
        lua.create_function(
            move |_,
                  (_ctx, player, prompt, candidates, resume_hook): (
                Table,
                u8,
                String,
                Table,
                String,
            )| {
                let options = candidates
                    .sequence_values::<String>()
                    .map(|candidate| {
                        candidate.map(|card_id| ChoiceOption {
                            label: card_id.clone(),
                            value: ChoiceValue::Card(card_id),
                        })
                    })
                    .collect::<mlua::Result<Vec<_>>>()?;
                output.borrow_mut().push(EffectSpec::RequestChoice {
                    player: parse_player(player)?,
                    source,
                    prompt,
                    options,
                    resume_hook,
                    continuation_owner: None,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "choose_options",
        lua.create_function(
            move |_,
                  (_ctx, player, prompt, choices, resume_hook): (
                Table,
                u8,
                String,
                Table,
                String,
            )| {
                let mut options = Vec::new();
                for choice in choices.sequence_values::<Table>() {
                    let choice = choice?;
                    options.push(ChoiceOption {
                        label: choice.get("label")?,
                        value: lua_to_choice_value(choice.get("value")?)?,
                    });
                }
                output.borrow_mut().push(EffectSpec::RequestChoice {
                    player: parse_player(player)?,
                    source,
                    prompt,
                    options,
                    resume_hook,
                    continuation_owner: None,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "discover_cards",
        lua.create_function(
            move |_,
                  (_ctx, player, prompt, candidates, count, resume_hook): (
                Table,
                u8,
                String,
                Table,
                usize,
                String,
            )| {
                let candidates = candidates
                    .sequence_values::<String>()
                    .collect::<mlua::Result<Vec<_>>>()?;
                output.borrow_mut().push(EffectSpec::DiscoverCards {
                    player: parse_player(player)?,
                    source,
                    prompt,
                    candidates,
                    count,
                    resume_hook,
                    continuation_owner: None,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "discover_entities",
        lua.create_function(
            move |_,
                  (_ctx, player, prompt, candidates, count, resume_hook): (
                Table,
                u8,
                String,
                Table,
                usize,
                String,
            )| {
                let candidates = candidates
                    .sequence_values::<u64>()
                    .map(|entity| entity.map(EntityId))
                    .collect::<mlua::Result<Vec<_>>>()?;
                output.borrow_mut().push(EffectSpec::DiscoverEntities {
                    player: parse_player(player)?,
                    source,
                    prompt,
                    candidates,
                    count,
                    resume_hook,
                    continuation_owner: None,
                });
                Ok(())
            },
        )?,
    )?;
    let output = effects.clone();
    ctx.set(
        "random_entity",
        lua.create_function(
            move |_, (_ctx, candidates, resume_hook): (Table, Table, String)| {
                let options = candidates
                    .sequence_values::<u64>()
                    .map(|candidate| candidate.map(|id| ChoiceValue::Entity(EntityId(id))))
                    .collect::<mlua::Result<Vec<_>>>()?;
                output.borrow_mut().push(EffectSpec::RandomChoice {
                    source,
                    options,
                    resume_hook,
                    continuation_owner: None,
                });
                Ok(())
            },
        )?,
    )?;
    ctx.set(
        "random_value",
        lua.create_function(
            move |_, (_ctx, candidates, resume_hook): (Table, Table, String)| {
                let options = candidates
                    .sequence_values::<Value>()
                    .map(|candidate| candidate.and_then(lua_to_choice_value))
                    .collect::<mlua::Result<Vec<_>>>()?;
                effects.borrow_mut().push(EffectSpec::RandomChoice {
                    source,
                    options,
                    resume_hook,
                    continuation_owner: None,
                });
                Ok(())
            },
        )?,
    )?;
    Ok(ctx)
}
