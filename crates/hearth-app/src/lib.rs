use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use hearth_bot::DifficultyBot;
use hearth_core::{
    CardDefinition, CardKind, CardRuntime, DEFAULT_HERO_POWER, Game, GameSnapshot, LegalAction,
    Locale, PlayerCommand, PlayerController, PlayerId, PlayerView, RuneCost,
};
use hearth_script::LuaCardRuntime;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod deckstring;

pub use deckstring::{DeckstringError, export_deckstring, import_deckstring};
pub use hearth_bot::BotDifficulty;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct DeckSideboard {
    /// Card in the main deck that owns this sideboard.
    pub owner: String,
    pub cards: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeckList {
    pub name: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default = "default_deck_class")]
    pub class: String,
    pub cards: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sideboards: Vec<DeckSideboard>,
    #[serde(default)]
    pub hero_power: Option<String>,
    #[serde(default)]
    pub unrestricted: bool,
}

#[derive(Clone, Debug)]
pub struct StoredDeck {
    pub path: PathBuf,
    pub deck: DeckList,
}

#[derive(Clone, Debug)]
pub struct CardCatalogEntry {
    pub id: String,
    pub name: String,
    pub text: String,
    pub set: String,
    pub kind: CardKind,
    pub collectible: bool,
    pub class: String,
    pub classes: Vec<String>,
    pub sideboard_size: u8,
    pub deck_size: Option<u8>,
    pub starting_health: Option<i32>,
    pub rune_cost: RuneCost,
    pub rarity: Option<String>,
    pub cost: u8,
    pub attack: i32,
    pub health: i32,
    pub armor: i32,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DeckLibrary {
    deck_root: PathBuf,
    decks: Vec<StoredDeck>,
    cards: Vec<CardCatalogEntry>,
    definitions: Vec<CardCatalogEntry>,
    dbf_ids: BTreeMap<String, u32>,
    card_ids_by_dbf: BTreeMap<u32, String>,
}

fn default_deck_class() -> String {
    "mage".to_owned()
}

pub fn basic_hero_power_for_class(class: &str) -> &'static str {
    match class {
        "warrior" => "HERO_01bp",
        "shaman" => "HERO_02bp",
        "rogue" => "HERO_03bp",
        "paladin" => "HERO_04bp",
        "hunter" => "HERO_05bp",
        "druid" => "HERO_06bp",
        "warlock" => "HERO_07bp",
        "priest" => "HERO_09bp",
        "demon_hunter" => "HERO_10bp",
        "death_knight" => "HERO_11bp",
        _ => DEFAULT_HERO_POWER,
    }
}

/// Selects a fair, deterministic first player from the match seed.
///
/// The SplitMix64 finalizer avoids coupling opening order to the engine's
/// replay RNG stream, so adding this rule does not perturb card shuffles.
pub fn starting_player_for_seed(seed: u64) -> PlayerId {
    let mut mixed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^= mixed >> 31;
    if mixed & 1 == 0 {
        PlayerId::TWO
    } else {
        PlayerId::ONE
    }
}

