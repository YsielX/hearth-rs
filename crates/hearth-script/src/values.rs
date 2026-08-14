use super::*;

pub(super) fn lua_to_choice_value(value: Value) -> mlua::Result<ChoiceValue> {
    struct Limits {
        nodes: usize,
        string_bytes: usize,
        tables: HashSet<usize>,
    }

    fn count_node(limits: &mut Limits, depth: usize) -> mlua::Result<()> {
        if depth > MAX_CHOICE_VALUE_DEPTH {
            return Err(mlua::Error::runtime(format!(
                "serialized value exceeds maximum depth {MAX_CHOICE_VALUE_DEPTH}"
            )));
        }
        limits.nodes += 1;
        if limits.nodes > MAX_CHOICE_VALUE_NODES {
            return Err(mlua::Error::runtime(format!(
                "serialized value exceeds maximum node count {MAX_CHOICE_VALUE_NODES}"
            )));
        }
        Ok(())
    }

    fn count_string(limits: &mut Limits, value: &str) -> mlua::Result<()> {
        limits.string_bytes = limits.string_bytes.saturating_add(value.len());
        if limits.string_bytes > MAX_CHOICE_VALUE_STRING_BYTES {
            return Err(mlua::Error::runtime(format!(
                "serialized value exceeds maximum string data {MAX_CHOICE_VALUE_STRING_BYTES} bytes"
            )));
        }
        Ok(())
    }

    fn convert(value: Value, depth: usize, limits: &mut Limits) -> mlua::Result<ChoiceValue> {
        count_node(limits, depth)?;
        match value {
            Value::Nil => Ok(ChoiceValue::Nil),
            Value::Boolean(value) => Ok(ChoiceValue::Boolean(value)),
            Value::Integer(value) => Ok(i32::try_from(value)
                .map(ChoiceValue::Number)
                .unwrap_or(ChoiceValue::Integer(value))),
            Value::String(value) => {
                let value = value.to_str()?.to_owned();
                count_string(limits, &value)?;
                Ok(ChoiceValue::Text(value))
            }
            Value::Table(table) => {
                let pointer = table.to_pointer() as usize;
                if !limits.tables.insert(pointer) {
                    return Err(mlua::Error::runtime(
                        "serialized tables cannot contain cycles or repeated table references",
                    ));
                }

                let mut integer_values = BTreeMap::new();
                let mut object_values = BTreeMap::new();
                for pair in table.pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    match key {
                        Value::Integer(index) if index >= 1 => {
                            let index = usize::try_from(index).map_err(|_| {
                                mlua::Error::runtime("serialized array index is too large")
                            })?;
                            integer_values.insert(index, value);
                        }
                        Value::String(key) => {
                            let key = key.to_str()?.to_owned();
                            count_string(limits, &key)?;
                            object_values.insert(key, value);
                        }
                        _ => {
                            return Err(mlua::Error::runtime(
                                "serialized table keys must be positive integers or UTF-8 strings",
                            ));
                        }
                    }
                }

                let result = if integer_values.is_empty() {
                    let mut values = BTreeMap::new();
                    for (key, value) in object_values {
                        values.insert(key, convert(value, depth + 1, limits)?);
                    }
                    ChoiceValue::Object(values)
                } else if object_values.is_empty()
                    && integer_values.keys().copied().eq(1..=integer_values.len())
                {
                    let mut values = Vec::with_capacity(integer_values.len());
                    for (_, value) in integer_values {
                        values.push(convert(value, depth + 1, limits)?);
                    }
                    ChoiceValue::List(values)
                } else {
                    return Err(mlua::Error::runtime(
                        "serialized table must be either a dense array or a string-keyed object",
                    ));
                };
                Ok(result)
            }
            Value::Number(_) => Err(mlua::Error::runtime(
                "serialized numbers must be integers; floating-point values are not supported",
            )),
            _ => Err(mlua::Error::runtime(
                "serialized values cannot contain functions, threads, userdata, or errors",
            )),
        }
    }

    convert(
        value,
        0,
        &mut Limits {
            nodes: 0,
            string_bytes: 0,
            tables: HashSet::new(),
        },
    )
}

