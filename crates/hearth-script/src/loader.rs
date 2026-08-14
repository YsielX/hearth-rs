use super::*;

#[derive(serde::Deserialize)]
struct LocaleCatalogEntry {
    id: String,
    name: String,
    #[serde(default)]
    text: String,
}

pub(super) fn load_locale_catalogs(
    root: &Path,
    cards: &mut BTreeMap<String, CardScript>,
    pack_hash: &mut u64,
) -> Result<(), ScriptLoadError> {
    for locale in Locale::ALL {
        let path = root.join("locales").join(format!("{}.json", locale.code()));
        if !path.exists() {
            continue;
        }
        let bytes = fs::read(&path).map_err(|source| ScriptLoadError::Io {
            path: path.clone(),
            source,
        })?;
        let entries: Vec<LocaleCatalogEntry> =
            serde_json::from_slice(&bytes).map_err(|source| ScriptLoadError::LocaleCatalog {
                path: path.clone(),
                source,
            })?;
        update_pack_hash(pack_hash, locale.code().as_bytes());
        update_pack_hash(pack_hash, &[0]);
        update_pack_hash(pack_hash, &bytes);
        update_pack_hash(pack_hash, &[0xff]);
        for entry in entries {
            if let Some(card) = cards.get_mut(&entry.id) {
                card.definition.localizations.insert(
                    locale,
                    LocalizedCardText {
                        name: entry.name,
                        text: entry.text,
                    },
                );
            }
        }
    }
    Ok(())
}

pub(super) fn register_card_module(
    lua: &Lua,
    cards: &mut BTreeMap<String, CardScript>,
    path: &Path,
    module: Table,
) -> Result<(), ScriptLoadError> {
    let definition = parse_definition(&module).map_err(|source| ScriptLoadError::Lua {
        path: path.to_owned(),
        source,
    })?;
    validate_module(&module, &definition).map_err(|source| ScriptLoadError::Lua {
        path: path.to_owned(),
        source,
    })?;
    let id = definition.id.clone();
    let module = lua
        .create_registry_value(module)
        .map_err(|source| ScriptLoadError::Lua {
            path: path.to_owned(),
            source,
        })?;
    if cards
        .insert(id.clone(), CardScript { definition, module })
        .is_some()
    {
        return Err(ScriptLoadError::DuplicateCard(id));
    }
    Ok(())
}

