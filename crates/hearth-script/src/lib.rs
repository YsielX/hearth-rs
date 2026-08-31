use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use hearth_core::{
    AuraSpec, CardActionSpec, CardCopyState, CardDefinition, CardKind, CardRuntime, ChoiceOption,
    ChoicePolicy, ChoiceValue, DeckAllowance, EffectDuration, EffectSpec, EntityId,
    EntityStatModification, EventId, EventTiming, FreshCopyStats, GameEvent, GameOutcome,
    GameState, Locale, LocalizedCardText, MAX_CHOICE_VALUE_DEPTH, MAX_CHOICE_VALUE_NODES,
    MAX_CHOICE_VALUE_STRING_BYTES, MinionStats, ModifierOperation, PlayerId, RuneCost, ScriptEvent,
    Stat, StatModifier, SummonStats, TargetMode, Zone, ZonePlacement,
};
use mlua::{Function, HookTriggers, Lua, RegistryKey, Table, Value, VmState};
use thiserror::Error;

mod context;
mod loader;
mod values;

use context::build_context;
use loader::*;
use values::*;

#[derive(Debug, Error)]
pub enum ScriptLoadError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid Lua card {path}: {source}")]
    Lua { path: PathBuf, source: mlua::Error },
    #[error("duplicate card id {0}")]
    DuplicateCard(String),
    #[error("duplicate keyword id {0}")]
    DuplicateKeyword(String),
    #[error("card {card} references unknown keyword {keyword}")]
    UnknownKeyword { card: String, keyword: String },
    #[error(
        "card {card} references keyword {keyword} but does not define required Lua hook {hook}"
    )]
    MissingKeywordHook {
        card: String,
        keyword: String,
        hook: String,
    },
    #[error(
        "card {card} references keyword {keyword} but does not define required action effect {action}"
    )]
    MissingKeywordAction {
        card: String,
        keyword: String,
        action: String,
    },
    #[error(
        "card {card} references keyword {keyword} but does not define required Lua field {field}"
    )]
    MissingKeywordField {
        card: String,
        keyword: String,
        field: String,
    },
    #[error("card {card} defines a parameter for unreferenced keyword {keyword}")]
    UnreferencedKeywordParam { card: String, keyword: String },
    #[error(
        "card {card} references keyword {keyword} but does not define keyword_params.{keyword}"
    )]
    MissingKeywordParam { card: String, keyword: String },
    #[error("no Lua card files found in {0}")]
    NoCards(PathBuf),
    #[error("invalid locale catalog {path}: {source}")]
    LocaleCatalog {
        path: PathBuf,
        source: serde_json::Error,
    },
}

struct CardScript {
    definition: CardDefinition,
    module: RegistryKey,
    source_path: Arc<str>,
    source: Arc<str>,
}

struct KeywordScript {
    module: RegistryKey,
}

/// Sandboxed Lua-backed card catalog and hook dispatcher.
pub struct LuaCardRuntime {
    lua: Lua,
    cards: BTreeMap<String, CardScript>,
    keywords: BTreeMap<String, KeywordScript>,
    catalog: Arc<BTreeMap<String, CardDefinition>>,
    instruction_blocks: Rc<Cell<u32>>,
    pack_hash: String,
    locale: Locale,
}

