mod deckstring;
mod library;
mod model;
mod validation;

pub use deckstring::{DeckstringError, export_deckstring, import_deckstring};
pub use library::DeckLibrary;
pub use model::{CardCatalogEntry, DeckList, DeckSideboard, StoredDeck};

pub(crate) use library::load_deck;
pub(crate) use validation::validate_deck;
pub(super) use validation::validate_editable_deck;

#[cfg(test)]
pub(crate) use library::deck_slug;

#[cfg(test)]
mod tests;