pub(super) fn register_keyword_module(
    lua: &Lua,
    keywords: &mut BTreeMap<String, KeywordScript>,
    path: &Path,
    module: Table,
) -> Result<(), ScriptLoadError> {
    let api_version: u32 = module
        .get("api_version")
        .map_err(|source| ScriptLoadError::Lua {
            path: path.to_owned(),
            source,
        })?;
    if api_version != 1 {
        return Err(ScriptLoadError::Lua {
            path: path.to_owned(),
            source: mlua::Error::runtime(format!(
                "unsupported keyword api_version {api_version}; expected 1"
            )),
        });
    }
    let id: String = module.get("id").map_err(|source| ScriptLoadError::Lua {
        path: path.to_owned(),
        source,
    })?;
    if id.trim().is_empty() {
        return Err(ScriptLoadError::Lua {
            path: path.to_owned(),
            source: mlua::Error::runtime("keyword id cannot be empty"),
        });
    }
    let _: Option<Table> = module.get("rules").map_err(|source| ScriptLoadError::Lua {
        path: path.to_owned(),
        source,
    })?;
    if let Some(hooks) =
        module
            .get::<Option<Table>>("hooks")
            .map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?
    {
        for pair in hooks.pairs::<String, Value>() {
            let (hook, value) = pair.map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?;
            if !matches!(hook.as_str(), "on_play" | "on_location_use") {
                return Err(ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source: mlua::Error::runtime(format!(
                        "keyword {id} uses unsupported lifecycle hook {hook}"
                    )),
                });
            }
            if !matches!(value, Value::Function(_)) {
                return Err(ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source: mlua::Error::runtime(format!(
                        "keyword {id} lifecycle hook {hook} must be a function"
                    )),
                });
            }
        }
    }
    if let Some(actions) =
        module
            .get::<Option<Table>>("actions")
            .map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?
    {
        for pair in actions.pairs::<String, Table>() {
            let (action, spec) = pair.map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?;
            if action.is_empty() || action.len() > 64 {
                return Err(ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source: mlua::Error::runtime("card action id must contain 1 to 64 bytes"),
                });
            }
            let _: Option<u8> = spec.get("cost").map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?;
            let _: Option<bool> =
                spec.get("spend_all_mana")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: path.to_owned(),
                        source,
                    })?;
            let _: Option<Function> =
                spec.get("condition")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: path.to_owned(),
                        source,
                    })?;
            let _: Option<Function> =
                spec.get("effect").map_err(|source| ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source,
                })?;
            if let Some(zones) = spec
                .get::<Option<Table>>("active_zones")
                .map_err(|source| ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source,
                })?
            {
                for zone in zones.sequence_values::<String>() {
                    let zone = zone.map_err(|source| ScriptLoadError::Lua {
                        path: path.to_owned(),
                        source,
                    })?;
                    if !matches!(zone.as_str(), "hand" | "board") {
                        return Err(ScriptLoadError::Lua {
                            path: path.to_owned(),
                            source: mlua::Error::runtime(format!(
                                "keyword {id} action {action} uses unsupported zone {zone}"
                            )),
                        });
                    }
                }
            }
        }
    }
    validate_triggers(&module, &format!("keyword {id}")).map_err(|source| {
        ScriptLoadError::Lua {
            path: path.to_owned(),
            source,
        }
    })?;
    if let Some(hooks) = module
        .get::<Option<Table>>("required_card_hooks")
        .map_err(|source| ScriptLoadError::Lua {
            path: path.to_owned(),
            source,
        })?
    {
        for hook in hooks.sequence_values::<String>() {
            let hook = hook.map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?;
            if hook.is_empty() || hook.len() > 64 {
                return Err(ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source: mlua::Error::runtime("required card hook must contain 1 to 64 bytes"),
                });
            }
        }
    }
    if let Some(actions) = module
        .get::<Option<Table>>("required_card_actions")
        .map_err(|source| ScriptLoadError::Lua {
            path: path.to_owned(),
            source,
        })?
    {
        for action in actions.sequence_values::<String>() {
            let action = action.map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?;
            if action.is_empty() || action.len() > 64 {
                return Err(ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source: mlua::Error::runtime("required card action must contain 1 to 64 bytes"),
                });
            }
        }
    }
    if let Some(fields) = module
        .get::<Option<Table>>("required_card_fields")
        .map_err(|source| ScriptLoadError::Lua {
            path: path.to_owned(),
            source,
        })?
    {
        for field in fields.sequence_values::<String>() {
            let field = field.map_err(|source| ScriptLoadError::Lua {
                path: path.to_owned(),
                source,
            })?;
            if field.is_empty() || field.len() > 64 {
                return Err(ScriptLoadError::Lua {
                    path: path.to_owned(),
                    source: mlua::Error::runtime("required card field must contain 1 to 64 bytes"),
                });
            }
        }
    }
    let module = lua
        .create_registry_value(module)
        .map_err(|source| ScriptLoadError::Lua {
            path: path.to_owned(),
            source,
        })?;
    if keywords
        .insert(id.clone(), KeywordScript { module })
        .is_some()
    {
        return Err(ScriptLoadError::DuplicateKeyword(id));
    }
    Ok(())
}

