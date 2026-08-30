use std::collections::{BTreeMap, BTreeSet};

use hearth_core::{CardDefinition, CardKind, CardRuntime, RuneCost};
use hearth_script::LuaCardRuntime;

use crate::AppError;

use super::model::{CardCatalogEntry, DeckList};

pub(crate) fn validate_editable_deck(
    deck: &DeckList,
    cards: &[CardCatalogEntry],
) -> Result<(), AppError> {
    if deck.name.trim().is_empty() || deck.name.chars().count() > 128 {
        return invalid_deck(deck, "name must contain 1 to 128 characters");
    }
    let mut copies = BTreeMap::<&str, usize>::new();
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
        .collect::<BTreeSet<_>>();
    let mut owners = BTreeSet::new();
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

pub(super) fn required_deck_size(deck: &DeckList, cards: &[CardCatalogEntry]) -> usize {
    deck.cards
        .iter()
        .filter_map(|card_id| cards.iter().find(|card| card.id == *card_id))
        .filter_map(|card| card.deck_size)
        .map(usize::from)
        .max()
        .unwrap_or(30)
}

pub(super) fn deck_rune_cost(deck: &DeckList, cards: &[CardCatalogEntry]) -> RuneCost {
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

pub(crate) fn validate_deck(runtime: &LuaCardRuntime, deck: &DeckList) -> Result<(), AppError> {
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
