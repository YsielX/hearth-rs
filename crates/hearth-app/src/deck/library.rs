use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use hearth_core::{CardDefinition, CardKind, Locale, RuneCost};
use hearth_script::LuaCardRuntime;
use serde::Deserialize;

use crate::AppError;

use super::model::{CardCatalogEntry, DeckList, StoredDeck};
use super::validation::{
    deck_rune_cost, required_deck_size, validate_deck, validate_editable_deck,
};

#[derive(Clone, Debug)]
pub struct DeckLibrary {
    deck_root: PathBuf,
    decks: Vec<StoredDeck>,
    cards: Vec<CardCatalogEntry>,
    definitions: Vec<CardCatalogEntry>,
    dbf_ids: BTreeMap<String, u32>,
    card_ids_by_dbf: BTreeMap<u32, String>,
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

pub(crate) fn load_deck(path: &Path) -> Result<DeckList, AppError> {
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

pub(crate) fn deck_slug(name: &str) -> String {
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