impl LuaCardRuntime {
    pub fn load_dir(path: impl AsRef<Path>) -> Result<Self, ScriptLoadError> {
        let root = path.as_ref();
        let lua = Lua::new();
        let globals = lua.globals();
        for name in [
            "dofile", "loadfile", "require", "package", "io", "os", "debug",
        ] {
            globals
                .set(name, Value::Nil)
                .map_err(|source| ScriptLoadError::Lua {
                    path: root.to_owned(),
                    source,
                })?;
        }
        if let Ok(math) = globals.get::<Table>("math") {
            math.set("random", Value::Nil)
                .and_then(|_| math.set("randomseed", Value::Nil))
                .map_err(|source| ScriptLoadError::Lua {
                    path: root.to_owned(),
                    source,
                })?;
        }
        globals
            .set(
                "cardlib",
                lua.create_table().map_err(|source| ScriptLoadError::Lua {
                    path: root.to_owned(),
                    source,
                })?,
            )
            .map_err(|source| ScriptLoadError::Lua {
                path: root.to_owned(),
                source,
            })?;
        // A runaway card should not be able to consume arbitrary process memory.
        lua.set_memory_limit(16 * 1024 * 1024)
            .map_err(|source| ScriptLoadError::Lua {
                path: root.to_owned(),
                source,
            })?;
        let instruction_blocks = Rc::new(Cell::new(0));
        let hook_blocks = instruction_blocks.clone();
        lua.set_hook(
            HookTriggers::new().every_nth_instruction(1_000),
            move |_, _| {
                let next = hook_blocks.get() + 1;
                hook_blocks.set(next);
                if next > 200 {
                    Err(mlua::Error::runtime(
                        "card script exceeded the 200,000 instruction budget",
                    ))
                } else {
                    Ok(VmState::Continue)
                }
            },
        )
        .map_err(|source| ScriptLoadError::Lua {
            path: root.to_owned(),
            source,
        })?;
        let mut files = Vec::new();
        collect_lua_files(root, &mut files)?;
        files.sort_by(|left, right| {
            let is_library = |path: &PathBuf| {
                path.strip_prefix(root).is_ok_and(|relative| {
                    relative
                        .components()
                        .any(|component| component.as_os_str() == "libraries")
                })
            };
            (!is_library(left), left).cmp(&(!is_library(right), right))
        });
        if files.is_empty() {
            return Err(ScriptLoadError::NoCards(root.to_owned()));
        }

        let mut cards = BTreeMap::new();
        let mut keywords = BTreeMap::new();
        let mut pack_hash = 0xcbf29ce484222325_u64;
        for path in files {
            instruction_blocks.set(0);
            let source = fs::read_to_string(&path).map_err(|source| ScriptLoadError::Io {
                path: path.clone(),
                source,
            })?;
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            update_pack_hash(&mut pack_hash, relative.as_bytes());
            update_pack_hash(&mut pack_hash, &[0]);
            update_pack_hash(&mut pack_hash, source.as_bytes());
            update_pack_hash(&mut pack_hash, &[0xff]);
            let source_path: Arc<str> = Arc::from(relative);
            let source: Arc<str> = Arc::from(source);
            let module: Table = lua
                .load(&*source)
                .set_name(path.to_string_lossy())
                .eval()
                .map_err(|source| ScriptLoadError::Lua {
                    path: path.clone(),
                    source,
                })?;
            let module_type = module
                .get::<Option<String>>("module_type")
                .map_err(|source| ScriptLoadError::Lua {
                    path: path.clone(),
                    source,
                })?
                .unwrap_or_else(|| "card".to_owned());
            if module_type == "library" {
                register_library_module(&lua, root, &path, module)?;
                continue;
            }
            if module_type == "keyword" {
                register_keyword_module(&lua, &mut keywords, &path, module)?;
                continue;
            }
            if module_type == "hero_power" {
                if let Some(declared_type) =
                    module
                        .get::<Option<String>>("type")
                        .map_err(|source| ScriptLoadError::Lua {
                            path: path.clone(),
                            source,
                        })?
                    && declared_type != "hero_power"
                {
                    return Err(ScriptLoadError::Lua {
                        path: path.clone(),
                        source: mlua::Error::runtime(
                            "hero_power modules cannot declare a different card type",
                        ),
                    });
                }
                module
                    .set("type", "hero_power")
                    .and_then(|_| module.set("collectible", false))
                    .map_err(|source| ScriptLoadError::Lua {
                        path: path.clone(),
                        source,
                    })?;
            } else if module_type != "card" {
                return Err(ScriptLoadError::Lua {
                    path: path.clone(),
                    source: mlua::Error::runtime(format!(
                        "unsupported module_type {module_type}; expected card, hero_power, keyword, or library"
                    )),
                });
            }
            let api_version: u32 =
                module
                    .get("api_version")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: path.clone(),
                        source,
                    })?;
            let tokens = module
                .get::<Option<Table>>("tokens")
                .map_err(|source| ScriptLoadError::Lua {
                    path: path.clone(),
                    source,
                })?
                .map(|tokens| {
                    tokens
                        .sequence_values::<Table>()
                        .collect::<mlua::Result<Vec<_>>>()
                })
                .transpose()
                .map_err(|source| ScriptLoadError::Lua {
                    path: path.clone(),
                    source,
                })?
                .unwrap_or_default();
            register_card_module(
                &lua,
                &mut cards,
                &path,
                module,
                source_path.clone(),
                source.clone(),
            )?;
            for token in tokens {
                if token
                    .get::<Option<u32>>("api_version")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: path.clone(),
                        source,
                    })?
                    .is_none()
                {
                    token.set("api_version", api_version).map_err(|source| {
                        ScriptLoadError::Lua {
                            path: path.clone(),
                            source,
                        }
                    })?;
                }
                if token
                    .get::<Option<bool>>("collectible")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: path.clone(),
                        source,
                    })?
                    .is_none()
                {
                    token
                        .set("collectible", false)
                        .map_err(|source| ScriptLoadError::Lua {
                            path: path.clone(),
                            source,
                        })?;
                }
                register_card_module(
                    &lua,
                    &mut cards,
                    &path,
                    token,
                    source_path.clone(),
                    source.clone(),
                )?;
            }
        }
        load_locale_catalogs(root, &mut cards, &mut pack_hash)?;
        for card in cards.values() {
            if card.definition.kind != CardKind::Hero {
                continue;
            }
            let Some(hero_power) = card.definition.hero_power.as_deref() else {
                return Err(ScriptLoadError::Lua {
                    path: root.to_owned(),
                    source: mlua::Error::runtime(format!(
                        "hero card {} must declare hero_power",
                        card.definition.id
                    )),
                });
            };
            if !cards
                .get(hero_power)
                .is_some_and(|power| power.definition.kind == CardKind::HeroPower)
            {
                return Err(ScriptLoadError::Lua {
                    path: root.to_owned(),
                    source: mlua::Error::runtime(format!(
                        "hero card {} references invalid hero power {hero_power}",
                        card.definition.id
                    )),
                });
            }
        }
        for card in cards.values() {
            let card_module: Table =
                lua.registry_value(&card.module)
                    .map_err(|source| ScriptLoadError::Lua {
                        path: root.to_owned(),
                        source,
                    })?;
            for keyword in &card.definition.keywords {
                let Some(keyword_script) = keywords.get(keyword) else {
                    return Err(ScriptLoadError::UnknownKeyword {
                        card: card.definition.id.clone(),
                        keyword: keyword.clone(),
                    });
                };
                let keyword_module: Table =
                    lua.registry_value(&keyword_script.module)
                        .map_err(|source| ScriptLoadError::Lua {
                            path: root.to_owned(),
                            source,
                        })?;
                let required_hooks = keyword_module
                    .get::<Option<Table>>("required_card_hooks")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: root.to_owned(),
                        source,
                    })?;
                if let Some(required_hooks) = required_hooks {
                    for hook in required_hooks.sequence_values::<String>() {
                        let hook = hook.map_err(|source| ScriptLoadError::Lua {
                            path: root.to_owned(),
                            source,
                        })?;
                        let implemented = card_module
                            .get::<Option<Function>>(hook.as_str())
                            .map_err(|source| ScriptLoadError::Lua {
                                path: root.to_owned(),
                                source,
                            })?
                            .is_some();
                        if !implemented {
                            return Err(ScriptLoadError::MissingKeywordHook {
                                card: card.definition.id.clone(),
                                keyword: keyword.clone(),
                                hook,
                            });
                        }
                    }
                }
                if let Some(required_actions) = keyword_module
                    .get::<Option<Table>>("required_card_actions")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: root.to_owned(),
                        source,
                    })?
                {
                    let action_effects = card_module
                        .get::<Option<Table>>("action_effects")
                        .map_err(|source| ScriptLoadError::Lua {
                            path: root.to_owned(),
                            source,
                        })?;
                    for action in required_actions.sequence_values::<String>() {
                        let action = action.map_err(|source| ScriptLoadError::Lua {
                            path: root.to_owned(),
                            source,
                        })?;
                        let implemented = action_effects
                            .as_ref()
                            .and_then(|effects| {
                                effects.get::<Option<Function>>(action.as_str()).ok()
                            })
                            .flatten()
                            .is_some();
                        if !implemented {
                            return Err(ScriptLoadError::MissingKeywordAction {
                                card: card.definition.id.clone(),
                                keyword: keyword.clone(),
                                action,
                            });
                        }
                    }
                }
                if let Some(required_fields) = keyword_module
                    .get::<Option<Table>>("required_card_fields")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: root.to_owned(),
                        source,
                    })?
                {
                    for field in required_fields.sequence_values::<String>() {
                        let field = field.map_err(|source| ScriptLoadError::Lua {
                            path: root.to_owned(),
                            source,
                        })?;
                        let present = !matches!(
                            card_module.get::<Value>(field.as_str()).map_err(|source| {
                                ScriptLoadError::Lua {
                                    path: root.to_owned(),
                                    source,
                                }
                            })?,
                            Value::Nil
                        );
                        if !present {
                            return Err(ScriptLoadError::MissingKeywordField {
                                card: card.definition.id.clone(),
                                keyword: keyword.clone(),
                                field,
                            });
                        }
                    }
                }
                let requires_param = keyword_module
                    .get::<Option<bool>>("requires_param")
                    .map_err(|source| ScriptLoadError::Lua {
                        path: root.to_owned(),
                        source,
                    })?
                    .unwrap_or(false);
                if requires_param && !card.definition.keyword_params.contains_key(keyword) {
                    return Err(ScriptLoadError::MissingKeywordParam {
                        card: card.definition.id.clone(),
                        keyword: keyword.clone(),
                    });
                }
            }
            for keyword in card.definition.keyword_params.keys() {
                if !card.definition.keywords.contains(keyword) {
                    return Err(ScriptLoadError::UnreferencedKeywordParam {
                        card: card.definition.id.clone(),
                        keyword: keyword.clone(),
                    });
                }
            }
        }
        let catalog = Arc::new(
            cards
                .iter()
                .map(|(id, card)| (id.clone(), card.definition.clone()))
                .collect(),
        );
        Ok(Self {
            lua,
            cards,
            keywords,
            catalog,
            instruction_blocks,
            pack_hash: format!("fnv1a64:{pack_hash:016x}"),
            locale: Locale::EnUs,
        })
    }

    pub fn load_dir_with_locale(
        path: impl AsRef<Path>,
        locale: Locale,
    ) -> Result<Self, ScriptLoadError> {
        let mut runtime = Self::load_dir(path)?;
        runtime.locale = locale;
        for card in runtime.cards.values_mut() {
            let localized = card.definition.localized(locale);
            card.definition.name = localized.name;
            card.definition.text = localized.text;
        }
        runtime.catalog = Arc::new(
            runtime
                .cards
                .iter()
                .map(|(id, card)| (id.clone(), card.definition.clone()))
                .collect(),
        );
        runtime.pack_hash = format!("{}:{}", runtime.pack_hash, locale.code());
        Ok(runtime)
    }

    pub fn definitions(&self) -> impl Iterator<Item = &CardDefinition> {
        self.cards.values().map(|card| &card.definition)
    }

    /// Card definitions paired with the portable Lua source that implements
    /// them. A parent card and its generated tokens intentionally share one
    /// source unit.
    pub fn scripted_definitions(&self) -> impl Iterator<Item = (&CardDefinition, &str, &str)> {
        self.cards.values().map(|card| {
            (
                &card.definition,
                card.source_path.as_ref(),
                card.source.as_ref(),
            )
        })
    }

    pub fn keyword_ids(&self) -> impl Iterator<Item = &str> {
        self.keywords.keys().map(String::as_str)
    }

    pub fn deck_allowances(&self, card_id: &str) -> Result<Vec<DeckAllowance>, String> {
        self.definition(card_id)
            .map(|definition| definition.deck_allowances.clone())
            .ok_or_else(|| format!("unknown card {card_id}"))
    }

    fn module(&self, card_id: &str) -> Result<Table, String> {
        let script = self
            .cards
            .get(card_id)
            .ok_or_else(|| format!("unknown Lua card {card_id}"))?;
        self.lua
            .registry_value(&script.module)
            .map_err(|error| error.to_string())
    }

    fn active_keyword_ids(
        &self,
        state: &GameState,
        entity: EntityId,
    ) -> Result<Vec<String>, String> {
        let entity = state
            .entity(entity)
            .ok_or_else(|| format!("unknown keyword rule entity {entity}"))?;
        let mut ids = entity.keywords.clone();
        if state.player(entity.controller).hero == entity.id {
            for keyword in &state.player(entity.controller).keywords {
                if !ids.contains(keyword) {
                    ids.push(keyword.clone());
                }
            }
        }
        // A weapon's combat keywords apply to its hero only while that hero is active.
        // This is generic composition: Rust does not know which keyword modules are present.
        if entity.kind == CardKind::Hero && entity.controller == state.active_player {
            if let Some(weapon) = state.player(entity.controller).weapon {
                for keyword in &state.entities[&weapon].keywords {
                    let inherits = self
                        .keyword_module(keyword)?
                        .get::<Option<bool>>("weapon_inherits_to_hero")
                        .map_err(|error| error.to_string())?
                        .unwrap_or(false);
                    if inherits && !ids.contains(keyword) {
                        ids.push(keyword.clone());
                    }
                }
            }
        }
        Ok(ids)
    }

    fn keyword_module(&self, keyword: &str) -> Result<Table, String> {
        let script = self
            .keywords
            .get(keyword)
            .ok_or_else(|| format!("unknown Lua keyword {keyword}"))?;
        self.lua
            .registry_value(&script.module)
            .map_err(|error| error.to_string())
    }

    fn invoke_triggers(
        &self,
        state: &GameState,
        listener: EntityId,
        event: &ScriptEvent,
        module: Table,
    ) -> Result<Vec<EffectSpec>, String> {
        let entity = state
            .entity(listener)
            .ok_or_else(|| format!("unknown listener entity {listener}"))?;
        let Some(triggers) = module
            .get::<Option<Table>>("triggers")
            .map_err(|error| error.to_string())?
        else {
            return Ok(Vec::new());
        };
        let mut output = Vec::new();
        for trigger in triggers.sequence_values::<Table>() {
            let trigger = trigger.map_err(|error| error.to_string())?;
            let trigger_event: String = trigger.get("event").map_err(|error| error.to_string())?;
            let timing = trigger
                .get::<Option<String>>("timing")
                .map_err(|error| error.to_string())?
                .unwrap_or_else(|| "after".to_owned());
            if trigger_event != event.event.script_name()
                || timing != timing_name(event.timing)
                || !zone_is_active(&trigger, entity.zone)?
            {
                continue;
            }
            let effects = Rc::new(RefCell::new(Vec::new()));
            let ctx = build_context(
                &self.lua,
                state,
                listener,
                effects.clone(),
                self.catalog.clone(),
                self.locale,
            )
            .map_err(|error| error.to_string())?;
            let event_table =
                event_to_table(&self.lua, event).map_err(|error| error.to_string())?;
            if let Some(condition) = trigger
                .get::<Option<Function>>("condition")
                .map_err(|error| error.to_string())?
            {
                let matches: bool = condition
                    .call((ctx.clone(), listener.0, event_table.clone()))
                    .map_err(|error| error.to_string())?;
                if !matches {
                    continue;
                }
            }
            let effect: Function = trigger.get("effect").map_err(|error| error.to_string())?;
            effect
                .call::<()>((ctx, listener.0, event_table))
                .map_err(|error| error.to_string())?;
            output.extend(effects.borrow_mut().drain(..));
        }
        Ok(output)
    }

    fn run_keyword_i32_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        rule: &str,
        mut value: i32,
        other: Option<EntityId>,
    ) -> Result<i32, String> {
        self.instruction_blocks.set(0);
        let card_id = state
            .entity(entity)
            .ok_or_else(|| format!("unknown card rule entity {entity}"))?
            .card_id
            .clone();
        if self.cards.contains_key(&card_id) {
            value = self.run_card_i32_rule(state, entity, &card_id, rule, value, other)?;
        }
        let rule_entity = state.entity(entity).unwrap();
        if rule_entity.kind == CardKind::Hero && rule_entity.controller == state.active_player {
            if let Some(weapon) = state.player(rule_entity.controller).weapon {
                let weapon_card = state.entities[&weapon].card_id.as_str();
                let module = self.module(weapon_card)?;
                if module
                    .get::<Option<bool>>("rules_inherit_to_hero")
                    .map_err(|error| error.to_string())?
                    .unwrap_or(false)
                {
                    value =
                        self.run_card_i32_rule(state, entity, weapon_card, rule, value, other)?;
                }
            }
        }
        for keyword in self.active_keyword_ids(state, entity)? {
            let module = self.keyword_module(&keyword)?;
            let Some(rules) = module
                .get::<Option<Table>>("rules")
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            let Some(function) = rules
                .get::<Option<Function>>(rule)
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            let emitted = Rc::new(RefCell::new(Vec::new()));
            let ctx = build_context(
                &self.lua,
                state,
                entity,
                emitted.clone(),
                self.catalog.clone(),
                self.locale,
            )
            .map_err(|error| error.to_string())?;
            value = function
                .call((ctx, entity.0, value, other.map(|id| id.0)))
                .map_err(|error| format!("keyword {keyword} rule {rule}: {error}"))?;
            if !emitted.borrow().is_empty() {
                return Err(format!(
                    "keyword {keyword} rule {rule} attempted to emit an effect"
                ));
            }
        }
        Ok(value)
    }

    fn run_card_i32_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        card_id: &str,
        rule: &str,
        value: i32,
        other: Option<EntityId>,
    ) -> Result<i32, String> {
        let card_module = self.module(card_id)?;
        let Some(rules) = card_module
            .get::<Option<Table>>("rules")
            .map_err(|error| error.to_string())?
        else {
            return Ok(value);
        };
        let Some(function) = rules
            .get::<Option<Function>>(rule)
            .map_err(|error| error.to_string())?
        else {
            return Ok(value);
        };
        let emitted = Rc::new(RefCell::new(Vec::new()));
        let ctx = build_context(
            &self.lua,
            state,
            entity,
            emitted.clone(),
            self.catalog.clone(),
            self.locale,
        )
        .map_err(|error| error.to_string())?;
        let value = function
            .call((ctx, entity.0, value, other.map(|id| id.0)))
            .map_err(|error| format!("card {card_id} rule {rule}: {error}"))?;
        if !emitted.borrow().is_empty() {
            return Err(format!(
                "card {card_id} rule {rule} attempted to emit an effect"
            ));
        }
        Ok(value)
    }

    fn run_keyword_bool_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        rule: &str,
        mut value: bool,
        other: Option<EntityId>,
    ) -> Result<bool, String> {
        self.instruction_blocks.set(0);
        let card_id = state
            .entity(entity)
            .ok_or_else(|| format!("unknown card rule entity {entity}"))?
            .card_id
            .clone();
        if self.cards.contains_key(&card_id) {
            value = self.run_card_bool_rule(state, entity, &card_id, rule, value, other)?;
        }
        let rule_entity = state.entity(entity).unwrap();
        if rule_entity.kind == CardKind::Hero && rule_entity.controller == state.active_player {
            if let Some(weapon) = state.player(rule_entity.controller).weapon {
                let weapon_card = state.entities[&weapon].card_id.as_str();
                let module = self.module(weapon_card)?;
                if module
                    .get::<Option<bool>>("rules_inherit_to_hero")
                    .map_err(|error| error.to_string())?
                    .unwrap_or(false)
                {
                    value =
                        self.run_card_bool_rule(state, entity, weapon_card, rule, value, other)?;
                }
            }
        }
        for keyword in self.active_keyword_ids(state, entity)? {
            let module = self.keyword_module(&keyword)?;
            let Some(rules) = module
                .get::<Option<Table>>("rules")
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            let Some(function) = rules
                .get::<Option<Function>>(rule)
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            let emitted = Rc::new(RefCell::new(Vec::new()));
            let ctx = build_context(
                &self.lua,
                state,
                entity,
                emitted.clone(),
                self.catalog.clone(),
                self.locale,
            )
            .map_err(|error| error.to_string())?;
            value = function
                .call((ctx, entity.0, value, other.map(|id| id.0)))
                .map_err(|error| format!("keyword {keyword} rule {rule}: {error}"))?;
            if !emitted.borrow().is_empty() {
                return Err(format!(
                    "keyword {keyword} rule {rule} attempted to emit an effect"
                ));
            }
        }
        Ok(value)
    }

    fn run_card_bool_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        card_id: &str,
        rule: &str,
        value: bool,
        other: Option<EntityId>,
    ) -> Result<bool, String> {
        let card_module = self.module(card_id)?;
        let Some(rules) = card_module
            .get::<Option<Table>>("rules")
            .map_err(|error| error.to_string())?
        else {
            return Ok(value);
        };
        let Some(function) = rules
            .get::<Option<Function>>(rule)
            .map_err(|error| error.to_string())?
        else {
            return Ok(value);
        };
        let emitted = Rc::new(RefCell::new(Vec::new()));
        let ctx = build_context(
            &self.lua,
            state,
            entity,
            emitted.clone(),
            self.catalog.clone(),
            self.locale,
        )
        .map_err(|error| error.to_string())?;
        let value = function
            .call((ctx, entity.0, value, other.map(|id| id.0)))
            .map_err(|error| format!("card {card_id} rule {rule}: {error}"))?;
        if !emitted.borrow().is_empty() {
            return Err(format!(
                "card {card_id} rule {rule} attempted to emit an effect"
            ));
        }
        Ok(value)
    }

    fn invoke_effect_hook(
        &self,
        state: &GameState,
        source: EntityId,
        function: Function,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String> {
        self.instruction_blocks.set(0);
        let effects = Rc::new(RefCell::new(Vec::new()));
        let ctx = build_context(
            &self.lua,
            state,
            source,
            effects.clone(),
            self.catalog.clone(),
            self.locale,
        )
        .map_err(|error| error.to_string())?;
        function
            .call::<()>((ctx, source.0, target.map(|id| id.0)))
            .map_err(|error| error.to_string())?;
        Ok(effects.borrow_mut().drain(..).collect())
    }

    fn invoke_keyword_effect_hooks(
        &self,
        state: &GameState,
        source: EntityId,
        hook: &str,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String> {
        let keywords = state
            .entity(source)
            .ok_or_else(|| format!("unknown keyword hook source entity {source}"))?
            .keywords
            .clone();
        let mut output = Vec::new();
        for keyword in keywords {
            let module = self.keyword_module(&keyword)?;
            let Some(hooks) = module
                .get::<Option<Table>>("hooks")
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            let Some(function) = hooks
                .get::<Option<Function>>(hook)
                .map_err(|error| format!("keyword {keyword} hook {hook}: {error}"))?
            else {
                continue;
            };
            output.extend(
                self.invoke_effect_hook(state, source, function, target)
                    .map_err(|error| format!("keyword {keyword} hook {hook}: {error}"))?,
            );
        }
        Ok(output)
    }

    fn bind_keyword_continuation_owners(
        &self,
        state: &GameState,
        source: EntityId,
        effects: &mut Vec<EffectSpec>,
    ) -> Result<(), String> {
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown keyword continuation source entity {source}"))?;
        let mut bound = Vec::with_capacity(effects.len());
        for effect in std::mem::take(effects) {
            let EffectSpec::Continue {
                hook,
                continuation_owner,
                ..
            } = &effect
            else {
                bound.push(effect);
                continue;
            };
            if continuation_owner.is_some() {
                bound.push(effect);
                continue;
            }

            let mut candidates = Vec::new();
            if !entity.silenced {
                candidates.push(entity.card_id.clone());
            }
            candidates.extend(entity.attached_cards.iter().cloned());
            if let Some(attachments) = entity.hook_attachments.get(hook) {
                candidates.extend(attachments.iter().cloned());
            }

            let mut owners = Vec::new();
            for card_id in candidates {
                let module = self.module(&card_id)?;
                if module
                    .get::<Option<Function>>(hook.as_str())
                    .map_err(|error| error.to_string())?
                    .is_some()
                {
                    owners.push(card_id);
                }
            }
            if owners.is_empty() {
                bound.push(effect);
                continue;
            }
            for owner in owners {
                let mut owned = effect.clone();
                let EffectSpec::Continue {
                    continuation_owner, ..
                } = &mut owned
                else {
                    unreachable!()
                };
                *continuation_owner = Some(owner);
                bound.push(owned);
            }
        }
        *effects = bound;
        Ok(())
    }

    fn action_target_mode(module: &Table, action: &str) -> Result<TargetMode, String> {
        let Some(modes) = module
            .get::<Option<Table>>("action_target_modes")
            .map_err(|error| error.to_string())?
        else {
            return Ok(TargetMode::Optional);
        };
        match modes
            .get::<Option<String>>(action)
            .map_err(|error| error.to_string())?
            .as_deref()
        {
            None | Some("optional") => Ok(TargetMode::Optional),
            Some("required") => Ok(TargetMode::Required),
            Some("required_if_available") => Ok(TargetMode::RequiredIfAvailable),
            Some(value) => Err(format!("unknown action target mode {value}")),
        }
    }
}

fn bind_continuation_owner(effects: &mut [EffectSpec], owner: &str) {
    for effect in effects {
        let continuation_owner = match effect {
            EffectSpec::Continue {
                continuation_owner, ..
            }
            | EffectSpec::RequestChoice {
                continuation_owner, ..
            }
            | EffectSpec::DiscoverCards {
                continuation_owner, ..
            }
            | EffectSpec::DiscoverEntities {
                continuation_owner, ..
            }
            | EffectSpec::RandomChoice {
                continuation_owner, ..
            }
            | EffectSpec::SpendPlayerResourceAndContinue {
                continuation_owner, ..
            } => continuation_owner,
            _ => continue,
        };
        if continuation_owner.is_none() {
            *continuation_owner = Some(owner.to_owned());
        }
    }
}

impl CardRuntime for LuaCardRuntime {
    fn pack_hash(&self) -> &str {
        &self.pack_hash
    }

    fn definition(&self, card_id: &str) -> Option<&CardDefinition> {
        self.cards.get(card_id).map(|card| &card.definition)
    }

    fn card_ids(&self) -> Vec<String> {
        self.cards.keys().cloned().collect()
    }

    fn keyword_i32_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        rule: &str,
        initial: i32,
        other: Option<EntityId>,
    ) -> Result<i32, String> {
        self.run_keyword_i32_rule(state, entity, rule, initial, other)
    }

    fn keyword_bool_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        rule: &str,
        initial: bool,
        other: Option<EntityId>,
    ) -> Result<bool, String> {
        self.run_keyword_bool_rule(state, entity, rule, initial, other)
    }

    fn valid_targets(&self, state: &GameState, source: EntityId) -> Result<Vec<EntityId>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown source entity {source}"))?;
        let mut card_ids = vec![entity.card_id.clone()];
        card_ids.extend(entity.attached_cards.iter().cloned());
        let mut seen = HashSet::new();
        let mut targets = Vec::new();
        for card_id in card_ids {
            let module = self.module(&card_id)?;
            let Some(function) = module
                .get::<Option<Function>>("targets")
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            let effects = Rc::new(RefCell::new(Vec::new()));
            let ctx = build_context(
                &self.lua,
                state,
                source,
                effects,
                self.catalog.clone(),
                self.locale,
            )
            .map_err(|error| error.to_string())?;
            let ids: Vec<u64> = function
                .call((ctx, source.0))
                .map_err(|error| format!("card {card_id} targets: {error}"))?;
            for id in ids.into_iter().map(EntityId) {
                if seen.insert(id) {
                    targets.push(id);
                }
            }
        }
        Ok(targets)
    }

    fn location_targets(
        &self,
        state: &GameState,
        source: EntityId,
    ) -> Result<Vec<EntityId>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown location entity {source}"))?;
        let module = self.module(&entity.card_id)?;
        let Some(function) = module
            .get::<Option<Function>>("location_targets")
            .map_err(|error| error.to_string())?
        else {
            return Ok(Vec::new());
        };
        let effects = Rc::new(RefCell::new(Vec::new()));
        let ctx = build_context(
            &self.lua,
            state,
            source,
            effects,
            self.catalog.clone(),
            self.locale,
        )
        .map_err(|error| error.to_string())?;
        let ids: Vec<u64> = function
            .call((ctx, source.0))
            .map_err(|error| error.to_string())?;
        Ok(ids.into_iter().map(EntityId).collect())
    }

    fn card_actions(
        &self,
        state: &GameState,
        source: EntityId,
    ) -> Result<Vec<CardActionSpec>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown card action entity {source}"))?;
        let card_module = self.module(&entity.card_id)?;
        let card_target_mode = |action: &str| Self::action_target_mode(&card_module, action);
        let semantic_card_id = |action: &str| -> Result<Option<String>, String> {
            let Some(mapping) = card_module
                .get::<Option<Table>>("action_semantic_cards")
                .map_err(|error| error.to_string())?
            else {
                return Ok(None);
            };
            let card_id = mapping
                .get::<Option<String>>(action)
                .map_err(|error| error.to_string())?;
            if let Some(card_id) = &card_id
                && !self.catalog.contains_key(card_id)
            {
                return Err(format!(
                    "card {} action {action} references unknown semantic card {card_id}",
                    entity.card_id
                ));
            }
            Ok(card_id)
        };
        let mut output = Vec::new();
        for keyword in &entity.keywords {
            let module = self.keyword_module(keyword)?;
            let Some(actions) = module
                .get::<Option<Table>>("actions")
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            for pair in actions.pairs::<String, Table>() {
                let (id, spec) = pair.map_err(|error| error.to_string())?;
                let zones = spec
                    .get::<Option<Table>>("active_zones")
                    .map_err(|error| error.to_string())?;
                let active = if let Some(zones) = zones {
                    let mut active = false;
                    for zone in zones.sequence_values::<String>() {
                        if zone.map_err(|error| error.to_string())? == zone_name(entity.zone) {
                            active = true;
                        }
                    }
                    active
                } else {
                    entity.zone == Zone::Hand
                };
                if !active {
                    continue;
                }
                if let Some(condition) = spec
                    .get::<Option<Function>>("condition")
                    .map_err(|error| error.to_string())?
                {
                    let emitted = Rc::new(RefCell::new(Vec::new()));
                    let ctx = build_context(
                        &self.lua,
                        state,
                        source,
                        emitted.clone(),
                        self.catalog.clone(),
                        self.locale,
                    )
                    .map_err(|error| error.to_string())?;
                    let available: bool = condition
                        .call((ctx, source.0))
                        .map_err(|error| format!("keyword {keyword} action {id}: {error}"))?;
                    if !emitted.borrow().is_empty() {
                        return Err(format!(
                            "keyword {keyword} action {id} condition attempted to emit an effect"
                        ));
                    }
                    if !available {
                        continue;
                    }
                }
                if output
                    .iter()
                    .any(|existing: &CardActionSpec| existing.id == id)
                {
                    return Err(format!("duplicate active card action {id} on {source}"));
                }
                output.push(CardActionSpec {
                    id: id.clone(),
                    semantic_card_id: spec
                        .get::<Option<String>>("semantic_card_id")
                        .map_err(|error| error.to_string())?
                        .or(semantic_card_id(&id)?),
                    cost: spec
                        .get::<Option<u8>>("cost")
                        .map_err(|error| error.to_string())?
                        .unwrap_or(0),
                    spend_all_mana: spec
                        .get::<Option<bool>>("spend_all_mana")
                        .map_err(|error| error.to_string())?
                        .unwrap_or(false),
                    target_mode: card_target_mode(&id)?,
                });
            }
        }
        if let Some(actions) = card_module
            .get::<Option<Table>>("card_actions")
            .map_err(|error| error.to_string())?
        {
            for pair in actions.pairs::<String, Table>() {
                let (id, spec) = pair.map_err(|error| error.to_string())?;
                let zones = spec
                    .get::<Option<Table>>("active_zones")
                    .map_err(|error| error.to_string())?;
                let active = if let Some(zones) = zones {
                    let mut active = false;
                    for zone in zones.sequence_values::<String>() {
                        if zone.map_err(|error| error.to_string())? == zone_name(entity.zone) {
                            active = true;
                        }
                    }
                    active
                } else {
                    entity.zone == Zone::Hand
                };
                if !active {
                    continue;
                }
                if let Some(condition) = spec
                    .get::<Option<Function>>("condition")
                    .map_err(|error| error.to_string())?
                {
                    let emitted = Rc::new(RefCell::new(Vec::new()));
                    let ctx = build_context(
                        &self.lua,
                        state,
                        source,
                        emitted.clone(),
                        self.catalog.clone(),
                        self.locale,
                    )
                    .map_err(|error| error.to_string())?;
                    let available: bool = condition
                        .call((ctx, source.0))
                        .map_err(|error| format!("card action {id}: {error}"))?;
                    if !emitted.borrow().is_empty() {
                        return Err(format!(
                            "card action {id} condition attempted to emit an effect"
                        ));
                    }
                    if !available {
                        continue;
                    }
                }
                if output.iter().any(|existing| existing.id == id) {
                    return Err(format!("duplicate active card action {id} on {source}"));
                }
                output.push(CardActionSpec {
                    id: id.clone(),
                    semantic_card_id: spec
                        .get::<Option<String>>("semantic_card_id")
                        .map_err(|error| error.to_string())?
                        .or(semantic_card_id(&id)?),
                    cost: spec
                        .get::<Option<u8>>("cost")
                        .map_err(|error| error.to_string())?
                        .unwrap_or(0),
                    spend_all_mana: spec
                        .get::<Option<bool>>("spend_all_mana")
                        .map_err(|error| error.to_string())?
                        .unwrap_or(false),
                    target_mode: card_target_mode(&id)?,
                });
            }
        }
        Ok(output)
    }

    fn action_targets(
        &self,
        state: &GameState,
        source: EntityId,
        action: &str,
    ) -> Result<Vec<EntityId>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown card action entity {source}"))?;
        let module = self.module(&entity.card_id)?;
        let Some(selectors) = module
            .get::<Option<Table>>("action_targets")
            .map_err(|error| error.to_string())?
        else {
            return Ok(Vec::new());
        };
        let Some(function) = selectors
            .get::<Option<Function>>(action)
            .map_err(|error| error.to_string())?
        else {
            return Ok(Vec::new());
        };
        let emitted = Rc::new(RefCell::new(Vec::new()));
        let ctx = build_context(
            &self.lua,
            state,
            source,
            emitted.clone(),
            self.catalog.clone(),
            self.locale,
        )
        .map_err(|error| error.to_string())?;
        let ids: Vec<u64> = function
            .call((ctx, source.0))
            .map_err(|error| error.to_string())?;
        if !emitted.borrow().is_empty() {
            return Err(format!(
                "action target selector {action} attempted to emit an effect"
            ));
        }
        Ok(ids.into_iter().map(EntityId).collect())
    }

    fn on_card_action(
        &self,
        state: &GameState,
        source: EntityId,
        action: &str,
        spent: u8,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown card action entity {source}"))?;
        let mut output = Vec::new();
        for keyword in &entity.keywords {
            let module = self.keyword_module(keyword)?;
            let Some(actions) = module
                .get::<Option<Table>>("actions")
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            let Some(spec) = actions
                .get::<Option<Table>>(action)
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            if let Some(function) = spec
                .get::<Option<Function>>("effect")
                .map_err(|error| error.to_string())?
            {
                let effects = Rc::new(RefCell::new(Vec::new()));
                let ctx = build_context(
                    &self.lua,
                    state,
                    source,
                    effects.clone(),
                    self.catalog.clone(),
                    self.locale,
                )
                .map_err(|error| error.to_string())?;
                function
                    .call::<()>((ctx, source.0, spent, target.map(|id| id.0)))
                    .map_err(|error| format!("keyword {keyword} action {action}: {error}"))?;
                output.extend(effects.borrow_mut().drain(..));
            }
        }
        self.bind_keyword_continuation_owners(state, source, &mut output)?;
        let module = self.module(&entity.card_id)?;
        if let Some(effects) = module
            .get::<Option<Table>>("action_effects")
            .map_err(|error| error.to_string())?
            && let Some(function) = effects
                .get::<Option<Function>>(action)
                .map_err(|error| error.to_string())?
        {
            let emitted = Rc::new(RefCell::new(Vec::new()));
            let ctx = build_context(
                &self.lua,
                state,
                source,
                emitted.clone(),
                self.catalog.clone(),
                self.locale,
            )
            .map_err(|error| error.to_string())?;
            function
                .call::<()>((ctx, source.0, spent, target.map(|id| id.0)))
                .map_err(|error| format!("card action {action}: {error}"))?;
            let mut generated = emitted.borrow_mut().drain(..).collect::<Vec<_>>();
            bind_continuation_owner(&mut generated, &entity.card_id);
            output.extend(generated);
        }
        Ok(output)
    }

    fn on_play(
        &self,
        state: &GameState,
        source: EntityId,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String> {
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown source entity {source}"))?;
        let module = self.module(&entity.card_id)?;
        let function = module
            .get::<Option<Function>>("on_play")
            .map_err(|error| error.to_string())?;
        let mut output = Vec::new();
        if let Some(function) = function {
            output.extend(self.invoke_effect_hook(state, source, function, target)?);
        }
        bind_continuation_owner(&mut output, &entity.card_id);
        let mut keyword_effects =
            self.invoke_keyword_effect_hooks(state, source, "on_play", target)?;
        // Keyword modules are generic dispatchers. Resolve an unambiguous
        // handler now so a summon trigger cannot transform the host before the
        // continuation runs. Keep multi-handler dispatches unowned so every
        // attached implementation still executes.
        self.bind_keyword_continuation_owners(state, source, &mut keyword_effects)?;
        output.extend(keyword_effects);
        Ok(output)
    }

    fn on_location_use(
        &self,
        state: &GameState,
        source: EntityId,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String> {
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown location entity {source}"))?;
        let module = self.module(&entity.card_id)?;
        let function = module
            .get::<Option<Function>>("on_location_use")
            .map_err(|error| error.to_string())?;
        let mut output = Vec::new();
        if let Some(function) = function {
            output.extend(self.invoke_effect_hook(state, source, function, target)?);
        }
        bind_continuation_owner(&mut output, &entity.card_id);
        let mut keyword_effects =
            self.invoke_keyword_effect_hooks(state, source, "on_location_use", target)?;
        self.bind_keyword_continuation_owners(state, source, &mut keyword_effects)?;
        output.extend(keyword_effects);
        Ok(output)
    }

    fn on_event(
        &self,
        state: &GameState,
        listener: EntityId,
        event: &ScriptEvent,
    ) -> Result<Vec<EffectSpec>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(listener)
            .ok_or_else(|| format!("unknown listener entity {listener}"))?;
        let mut output = Vec::new();
        if !entity.silenced
            && let Some(script) = self.cards.get(&entity.card_id)
        {
            let module = self
                .lua
                .registry_value(&script.module)
                .map_err(|error| error.to_string())?;
            let mut generated = self.invoke_triggers(state, listener, event, module)?;
            bind_continuation_owner(&mut generated, &entity.card_id);
            output.extend(generated);
        }
        for attached in &entity.attached_cards {
            if let Some(script) = self.cards.get(attached) {
                let module = self
                    .lua
                    .registry_value(&script.module)
                    .map_err(|error| error.to_string())?;
                let mut generated = self.invoke_triggers(state, listener, event, module)?;
                bind_continuation_owner(&mut generated, attached);
                output.extend(generated);
            }
        }
        for keyword in self.active_keyword_ids(state, listener)? {
            let module = self.keyword_module(&keyword)?;
            let mut generated = self.invoke_triggers(state, listener, event, module)?;
            self.bind_keyword_continuation_owners(state, listener, &mut generated)?;
            output.extend(generated);
        }
        Ok(output)
    }

    fn on_resume(
        &self,
        state: &GameState,
        source: EntityId,
        continuation_owner: Option<&str>,
        hook: &str,
        choice: &ChoiceValue,
    ) -> Result<Vec<EffectSpec>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown source entity {source}"))?;
        let mut card_ids = continuation_owner
            .map(|owner| vec![owner.to_owned()])
            .unwrap_or_else(|| {
                let mut ids = vec![entity.card_id.clone()];
                ids.extend(entity.attached_cards.iter().cloned());
                ids
            });
        // A hook attachment executes with the host entity as `self`, so a
        // choice or random-value continuation emitted by that script also
        // resumes against the host. Include those script modules when looking
        // up the named resume hook (for example, Infest's attached
        // Deathrattle choosing a random Beast).
        if continuation_owner.is_none() {
            card_ids.extend(
                entity
                    .hook_attachments
                    .values()
                    .flat_map(|attachments| attachments.iter().cloned()),
            );
        }
        let mut function = None;
        let mut function_owner = None;
        for card_id in &card_ids {
            let module = self.module(card_id)?;
            if let Some(candidate) = module
                .get::<Option<Function>>(hook)
                .map_err(|error| error.to_string())?
            {
                function = Some(candidate);
                function_owner = Some(card_id.clone());
                break;
            }
        }
        let function = function.ok_or_else(|| {
            format!(
                "card {} and its attachments have no resume hook {hook}",
                entity.card_id
            )
        })?;
        let effects = Rc::new(RefCell::new(Vec::new()));
        let ctx = build_context(
            &self.lua,
            state,
            source,
            effects.clone(),
            self.catalog.clone(),
            self.locale,
        )
        .map_err(|error| error.to_string())?;
        let choice = choice_value_to_lua(&self.lua, choice).map_err(|error| error.to_string())?;
        function
            .call::<()>((ctx, source.0, choice))
            .map_err(|error| error.to_string())?;
        let mut generated = effects.borrow_mut().drain(..).collect::<Vec<_>>();
        bind_continuation_owner(
            &mut generated,
            function_owner
                .as_deref()
                .expect("resume function has an owner"),
        );
        Ok(generated)
    }

    fn on_continue(
        &self,
        state: &GameState,
        source: EntityId,
        continuation_owner: Option<&str>,
        hook: &str,
        payload: Option<&ChoiceValue>,
    ) -> Result<Vec<EffectSpec>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown continuation source {source}"))?;
        let mut card_ids = continuation_owner
            .map(|owner| vec![owner.to_owned()])
            .unwrap_or_default();
        if continuation_owner.is_none() {
            if !entity.silenced {
                card_ids.push(entity.card_id.clone());
            }
            card_ids.extend(entity.attached_cards.iter().cloned());
            if let Some(attachments) = entity.hook_attachments.get(hook) {
                card_ids.extend(attachments.iter().cloned());
            }
        }
        let searched_card_ids = card_ids.clone();
        let mut output = Vec::new();
        let mut found = false;
        for card_id in card_ids {
            let module = self.module(&card_id)?;
            let Some(function) = module
                .get::<Option<Function>>(hook)
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            found = true;
            // A fused minion can inherit several targeted Battlecries whose
            // target sets differ. The play target is the union of those sets,
            // but only the attached Battlecries for which that target is legal
            // should execute (for example, Fire Plume Phoenix may target an
            // enemy while Princess Huhuran may only target a friendly
            // Deathrattle minion).
            if hook == "on_battlecry"
                && entity.attached_cards.len() > 1
                && let Some(ChoiceValue::Entity(target)) = payload
                && let Some(targets) = module
                    .get::<Option<Function>>("targets")
                    .map_err(|error| error.to_string())?
            {
                let emitted = Rc::new(RefCell::new(Vec::new()));
                let ctx = build_context(
                    &self.lua,
                    state,
                    source,
                    emitted,
                    self.catalog.clone(),
                    self.locale,
                )
                .map_err(|error| error.to_string())?;
                let legal_targets: Vec<u64> = targets
                    .call((ctx, source.0))
                    .map_err(|error| format!("card {card_id} targets: {error}"))?;
                if !legal_targets.contains(&target.0) {
                    continue;
                }
            }
            let effects = Rc::new(RefCell::new(Vec::new()));
            let ctx = build_context(
                &self.lua,
                state,
                source,
                effects.clone(),
                self.catalog.clone(),
                self.locale,
            )
            .map_err(|error| error.to_string())?;
            match payload {
                None => function
                    .call::<()>((ctx, source.0))
                    .map_err(|error| error.to_string())?,
                Some(payload) => {
                    let payload = choice_value_to_lua(&self.lua, payload)
                        .map_err(|error| error.to_string())?;
                    function
                        .call::<()>((ctx, source.0, payload))
                        .map_err(|error| error.to_string())?;
                }
            }
            let mut generated = effects.borrow_mut().drain(..).collect::<Vec<_>>();
            bind_continuation_owner(&mut generated, &card_id);
            output.extend(generated);
        }
        if !found {
            return Err(format!(
                "card {} and searched scripts {:?} have no continuation hook {hook}",
                entity.card_id, searched_card_ids
            ));
        }
        Ok(output)
    }

    fn auras(&self, state: &GameState, source: EntityId) -> Result<Vec<AuraSpec>, String> {
        self.instruction_blocks.set(0);
        let entity = state
            .entity(source)
            .ok_or_else(|| format!("unknown aura source entity {source}"))?;
        let mut output = Vec::new();
        let mut modules = Vec::new();
        let mut card_ids = Vec::new();
        if !entity.silenced {
            card_ids.push(entity.card_id.clone());
        }
        card_ids.extend(entity.attached_cards.iter().cloned());
        for card_id in card_ids {
            if !self.cards.contains_key(&card_id) {
                continue;
            }
            modules.push((card_id.clone(), self.module(&card_id)?));
        }
        for keyword in self.active_keyword_ids(state, source)? {
            modules.push((format!("keyword {keyword}"), self.keyword_module(&keyword)?));
        }
        for (owner, module) in modules {
            let Some(auras) = module
                .get::<Option<Table>>("auras")
                .map_err(|error| error.to_string())?
            else {
                continue;
            };
            for aura in auras.sequence_values::<Table>() {
                let aura = aura.map_err(|error| error.to_string())?;
                if !aura_active_in_zone(&aura, entity.zone).map_err(|error| error.to_string())? {
                    continue;
                }
                let targets: Function = aura.get("targets").map_err(|error| error.to_string())?;
                let emitted = Rc::new(RefCell::new(Vec::new()));
                let ctx = build_context(
                    &self.lua,
                    state,
                    source,
                    emitted.clone(),
                    self.catalog.clone(),
                    self.locale,
                )
                .map_err(|error| error.to_string())?;
                let targets: Vec<u64> = targets
                    .call((ctx.clone(), source.0))
                    .map_err(|error| error.to_string())?;
                let attack = aura_stat_value(&aura, "attack", &ctx, source)
                    .map_err(|error| error.to_string())?;
                let health = aura_stat_value(&aura, "health", &ctx, source)
                    .map_err(|error| error.to_string())?;
                let cost = aura_stat_value(&aura, "cost", &ctx, source)
                    .map_err(|error| error.to_string())?;
                let cost_set = match aura.get::<Value>("cost_set").map_err(|e| e.to_string())? {
                    Value::Nil => None,
                    Value::Integer(value) => Some(i32::try_from(value).map_err(|_| {
                        "aura cost_set must fit in a signed 32-bit value".to_owned()
                    })?),
                    Value::Function(function) => Some(
                        function
                            .call((ctx.clone(), source.0))
                            .map_err(|error| error.to_string())?,
                    ),
                    _ => return Err("aura cost_set must be an integer or function".to_owned()),
                };
                let cost_cap = match aura.get::<Value>("cost_cap").map_err(|e| e.to_string())? {
                    Value::Nil => None,
                    Value::Integer(value) => Some(i32::try_from(value).map_err(|_| {
                        "aura cost_cap must fit in a signed 32-bit value".to_owned()
                    })?),
                    Value::Function(function) => Some(
                        function
                            .call((ctx.clone(), source.0))
                            .map_err(|error| error.to_string())?,
                    ),
                    _ => return Err("aura cost_cap must be an integer or function".to_owned()),
                };
                let spell_damage = aura_stat_value(&aura, "spell_damage", &ctx, source)
                    .map_err(|error| error.to_string())?;
                if !emitted.borrow().is_empty() {
                    return Err(format!(
                        "aura selector on {owner} attempted to emit an effect"
                    ));
                }
                let keywords = aura
                    .get::<Option<Table>>("keywords")
                    .map_err(|error| error.to_string())?
                    .map(|values| {
                        values
                            .sequence_values::<String>()
                            .collect::<mlua::Result<Vec<_>>>()
                    })
                    .transpose()
                    .map_err(|error| error.to_string())?
                    .unwrap_or_default();
                output.push(AuraSpec {
                    source,
                    targets: targets.into_iter().map(EntityId).collect(),
                    attack,
                    health,
                    cost,
                    cost_set,
                    cost_cap,
                    spell_damage,
                    keywords,
                });
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_repository_cards() {
        let cards = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let runtime = LuaCardRuntime::load_dir(cards).unwrap();
        assert!(runtime.definition("CS2_029").is_some());
        assert!(runtime.definition("CS2_022").unwrap().collectible);
        assert!(!runtime.definition("CS2_tk1").unwrap().collectible);
        assert!(runtime.card_ids().len() >= 4);
    }

    #[test]
    fn resource_spend_continuations_keep_the_emitting_script_owner() {
        let mut effects = vec![EffectSpec::SpendPlayerResourceAndContinue {
            source: EntityId(7),
            player: PlayerId::ONE,
            resource: "test_resource".to_owned(),
            minimum: 1,
            maximum: 1,
            hook: "after_payment".to_owned(),
            continuation_owner: None,
        }];

        bind_continuation_owner(&mut effects, "OWNER_CARD");

        assert!(matches!(
            &effects[0],
            EffectSpec::SpendPlayerResourceAndContinue {
                continuation_owner: Some(owner),
                ..
            } if owner == "OWNER_CARD"
        ));
    }

    #[test]
    fn rejects_invalid_death_knight_rune_cost_metadata() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "hearth-script-invalid-runes-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("card.lua"),
            r#"return {
                api_version = 1,
                id = "INVALID_RUNES",
                name = "Invalid Runes",
                set = "TEST",
                type = "spell",
                class = "death_knight",
                cost = 1,
                rune_cost = { blood = 2, frost = 2 },
            }"#,
        )
        .unwrap();

        let error = match LuaCardRuntime::load_dir(&root) {
            Ok(_) => panic!("four-slot Death Knight card unexpectedly loaded"),
            Err(error) => error,
        };
        std::fs::remove_dir_all(&root).unwrap();
        assert!(
            error
                .to_string()
                .contains("INVALID_RUNES has an invalid Death Knight rune_cost")
        );
    }

    #[test]
    fn rejects_card_that_omits_a_keyword_required_hook() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "hearth-script-required-hook-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("keyword.lua"),
            r#"return {
                api_version = 1,
                module_type = "keyword",
                id = "contract_keyword",
                required_card_hooks = { "on_contract" },
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("card.lua"),
            r#"return {
                api_version = 1,
                id = "BROKEN_CONTRACT",
                name = "Broken Contract",
                set = "TEST",
                type = "minion",
                cost = 1,
                attack = 1,
                health = 1,
                keywords = { "contract_keyword" },
            }"#,
        )
        .unwrap();

        let error = match LuaCardRuntime::load_dir(&root) {
            Ok(_) => panic!("card without its keyword-required hook unexpectedly loaded"),
            Err(error) => error,
        };
        std::fs::remove_dir_all(&root).unwrap();
        assert!(matches!(
            error,
            ScriptLoadError::MissingKeywordHook { card, keyword, hook }
                if card == "BROKEN_CONTRACT"
                    && keyword == "contract_keyword"
                    && hook == "on_contract"
        ));
    }

    #[test]
    fn rejects_card_that_omits_a_required_keyword_parameter() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "hearth-script-required-param-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("keyword.lua"),
            r#"return {
                api_version = 1,
                module_type = "keyword",
                id = "numbered_keyword",
                requires_param = true,
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("card.lua"),
            r#"return {
                api_version = 1,
                id = "MISSING_NUMBER",
                name = "Missing Number",
                set = "TEST",
                type = "spell",
                cost = 1,
                keywords = { "numbered_keyword" },
            }"#,
        )
        .unwrap();

        let error = match LuaCardRuntime::load_dir(&root) {
            Ok(_) => panic!("card without its keyword parameter unexpectedly loaded"),
            Err(error) => error,
        };
        std::fs::remove_dir_all(&root).unwrap();
        assert!(matches!(
            error,
            ScriptLoadError::MissingKeywordParam { card, keyword }
                if card == "MISSING_NUMBER" && keyword == "numbered_keyword"
        ));
    }

    #[test]
    fn keyword_required_fields_and_deck_allowances_are_generic() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "hearth-script-required-field-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("keyword.lua"),
            r#"return {
                api_version = 1,
                module_type = "keyword",
                id = "deck_guest",
                required_card_fields = { "deck_allowances" },
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("card.lua"),
            r#"return {
                api_version = 1,
                id = "DECK_GUEST",
                name = "Deck Guest",
                set = "HOST_SET",
                type = "minion",
                class = "host",
                cost = 1,
                attack = 1,
                health = 1,
                keywords = { "deck_guest" },
                deck_allowances = {
                    {
                        class = "guest",
                        set = "GUEST_SET",
                        excluded_keywords = { "deck_guest" },
                    },
                },
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("support.lua"),
            r#"return {
                api_version = 1, id = "NEUTRAL_FILLER", name = "Neutral Filler",
                set = "TEST", type = "minion", cost = 1, attack = 1, health = 1,
                tokens = {
                    { id = "GUEST_CARD", name = "Guest Card", set = "GUEST_SET",
                      type = "minion", class = "guest", collectible = true,
                      cost = 1, attack = 1, health = 1 },
                    { id = "TEST_HP", name = "Test Power", set = "TEST",
                      type = "hero_power", collectible = false, cost = 2 },
                    { id = "GAME_005", name = "The Coin", set = "CORE",
                      type = "spell", collectible = false, cost = 0 },
                },
            }"#,
        )
        .unwrap();

        let runtime = LuaCardRuntime::load_dir(&root).unwrap();
        let allowances = runtime.deck_allowances("DECK_GUEST").unwrap();
        assert_eq!(
            allowances,
            vec![DeckAllowance {
                class: "guest".to_owned(),
                set: "GUEST_SET".to_owned(),
                excluded_keywords: vec!["deck_guest".to_owned()],
            }]
        );
        let allowed_deck = ["DECK_GUEST", "GUEST_CARD"]
            .into_iter()
            .cycle()
            .take(20)
            .map(str::to_owned)
            .collect();
        let neutral_deck = std::iter::repeat_n("NEUTRAL_FILLER".to_owned(), 20).collect();
        assert!(
            hearth_core::Game::new_with_hero_powers_and_classes(
                runtime,
                allowed_deck,
                neutral_deck,
                1,
                ["TEST_HP".to_owned(), "TEST_HP".to_owned()],
                ["host".to_owned(), "host".to_owned()],
            )
            .is_ok()
        );

        let invalid = hearth_core::Game::new_with_hero_powers_and_classes(
            LuaCardRuntime::load_dir(&root).unwrap(),
            std::iter::repeat_n("GUEST_CARD".to_owned(), 20).collect(),
            std::iter::repeat_n("NEUTRAL_FILLER".to_owned(), 20).collect(),
            1,
            ["TEST_HP".to_owned(), "TEST_HP".to_owned()],
            ["host".to_owned(), "host".to_owned()],
        );
        assert!(matches!(
            invalid,
            Err(hearth_core::GameError::InvalidDeckClassCard { card, .. })
                if card == "GUEST_CARD"
        ));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rejects_card_that_omits_a_keyword_required_field() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "hearth-script-missing-field-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("keyword.lua"),
            r#"return {
                api_version = 1,
                module_type = "keyword",
                id = "field_keyword",
                required_card_fields = { "deck_allowances" },
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("card.lua"),
            r#"return {
                api_version = 1,
                id = "MISSING_FIELD",
                name = "Missing Field",
                set = "TEST",
                type = "spell",
                cost = 1,
                keywords = { "field_keyword" },
            }"#,
        )
        .unwrap();

        let error = match LuaCardRuntime::load_dir(&root) {
            Ok(_) => panic!("card without its keyword-required field unexpectedly loaded"),
            Err(error) => error,
        };
        std::fs::remove_dir_all(&root).unwrap();
        assert!(matches!(
            error,
            ScriptLoadError::MissingKeywordField { card, keyword, field }
                if card == "MISSING_FIELD"
                    && keyword == "field_keyword"
                    && field == "deck_allowances"
        ));
    }

    #[test]
    fn rejects_parameter_for_an_unreferenced_keyword() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "hearth-script-unreferenced-param-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("card.lua"),
            r#"return {
                api_version = 1,
                id = "STRAY_NUMBER",
                name = "Stray Number",
                set = "TEST",
                type = "spell",
                cost = 1,
                keyword_params = { missing_keyword = 2 },
            }"#,
        )
        .unwrap();

        let error = match LuaCardRuntime::load_dir(&root) {
            Ok(_) => panic!("unreferenced keyword parameter unexpectedly loaded"),
            Err(error) => error,
        };
        std::fs::remove_dir_all(&root).unwrap();
        assert!(matches!(
            error,
            ScriptLoadError::UnreferencedKeywordParam { card, keyword }
                if card == "STRAY_NUMBER" && keyword == "missing_keyword"
        ));
    }

    #[test]
    fn rejects_unknown_keyword_lifecycle_hook() {
        let lua = Lua::new();
        let module: Table = lua
            .load(
                r#"return {
                    api_version = 1,
                    module_type = "keyword",
                    id = "broken_lifecycle",
                    hooks = {
                        on_draw = function(ctx, self) end,
                    },
                }"#,
            )
            .eval()
            .unwrap();
        let mut keywords = BTreeMap::new();
        let error = register_keyword_module(
            &lua,
            &mut keywords,
            Path::new("broken_lifecycle.lua"),
            module,
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unsupported lifecycle hook on_draw")
        );
    }

    #[test]
    fn rejects_targeted_card_without_selector() {
        let lua = Lua::new();
        let module: Table = lua
            .load(
                r#"return {
                    api_version = 1,
                    id = "BROKEN",
                    name = "Broken",
                    type = "spell",
                    cost = 1,
                    requires_target = true
                }"#,
            )
            .eval()
            .unwrap();
        let definition = parse_definition(&module).unwrap();
        let error = validate_module(&module, &definition).unwrap_err();
        assert!(error.to_string().contains("no targets function"));
    }

    #[test]
    fn structured_choice_value_round_trips_between_lua_and_rust() {
        let lua = Lua::new();
        let value: Value = lua
            .load(
                r#"return {
                    enabled = true,
                    name = "plan",
                    nested = { 3, false, { key = "value" } },
                }"#,
            )
            .eval()
            .unwrap();
        let value = lua_to_choice_value(value).unwrap();
        let ChoiceValue::Object(fields) = &value else {
            panic!("expected object")
        };
        assert_eq!(fields.get("enabled"), Some(&ChoiceValue::Boolean(true)));
        assert!(
            matches!(fields.get("nested"), Some(ChoiceValue::List(values)) if values.len() == 3)
        );

        let restored = choice_value_to_lua(&lua, &value).unwrap();
        assert_eq!(lua_to_choice_value(restored).unwrap(), value);
    }

    #[test]
    fn structured_choice_value_rejects_cycles_and_mixed_tables() {
        let lua = Lua::new();
        let cyclic: Value = lua
            .load("local value = {}; value.self = value; return value")
            .eval()
            .unwrap();
        assert!(
            lua_to_choice_value(cyclic)
                .unwrap_err()
                .to_string()
                .contains("cycles")
        );

        let mixed: Value = lua.load("return { 1, name = 'mixed' }").eval().unwrap();
        assert!(
            lua_to_choice_value(mixed)
                .unwrap_err()
                .to_string()
                .contains("dense array or a string-keyed object")
        );
    }

    #[test]
    fn structured_choice_value_enforces_serialization_limits() {
        let lua = Lua::new();
        let too_many: Value = lua
            .load("local value = {}; for i = 1, 512 do value[i] = i end; return value")
            .eval()
            .unwrap();
        assert!(
            lua_to_choice_value(too_many)
                .unwrap_err()
                .to_string()
                .contains("maximum node count")
        );

        let too_deep: Value = lua
            .load("local root = {}; local value = root; for _ = 1, 17 do value.next = {}; value = value.next end; return root")
            .eval()
            .unwrap();
        assert!(
            lua_to_choice_value(too_deep)
                .unwrap_err()
                .to_string()
                .contains("maximum depth")
        );
    }
}
