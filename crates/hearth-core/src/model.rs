use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

pub type CardId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "enUS")]
    EnUs,
    #[serde(rename = "zhCN")]
    ZhCn,
    #[serde(rename = "zhTW")]
    ZhTw,
}

impl Locale {
    pub const ALL: [Self; 3] = [Self::EnUs, Self::ZhCn, Self::ZhTw];

    pub const fn code(self) -> &'static str {
        match self {
            Self::EnUs => "enUS",
            Self::ZhCn => "zhCN",
            Self::ZhTw => "zhTW",
        }
    }
}

impl std::str::FromStr for Locale {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.replace(['-', '_'], "").to_ascii_lowercase().as_str() {
            "en" | "enus" => Ok(Self::EnUs),
            "zh" | "zhcn" | "hans" | "zhhans" => Ok(Self::ZhCn),
            "zhtw" | "hant" | "zhhant" => Ok(Self::ZhTw),
            _ => Err(format!(
                "unsupported locale {value}; expected enUS, zhCN, or zhTW"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedCardText {
    pub name: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EnchantmentId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventTiming {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enchantment {
    pub id: EnchantmentId,
    pub source: EntityId,
    pub attack: i32,
    pub health: i32,
    pub modifiers: Vec<StatModifier>,
    pub keywords: Vec<String>,
    pub silenciable: bool,
    pub expires_at: Option<EnchantmentExpiry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stat {
    Attack,
    Health,
    Cost,
    SpellDamage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModifierOperation {
    Set,
    Add,
    Multiply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatModifier {
    pub stat: Stat,
    pub operation: ModifierOperation,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnchantmentExpiry {
    EndOfTurn { turn: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectDuration {
    Permanent,
    UntilEndOfTurn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuraSpec {
    pub source: EntityId,
    pub targets: Vec<EntityId>,
    pub attack: i32,
    pub health: i32,
    pub cost: i32,
    pub spell_damage: i32,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const ONE: Self = Self(0);
    pub const TWO: Self = Self(1);

    pub fn opponent(self) -> Self {
        Self(1 - self.0)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{}", self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    Hero,
    Minion,
    Spell,
    Weapon,
    Location,
    HeroPower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetMode {
    /// The action may be performed without choosing a target.
    #[default]
    Optional,
    /// A target is always required. With no legal target, the action is unavailable.
    Required,
    /// A target is required when at least one is legal; otherwise the card may be played
    /// and its targeted hook receives nil. This is Hearthstone's targeted Battlecry rule.
    RequiredIfAvailable,
}

impl TargetMode {
    pub fn requires_target(self, available_targets: usize) -> bool {
        match self {
            Self::Optional => false,
            Self::Required => true,
            Self::RequiredIfAvailable => available_targets > 0,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Optional => "optional",
            Self::Required => "required",
            Self::RequiredIfAvailable => "required_if_available",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Hero,
    SetAside,
    Deck,
    Hand,
    Board,
    Weapon,
    HeroPower,
    Secret,
    Graveyard,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZonePlacement {
    Hand,
    DeckTop,
    DeckBottom,
    DeckRandom,
    Graveyard,
    Removed,
}

impl ZonePlacement {
    pub fn zone(self) -> Zone {
        match self {
            Self::Hand => Zone::Hand,
            Self::DeckTop | Self::DeckBottom | Self::DeckRandom => Zone::Deck,
            Self::Graveyard => Zone::Graveyard,
            Self::Removed => Zone::Removed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardDefinition {
    pub id: CardId,
    pub name: String,
    pub text: String,
    #[serde(default)]
    pub localizations: BTreeMap<Locale, LocalizedCardText>,
    #[serde(default)]
    pub set: String,
    pub kind: CardKind,
    #[serde(default = "default_collectible")]
    pub collectible: bool,
    #[serde(default = "default_card_class")]
    pub class: String,
    /// Printed rarity, normalized to lowercase for generation pool filters.
    #[serde(default)]
    pub rarity: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Printed spell school, normalized to lowercase for Lua pool filters.
    #[serde(default)]
    pub spell_school: Option<String>,
    pub cost: u8,
    pub attack: i32,
    pub health: i32,
    /// Armor gained when a Hero card replaces the current hero.
    #[serde(default)]
    pub armor: i32,
    /// Hero Power installed by a Hero card.
    #[serde(default)]
    pub hero_power: Option<CardId>,
    pub keywords: Vec<String>,
    /// Static numeric configuration consumed by Lua keyword modules.
    #[serde(default)]
    pub keyword_params: BTreeMap<String, i64>,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub target_mode: TargetMode,
}

impl CardDefinition {
    pub fn localized(&self, locale: Locale) -> LocalizedCardText {
        self.localizations
            .get(&locale)
            .cloned()
            .unwrap_or_else(|| LocalizedCardText {
                name: self.name.clone(),
                text: self.text.clone(),
            })
    }
}

fn default_collectible() -> bool {
    true
}

fn default_card_class() -> String {
    "neutral".to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub card_id: CardId,
    pub name: String,
    pub kind: CardKind,
    pub owner: PlayerId,
    pub controller: PlayerId,
    pub zone: Zone,
    pub base_attack: i32,
    pub base_health: i32,
    pub base_cost: u8,
    #[serde(default)]
    pub base_spell_damage: i32,
    pub base_keywords: Vec<String>,
    pub attack: i32,
    pub max_health: i32,
    pub damage: i32,
    #[serde(default)]
    pub armor: i32,
    pub cost: u8,
    #[serde(default)]
    pub spell_damage: i32,
    pub exhausted: bool,
    pub frozen: bool,
    /// Turn on which Freeze was most recently applied. Frozen characters only
    /// thaw after a later turn belonging to their controller, so freezing a
    /// character after it attacked cannot be cleared immediately.
    #[serde(default)]
    pub frozen_since_turn: Option<u32>,
    pub attacks_this_turn: u8,
    #[serde(default)]
    pub location_cooldown: u8,
    pub timestamp: u64,
    pub keywords: Vec<String>,
    pub disabled_keywords: Vec<String>,
    pub aura_attack: i32,
    pub aura_health: i32,
    #[serde(default)]
    pub aura_cost: i32,
    #[serde(default)]
    pub aura_spell_damage: i32,
    pub aura_keywords: Vec<String>,
    pub enchantments: Vec<Enchantment>,
    pub silenced: bool,
    /// Number of cards its controller had already played when this entity was
    /// most recently played from hand. This preserves Combo context across a
    /// suspended choice and snapshot round-trip.
    #[serde(default)]
    pub cards_played_before: u32,
    /// Zero-based position occupied immediately before this card left the hand
    /// to be played. Used by the Lua Outcast module after the card is set aside.
    #[serde(default)]
    pub hand_position_before_play: Option<usize>,
    /// Game turn on which this entity most recently entered its owner's hand.
    #[serde(default)]
    pub entered_hand_turn: Option<u32>,
    pub script_data: BTreeMap<String, i64>,
    /// Card scripts merged into this minion by attachment mechanics such as
    /// Magnetic. Duplicates preserve attachment order.
    #[serde(default)]
    pub attached_cards: Vec<CardId>,
}

impl Entity {
    pub fn health(&self) -> i32 {
        self.max_health - self.damage
    }

    pub fn is_mortally_wounded(&self) -> bool {
        self.zone == Zone::Board && self.kind == CardKind::Minion && self.health() <= 0
    }

    pub fn has_keyword(&self, keyword: &str) -> bool {
        self.keywords.iter().any(|value| value == keyword)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    #[serde(default = "default_player_class")]
    pub class: String,
    pub hero: EntityId,
    pub deck: VecDeque<EntityId>,
    pub hand: Vec<EntityId>,
    pub board: Vec<EntityId>,
    pub weapon: Option<EntityId>,
    pub hero_power: EntityId,
    pub hero_power_used: bool,
    pub secrets: Vec<EntityId>,
    pub graveyard: Vec<EntityId>,
    pub mana: u8,
    pub max_mana: u8,
    #[serde(default)]
    pub temporary_mana: u8,
    #[serde(default)]
    pub overload_pending: u8,
    #[serde(default)]
    pub overloaded_mana: u8,
    #[serde(default)]
    pub cards_played_this_turn: u32,
    /// Card definition IDs frozen at play/cast time. Card play history also
    /// includes countered cards, while the typed histories only include plays
    /// that passed the generic card counter phase.
    #[serde(default)]
    pub cards_played_history: Vec<CardId>,
    /// Cards played during this player's previous and current turns. These
    /// frozen definition IDs let Kindred remain deterministic across snapshots.
    #[serde(default)]
    pub cards_played_last_turn: Vec<CardId>,
    #[serde(default)]
    pub cards_played_current_turn: Vec<CardId>,
    #[serde(default)]
    pub spells_cast_history: Vec<CardId>,
    #[serde(default)]
    pub minions_played_history: Vec<CardId>,
    #[serde(default)]
    pub weapons_played_history: Vec<CardId>,
    #[serde(default)]
    pub locations_played_history: Vec<CardId>,
    pub fatigue: u32,
    #[serde(default)]
    pub script_data: BTreeMap<String, i64>,
}

fn default_player_class() -> String {
    "neutral".to_owned()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOutcome {
    Winner(PlayerId),
    Draw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub rng_seed: u64,
    pub random_counter: u64,
    pub turn: u32,
    pub active_player: PlayerId,
    pub players: [PlayerState; 2],
    pub entities: BTreeMap<EntityId, Entity>,
    pub next_entity_id: u64,
    pub next_timestamp: u64,
    pub next_enchantment_id: u64,
    pub next_event_id: u64,
    pub outcome: Option<GameOutcome>,
    #[serde(default)]
    pub mulligan: Option<MulliganState>,
    pub pending_input: Option<PendingInput>,
    pub log: Vec<GameEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MulliganState {
    pub current_player: PlayerId,
    pub eligible: [Vec<EntityId>; 2],
}

impl GameState {
    pub fn player(&self, player: PlayerId) -> &PlayerState {
        &self.players[player.index()]
    }

    pub fn player_mut(&mut self, player: PlayerId) -> &mut PlayerState {
        &mut self.players[player.index()]
    }

    pub fn entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn hero(&self, player: PlayerId) -> &Entity {
        &self.entities[&self.player(player).hero]
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        for expected in [PlayerId::ONE, PlayerId::TWO] {
            let player = self.player(expected);
            if player.id != expected {
                return Err(format!("player slot {expected} contains {}", player.id));
            }
            if player.class.trim().is_empty() || player.class.len() > 64 {
                return Err(format!("{expected} has invalid class {:?}", player.class));
            }
            if player.hand.len() > 10 || player.board.len() > 7 || player.secrets.len() > 5 {
                return Err(format!("{expected} exceeds a zone capacity"));
            }
            if player.max_mana > 10 {
                return Err(format!("{expected} has invalid mana"));
            }
            if player.temporary_mana > player.mana {
                return Err(format!("{expected} has invalid temporary mana"));
            }
            if player.overloaded_mana > player.max_mana {
                return Err(format!("{expected} has invalid locked mana"));
            }
            if player
                .mana
                .saturating_sub(player.temporary_mana)
                .saturating_add(player.overloaded_mana)
                > player.max_mana
            {
                return Err(format!("{expected} has mana exceeding unlocked crystals"));
            }

            let mut check = |id: EntityId, zone: Zone| -> Result<(), String> {
                let entity = self
                    .entity(id)
                    .ok_or_else(|| format!("{expected} references missing entity {id}"))?;
                if entity.zone != zone {
                    return Err(format!(
                        "entity {id} is listed in {zone:?} but says {:?}",
                        entity.zone
                    ));
                }
                if entity.controller != expected {
                    return Err(format!(
                        "entity {id} is listed for {expected} but controlled by {}",
                        entity.controller
                    ));
                }
                if !seen.insert(id) {
                    return Err(format!("entity {id} occurs in multiple zones"));
                }
                Ok(())
            };

            check(player.hero, Zone::Hero)?;
            check(player.hero_power, Zone::HeroPower)?;
            for id in &player.deck {
                check(*id, Zone::Deck)?;
            }
            for id in &player.hand {
                check(*id, Zone::Hand)?;
            }
            for id in &player.board {
                check(*id, Zone::Board)?;
                let entity = &self.entities[id];
                if !matches!(entity.kind, CardKind::Minion | CardKind::Location) {
                    return Err(format!(
                        "entity {id} has invalid {:?} kind for the board",
                        entity.kind
                    ));
                }
            }
            if let Some(weapon) = player.weapon {
                check(weapon, Zone::Weapon)?;
            }
            for id in &player.secrets {
                check(*id, Zone::Secret)?;
            }
            for id in &player.graveyard {
                check(*id, Zone::Graveyard)?;
            }
        }

        for entity in self.entities.values() {
            if entity.armor < 0 || (entity.kind != CardKind::Hero && entity.armor != 0) {
                return Err(format!(
                    "entity {} has invalid armor {}",
                    entity.id, entity.armor
                ));
            }
            if entity.spell_damage < 0 {
                return Err(format!(
                    "entity {} has invalid spell damage {}",
                    entity.id, entity.spell_damage
                ));
            }
            if entity.kind != CardKind::Location && entity.location_cooldown != 0 {
                return Err(format!(
                    "non-location entity {} has location cooldown {}",
                    entity.id, entity.location_cooldown
                ));
            }
            if entity.location_cooldown > 2 {
                return Err(format!(
                    "entity {} has invalid location cooldown {}",
                    entity.id, entity.location_cooldown
                ));
            }
            match entity.zone {
                Zone::SetAside => {
                    if self.pending_input.is_none() {
                        return Err(format!(
                            "entity {} remains set aside without pending input",
                            entity.id
                        ));
                    }
                }
                Zone::Removed => {}
                _ if !seen.contains(&entity.id) => {
                    return Err(format!(
                        "entity {} says {:?} but is absent from that zone",
                        entity.id, entity.zone
                    ));
                }
                _ => {}
            }
        }
        if self.outcome.is_some() && self.pending_input.is_some() {
            return Err("finished game still has pending input".to_owned());
        }
        if let Some(mulligan) = &self.mulligan {
            if self.turn != 0 {
                return Err("mulligan remains active after the first turn started".to_owned());
            }
            if self.active_player != mulligan.current_player {
                return Err("active player does not match the mulligan player".to_owned());
            }
            if self.pending_input.is_some() {
                return Err("mulligan and a scripted choice are active together".to_owned());
            }
        }
        if self
            .entities
            .keys()
            .next_back()
            .is_some_and(|id| id.0 >= self.next_entity_id)
        {
            return Err("next_entity_id is not above all allocated IDs".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerCommand {
    Mulligan {
        replace: Vec<EntityId>,
    },
    PlayCard {
        card: EntityId,
        target: Option<EntityId>,
    },
    PlayCardAt {
        card: EntityId,
        target: Option<EntityId>,
        position: usize,
    },
    TradeCard {
        card: EntityId,
    },
    UseCardAction {
        card: EntityId,
        action: String,
        target: Option<EntityId>,
    },
    Attack {
        attacker: EntityId,
        defender: EntityId,
    },
    UseHeroPower {
        target: Option<EntityId>,
    },
    UseLocation {
        location: EntityId,
        target: Option<EntityId>,
    },
    EndTurn,
    Concede,
    Choose {
        index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardActionSpec {
    pub id: String,
    pub cost: u8,
    #[serde(default)]
    pub spend_all_mana: bool,
    #[serde(default)]
    pub target_mode: TargetMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub format_version: u32,
    pub card_pack_hash: String,
    pub seed: u64,
    pub decks: [Vec<CardId>; 2],
    pub hero_powers: [CardId; 2],
    pub classes: [String; 2],
    pub commands: Vec<PlayerCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub format_version: u32,
    pub replay: Replay,
    pub state: GameState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceValue {
    Entity(EntityId),
    Card(CardId),
    Number(i32),
    Integer(i64),
    Nil,
    Boolean(bool),
    Text(String),
    List(Vec<ChoiceValue>),
    Object(BTreeMap<String, ChoiceValue>),
}

pub const MAX_CHOICE_VALUE_DEPTH: usize = 16;
pub const MAX_CHOICE_VALUE_NODES: usize = 512;
pub const MAX_CHOICE_VALUE_STRING_BYTES: usize = 16 * 1024;

impl ChoiceValue {
    pub fn validate(&self) -> Result<(), String> {
        let mut nodes = 0;
        let mut string_bytes = 0;
        self.validate_inner(0, &mut nodes, &mut string_bytes)
    }

    fn validate_inner(
        &self,
        depth: usize,
        nodes: &mut usize,
        string_bytes: &mut usize,
    ) -> Result<(), String> {
        if depth > MAX_CHOICE_VALUE_DEPTH {
            return Err(format!(
                "choice value exceeds maximum depth {MAX_CHOICE_VALUE_DEPTH}"
            ));
        }
        *nodes += 1;
        if *nodes > MAX_CHOICE_VALUE_NODES {
            return Err(format!(
                "choice value exceeds maximum node count {MAX_CHOICE_VALUE_NODES}"
            ));
        }
        fn count_string(string_bytes: &mut usize, value: &str) -> Result<(), String> {
            *string_bytes = string_bytes.saturating_add(value.len());
            if *string_bytes > MAX_CHOICE_VALUE_STRING_BYTES {
                return Err(format!(
                    "choice value exceeds maximum string data {MAX_CHOICE_VALUE_STRING_BYTES} bytes"
                ));
            }
            Ok(())
        }
        match self {
            Self::Card(card) | Self::Text(card) => count_string(string_bytes, card),
            Self::List(values) => {
                for value in values {
                    value.validate_inner(depth + 1, nodes, string_bytes)?;
                }
                Ok(())
            }
            Self::Object(values) => {
                for (key, value) in values {
                    count_string(string_bytes, key)?;
                    value.validate_inner(depth + 1, nodes, string_bytes)?;
                }
                Ok(())
            }
            Self::Entity(_) | Self::Number(_) | Self::Integer(_) | Self::Nil | Self::Boolean(_) => {
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub label: String,
    pub value: ChoiceValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingInput {
    pub player: PlayerId,
    pub source: EntityId,
    pub prompt: String,
    pub options: Vec<ChoiceOption>,
    pub resume_hook: String,
    pub remaining_resolution: Vec<ResolutionItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSpec {
    Damage {
        source: EntityId,
        target: EntityId,
        amount: i32,
    },
    DamageGroup {
        source: EntityId,
        targets: Vec<EntityId>,
        amount: i32,
    },
    Heal {
        source: EntityId,
        target: EntityId,
        amount: i32,
    },
    GainArmor {
        source: EntityId,
        player: PlayerId,
        amount: i32,
    },
    Overload {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    UnlockMana {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    ClearOverload {
        source: EntityId,
        player: PlayerId,
    },
    GainTemporaryMana {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    GainManaCrystals {
        source: EntityId,
        player: PlayerId,
        amount: u8,
        filled: bool,
    },
    DestroyManaCrystals {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    Draw {
        player: PlayerId,
        count: u8,
    },
    GiveCard {
        source: EntityId,
        player: PlayerId,
        card_id: CardId,
    },
    GiveCardAt {
        source: EntityId,
        player: PlayerId,
        card_id: CardId,
        position: usize,
    },
    ShuffleCardIntoDeck {
        source: EntityId,
        player: PlayerId,
        card_id: CardId,
    },
    ReplaceHeroPower {
        source: EntityId,
        player: PlayerId,
        card_id: CardId,
    },
    RefreshHeroPower {
        source: EntityId,
        player: PlayerId,
    },
    GiveMergedMinion {
        source: EntityId,
        player: PlayerId,
        template: CardId,
        first: CardId,
        second: CardId,
    },
    EquipWeapon {
        source: EntityId,
        player: PlayerId,
        card_id: CardId,
    },
    Discard {
        source: EntityId,
        player: PlayerId,
        target: EntityId,
    },
    CastSpell {
        source: EntityId,
        player: PlayerId,
        card_id: CardId,
        #[serde(default)]
        target: Option<EntityId>,
    },
    CastDrawn {
        card: EntityId,
    },
    Summon {
        player: PlayerId,
        card_id: CardId,
        #[serde(default)]
        position: Option<usize>,
    },
    SummonFromHand {
        card: EntityId,
    },
    SummonCopy {
        player: PlayerId,
        target: EntityId,
        #[serde(default)]
        position: Option<usize>,
    },
    Recruit {
        source: EntityId,
        player: PlayerId,
        target: EntityId,
        #[serde(default)]
        position: Option<usize>,
    },
    MoveEntity {
        source: EntityId,
        target: EntityId,
        destination: ZonePlacement,
    },
    ChangeController {
        source: EntityId,
        target: EntityId,
        player: PlayerId,
    },
    Transform {
        source: EntityId,
        target: EntityId,
        card_id: CardId,
    },
    Destroy {
        source: EntityId,
        target: EntityId,
    },
    Buff {
        source: EntityId,
        target: EntityId,
        attack: i32,
        health: i32,
        keywords: Vec<String>,
        duration: EffectDuration,
    },
    DisableKeyword {
        source: EntityId,
        target: EntityId,
        keyword: String,
    },
    SummonFreshCopy {
        source: EntityId,
        player: PlayerId,
        target: EntityId,
        #[serde(default)]
        position: Option<usize>,
        health: i32,
        #[serde(default)]
        without_keywords: Vec<String>,
    },
    ModifyStat {
        source: EntityId,
        target: EntityId,
        modifier: StatModifier,
        duration: EffectDuration,
        #[serde(default = "default_true")]
        silenciable: bool,
    },
    RemoveEnchantmentsFromSource {
        source: EntityId,
        target: EntityId,
        enchantment_source: EntityId,
    },
    SetScriptData {
        source: EntityId,
        target: EntityId,
        key: String,
        value: i64,
    },
    SetPlayerScriptData {
        source: EntityId,
        player: PlayerId,
        key: String,
        value: i64,
    },
    Silence {
        source: EntityId,
        target: EntityId,
    },
    Freeze {
        source: EntityId,
        target: EntityId,
    },
    RevealSecret {
        source: EntityId,
        secret: EntityId,
    },
    CancelEvent {
        source: EntityId,
        event: EventId,
    },
    SetEventAmount {
        source: EntityId,
        event: EventId,
        amount: i32,
    },
    SetTradeDraw {
        source: EntityId,
        event: EventId,
        replacement: EntityId,
    },
    Continue {
        source: EntityId,
        hook: String,
        payload: Option<ChoiceValue>,
    },
    RequestChoice {
        player: PlayerId,
        source: EntityId,
        prompt: String,
        options: Vec<ChoiceOption>,
        resume_hook: String,
    },
    DiscoverCards {
        player: PlayerId,
        source: EntityId,
        prompt: String,
        candidates: Vec<CardId>,
        count: usize,
        resume_hook: String,
    },
    DiscoverEntities {
        player: PlayerId,
        source: EntityId,
        prompt: String,
        candidates: Vec<EntityId>,
        count: usize,
        resume_hook: String,
    },
    RandomChoice {
        source: EntityId,
        options: Vec<ChoiceValue>,
        resume_hook: String,
    },
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingEvent {
    pub id: EventId,
    pub event: GameEvent,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReservedSummonOrigin {
    #[default]
    Generated,
    Deck {
        player: PlayerId,
        position: usize,
        previous: Option<EntityId>,
        next: Option<EntityId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionItem {
    Effect(EffectSpec),
    DrawOne {
        player: PlayerId,
    },
    CommitFatigue(PendingEvent),
    CommitEvent(PendingEvent),
    CommitCardPlay {
        play: PendingEvent,
        target: Option<EntityId>,
        #[serde(default)]
        position: Option<usize>,
    },
    CommitDiscard(PendingEvent),
    CommitTradeDraw(PendingEvent),
    CompleteTrade {
        player: PlayerId,
        card: EntityId,
    },
    CommitHeroPower {
        use_event: PendingEvent,
        target: Option<EntityId>,
    },
    CommitLocationUse(PendingEvent),
    DestroySpentLocation {
        player: PlayerId,
        location: EntityId,
    },
    CommitWeaponEquip {
        equip: PendingEvent,
        card_play_id: EventId,
        target: Option<EntityId>,
        replacement: Option<PendingEvent>,
    },
    CommitWeaponDestruction(PendingEvent),
    CommitCombat {
        attack: PendingEvent,
        damage: Vec<PendingEvent>,
    },
    CommitDamageGroup {
        damage: Vec<PendingEvent>,
    },
    CommitSummon {
        summon: PendingEvent,
        #[serde(default)]
        position: Option<usize>,
        #[serde(default)]
        origin: ReservedSummonOrigin,
    },
    CommitZoneChange {
        change: PendingEvent,
        destination: ZonePlacement,
    },
    CommitControllerChange(PendingEvent),
    CommitTransform(PendingEvent),
    SummonFreshCopy {
        player: PlayerId,
        card_id: CardId,
        #[serde(default)]
        position: usize,
        health: i32,
        #[serde(default)]
        without_keywords: Vec<String>,
    },
    PublishAfter {
        id: EventId,
        event: GameEvent,
    },
    PublishAfterGroup {
        events: Vec<(EventId, GameEvent)>,
    },
    DeathCheck,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptEvent {
    pub id: EventId,
    pub timing: EventTiming,
    pub event: GameEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    GameStarted,
    TurnStarted {
        player: PlayerId,
        turn: u32,
    },
    CardDrawn {
        player: PlayerId,
        card: EntityId,
    },
    CardBurned {
        player: PlayerId,
        card: EntityId,
    },
    CardCreated {
        source: EntityId,
        player: PlayerId,
        card: EntityId,
    },
    Fatigue {
        player: PlayerId,
        amount: u32,
    },
    CardPlayed {
        player: PlayerId,
        card: EntityId,
    },
    SpellCast {
        player: PlayerId,
        spell: EntityId,
        #[serde(default)]
        generated_by: Option<EntityId>,
    },
    MinionPlayed {
        player: PlayerId,
        minion: EntityId,
    },
    WeaponPlayed {
        player: PlayerId,
        weapon: EntityId,
    },
    LocationPlayed {
        player: PlayerId,
        location: EntityId,
    },
    CardCountered {
        player: PlayerId,
        card: EntityId,
    },
    CardDiscarded {
        source: EntityId,
        player: PlayerId,
        card: EntityId,
    },
    CardTraded {
        player: PlayerId,
        card: EntityId,
    },
    TradeDraw {
        player: PlayerId,
        card: EntityId,
        #[serde(default)]
        replacement: Option<EntityId>,
    },
    MinionSummoned {
        player: PlayerId,
        entity: EntityId,
    },
    Magnetized {
        player: PlayerId,
        attachment: EntityId,
        target: EntityId,
    },
    WeaponEquipped {
        player: PlayerId,
        weapon: EntityId,
    },
    WeaponDestroyed {
        player: PlayerId,
        weapon: EntityId,
    },
    LocationUsed {
        player: PlayerId,
        location: EntityId,
        target: Option<EntityId>,
    },
    LocationDestroyed {
        player: PlayerId,
        location: EntityId,
    },
    HeroPowerUsed {
        player: PlayerId,
        hero_power: EntityId,
        target: Option<EntityId>,
    },
    HeroPowerReplaced {
        source: EntityId,
        player: PlayerId,
        old: EntityId,
        new: EntityId,
    },
    HeroReplaced {
        player: PlayerId,
        old: EntityId,
        new: EntityId,
    },
    SecretPlayed {
        player: PlayerId,
        secret: EntityId,
    },
    SecretRevealed {
        player: PlayerId,
        secret: EntityId,
    },
    ZoneChanged {
        entity: EntityId,
        from: Zone,
        to: Zone,
    },
    ControllerChanged {
        source: EntityId,
        entity: EntityId,
        from: PlayerId,
        to: PlayerId,
    },
    Transformed {
        source: EntityId,
        entity: EntityId,
        from_card: CardId,
        to_card: CardId,
    },
    Attack {
        attacker: EntityId,
        defender: EntityId,
    },
    Damaged {
        source: EntityId,
        target: EntityId,
        amount: i32,
    },
    DamagePrevented {
        source: EntityId,
        target: EntityId,
        reason: String,
    },
    Healed {
        source: EntityId,
        target: EntityId,
        amount: i32,
    },
    ArmorGained {
        source: EntityId,
        target: EntityId,
        amount: i32,
    },
    OverloadQueued {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    ManaLocked {
        player: PlayerId,
        amount: u8,
    },
    ManaUnlocked {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    OverloadCleared {
        source: EntityId,
        player: PlayerId,
        pending: u8,
        locked: u8,
    },
    TemporaryManaGained {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    TemporaryManaExpired {
        player: PlayerId,
        amount: u8,
    },
    ManaCrystalsGained {
        source: EntityId,
        player: PlayerId,
        amount: u8,
        filled: bool,
    },
    ManaCrystalsDestroyed {
        source: EntityId,
        player: PlayerId,
        amount: u8,
    },
    ManaSpent {
        player: PlayerId,
        source: EntityId,
        amount: u8,
        temporary: u8,
    },
    KeywordDisabled {
        source: EntityId,
        target: EntityId,
        keyword: String,
    },
    Frozen {
        source: EntityId,
        target: EntityId,
    },
    EntityDied {
        entity: EntityId,
        player: PlayerId,
        position: usize,
    },
    TurnEnded {
        player: PlayerId,
        turn: u32,
    },
    Conceded {
        player: PlayerId,
    },
    GameEnded {
        outcome: GameOutcome,
    },
    ChoiceRequested {
        player: PlayerId,
        source: EntityId,
        options: usize,
    },
    ChoiceMade {
        player: PlayerId,
        source: EntityId,
        index: usize,
    },
    RandomChoiceMade {
        source: EntityId,
        index: usize,
        options: usize,
    },
    RandomCardsSampled {
        source: EntityId,
        cards: Vec<CardId>,
        population: usize,
    },
    RandomEntitiesSampled {
        source: EntityId,
        entities: Vec<EntityId>,
        population: usize,
    },
}

impl GameEvent {
    pub fn script_name(&self) -> &'static str {
        match self {
            Self::GameStarted => "game_started",
            Self::TurnStarted { .. } => "turn_started",
            Self::CardDrawn { .. } => "card_drawn",
            Self::CardBurned { .. } => "card_burned",
            Self::CardCreated { .. } => "card_created",
            Self::Fatigue { .. } => "fatigue",
            Self::CardPlayed { .. } => "card_played",
            Self::SpellCast { .. } => "spell_cast",
            Self::MinionPlayed { .. } => "minion_played",
            Self::WeaponPlayed { .. } => "weapon_played",
            Self::LocationPlayed { .. } => "location_played",
            Self::CardCountered { .. } => "card_countered",
            Self::CardDiscarded { .. } => "card_discarded",
            Self::CardTraded { .. } => "card_traded",
            Self::TradeDraw { .. } => "trade_draw",
            Self::MinionSummoned { .. } => "minion_summoned",
            Self::Magnetized { .. } => "magnetized",
            Self::WeaponEquipped { .. } => "weapon_equipped",
            Self::WeaponDestroyed { .. } => "weapon_destroyed",
            Self::LocationUsed { .. } => "location_used",
            Self::LocationDestroyed { .. } => "location_destroyed",
            Self::HeroPowerUsed { .. } => "hero_power_used",
            Self::HeroPowerReplaced { .. } => "hero_power_replaced",
            Self::HeroReplaced { .. } => "hero_replaced",
            Self::SecretPlayed { .. } => "secret_played",
            Self::SecretRevealed { .. } => "secret_revealed",
            Self::ZoneChanged { .. } => "zone_changed",
            Self::ControllerChanged { .. } => "controller_changed",
            Self::Transformed { .. } => "transformed",
            Self::Attack { .. } => "attack",
            Self::Damaged { .. } => "damaged",
            Self::DamagePrevented { .. } => "damage_prevented",
            Self::Healed { .. } => "healed",
            Self::ArmorGained { .. } => "armor_gained",
            Self::OverloadQueued { .. } => "overload_queued",
            Self::ManaLocked { .. } => "mana_locked",
            Self::ManaUnlocked { .. } => "mana_unlocked",
            Self::OverloadCleared { .. } => "overload_cleared",
            Self::TemporaryManaGained { .. } => "temporary_mana_gained",
            Self::TemporaryManaExpired { .. } => "temporary_mana_expired",
            Self::ManaCrystalsGained { .. } => "mana_crystals_gained",
            Self::ManaCrystalsDestroyed { .. } => "mana_crystals_destroyed",
            Self::ManaSpent { .. } => "mana_spent",
            Self::KeywordDisabled { .. } => "keyword_disabled",
            Self::Frozen { .. } => "frozen",
            Self::EntityDied { .. } => "entity_died",
            Self::TurnEnded { .. } => "turn_ended",
            Self::Conceded { .. } => "conceded",
            Self::GameEnded { .. } => "game_ended",
            Self::ChoiceRequested { .. } => "choice_requested",
            Self::ChoiceMade { .. } => "choice_made",
            Self::RandomChoiceMade { .. } => "random_choice_made",
            Self::RandomCardsSampled { .. } => "random_cards_sampled",
            Self::RandomEntitiesSampled { .. } => "random_entities_sampled",
        }
    }
}