#[derive(Clone, Debug)]
pub struct MatchConfig {
    pub data_dir: PathBuf,
    pub deck_one: PathBuf,
    pub deck_two: PathBuf,
    pub seed: u64,
    pub locale: Locale,
    pub human_player: PlayerId,
    pub match_mode: MatchMode,
    pub bot_difficulty: BotDifficulty,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum MatchMode {
    #[default]
    VsBot,
    Hotseat,
}

impl MatchConfig {
    pub fn demo(workspace_root: impl AsRef<Path>) -> Self {
        let root = workspace_root.as_ref();
        Self {
            data_dir: root.join("data"),
            deck_one: root.join("decks/demo.json"),
            deck_two: root.join("decks/demo.json"),
            seed: 20260829,
            locale: Locale::EnUs,
            human_player: PlayerId::ONE,
            match_mode: MatchMode::VsBot,
            bot_difficulty: BotDifficulty::Normal,
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse deck {path}: {source}")]
    ParseDeck {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to parse card metadata {path}: {source}")]
    ParseCardMetadata {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid card metadata {path}: {message}")]
    InvalidCardMetadata { path: PathBuf, message: String },
    #[error("failed to serialize deck {path}: {source}")]
    SerializeDeck {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to delete {path}: {source}")]
    Delete {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("deck index {0} does not exist")]
    UnknownDeckIndex(usize),
    #[error("deck path is not loaded: {0}")]
    UnknownDeckPath(PathBuf),
    #[error("refusing to delete protected repository deck {0}")]
    ProtectedDeck(PathBuf),
    #[error("another custom deck already uses {0}")]
    DeckNameConflict(PathBuf),
    #[error("invalid deck {deck}: {message}")]
    InvalidDeck { deck: String, message: String },
    #[error("failed to load card scripts: {0}")]
    Script(#[from] hearth_script::ScriptLoadError),
    #[error("game error: {0}")]
    Game(#[from] hearth_core::GameError),
    #[error("controller error: {0}")]
    Controller(String),
    #[error("runtime rule error: {0}")]
    RuntimeRule(String),
    #[error("automated opponent exceeded {0} consecutive actions")]
    BotActionLimit(usize),
    #[error("unsupported saved session format {0}")]
    UnsupportedSessionSnapshot(u32),
}

impl DeckLibrary {
    pub fn load(
        deck_root: impl AsRef<Path>,
        data_dir: impl AsRef<Path>,
        locale: Locale,
    ) -> Result<Self, AppError> {
        let deck_root = deck_root.as_ref().to_owned();
        let data_dir = data_dir.as_ref();
        let mut paths = Vec::new();
        collect_deck_paths(&deck_root, &mut paths)?;
        paths.sort();
        let mut decks = paths
            .into_iter()
            .map(|path| load_deck(&path).map(|deck| StoredDeck { path, deck }))
            .collect::<Result<Vec<_>, _>>()?;
        decks.sort_by(|left, right| {
            left.deck
                .name
                .to_lowercase()
                .cmp(&right.deck.name.to_lowercase())
                .then_with(|| left.path.cmp(&right.path))
        });

        let (dbf_ids, card_ids_by_dbf) = load_card_dbf_ids(data_dir)?;
        let runtime = LuaCardRuntime::load_dir_with_locale(data_dir, locale)?;
        for stored in &decks {
            validate_deck(&runtime, &stored.deck)?;
        }
        let mut definitions = runtime
            .definitions()
            .map(card_catalog_entry)
            .collect::<Vec<_>>();
        definitions.sort_by(|left, right| left.id.cmp(&right.id));
        let mut cards = definitions
            .iter()
            .filter(|definition| {
                definition.collectible
                    && !definition.set.eq_ignore_ascii_case("HERO_SKINS")
                    && matches!(
                        definition.kind,
                        CardKind::Hero
                            | CardKind::Minion
                            | CardKind::Spell
                            | CardKind::Weapon
                            | CardKind::Location
                    )
            })
            .cloned()
            .collect::<Vec<_>>();
        cards.sort_by(|left, right| {
            left.cost
                .cmp(&right.cost)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(Self {
            deck_root,
            decks,
            cards,
            definitions,
            dbf_ids,
            card_ids_by_dbf,
        })
    }

    pub fn decks(&self) -> &[StoredDeck] {
        &self.decks
    }

    pub fn cards(&self) -> &[CardCatalogEntry] {
        &self.cards
    }

    /// Required constructed main-deck size after applying cards such as
    /// Prince Renathal. Unknown cards cannot grant extra capacity.
    pub fn required_deck_size(&self, deck: &DeckList) -> usize {
        required_deck_size(deck, &self.cards)
    }

    /// Minimum Death Knight rune commitment required by the main deck and all
    /// constructed sideboards.
    pub fn deck_rune_cost(&self, deck: &DeckList) -> RuneCost {
        deck_rune_cost(deck, &self.cards)
    }

    /// Whether adding a card preserves the official three-slot Death Knight
    /// rune constraint. Other classes and mechanics sandboxes are unaffected.
    pub fn card_fits_deck_runes(&self, deck: &DeckList, card: &CardCatalogEntry) -> bool {
        deck.unrestricted
            || !deck.class.eq_ignore_ascii_case("death_knight")
            || self
                .deck_rune_cost(deck)
                .combined(card.rune_cost)
                .fits_death_knight_deck()
    }

    pub fn definition(&self, card_id: &str) -> Option<&CardCatalogEntry> {
        self.definitions
            .binary_search_by_key(&card_id, |definition| definition.id.as_str())
            .ok()
            .map(|index| &self.definitions[index])
    }

    pub fn dbf_id(&self, card_id: &str) -> Option<u32> {
        self.dbf_ids.get(card_id).copied()
    }

    pub fn card_id_for_dbf(&self, dbf_id: u32) -> Option<&str> {
        self.card_ids_by_dbf.get(&dbf_id).map(String::as_str)
    }

    pub fn reload_locale(&mut self, data_dir: &Path, locale: Locale) -> Result<(), AppError> {
        let reloaded = Self::load(&self.deck_root, data_dir, locale)?;
        *self = reloaded;
        Ok(())
    }

    pub fn deck(&self, index: usize) -> Option<&StoredDeck> {
        self.decks.get(index)
    }

    pub fn index_of_path(&self, path: &Path) -> Option<usize> {
        self.decks.iter().position(|deck| deck.path == path)
    }

    pub fn is_custom(&self, index: usize) -> bool {
        self.decks
            .get(index)
            .is_some_and(|stored| self.is_custom_path(&stored.path))
    }

    pub fn delete_custom(&mut self, index: usize) -> Result<StoredDeck, AppError> {
        let stored = self
            .decks
            .get(index)
            .cloned()
            .ok_or(AppError::UnknownDeckIndex(index))?;
        if !self.is_custom_path(&stored.path) {
            return Err(AppError::ProtectedDeck(stored.path));
        }
        fs::remove_file(&stored.path).map_err(|source| AppError::Delete {
            path: stored.path.clone(),
            source,
        })?;
        self.decks.remove(index);
        Ok(stored)
    }

    fn is_custom_path(&self, path: &Path) -> bool {
        path.parent() == Some(self.deck_root.join("custom").as_path())
    }

    pub fn save_custom(&mut self, deck: &DeckList) -> Result<PathBuf, AppError> {
        self.save_custom_internal(None, deck)
    }

    pub fn replace_custom(&mut self, source: &Path, deck: &DeckList) -> Result<PathBuf, AppError> {
        self.save_custom_internal(Some(source), deck)
    }

    fn save_custom_internal(
        &mut self,
        source: Option<&Path>,
        deck: &DeckList,
    ) -> Result<PathBuf, AppError> {
        validate_editable_deck(deck, &self.cards)?;
        let directory = self.deck_root.join("custom");
        fs::create_dir_all(&directory).map_err(|source| AppError::Write {
            path: directory.clone(),
            source,
        })?;
        let slug = deck_slug(&deck.name);
        let path = directory.join(format!("{slug}.json"));
        let source_index = match source {
            Some(source) => {
                if !self.is_custom_path(source) {
                    return Err(AppError::ProtectedDeck(source.to_owned()));
                }
                Some(
                    self.index_of_path(source)
                        .ok_or_else(|| AppError::UnknownDeckPath(source.to_owned()))?,
                )
            }
            None => None,
        };
        if source != Some(path.as_path()) && path.exists() {
            return Err(AppError::DeckNameConflict(path));
        }
        let json =
            serde_json::to_string_pretty(deck).map_err(|source| AppError::SerializeDeck {
                path: path.clone(),
                source,
            })?;
        fs::write(&path, format!("{json}\n")).map_err(|source| AppError::Write {
            path: path.clone(),
            source,
        })?;
        if let Some(source) = source
            && source != path
            && let Err(delete_error) = fs::remove_file(source)
        {
            let _ = fs::remove_file(&path);
            return Err(AppError::Delete {
                path: source.to_owned(),
                source: delete_error,
            });
        }
        let stored = StoredDeck {
            path: path.clone(),
            deck: deck.clone(),
        };
        if let Some(source_index) = source_index {
            self.decks.remove(source_index);
        }
        self.decks.push(stored);
        self.decks.sort_by(|left, right| {
            left.deck
                .name
                .to_lowercase()
                .cmp(&right.deck.name.to_lowercase())
                .then_with(|| left.path.cmp(&right.path))
        });
        Ok(path)
    }
}

fn card_catalog_entry(definition: &CardDefinition) -> CardCatalogEntry {
    CardCatalogEntry {
        id: definition.id.clone(),
        name: definition.name.clone(),
        text: definition.text.clone(),
        set: definition.set.clone(),
        kind: definition.kind,
        collectible: definition.collectible,
        class: definition.class.clone(),
        classes: definition.classes.clone(),
        sideboard_size: definition.sideboard_size,
        deck_size: definition.deck_size,
        starting_health: definition.starting_health,
        rune_cost: definition.rune_cost,
        rarity: definition.rarity.clone(),
        cost: definition.cost,
        attack: definition.attack,
        health: definition.health,
        armor: definition.armor,
        keywords: definition.keywords.clone(),
    }
}

pub struct GameSession {
    game: Game<LuaCardRuntime>,
    human_player: PlayerId,
    match_mode: MatchMode,
    bot: DifficultyBot,
    deck_names: [String; 2],
    locale: Locale,
}

pub const GAME_SESSION_SNAPSHOT_VERSION: u32 = 1;

/// An authoritative local checkpoint. Frontends must treat this as private because it contains
/// both players' hidden zones; ordinary player-facing export should continue to use `PlayerView`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameSessionSnapshot {
    pub format_version: u32,
    pub game: GameSnapshot,
    pub human_player: PlayerId,
    pub match_mode: MatchMode,
    #[serde(default)]
    pub bot_difficulty: BotDifficulty,
    pub deck_names: [String; 2],
}

impl GameSession {
    pub fn load(config: &MatchConfig) -> Result<Self, AppError> {
        let runtime = LuaCardRuntime::load_dir_with_locale(&config.data_dir, config.locale)?;
        let deck_one = load_deck(&config.deck_one)?;
        let deck_two = load_deck(&config.deck_two)?;
        validate_deck(&runtime, &deck_one)?;
        validate_deck(&runtime, &deck_two)?;

        let hero_powers = [
            deck_one
                .hero_power
                .clone()
                .unwrap_or_else(|| basic_hero_power_for_class(&deck_one.class).to_owned()),
            deck_two
                .hero_power
                .clone()
                .unwrap_or_else(|| basic_hero_power_for_class(&deck_two.class).to_owned()),
        ];
        let classes = [deck_one.class.clone(), deck_two.class.clone()];
        let unrestricted = deck_one.unrestricted || deck_two.unrestricted;
        let starting_player = starting_player_for_seed(config.seed);
        let sideboards = [deck_sideboards(&deck_one), deck_sideboards(&deck_two)];
        let game = Game::new_with_sideboards_hero_powers_classes_and_starting_player(
            runtime,
            deck_one.cards,
            deck_two.cards,
            sideboards,
            config.seed,
            hero_powers,
            classes,
            starting_player,
            unrestricted,
        )?;

        Ok(Self {
            game,
            human_player: config.human_player,
            match_mode: config.match_mode,
            bot: DifficultyBot::new(config.bot_difficulty),
            deck_names: [deck_one.name, deck_two.name],
            locale: config.locale,
        })
    }

    pub fn snapshot(&self) -> GameSessionSnapshot {
        GameSessionSnapshot {
            format_version: GAME_SESSION_SNAPSHOT_VERSION,
            game: self.game.snapshot(),
            human_player: self.human_player,
            match_mode: self.match_mode,
            bot_difficulty: self.bot.difficulty(),
            deck_names: self.deck_names.clone(),
        }
    }

    pub fn from_snapshot(
        data_dir: impl AsRef<Path>,
        locale: Locale,
        snapshot: &GameSessionSnapshot,
    ) -> Result<Self, AppError> {
        if snapshot.format_version != GAME_SESSION_SNAPSHOT_VERSION {
            return Err(AppError::UnsupportedSessionSnapshot(
                snapshot.format_version,
            ));
        }
        let runtime = LuaCardRuntime::load_dir_with_locale(data_dir, locale)?;
        let game = Game::from_snapshot(runtime, &snapshot.game)?;
        Ok(Self {
            game,
            human_player: snapshot.human_player,
            match_mode: snapshot.match_mode,
            bot: DifficultyBot::new(snapshot.bot_difficulty),
            deck_names: snapshot.deck_names.clone(),
            locale,
        })
    }

    pub fn human_player(&self) -> PlayerId {
        if self.match_mode == MatchMode::Hotseat {
            self.game.state().input_player()
        } else {
            self.human_player
        }
    }

    pub fn match_mode(&self) -> MatchMode {
        self.match_mode
    }

    pub fn bot_difficulty(&self) -> BotDifficulty {
        self.bot.difficulty()
    }

    pub fn starting_player(&self) -> PlayerId {
        self.game.state().starting_player
    }

    pub fn is_hotseat(&self) -> bool {
        self.match_mode == MatchMode::Hotseat
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn deck_name(&self, player: PlayerId) -> &str {
        &self.deck_names[player.index()]
    }

    pub fn view(&self) -> PlayerView {
        self.game.state().player_view(self.human_player())
    }

    pub fn legal_actions(&self) -> Result<Vec<LegalAction>, AppError> {
        if self.game.state().input_player() != self.human_player() {
            return Ok(Vec::new());
        }
        self.game.legal_action_options().map_err(AppError::from)
    }

    pub fn dispatch_human(&mut self, command: PlayerCommand) -> Result<(), AppError> {
        self.dispatch_human_only(command)?;
        self.advance_bot(10_000)
    }

    /// Dispatches one human command without consuming any following bot input.
    ///
    /// Interactive frontends can use this to present automated actions one at a
    /// time. Batch frontends should normally keep using [`Self::dispatch_human`].
    pub fn dispatch_human_only(&mut self, command: PlayerCommand) -> Result<(), AppError> {
        let human_player = self.human_player();
        if self.game.state().input_player() != human_player {
            return Err(AppError::Controller(format!(
                "{} cannot act while {} has input",
                human_player,
                self.game.state().input_player()
            )));
        }
        let legal = self.game.legal_actions()?;
        if !legal.contains(&command) {
            return Err(AppError::Controller(format!(
                "the selected command is no longer legal: {command:?}"
            )));
        }
        self.game.dispatch(command)?;
        Ok(())
    }

    /// Concedes the locally controlled side even when another controller owns
    /// the current input (for example while the built-in AI is acting).
    pub fn concede_human(&mut self) -> Result<(), AppError> {
        let player = self.human_player();
        self.game
            .dispatch(PlayerCommand::ConcedePlayer { player })?;
        Ok(())
    }

    pub fn is_bot_turn(&self) -> bool {
        !self.is_hotseat()
            && self.game.state().outcome.is_none()
            && self.game.state().input_player() != self.human_player
    }

    /// Advances at most one automated action, returning whether one was
    /// dispatched. This is intentionally deterministic and uses the same bot
    /// policy as [`Self::advance_bot`].
    pub fn advance_bot_once(&mut self) -> Result<bool, AppError> {
        if !self.is_bot_turn() {
            return Ok(false);
        }
        let player = self.game.state().input_player();
        let view = self.game.state().player_view(player);
        let legal = self.game.legal_action_options()?;
        let command = self
            .bot
            .choose_action(&view, &legal)
            .map_err(AppError::Controller)?;
        self.game.dispatch(command)?;
        Ok(true)
    }

    pub fn advance_bot(&mut self, action_limit: usize) -> Result<(), AppError> {
        if self.is_hotseat() {
            return Ok(());
        }
        let mut actions = 0usize;
        while self.game.state().outcome.is_none()
            && self.game.state().input_player() != self.human_player
        {
            actions += 1;
            if actions > action_limit {
                return Err(AppError::BotActionLimit(action_limit));
            }
            let advanced = self.advance_bot_once()?;
            debug_assert!(advanced, "bot loop condition requires one action");
        }
        Ok(())
    }

    pub fn card_name(&self, card_id: &str) -> String {
        self.game
            .runtime()
            .definition(card_id)
            .map(|definition| definition.name.clone())
            .unwrap_or_else(|| card_id.to_owned())
    }

    pub fn card_text(&self, card_id: &str) -> String {
        self.game
            .runtime()
            .definition(card_id)
            .map(|definition| definition.text.clone())
            .unwrap_or_default()
    }

    pub fn turn_time_limit_seconds(&self) -> Result<Option<u64>, AppError> {
        let mut limit = 0;
        for player in [PlayerId::ONE, PlayerId::TWO] {
            for entity in &self.game.state().player(player).board {
                limit = self
                    .game
                    .runtime()
                    .keyword_i32_rule(
                        self.game.state(),
                        *entity,
                        "turn_time_limit_seconds",
                        limit,
                        None,
                    )
                    .map_err(AppError::RuntimeRule)?;
            }
        }
        Ok((limit > 0).then_some(limit as u64))
    }
}

fn deck_sideboards(deck: &DeckList) -> BTreeMap<String, Vec<String>> {
    deck.sideboards
        .iter()
        .map(|sideboard| (sideboard.owner.clone(), sideboard.cards.clone()))
        .collect()
}

fn load_deck(path: &Path) -> Result<DeckList, AppError> {
    let source = fs::read_to_string(path).map_err(|source| AppError::Read {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&source).map_err(|source| AppError::ParseDeck {
        path: path.to_owned(),
        source,
    })
}

#[derive(Deserialize)]
struct CardDbfMetadata {
    id: String,
    #[serde(rename = "dbfId")]
    dbf_id: u32,
}

type CardDbfMaps = (BTreeMap<String, u32>, BTreeMap<u32, String>);

fn load_card_dbf_ids(data_dir: &Path) -> Result<CardDbfMaps, AppError> {
    let path = data_dir.join("hearthstonejson/selected.enUS.json");
    let json = match fs::read_to_string(&path) {
        Ok(json) => json,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok((BTreeMap::new(), BTreeMap::new()));
        }
        Err(source) => return Err(AppError::Read { path, source }),
    };
    let records = serde_json::from_str::<Vec<CardDbfMetadata>>(&json).map_err(|source| {
        AppError::ParseCardMetadata {
            path: path.clone(),
            source,
        }
    })?;
    let mut by_card_id = BTreeMap::new();
    let mut by_dbf_id = BTreeMap::new();
    for record in records {
        if let Some(previous) = by_card_id.insert(record.id.clone(), record.dbf_id)
            && previous != record.dbf_id
        {
            return Err(AppError::InvalidCardMetadata {
                path: path.clone(),
                message: format!(
                    "{} has conflicting dbfIds {previous} and {}",
                    record.id, record.dbf_id
                ),
            });
        }
        if let Some(previous) = by_dbf_id.insert(record.dbf_id, record.id.clone())
            && previous != record.id
        {
            return Err(AppError::InvalidCardMetadata {
                path: path.clone(),
                message: format!(
                    "dbfId {} belongs to both {previous} and {}",
                    record.dbf_id, record.id
                ),
            });
        }
    }
    Ok((by_card_id, by_dbf_id))
}

fn collect_deck_paths(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), AppError> {
    let entries = fs::read_dir(directory).map_err(|source| AppError::Read {
        path: directory.to_owned(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| AppError::Read {
            path: directory.to_owned(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| AppError::Read {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            collect_deck_paths(&path, paths)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == "json") {
            paths.push(path);
        }
    }
    Ok(())
}

fn validate_editable_deck(deck: &DeckList, cards: &[CardCatalogEntry]) -> Result<(), AppError> {
    if deck.name.trim().is_empty() || deck.name.chars().count() > 128 {
        return invalid_deck(deck, "name must contain 1 to 128 characters");
    }
    let mut copies = std::collections::BTreeMap::<&str, usize>::new();
    for card_id in &deck.cards {
        let Some(card) = cards.iter().find(|card| card.id == *card_id) else {
            return invalid_deck(deck, format!("{card_id} is not a collectible card"));
        };
        let count = copies.entry(card_id).or_default();
        *count += 1;
        if !deck.unrestricted {
            let maximum = if card.rarity.as_deref() == Some("legendary") {
                1
            } else {
                2
            };
            if *count > maximum {
                return invalid_deck(
                    deck,
                    format!("{} exceeds its {maximum}-copy limit", card.name),
                );
            }
        }
    }
    let required_size = required_deck_size(deck, cards);
    if deck.cards.len() != required_size {
        return invalid_deck(
            deck,
            format!(
                "constructed decks require exactly {required_size} cards, got {}",
                deck.cards.len()
            ),
        );
    }
    let main_cards = deck
        .cards
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let mut owners = std::collections::BTreeSet::new();
    for sideboard in &deck.sideboards {
        if !owners.insert(sideboard.owner.as_str()) {
            return invalid_deck(
                deck,
                format!("{} has more than one sideboard", sideboard.owner),
            );
        }
        if !main_cards.contains(sideboard.owner.as_str()) {
            return invalid_deck(
                deck,
                format!(
                    "sideboard owner {} is not in the main deck",
                    sideboard.owner
                ),
            );
        }
        let Some(owner) = cards.iter().find(|card| card.id == sideboard.owner) else {
            return invalid_deck(
                deck,
                format!(
                    "sideboard owner {} is not a collectible card",
                    sideboard.owner
                ),
            );
        };
        if owner.sideboard_size == 0 {
            return invalid_deck(deck, format!("{} does not support a sideboard", owner.name));
        }
        if sideboard.cards.len() != usize::from(owner.sideboard_size) {
            return invalid_deck(
                deck,
                format!(
                    "{} requires exactly {} sideboard cards, got {}",
                    owner.name,
                    owner.sideboard_size,
                    sideboard.cards.len()
                ),
            );
        }
        for card_id in &sideboard.cards {
            if card_id == &sideboard.owner {
                return invalid_deck(deck, format!("{} cannot contain itself", owner.name));
            }
            let Some(card) = cards.iter().find(|card| card.id == *card_id) else {
                return invalid_deck(deck, format!("{card_id} is not a collectible card"));
            };
            let count = copies.entry(card_id).or_default();
            *count += 1;
            if !deck.unrestricted {
                let maximum = if card.rarity.as_deref() == Some("legendary") {
                    1
                } else {
                    2
                };
                if *count > maximum {
                    return invalid_deck(
                        deck,
                        format!("{} exceeds its {maximum}-copy limit", card.name),
                    );
                }
            }
        }
    }
    if !deck.unrestricted && deck.class.eq_ignore_ascii_case("death_knight") {
        let runes = deck_rune_cost(deck, cards);
        if !runes.fits_death_knight_deck() {
            return invalid_deck(
                deck,
                format!(
                    "Death Knight rune requirements need {} slots (Blood {}, Frost {}, Unholy {}), but only {} are available",
                    runes.total(),
                    runes.blood,
                    runes.frost,
                    runes.unholy,
                    RuneCost::SLOTS
                ),
            );
        }
    }
    Ok(())
}

fn required_deck_size(deck: &DeckList, cards: &[CardCatalogEntry]) -> usize {
    deck.cards
        .iter()
        .filter_map(|card_id| cards.iter().find(|card| card.id == *card_id))
        .filter_map(|card| card.deck_size)
        .map(usize::from)
        .max()
        .unwrap_or(30)
}

fn deck_rune_cost(deck: &DeckList, cards: &[CardCatalogEntry]) -> RuneCost {
    deck.cards
        .iter()
        .chain(
            deck.sideboards
                .iter()
                .flat_map(|sideboard| sideboard.cards.iter()),
        )
        .filter_map(|card_id| cards.iter().find(|card| card.id == *card_id))
        .fold(RuneCost::default(), |runes, card| {
            runes.combined(card.rune_cost)
        })
}

fn deck_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;
    for character in name.trim().chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            slug.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
        if slug.chars().count() >= 64 {
            break;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    if slug.is_empty() {
        "custom_deck".to_owned()
    } else {
        slug
    }
}

fn validate_deck(runtime: &LuaCardRuntime, deck: &DeckList) -> Result<(), AppError> {
    if deck.class.trim().is_empty() || deck.class.len() > 64 {
        return invalid_deck(deck, "class must contain 1 to 64 bytes");
    }
    for card in &deck.cards {
        let deckable = runtime
            .definition(card)
            .is_some_and(CardDefinition::is_deckable);
        if !deckable {
            return invalid_deck(deck, format!("{card} is not a deckable card"));
        }
    }
    for sideboard in &deck.sideboards {
        for card in &sideboard.cards {
            let deckable = runtime
                .definition(card)
                .is_some_and(CardDefinition::is_deckable);
            if !deckable {
                return invalid_deck(deck, format!("{card} is not a deckable sideboard card"));
            }
        }
    }
    if let Some(hero_power) = deck.hero_power.as_deref() {
        let valid = runtime
            .definition(hero_power)
            .is_some_and(|definition| definition.kind == CardKind::HeroPower);
        if !valid {
            return invalid_deck(deck, format!("{hero_power} is not a Hero Power"));
        }
    }
    Ok(())
}

fn invalid_deck<T>(deck: &DeckList, message: impl Into<String>) -> Result<T, AppError> {
    Err(AppError::InvalidDeck {
        deck: deck.name.clone(),
        message: message.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_constructed_class_has_its_canonical_hero_power() {
        assert_eq!(basic_hero_power_for_class("warrior"), "HERO_01bp");
        assert_eq!(basic_hero_power_for_class("shaman"), "HERO_02bp");
        assert_eq!(basic_hero_power_for_class("rogue"), "HERO_03bp");
        assert_eq!(basic_hero_power_for_class("paladin"), "HERO_04bp");
        assert_eq!(basic_hero_power_for_class("hunter"), "HERO_05bp");
        assert_eq!(basic_hero_power_for_class("druid"), "HERO_06bp");
        assert_eq!(basic_hero_power_for_class("warlock"), "HERO_07bp");
        assert_eq!(basic_hero_power_for_class("mage"), "HERO_08bp");
        assert_eq!(basic_hero_power_for_class("priest"), "HERO_09bp");
        assert_eq!(basic_hero_power_for_class("demon_hunter"), "HERO_10bp");
        assert_eq!(basic_hero_power_for_class("death_knight"), "HERO_11bp");
        assert_eq!(basic_hero_power_for_class("future_class"), "HERO_08bp");
    }

    #[test]
    fn demo_session_reaches_human_mulligan() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let session = GameSession::load(&MatchConfig::demo(root)).unwrap();
        let view = session.view();
        assert_eq!(view.hero(PlayerId::ONE).card_id, "HERO_08");
        assert_eq!(view.hero(PlayerId::TWO).card_id, "HERO_08");
        assert_eq!(view.input_player, PlayerId::ONE);
        assert_eq!(view.mulligan_eligible.len(), 3);
        assert!(session.legal_actions().unwrap().iter().any(|action| {
            matches!(
                action.command,
                PlayerCommand::Mulligan { ref replace } if replace.is_empty()
            )
        }));
    }

    #[test]
    fn keeping_the_hand_advances_the_bot_and_starts_the_match() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut session = GameSession::load(&MatchConfig::demo(root)).unwrap();
        session
            .dispatch_human(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();

        let view = session.view();
        assert!(view.mulligan_eligible.is_empty());
        assert_eq!(view.input_player, PlayerId::ONE);
        assert_eq!(view.active_player, PlayerId::ONE);
        assert_eq!(view.turn, 1);
        assert!(
            session
                .legal_actions()
                .unwrap()
                .iter()
                .any(|action| matches!(action.command, PlayerCommand::EndTurn))
        );
    }

    #[test]
    fn deferred_dispatch_exposes_exactly_one_bot_action_at_a_time() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut session = GameSession::load(&MatchConfig::demo(root)).unwrap();
        assert!(!session.is_bot_turn());
        assert!(!session.advance_bot_once().unwrap());

        session
            .dispatch_human_only(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();
        assert!(session.is_bot_turn());
        let history_before_bot = session.view().history.len();

        assert!(session.advance_bot_once().unwrap());
        assert!(session.view().history.len() > history_before_bot);
        assert!(!session.is_bot_turn());
        assert!(!session.advance_bot_once().unwrap());
        assert_eq!(session.view().turn, 1);
    }

    #[test]
    fn human_can_concede_while_the_bot_owns_input() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut session = GameSession::load(&MatchConfig::demo(root)).unwrap();
        session
            .dispatch_human_only(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();
        assert!(session.is_bot_turn());

        session.concede_human().unwrap();

        assert_eq!(
            session.view().outcome,
            Some(hearth_core::GameOutcome::Winner(PlayerId::TWO))
        );
        assert!(matches!(
            session.snapshot().game.replay.commands.last(),
            Some(PlayerCommand::ConcedePlayer {
                player: PlayerId::ONE
            })
        ));
    }

    #[test]
    fn hotseat_keeps_both_players_interactive_and_switches_the_viewer() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut config = MatchConfig::demo(root);
        config.match_mode = MatchMode::Hotseat;
        let mut session = GameSession::load(&config).unwrap();

        assert!(session.is_hotseat());
        assert_eq!(session.human_player(), PlayerId::ONE);
        session
            .dispatch_human(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();

        assert_eq!(session.human_player(), PlayerId::TWO);
        assert_eq!(session.view().input_player, PlayerId::TWO);
        assert_eq!(session.view().mulligan_eligible.len(), 4);
        let before = session.view().turn;
        session.advance_bot(10_000).unwrap();
        assert_eq!(session.human_player(), PlayerId::TWO);
        assert_eq!(session.view().turn, before);
    }

    #[test]
    fn seeded_opening_order_gives_the_second_player_four_cards_and_the_coin() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut config = MatchConfig::demo(&root);
        config.match_mode = MatchMode::Hotseat;

        assert_eq!(starting_player_for_seed(20260829), PlayerId::ONE);
        assert_eq!(starting_player_for_seed(20260830), PlayerId::TWO);
        config.seed = 20260830;
        let mut session = GameSession::load(&config).unwrap();
        assert_eq!(session.starting_player(), PlayerId::TWO);
        assert_eq!(session.human_player(), PlayerId::TWO);
        assert_eq!(session.view().mulligan_eligible.len(), 3);

        session
            .dispatch_human(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();
        assert_eq!(session.human_player(), PlayerId::ONE);
        assert_eq!(session.view().mulligan_eligible.len(), 4);
        session
            .dispatch_human(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();

        let snapshot = session.snapshot();
        assert_eq!(snapshot.game.replay.starting_player, PlayerId::TWO);
        assert_eq!(snapshot.game.state.starting_player, PlayerId::TWO);
        assert_eq!(snapshot.game.state.active_player, PlayerId::TWO);
        assert_eq!(snapshot.game.state.turn, 1);
        let second_player = snapshot.game.state.player(PlayerId::ONE);
        assert_eq!(second_player.hand.len(), 5);
        assert!(second_player.hand.iter().any(|entity| {
            snapshot.game.state.entity(*entity).unwrap().card_id == hearth_core::DEFAULT_COIN
        }));
    }

    #[test]
    fn session_snapshot_round_trips_and_rejects_unknown_versions() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut config = MatchConfig::demo(&root);
        config.match_mode = MatchMode::Hotseat;
        config.bot_difficulty = BotDifficulty::Hard;
        let mut session = GameSession::load(&config).unwrap();
        session
            .dispatch_human(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();

        let json = serde_json::to_string(&session.snapshot()).unwrap();
        let snapshot = serde_json::from_str::<GameSessionSnapshot>(&json).unwrap();
        let restored =
            GameSession::from_snapshot(&config.data_dir, config.locale, &snapshot).unwrap();
        assert_eq!(restored.view(), session.view());
        assert_eq!(
            restored.legal_actions().unwrap(),
            session.legal_actions().unwrap()
        );
        assert_eq!(restored.match_mode(), MatchMode::Hotseat);
        assert_eq!(restored.bot_difficulty(), BotDifficulty::Hard);
        assert_eq!(
            restored.deck_name(PlayerId::ONE),
            session.deck_name(PlayerId::ONE)
        );

        let mut legacy_value = serde_json::to_value(&snapshot).unwrap();
        legacy_value
            .as_object_mut()
            .unwrap()
            .remove("bot_difficulty");
        legacy_value["game"]["replay"]
            .as_object_mut()
            .unwrap()
            .remove("starting_player");
        legacy_value["game"]["state"]
            .as_object_mut()
            .unwrap()
            .remove("starting_player");
        let legacy = serde_json::from_value::<GameSessionSnapshot>(legacy_value).unwrap();
        let legacy_restored =
            GameSession::from_snapshot(&config.data_dir, config.locale, &legacy).unwrap();
        assert_eq!(legacy_restored.bot_difficulty(), BotDifficulty::Normal);

        let mut unsupported = snapshot;
        unsupported.format_version += 1;
        assert!(matches!(
            GameSession::from_snapshot(&config.data_dir, config.locale, &unsupported),
            Err(AppError::UnsupportedSessionSnapshot(_))
        ));
    }

    #[test]
    fn deck_library_discovers_repository_decks_and_collectible_cards() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        assert!(library.decks().len() >= 300);
        assert!(library.cards().len() >= 1_000);
        assert!(
            library
                .decks()
                .iter()
                .any(|stored| stored.deck.name == "Official Representative Card Demo")
        );
        assert!(
            library
                .cards()
                .iter()
                .any(|card| card.id == "EX1_008" && card.name == "Argent Squire")
        );
        assert_eq!(
            library.definition("HERO_08").map(|hero| hero.name.as_str()),
            Some("Jaina Proudmoore")
        );
        assert!(
            library.cards().iter().all(|card| card.set != "HERO_SKINS"),
            "collectible Hero portraits must not occupy constructed deck slots"
        );
        assert_eq!(
            library
                .definition("FP1_002t")
                .map(|card| card.name.as_str()),
            Some("Spectral Spider")
        );
        assert_eq!(
            library.definition("RLK_067").map(|card| card.rune_cost),
            Some(RuneCost {
                blood: 2,
                frost: 0,
                unholy: 0,
            })
        );
    }

    #[test]
    fn death_knight_runes_cover_main_deck_sideboards_and_candidate_filtering() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let neutrals = library
            .cards()
            .iter()
            .filter(|card| {
                card.class == "neutral"
                    && card.id != "ETC_080"
                    && card.deck_size.is_none()
                    && card.sideboard_size == 0
            })
            .map(|card| card.id.clone())
            .take(30)
            .collect::<Vec<_>>();
        assert_eq!(neutrals.len(), 30);
        let mut cards = vec!["ETC_080".to_owned(), "RLK_067".to_owned()];
        cards.extend(neutrals[..28].iter().cloned());
        let mut deck = DeckList {
            name: "Runes".to_owned(),
            format: Some("wild".to_owned()),
            class: "death_knight".to_owned(),
            cards,
            sideboards: vec![DeckSideboard {
                owner: "ETC_080".to_owned(),
                cards: vec![
                    "RLK_048".to_owned(),
                    neutrals[28].clone(),
                    neutrals[29].clone(),
                ],
            }],
            hero_power: None,
            unrestricted: false,
        };

        let runes = library.deck_rune_cost(&deck);
        assert_eq!(
            runes,
            RuneCost {
                blood: 2,
                frost: 0,
                unholy: 1,
            }
        );
        assert!(validate_editable_deck(&deck, library.cards()).is_ok());
        assert!(!library.card_fits_deck_runes(
            &deck,
            library.definition("RLK_063").expect("Frostwyrm's Fury")
        ));

        deck.sideboards[0].cards[0] = "RLK_063".to_owned();
        let error = validate_editable_deck(&deck, library.cards()).unwrap_err();
        assert!(error.to_string().contains("5 slots"));

        deck.unrestricted = true;
        assert!(validate_editable_deck(&deck, library.cards()).is_ok());
    }

    #[test]
    fn reloading_the_deck_library_localizes_cards_without_losing_deck_paths() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let paths = library
            .decks()
            .iter()
            .map(|stored| stored.path.clone())
            .collect::<Vec<_>>();
        assert_eq!(library.definition("EX1_008").unwrap().name, "Argent Squire");

        library
            .reload_locale(&root.join("data"), Locale::ZhCn)
            .unwrap();

        assert_eq!(library.definition("EX1_008").unwrap().name, "银色侍从");
        assert_eq!(
            library
                .decks()
                .iter()
                .map(|stored| stored.path.clone())
                .collect::<Vec<_>>(),
            paths
        );
        assert!(
            library
                .reload_locale(&root.join("missing-card-data"), Locale::ZhTw)
                .is_err()
        );
        assert_eq!(library.definition("EX1_008").unwrap().name, "银色侍从");
    }

    #[test]
    fn editable_decks_enforce_size_and_copy_limits() {
        let cards = vec![
            CardCatalogEntry {
                id: "common".to_owned(),
                name: "Common".to_owned(),
                text: String::new(),
                set: "test".to_owned(),
                kind: CardKind::Minion,
                collectible: true,
                class: "neutral".to_owned(),
                classes: Vec::new(),
                sideboard_size: 0,
                deck_size: None,
                starting_health: None,
                rune_cost: RuneCost::default(),
                rarity: Some("common".to_owned()),
                cost: 1,
                attack: 1,
                health: 1,
                armor: 0,
                keywords: Vec::new(),
            },
            CardCatalogEntry {
                id: "legendary".to_owned(),
                name: "Legendary".to_owned(),
                text: String::new(),
                set: "test".to_owned(),
                kind: CardKind::Minion,
                collectible: true,
                class: "neutral".to_owned(),
                classes: Vec::new(),
                sideboard_size: 0,
                deck_size: None,
                starting_health: None,
                rune_cost: RuneCost::default(),
                rarity: Some("legendary".to_owned()),
                cost: 1,
                attack: 1,
                health: 1,
                armor: 0,
                keywords: Vec::new(),
            },
        ];
        let mut deck = DeckList {
            name: "Custom".to_owned(),
            format: None,
            class: "mage".to_owned(),
            cards: vec!["common".to_owned(); 30],
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };
        assert!(validate_editable_deck(&deck, &cards).is_err());
        deck.unrestricted = true;
        assert!(validate_editable_deck(&deck, &cards).is_ok());
        deck.unrestricted = false;
        deck.cards = vec!["common".to_owned(); 29];
        deck.cards.push("legendary".to_owned());
        assert!(validate_editable_deck(&deck, &cards).is_err());
    }

    #[test]
    fn custom_deck_file_names_cannot_escape_the_custom_directory() {
        assert_eq!(deck_slug("../../My Deck!?"), "my_deck");
        assert_eq!(deck_slug("法师套牌"), "法师套牌");
        assert_eq!(deck_slug("///"), "custom_deck");
    }

    #[test]
    fn custom_deck_mutations_are_confined_and_rename_safely() {
        struct TempDeckRoot(PathBuf);

        impl Drop for TempDeckRoot {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos();
        let root = TempDeckRoot(
            std::env::temp_dir().join(format!("hearth-app-delete-{}-{nonce}", std::process::id())),
        );
        let custom_dir = root.0.join("custom");
        fs::create_dir_all(&custom_dir).expect("temporary custom directory should be created");
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let source = workspace.join("decks/demo.json");
        let protected_path = root.0.join("protected.json");
        let custom_path = custom_dir.join("delete_me.json");
        fs::copy(&source, &protected_path).expect("protected fixture should be copied");
        fs::copy(&source, &custom_path).expect("custom fixture should be copied");

        let mut library = DeckLibrary::load(&root.0, workspace.join("data"), Locale::EnUs)
            .expect("temporary deck library should load");
        let protected = library
            .index_of_path(&protected_path)
            .expect("protected fixture should be indexed");
        assert!(!library.is_custom(protected));
        assert!(matches!(
            library.delete_custom(protected),
            Err(AppError::ProtectedDeck(path)) if path == protected_path
        ));
        let protected_deck = library
            .deck(protected)
            .expect("protected fixture remains loaded")
            .deck
            .clone();
        assert!(matches!(
            library.replace_custom(&protected_path, &protected_deck),
            Err(AppError::ProtectedDeck(path)) if path == protected_path
        ));
        assert!(protected_path.exists());

        let custom = library
            .index_of_path(&custom_path)
            .expect("custom fixture should be indexed");
        assert!(library.is_custom(custom));
        let original_count = library.decks().len();
        let mut renamed = library
            .deck(custom)
            .expect("custom fixture remains loaded")
            .deck
            .clone();
        renamed.name = "Renamed Custom Fixture".to_owned();
        let renamed_path = custom_dir.join("renamed_custom_fixture.json");
        let saved_path = library
            .replace_custom(&custom_path, &renamed)
            .expect("custom fixture should be renamed");
        assert_eq!(saved_path, renamed_path);
        assert!(!custom_path.exists());
        assert!(renamed_path.exists());
        assert_eq!(library.decks().len(), original_count);
        let renamed_index = library
            .index_of_path(&renamed_path)
            .expect("renamed fixture should be indexed");
        assert_eq!(library.deck(renamed_index).unwrap().deck.name, renamed.name);
        assert!(matches!(
            library.save_custom(&renamed),
            Err(AppError::DeckNameConflict(path)) if path == renamed_path
        ));

        let custom = library
            .index_of_path(&renamed_path)
            .expect("renamed fixture should still be indexed");
        let deleted = library
            .delete_custom(custom)
            .expect("custom fixture should be deleted");
        assert_eq!(deleted.path, renamed_path);
        assert!(!deleted.path.exists());
        assert!(library.index_of_path(&deleted.path).is_none());
        assert!(matches!(
            library.delete_custom(usize::MAX),
            Err(AppError::UnknownDeckIndex(index)) if index == usize::MAX
        ));
    }
}
