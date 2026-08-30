use std::path::PathBuf;

use hearth_core::{CardKind, RuneCost};
use serde::{Deserialize, Serialize};

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

fn default_deck_class() -> String {
    "mage".to_owned()
}