pub(super) fn choice_value_to_lua(lua: &Lua, value: &ChoiceValue) -> mlua::Result<Value> {
    match value {
        ChoiceValue::Entity(entity) => {
            Ok(Value::Integer(mlua::Integer::try_from(entity.0).map_err(
                |_| mlua::Error::runtime("entity id cannot be represented as a Lua integer"),
            )?))
        }
        ChoiceValue::Card(card) | ChoiceValue::Text(card) => {
            Ok(Value::String(lua.create_string(card)?))
        }
        ChoiceValue::Number(number) => Ok(Value::Integer((*number).into())),
        ChoiceValue::Integer(number) => Ok(Value::Integer(*number)),
        ChoiceValue::Nil => Ok(Value::Nil),
        ChoiceValue::Boolean(value) => Ok(Value::Boolean(*value)),
        ChoiceValue::List(values) => {
            let table = lua.create_table_with_capacity(values.len(), 0)?;
            for (index, value) in values.iter().enumerate() {
                table.raw_set(index + 1, choice_value_to_lua(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
        ChoiceValue::Object(values) => {
            let table = lua.create_table_with_capacity(0, values.len())?;
            for (key, value) in values {
                table.raw_set(key.as_str(), choice_value_to_lua(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

pub(super) fn aura_active_in_zone(aura: &Table, zone: Zone) -> mlua::Result<bool> {
    let Some(zones) = aura.get::<Option<Table>>("active_zones")? else {
        return Ok(zone == Zone::Board);
    };
    for active in zones.sequence_values::<String>() {
        if active? == zone_name(zone) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn aura_stat_value(
    aura: &Table,
    field: &str,
    ctx: &Table,
    source: EntityId,
) -> mlua::Result<i32> {
    match aura.get::<Value>(field)? {
        Value::Nil => Ok(0),
        Value::Integer(value) => i32::try_from(value).map_err(|_| {
            mlua::Error::runtime(format!("aura {field} must fit in a signed 32-bit value"))
        }),
        Value::Function(function) => function.call((ctx.clone(), source.0)),
        _ => Err(mlua::Error::runtime(format!(
            "aura {field} must be an integer or function"
        ))),
    }
}

pub(super) fn entity_to_table(lua: &Lua, entity: &hearth_core::Entity) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("id", entity.id.0)?;
    table.set("card_id", entity.card_id.as_str())?;
    table.set("name", entity.name.as_str())?;
    table.set("controller", entity.controller.0)?;
    table.set("owner", entity.owner.0)?;
    table.set("attack", entity.attack)?;
    table.set("health", entity.health())?;
    table.set("max_health", entity.max_health)?;
    table.set("damage", entity.damage)?;
    table.set("armor", entity.armor)?;
    table.set("cost", entity.cost)?;
    table.set("spell_damage", entity.spell_damage)?;
    table.set("silenced", entity.silenced)?;
    table.set("frozen", entity.frozen)?;
    table.set("location_cooldown", entity.location_cooldown)?;
    table.set("attacks_this_turn", entity.attacks_this_turn)?;
    table.set("enchantments", entity.enchantments.len())?;
    table.set("cards_played_before", entity.cards_played_before)?;
    table.set("attack_at_death", entity.attack_at_death)?;
    table.set("started_in_deck", entity.started_in_deck)?;
    table.set("combo_active", entity.cards_played_before > 0)?;
    let keywords = lua.create_table()?;
    for (index, keyword) in entity.keywords.iter().enumerate() {
        keywords.set(index + 1, keyword.as_str())?;
    }
    table.set("keywords", keywords)?;
    let attached_cards = lua.create_table()?;
    for (index, card_id) in entity.attached_cards.iter().enumerate() {
        attached_cards.set(index + 1, card_id.as_str())?;
    }
    table.set("attached_cards", attached_cards)?;
    let attached_deathrattles = lua.create_table()?;
    for (index, card_id) in entity.attached_deathrattles.iter().enumerate() {
        attached_deathrattles.set(index + 1, card_id.as_str())?;
    }
    table.set("attached_deathrattles", attached_deathrattles)?;
    table.set("zone", zone_name(entity.zone))?;
    table.set(
        "type",
        match entity.kind {
            CardKind::Hero => "hero",
            CardKind::Minion => "minion",
            CardKind::Spell => "spell",
            CardKind::Weapon => "weapon",
            CardKind::Location => "location",
            CardKind::HeroPower => "hero_power",
        },
    )?;
    Ok(table)
}

pub(super) fn card_definition_to_table(lua: &Lua, card: &CardDefinition) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("id", card.id.as_str())?;
    table.set("name", card.name.as_str())?;
    table.set("text", card.text.as_str())?;
    table.set("set", card.set.as_str())?;
    table.set("type", card_kind_name(card.kind))?;
    table.set("collectible", card.collectible)?;
    table.set("class", card.class.as_str())?;
    table.set(
        "classes",
        lua.create_sequence_from(card.classes.iter().map(String::as_str))?,
    )?;
    table.set("rarity", card.rarity.as_deref())?;
    table.set("spell_school", card.spell_school.as_deref())?;
    table.set("cost", card.cost)?;
    table.set("attack", card.attack)?;
    table.set("health", card.health)?;
    table.set("armor", card.armor)?;
    table.set("hero_power", card.hero_power.as_deref())?;
    table.set("secret", card.secret)?;
    table.set("target_mode", card.target_mode.as_str())?;
    table.set("requires_target", card.target_mode == TargetMode::Required)?;
    let keywords = lua.create_table()?;
    for (index, keyword) in card.keywords.iter().enumerate() {
        keywords.set(index + 1, keyword.as_str())?;
    }
    table.set("keywords", keywords)?;
    let keyword_params = lua.create_table()?;
    for (keyword, value) in &card.keyword_params {
        keyword_params.set(keyword.as_str(), *value)?;
    }
    table.set("keyword_params", keyword_params)?;
    let tags = lua.create_table()?;
    for (index, tag) in card.tags.iter().enumerate() {
        tags.set(index + 1, tag.as_str())?;
    }
    table.set("tags", tags)?;
    Ok(table)
}

pub(super) fn card_kind_name(kind: CardKind) -> &'static str {
    match kind {
        CardKind::Hero => "hero",
        CardKind::Minion => "minion",
        CardKind::Spell => "spell",
        CardKind::Weapon => "weapon",
        CardKind::Location => "location",
        CardKind::HeroPower => "hero_power",
    }
}

pub(super) fn event_to_table(lua: &Lua, script_event: &ScriptEvent) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let event = &script_event.event;
    table.set("name", event.script_name())?;
    table.set("event_id", script_event.id.0)?;
    table.set("timing", timing_name(script_event.timing))?;
    match event {
        GameEvent::GameStarted => {}
        GameEvent::TurnStarted { player, turn } | GameEvent::TurnEnded { player, turn } => {
            table.set("player", player.0)?;
            table.set("turn", *turn)?;
        }
        GameEvent::CardDrawn {
            player,
            card,
            source,
        }
        | GameEvent::CardBurned {
            player,
            card,
            source,
        } => {
            table.set("player", player.0)?;
            table.set("entity", card.0)?;
            table.set("source", source.map(|entity| entity.0))?;
        }
        GameEvent::CardPlayed { player, card, cost } => {
            table.set("player", player.0)?;
            table.set("entity", card.0)?;
            table.set("cost", *cost)?;
        }
        GameEvent::CardCountered { player, card } | GameEvent::CardTraded { player, card } => {
            table.set("player", player.0)?;
            table.set("entity", card.0)?;
        }
        GameEvent::CardCreated {
            source,
            player,
            card,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("entity", card.0)?;
        }
        GameEvent::TradeDraw {
            player,
            card,
            replacement,
        } => {
            table.set("player", player.0)?;
            table.set("entity", card.0)?;
            table.set("replacement", replacement.map(|entity| entity.0))?;
        }
        GameEvent::CardDiscarded {
            source,
            player,
            card,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("entity", card.0)?;
        }
        GameEvent::SpellCast {
            player,
            spell,
            generated_by,
            target,
            ..
        } => {
            table.set("player", player.0)?;
            table.set("entity", spell.0)?;
            table.set("generated", generated_by.is_some())?;
            table.set("player_cast", generated_by.is_none())?;
            table.set("generated_by", generated_by.map(|entity| entity.0))?;
            table.set("target", target.map(|entity| entity.0))?;
        }
        GameEvent::SpellTargeted {
            player,
            spell,
            target,
            generated_by,
        } => {
            table.set("player", player.0)?;
            table.set("entity", spell.0)?;
            table.set("target", target.0)?;
            table.set("generated", generated_by.is_some())?;
            table.set("player_cast", generated_by.is_none())?;
            table.set("generated_by", generated_by.map(|entity| entity.0))?;
        }
        GameEvent::MinionPlayed { player, minion } => {
            table.set("player", player.0)?;
            table.set("entity", minion.0)?;
        }
        GameEvent::WeaponPlayed { player, weapon } => {
            table.set("player", player.0)?;
            table.set("entity", weapon.0)?;
        }
        GameEvent::LocationPlayed { player, location }
        | GameEvent::LocationDestroyed { player, location } => {
            table.set("player", player.0)?;
            table.set("entity", location.0)?;
        }
        GameEvent::LocationUsed {
            player,
            location,
            target,
        } => {
            table.set("player", player.0)?;
            table.set("entity", location.0)?;
            table.set("target", target.map(|entity| entity.0))?;
        }
        GameEvent::Fatigue { player, amount } => {
            table.set("player", player.0)?;
            table.set("amount", *amount)?;
        }
        GameEvent::MinionSummoned { player, entity } => {
            table.set("player", player.0)?;
            table.set("entity", entity.0)?;
        }
        GameEvent::Magnetized {
            player,
            attachment,
            target,
        } => {
            table.set("player", player.0)?;
            table.set("entity", attachment.0)?;
            table.set("target", target.0)?;
        }
        GameEvent::WeaponEquipped { player, weapon }
        | GameEvent::WeaponDestroyed { player, weapon } => {
            table.set("player", player.0)?;
            table.set("entity", weapon.0)?;
        }
        GameEvent::HeroPowerUsed {
            player,
            hero_power,
            target,
        } => {
            table.set("player", player.0)?;
            table.set("entity", hero_power.0)?;
            table.set("target", target.map(|entity| entity.0))?;
        }
        GameEvent::HeroPowerReplaced {
            source,
            player,
            old,
            new,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("old", old.0)?;
            table.set("new", new.0)?;
        }
        GameEvent::HeroReplaced { player, old, new } => {
            table.set("player", player.0)?;
            table.set("old", old.0)?;
            table.set("new", new.0)?;
        }
        GameEvent::SecretPlayed { player, secret }
        | GameEvent::SecretRevealed { player, secret } => {
            table.set("player", player.0)?;
            table.set("entity", secret.0)?;
        }
        GameEvent::ZoneChanged { entity, from, to } => {
            table.set("entity", entity.0)?;
            table.set("from", zone_name(*from))?;
            table.set("to", zone_name(*to))?;
        }
        GameEvent::ControllerChanged {
            source,
            entity,
            from,
            to,
        } => {
            table.set("source", source.0)?;
            table.set("entity", entity.0)?;
            table.set("from", from.0)?;
            table.set("to", to.0)?;
        }
        GameEvent::Transformed {
            source,
            entity,
            from_card,
            to_card,
        } => {
            table.set("source", source.0)?;
            table.set("entity", entity.0)?;
            table.set("from_card", from_card.as_str())?;
            table.set("to_card", to_card.as_str())?;
        }
        GameEvent::Attack {
            attacker, defender, ..
        } => {
            table.set("attacker", attacker.0)?;
            table.set("defender", defender.0)?;
        }
        GameEvent::Damaged {
            source,
            target,
            amount,
        }
        | GameEvent::Healed {
            source,
            target,
            amount,
        } => {
            table.set("source", source.0)?;
            table.set("target", target.0)?;
            table.set("amount", *amount)?;
        }
        GameEvent::DamagePrevented {
            source,
            target,
            reason,
        } => {
            table.set("source", source.0)?;
            table.set("target", target.0)?;
            table.set("reason", reason.as_str())?;
        }
        GameEvent::ArmorGained {
            source,
            target,
            amount,
        } => {
            table.set("source", source.0)?;
            table.set("target", target.0)?;
            table.set("amount", *amount)?;
        }
        GameEvent::OverloadQueued {
            source,
            player,
            amount,
        }
        | GameEvent::ManaUnlocked {
            source,
            player,
            amount,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("amount", *amount)?;
        }
        GameEvent::OverloadCleared {
            source,
            player,
            pending,
            locked,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("pending", *pending)?;
            table.set("locked", *locked)?;
        }
        GameEvent::ManaLocked { player, amount } => {
            table.set("player", player.0)?;
            table.set("amount", *amount)?;
        }
        GameEvent::TemporaryManaGained {
            source,
            player,
            amount,
        }
        | GameEvent::ManaCrystalsDestroyed {
            source,
            player,
            amount,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("amount", *amount)?;
        }
        GameEvent::TemporaryManaExpired { player, amount } => {
            table.set("player", player.0)?;
            table.set("amount", *amount)?;
        }
        GameEvent::ManaCrystalsGained {
            source,
            player,
            amount,
            filled,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("amount", *amount)?;
            table.set("filled", *filled)?;
        }
        GameEvent::ManaSpent {
            player,
            source,
            amount,
            temporary,
        } => {
            table.set("player", player.0)?;
            table.set("source", source.0)?;
            table.set("amount", *amount)?;
            table.set("temporary", *temporary)?;
        }
        GameEvent::PlayerScriptDataChanged {
            source,
            player,
            key,
            old,
            new,
        } => {
            table.set("source", source.0)?;
            table.set("player", player.0)?;
            table.set("key", key.as_str())?;
            table.set("old", *old)?;
            table.set("new", *new)?;
            table.set("delta", new.saturating_sub(*old))?;
        }
        GameEvent::KeywordDisabled {
            source,
            target,
            keyword,
        } => {
            table.set("source", source.0)?;
            table.set("target", target.0)?;
            table.set("keyword", keyword.as_str())?;
        }
        GameEvent::Frozen { source, target } => {
            table.set("source", source.0)?;
            table.set("target", target.0)?;
        }
        GameEvent::EntityDied {
            entity,
            player,
            position,
            source,
            repetitions,
        } => {
            table.set("entity", entity.0)?;
            table.set("player", player.0)?;
            table.set("position", *position)?;
            table.set("source", source.map(|source| source.0))?;
            table.set("repetitions", *repetitions)?;
        }
        GameEvent::Conceded { player } => table.set("player", player.0)?,
        GameEvent::GameEnded { outcome } => match outcome {
            GameOutcome::Winner(winner) => {
                table.set("outcome", "winner")?;
                table.set("winner", winner.0)?;
            }
            GameOutcome::Draw => table.set("outcome", "draw")?,
        },
        GameEvent::ChoiceRequested {
            player,
            source,
            options,
        } => {
            table.set("player", player.0)?;
            table.set("source", source.0)?;
            table.set("options", *options)?;
        }
        GameEvent::ChoiceMade {
            player,
            source,
            index,
        } => {
            table.set("player", player.0)?;
            table.set("source", source.0)?;
            table.set("index", *index)?;
        }
        GameEvent::RandomChoiceMade {
            source,
            index,
            options,
        } => {
            table.set("source", source.0)?;
            table.set("index", *index)?;
            table.set("options", *options)?;
        }
        GameEvent::RandomCardsSampled {
            source,
            cards,
            population,
        } => {
            table.set("source", source.0)?;
            table.set("population", *population)?;
            let sampled = lua.create_table()?;
            for (index, card) in cards.iter().enumerate() {
                sampled.set(index + 1, card.as_str())?;
            }
            table.set("cards", sampled)?;
        }
        GameEvent::RandomEntitiesSampled {
            source,
            entities,
            population,
        } => {
            table.set("source", source.0)?;
            table.set("population", *population)?;
            let sampled = lua.create_table()?;
            for (index, entity) in entities.iter().enumerate() {
                sampled.set(index + 1, entity.0)?;
            }
            table.set("entities", sampled)?;
        }
    }
    Ok(table)
}

pub(super) fn timing_name(timing: EventTiming) -> &'static str {
    match timing {
        EventTiming::Before => "before",
        EventTiming::After => "after",
    }
}

pub(super) fn zone_is_active(trigger: &Table, current: Zone) -> Result<bool, String> {
    let zones = trigger
        .get::<Option<Table>>("active_zones")
        .map_err(|error| error.to_string())?;
    let Some(zones) = zones else {
        return Ok(current == Zone::Board);
    };
    for zone in zones.sequence_values::<String>() {
        if zone.map_err(|error| error.to_string())? == zone_name(current) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn zone_name(zone: Zone) -> &'static str {
    match zone {
        Zone::Hero => "hero",
        Zone::SetAside => "set_aside",
        Zone::HeroPower => "hero_power",
        Zone::Deck => "deck",
        Zone::Hand => "hand",
        Zone::Board => "board",
        Zone::Weapon => "weapon",
        Zone::Secret => "secret",
        Zone::Graveyard => "graveyard",
        Zone::Removed => "removed",
    }
}

pub(super) fn parse_stat(value: &str) -> mlua::Result<Stat> {
    match value {
        "attack" => Ok(Stat::Attack),
        "health" => Ok(Stat::Health),
        "cost" => Ok(Stat::Cost),
        "spell_damage" => Ok(Stat::SpellDamage),
        _ => Err(mlua::Error::runtime(format!(
            "unknown modifier stat {value}"
        ))),
    }
}

pub(super) fn parse_player(value: u8) -> mlua::Result<PlayerId> {
    match value {
        0 => Ok(PlayerId::ONE),
        1 => Ok(PlayerId::TWO),
        _ => Err(mlua::Error::runtime(format!(
            "player must be 0 or 1, got {value}"
        ))),
    }
}

pub(super) fn parse_modifier_operation(value: &str) -> mlua::Result<ModifierOperation> {
    match value {
        "set" => Ok(ModifierOperation::Set),
        "add" => Ok(ModifierOperation::Add),
        "pre_final_add" => Ok(ModifierOperation::PreFinalAdd),
        "multiply" => Ok(ModifierOperation::Multiply),
        "final_set" => Ok(ModifierOperation::FinalSet),
        _ => Err(mlua::Error::runtime(format!(
            "unknown modifier operation {value}"
        ))),
    }
}

pub(super) fn parse_duration(value: &str) -> mlua::Result<EffectDuration> {
    match value {
        "permanent" => Ok(EffectDuration::Permanent),
        "end_of_turn" => Ok(EffectDuration::UntilEndOfTurn),
        _ => Err(mlua::Error::runtime(format!(
            "unknown modifier duration {value}"
        ))),
    }
}

pub(super) fn parse_zone_placement(value: &str) -> mlua::Result<ZonePlacement> {
    match value {
        "hand" => Ok(ZonePlacement::Hand),
        "board" => Ok(ZonePlacement::Board),
        "secret" => Ok(ZonePlacement::Secret),
        "deck_top" => Ok(ZonePlacement::DeckTop),
        "deck_bottom" => Ok(ZonePlacement::DeckBottom),
        "deck_random" => Ok(ZonePlacement::DeckRandom),
        "graveyard" => Ok(ZonePlacement::Graveyard),
        "removed" => Ok(ZonePlacement::Removed),
        _ => Err(mlua::Error::runtime(format!(
            "unknown zone destination {value}"
        ))),
    }
}