pub(super) fn update_pack_hash(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

pub(super) fn collect_lua_files(
    path: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), ScriptLoadError> {
    let entries = fs::read_dir(path).map_err(|source| ScriptLoadError::Io {
        path: path.to_owned(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ScriptLoadError::Io {
            path: path.to_owned(),
            source,
        })?;
        let child = entry.path();
        if child.is_dir() {
            collect_lua_files(&child, files)?;
        } else if child
            .extension()
            .is_some_and(|extension| extension == "lua")
        {
            files.push(child);
        }
    }
    Ok(())
}

pub(super) fn parse_definition(module: &Table) -> mlua::Result<CardDefinition> {
    let api_version: u32 = module.get("api_version")?;
    if api_version != 1 {
        return Err(mlua::Error::runtime(format!(
            "unsupported api_version {api_version}; expected 1"
        )));
    }
    let kind = match module.get::<String>("type")?.as_str() {
        "minion" => CardKind::Minion,
        "spell" => CardKind::Spell,
        "weapon" => CardKind::Weapon,
        "location" => CardKind::Location,
        "hero" => CardKind::Hero,
        "hero_power" => CardKind::HeroPower,
        value => {
            return Err(mlua::Error::runtime(format!(
                "unsupported card type {value}"
            )));
        }
    };
    let keywords = module
        .get::<Option<Table>>("keywords")?
        .map(|values| values.sequence_values::<String>().collect())
        .transpose()?
        .unwrap_or_default();
    let keyword_params = module
        .get::<Option<Table>>("keyword_params")?
        .map(|values| values.pairs::<String, i64>().collect())
        .transpose()?
        .unwrap_or_default();
    let legacy_requires_target = module
        .get::<Option<bool>>("requires_target")?
        .unwrap_or(false);
    let target_mode = match module.get::<Option<String>>("target_mode")?.as_deref() {
        Some("optional") => TargetMode::Optional,
        Some("required") => TargetMode::Required,
        Some("required_if_available") => TargetMode::RequiredIfAvailable,
        Some(value) => {
            return Err(mlua::Error::runtime(format!(
                "unsupported target_mode {value}; expected optional, required, or required_if_available"
            )));
        }
        None if legacy_requires_target => TargetMode::Required,
        None => TargetMode::Optional,
    };
    Ok(CardDefinition {
        id: module.get("id")?,
        name: module.get("name")?,
        text: module.get::<Option<String>>("text")?.unwrap_or_default(),
        localizations: BTreeMap::new(),
        set: module.get::<Option<String>>("set")?.unwrap_or_default(),
        kind,
        collectible: module.get::<Option<bool>>("collectible")?.unwrap_or(true),
        class: module
            .get::<Option<String>>("class")?
            .unwrap_or_else(|| "neutral".to_owned()),
        tags: module
            .get::<Option<Table>>("tags")?
            .map(|values| values.sequence_values::<String>().collect())
            .transpose()?
            .unwrap_or_default(),
        cost: module.get("cost")?,
        attack: module.get::<Option<i32>>("attack")?.unwrap_or(0),
        health: module.get::<Option<i32>>("health")?.unwrap_or(0),
        armor: module.get::<Option<i32>>("armor")?.unwrap_or(0),
        hero_power: module.get::<Option<String>>("hero_power")?,
        keywords,
        keyword_params,
        secret: module.get::<Option<bool>>("secret")?.unwrap_or(false),
        target_mode,
    })
}

pub(super) fn validate_module(module: &Table, definition: &CardDefinition) -> mlua::Result<()> {
    if definition.id.trim().is_empty() {
        return Err(mlua::Error::runtime("card id cannot be empty"));
    }
    if definition.class.trim().is_empty() {
        return Err(mlua::Error::runtime(format!(
            "card {} has an empty class",
            definition.id
        )));
    }
    if matches!(
        definition.kind,
        CardKind::Minion | CardKind::Weapon | CardKind::Location
    ) && definition.health <= 0
    {
        return Err(mlua::Error::runtime(format!(
            "minion, weapon, or location {} must have positive health/durability",
            definition.id
        )));
    }
    if definition.armor < 0 {
        return Err(mlua::Error::runtime(format!(
            "card {} cannot grant negative armor",
            definition.id
        )));
    }
    if definition.kind == CardKind::Hero {
        if definition.health <= 0 {
            return Err(mlua::Error::runtime(format!(
                "hero card {} must have positive health metadata",
                definition.id
            )));
        }
        if definition.hero_power.is_none() {
            return Err(mlua::Error::runtime(format!(
                "hero card {} must declare hero_power",
                definition.id
            )));
        }
    } else if definition.armor != 0 || definition.hero_power.is_some() {
        return Err(mlua::Error::runtime(format!(
            "only hero cards may declare armor or hero_power ({})",
            definition.id
        )));
    }
    let target_hook = if definition.kind == CardKind::Location {
        "location_targets"
    } else {
        "targets"
    };
    let targets = module.get::<Option<Function>>(target_hook)?;
    if definition.target_mode != TargetMode::Optional && targets.is_none() {
        return Err(mlua::Error::runtime(format!(
            "card {} has target_mode={} but no {target_hook} function",
            definition.id,
            definition.target_mode.as_str(),
        )));
    }
    let _: Option<Function> = module.get("on_play")?;
    let _: Option<Function> = module.get("on_location_use")?;
    if let Some(effects) = module.get::<Option<Table>>("action_effects")? {
        for pair in effects.pairs::<String, Value>() {
            let (action, value) = pair?;
            if action.is_empty() || action.len() > 64 || !matches!(value, Value::Function(_)) {
                return Err(mlua::Error::runtime(format!(
                    "card {} action effect {action:?} must be a function with a valid id",
                    definition.id
                )));
            }
        }
    }
    if let Some(actions) = module.get::<Option<Table>>("card_actions")? {
        for pair in actions.pairs::<String, Table>() {
            let (action, spec) = pair?;
            if action.is_empty() || action.len() > 64 {
                return Err(mlua::Error::runtime(format!(
                    "card {} has an invalid card action id",
                    definition.id
                )));
            }
            let _: Option<u8> = spec.get("cost")?;
            let _: Option<bool> = spec.get("spend_all_mana")?;
            let _: Option<Function> = spec.get("condition")?;
            if let Some(zones) = spec.get::<Option<Table>>("active_zones")? {
                for zone in zones.sequence_values::<String>() {
                    let zone = zone?;
                    if !matches!(zone.as_str(), "hand" | "board") {
                        return Err(mlua::Error::runtime(format!(
                            "card {} action {action} uses unsupported zone {zone}",
                            definition.id
                        )));
                    }
                }
            }
        }
    }
    if let Some(targets) = module.get::<Option<Table>>("action_targets")? {
        for pair in targets.pairs::<String, Value>() {
            let (action, value) = pair?;
            if action.is_empty() || action.len() > 64 || !matches!(value, Value::Function(_)) {
                return Err(mlua::Error::runtime(format!(
                    "card {} action target selector {action:?} must be a function with a valid id",
                    definition.id
                )));
            }
        }
    }
    if let Some(modes) = module.get::<Option<Table>>("action_target_modes")? {
        for pair in modes.pairs::<String, String>() {
            let (action, mode) = pair?;
            if action.is_empty()
                || action.len() > 64
                || !matches!(
                    mode.as_str(),
                    "optional" | "required" | "required_if_available"
                )
            {
                return Err(mlua::Error::runtime(format!(
                    "card {} action {action:?} has invalid target mode {mode:?}",
                    definition.id
                )));
            }
        }
    }
    validate_triggers(module, &format!("card {}", definition.id))?;

    if let Some(auras) = module.get::<Option<Table>>("auras")? {
        for (index, aura) in auras.sequence_values::<Table>().enumerate() {
            let aura = aura?;
            let _: Function = aura.get("targets")?;
            for field in ["attack", "health", "cost", "spell_damage"] {
                match aura.get::<Value>(field)? {
                    Value::Nil | Value::Function(_) => {}
                    Value::Integer(value) if i32::try_from(value).is_ok() => {}
                    _ => {
                        return Err(mlua::Error::runtime(format!(
                            "card {} aura {index} field {field} must be an integer or function",
                            definition.id
                        )));
                    }
                }
            }
            if let Some(keywords) = aura.get::<Option<Table>>("keywords")? {
                keywords
                    .sequence_values::<String>()
                    .collect::<mlua::Result<Vec<_>>>()?;
            }
            if let Some(zones) = aura.get::<Option<Table>>("active_zones")? {
                for zone in zones.sequence_values::<String>() {
                    let zone = zone?;
                    if !matches!(
                        zone.as_str(),
                        "hero"
                            | "hero_power"
                            | "deck"
                            | "hand"
                            | "board"
                            | "weapon"
                            | "secret"
                            | "graveyard"
                            | "removed"
                    ) {
                        return Err(mlua::Error::runtime(format!(
                            "card {} aura {index} uses unknown zone {zone}",
                            definition.id
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn validate_triggers(module: &Table, owner: &str) -> mlua::Result<()> {
    let Some(triggers) = module.get::<Option<Table>>("triggers")? else {
        return Ok(());
    };
    for (index, trigger) in triggers.sequence_values::<Table>().enumerate() {
        let trigger = trigger?;
        let event: String = trigger.get("event")?;
        if !matches!(
            event.as_str(),
            "game_started"
                | "turn_started"
                | "turn_ended"
                | "card_drawn"
                | "card_burned"
                | "card_created"
                | "fatigue"
                | "card_played"
                | "spell_cast"
                | "minion_played"
                | "weapon_played"
                | "location_played"
                | "card_countered"
                | "card_discarded"
                | "card_traded"
                | "trade_draw"
                | "minion_summoned"
                | "magnetized"
                | "weapon_equipped"
                | "weapon_destroyed"
                | "location_used"
                | "location_destroyed"
                | "hero_power_used"
                | "hero_power_replaced"
                | "hero_replaced"
                | "secret_played"
                | "secret_revealed"
                | "zone_changed"
                | "controller_changed"
                | "transformed"
                | "attack"
                | "damaged"
                | "damage_prevented"
                | "healed"
                | "armor_gained"
                | "overload_queued"
                | "mana_locked"
                | "mana_unlocked"
                | "overload_cleared"
                | "temporary_mana_gained"
                | "temporary_mana_expired"
                | "mana_crystals_gained"
                | "mana_crystals_destroyed"
                | "mana_spent"
                | "keyword_disabled"
                | "frozen"
                | "entity_died"
                | "conceded"
                | "game_ended"
                | "choice_requested"
                | "choice_made"
                | "random_choice_made"
                | "random_cards_sampled"
                | "random_entities_sampled"
        ) {
            return Err(mlua::Error::runtime(format!(
                "{owner} trigger {index} uses unknown event {event}"
            )));
        }
        let _: Option<Function> = trigger.get("condition")?;
        let _: Function = trigger.get("effect")?;
        if let Some(timing) = trigger.get::<Option<String>>("timing")?
            && !matches!(timing.as_str(), "before" | "after")
        {
            return Err(mlua::Error::runtime(format!(
                "{owner} trigger {index} uses unknown timing {timing}"
            )));
        }
        if let Some(zones) = trigger.get::<Option<Table>>("active_zones")? {
            for zone in zones.sequence_values::<String>() {
                let zone = zone?;
                if !matches!(
                    zone.as_str(),
                    "hero"
                        | "hero_power"
                        | "deck"
                        | "hand"
                        | "board"
                        | "weapon"
                        | "secret"
                        | "graveyard"
                        | "removed"
                ) {
                    return Err(mlua::Error::runtime(format!(
                        "{owner} trigger {index} uses unknown zone {zone}"
                    )));
                }
            }
        }
    }
    Ok(())
}
