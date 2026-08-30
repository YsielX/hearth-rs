use std::collections::BTreeMap;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use thiserror::Error;

use super::{DeckLibrary, DeckList, DeckSideboard, validate_editable_deck};

const DECKSTRING_VERSION: u64 = 1;
const FORMAT_WILD: u64 = 1;
const FORMAT_STANDARD: u64 = 2;
const FORMAT_CLASSIC: u64 = 3;
const FORMAT_TWIST: u64 = 4;
const MAX_DECKSTRING_ENTRIES: usize = 256;
const MAX_CONSTRUCTED_DECK_SIZE: usize = 60;

#[derive(Debug, Error)]
pub enum DeckstringError {
    #[error("no deck code was found")]
    MissingCode,
    #[error("invalid base64 deck code: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("the deck code ended unexpectedly")]
    UnexpectedEof,
    #[error("invalid deck code header")]
    InvalidHeader,
    #[error("unsupported deck code version {0}")]
    UnsupportedVersion(u64),
    #[error("unsupported deck format {0}")]
    UnsupportedFormat(u64),
    #[error("expected exactly one deck hero, found {0}")]
    InvalidHeroCount(u64),
    #[error("unsupported deck hero dbfId {0}")]
    UnsupportedHero(u64),
    #[error("card {0} has no Hearthstone dbfId metadata")]
    MissingCardDbfId(String),
    #[error("deck code references unknown card dbfId {0}")]
    UnknownCardDbfId(u64),
    #[error("deck code references unsupported card {card_id} (dbfId {dbf_id})")]
    UnsupportedCard { dbf_id: u64, card_id: String },
    #[error("deck code contains invalid count {count} for dbfId {dbf_id}")]
    InvalidCardCount { dbf_id: u64, count: u64 },
    #[error("deck code contains too many card entries")]
    TooManyEntries,
    #[error("invalid deck code footer {0}")]
    InvalidFooter(u8),
    #[error("unexpected trailing data in deck code")]
    TrailingData,
    #[error("invalid imported deck: {0}")]
    InvalidDeck(String),
}

#[derive(Debug, PartialEq, Eq)]
struct RawDeckstring {
    format: u64,
    hero: u64,
    cards: Vec<(u64, u64)>,
    /// `(card dbfId, count, owner dbfId)` entries.
    sideboards: Vec<(u64, u64, u64)>,
}

pub fn export_deckstring(
    library: &DeckLibrary,
    deck: &DeckList,
) -> Result<String, DeckstringError> {
    validate_editable_deck(deck, library.cards())
        .map_err(|error| DeckstringError::InvalidDeck(error.to_string()))?;
    let hero = class_hero_dbf_id(&deck.class)
        .ok_or_else(|| DeckstringError::InvalidDeck(format!("unknown class {}", deck.class)))?;
    let mut counts = BTreeMap::<&str, u64>::new();
    for card_id in &deck.cards {
        *counts.entry(card_id).or_default() += 1;
    }
    let mut cards = counts
        .into_iter()
        .map(|(card_id, count)| {
            let dbf_id = library
                .dbf_id(card_id)
                .ok_or_else(|| DeckstringError::MissingCardDbfId(card_id.to_owned()))?;
            Ok((u64::from(dbf_id), count))
        })
        .collect::<Result<Vec<_>, DeckstringError>>()?;
    cards.sort_unstable_by_key(|(dbf_id, _)| *dbf_id);
    let mut sideboards = Vec::new();
    for sideboard in &deck.sideboards {
        let owner = library
            .dbf_id(&sideboard.owner)
            .ok_or_else(|| DeckstringError::MissingCardDbfId(sideboard.owner.clone()))?;
        let mut counts = BTreeMap::<&str, u64>::new();
        for card_id in &sideboard.cards {
            *counts.entry(card_id).or_default() += 1;
        }
        for (card_id, count) in counts {
            let dbf_id = library
                .dbf_id(card_id)
                .ok_or_else(|| DeckstringError::MissingCardDbfId(card_id.to_owned()))?;
            sideboards.push((u64::from(dbf_id), count, u64::from(owner)));
        }
    }
    sideboards.sort_unstable_by_key(|(dbf_id, _, owner)| (*owner, *dbf_id));
    encode_raw(&RawDeckstring {
        format: deck_format_code(deck.format.as_deref()),
        hero,
        cards,
        sideboards,
    })
}

pub fn import_deckstring(
    library: &DeckLibrary,
    input: &str,
    default_name: &str,
) -> Result<DeckList, DeckstringError> {
    let (name, code) = extract_name_and_code(input, default_name)?;
    let raw = decode_raw(code)?;
    let class = hero_class(raw.hero)
        .ok_or(DeckstringError::UnsupportedHero(raw.hero))?
        .to_owned();
    let mut cards = Vec::new();
    for (dbf_id, count) in raw.cards {
        if count == 0 || count > MAX_CONSTRUCTED_DECK_SIZE as u64 {
            return Err(DeckstringError::InvalidCardCount { dbf_id, count });
        }
        let dbf_id_u32 =
            u32::try_from(dbf_id).map_err(|_| DeckstringError::UnknownCardDbfId(dbf_id))?;
        let card_id = library
            .card_id_for_dbf(dbf_id_u32)
            .ok_or(DeckstringError::UnknownCardDbfId(dbf_id))?;
        if !library.cards().iter().any(|card| card.id == card_id) {
            return Err(DeckstringError::UnsupportedCard {
                dbf_id,
                card_id: card_id.to_owned(),
            });
        }
        let count = usize::try_from(count)
            .map_err(|_| DeckstringError::InvalidCardCount { dbf_id, count })?;
        if cards.len().saturating_add(count) > MAX_CONSTRUCTED_DECK_SIZE {
            return Err(DeckstringError::InvalidDeck(format!(
                "the deck contains more than {MAX_CONSTRUCTED_DECK_SIZE} cards"
            )));
        }
        cards.extend(std::iter::repeat_n(card_id.to_owned(), count));
    }
    let mut sideboards_by_owner = BTreeMap::<String, Vec<String>>::new();
    for (dbf_id, count, owner_dbf_id) in raw.sideboards {
        if count == 0 || count > 30 {
            return Err(DeckstringError::InvalidCardCount { dbf_id, count });
        }
        let card_id = card_id_for_dbf(library, dbf_id)?;
        let owner = card_id_for_dbf(library, owner_dbf_id)?;
        let count = usize::try_from(count)
            .map_err(|_| DeckstringError::InvalidCardCount { dbf_id, count })?;
        sideboards_by_owner
            .entry(owner.to_owned())
            .or_default()
            .extend(std::iter::repeat_n(card_id.to_owned(), count));
    }
    let deck = DeckList {
        name,
        format: Some(deck_format_name(raw.format)?.to_owned()),
        class,
        cards,
        sideboards: sideboards_by_owner
            .into_iter()
            .map(|(owner, cards)| DeckSideboard { owner, cards })
            .collect(),
        hero_power: None,
        unrestricted: false,
    };
    validate_editable_deck(&deck, library.cards())
        .map_err(|error| DeckstringError::InvalidDeck(error.to_string()))?;
    Ok(deck)
}

fn card_id_for_dbf(library: &DeckLibrary, dbf_id: u64) -> Result<&str, DeckstringError> {
    let dbf_id_u32 =
        u32::try_from(dbf_id).map_err(|_| DeckstringError::UnknownCardDbfId(dbf_id))?;
    let card_id = library
        .card_id_for_dbf(dbf_id_u32)
        .ok_or(DeckstringError::UnknownCardDbfId(dbf_id))?;
    if !library.cards().iter().any(|card| card.id == card_id) {
        return Err(DeckstringError::UnsupportedCard {
            dbf_id,
            card_id: card_id.to_owned(),
        });
    }
    Ok(card_id)
}

fn extract_name_and_code<'a>(
    input: &'a str,
    default_name: &str,
) -> Result<(String, &'a str), DeckstringError> {
    let mut name = None;
    let mut code = None;
    for line in input.lines() {
        let line = line.trim();
        if let Some(candidate) = line.strip_prefix("### ")
            && !candidate.trim().is_empty()
            && name.is_none()
        {
            name = Some(candidate.trim().to_owned());
        } else if !line.is_empty() && !line.starts_with('#') {
            code = Some(line);
        }
    }
    if input.lines().count() == 0 && !input.trim().is_empty() {
        code = Some(input.trim());
    }
    let code = code.ok_or(DeckstringError::MissingCode)?;
    Ok((name.unwrap_or_else(|| default_name.to_owned()), code))
}

fn encode_raw(deck: &RawDeckstring) -> Result<String, DeckstringError> {
    deck_format_name(deck.format)?;
    if let Some((dbf_id, count)) = deck.cards.iter().find(|(_, count)| *count == 0) {
        return Err(DeckstringError::InvalidCardCount {
            dbf_id: *dbf_id,
            count: *count,
        });
    }
    let mut bytes = vec![0];
    write_varint(&mut bytes, DECKSTRING_VERSION);
    write_varint(&mut bytes, deck.format);
    write_varint(&mut bytes, 1);
    write_varint(&mut bytes, deck.hero);

    for count in [1, 2] {
        let group = deck
            .cards
            .iter()
            .filter(|(_, copies)| *copies == count)
            .collect::<Vec<_>>();
        write_varint(&mut bytes, group.len() as u64);
        for (dbf_id, _) in group {
            write_varint(&mut bytes, *dbf_id);
        }
    }
    let multiple = deck
        .cards
        .iter()
        .filter(|(_, copies)| *copies > 2)
        .collect::<Vec<_>>();
    write_varint(&mut bytes, multiple.len() as u64);
    for (dbf_id, copies) in multiple {
        write_varint(&mut bytes, *dbf_id);
        write_varint(&mut bytes, *copies);
    }
    if deck.sideboards.is_empty() {
        bytes.push(0);
    } else {
        bytes.push(1);
        for count in [1, 2] {
            let group = deck
                .sideboards
                .iter()
                .filter(|(_, copies, _)| *copies == count)
                .collect::<Vec<_>>();
            write_varint(&mut bytes, group.len() as u64);
            for (dbf_id, _, owner) in group {
                write_varint(&mut bytes, *dbf_id);
                write_varint(&mut bytes, *owner);
            }
        }
        let multiple = deck
            .sideboards
            .iter()
            .filter(|(_, copies, _)| *copies > 2)
            .collect::<Vec<_>>();
        write_varint(&mut bytes, multiple.len() as u64);
        for (dbf_id, copies, owner) in multiple {
            write_varint(&mut bytes, *dbf_id);
            write_varint(&mut bytes, *copies);
            write_varint(&mut bytes, *owner);
        }
    }
    Ok(STANDARD.encode(bytes))
}

fn decode_raw(code: &str) -> Result<RawDeckstring, DeckstringError> {
    let bytes = STANDARD.decode(code.trim())?;
    let mut cursor = 0;
    if take_byte(&bytes, &mut cursor)? != 0 {
        return Err(DeckstringError::InvalidHeader);
    }
    let version = read_varint(&bytes, &mut cursor)?;
    if version != DECKSTRING_VERSION {
        return Err(DeckstringError::UnsupportedVersion(version));
    }
    let format = read_varint(&bytes, &mut cursor)?;
    deck_format_name(format)?;
    let hero_count = read_varint(&bytes, &mut cursor)?;
    if hero_count != 1 {
        return Err(DeckstringError::InvalidHeroCount(hero_count));
    }
    let hero = read_varint(&bytes, &mut cursor)?;
    let mut cards = Vec::new();
    for copies in [1, 2] {
        read_card_group(&bytes, &mut cursor, copies, &mut cards)?;
    }
    let multiple_count = bounded_entry_count(read_varint(&bytes, &mut cursor)?)?;
    for _ in 0..multiple_count {
        let dbf_id = read_varint(&bytes, &mut cursor)?;
        let copies = read_varint(&bytes, &mut cursor)?;
        if copies <= 2 {
            return Err(DeckstringError::InvalidCardCount {
                dbf_id,
                count: copies,
            });
        }
        cards.push((dbf_id, copies));
    }
    let mut sideboards = Vec::new();
    if cursor < bytes.len() {
        match take_byte(&bytes, &mut cursor)? {
            0 => {}
            1 => {
                for copies in [1, 2] {
                    let entries = bounded_entry_count(read_varint(&bytes, &mut cursor)?)?;
                    for _ in 0..entries {
                        let dbf_id = read_varint(&bytes, &mut cursor)?;
                        let owner = read_varint(&bytes, &mut cursor)?;
                        sideboards.push((dbf_id, copies, owner));
                    }
                }
                let entries = bounded_entry_count(read_varint(&bytes, &mut cursor)?)?;
                for _ in 0..entries {
                    let dbf_id = read_varint(&bytes, &mut cursor)?;
                    let copies = read_varint(&bytes, &mut cursor)?;
                    if copies <= 2 {
                        return Err(DeckstringError::InvalidCardCount {
                            dbf_id,
                            count: copies,
                        });
                    }
                    let owner = read_varint(&bytes, &mut cursor)?;
                    sideboards.push((dbf_id, copies, owner));
                }
            }
            footer => return Err(DeckstringError::InvalidFooter(footer)),
        }
    }
    if cursor != bytes.len() {
        return Err(DeckstringError::TrailingData);
    }
    cards.sort_unstable_by_key(|(dbf_id, _)| *dbf_id);
    sideboards.sort_unstable_by_key(|(dbf_id, _, owner)| (*owner, *dbf_id));
    Ok(RawDeckstring {
        format,
        hero,
        cards,
        sideboards,
    })
}

fn read_card_group(
    bytes: &[u8],
    cursor: &mut usize,
    copies: u64,
    cards: &mut Vec<(u64, u64)>,
) -> Result<(), DeckstringError> {
    let entries = bounded_entry_count(read_varint(bytes, cursor)?)?;
    for _ in 0..entries {
        cards.push((read_varint(bytes, cursor)?, copies));
    }
    Ok(())
}

fn bounded_entry_count(count: u64) -> Result<usize, DeckstringError> {
    let count = usize::try_from(count).map_err(|_| DeckstringError::TooManyEntries)?;
    if count > MAX_DECKSTRING_ENTRIES {
        return Err(DeckstringError::TooManyEntries);
    }
    Ok(count)
}

fn write_varint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(bytes: &[u8], cursor: &mut usize) -> Result<u64, DeckstringError> {
    let mut value = 0u64;
    for shift in (0..=63).step_by(7) {
        let byte = take_byte(bytes, cursor)?;
        if shift == 63 && byte & 0x7e != 0 {
            return Err(DeckstringError::InvalidHeader);
        }
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(DeckstringError::InvalidHeader)
}

fn take_byte(bytes: &[u8], cursor: &mut usize) -> Result<u8, DeckstringError> {
    let byte = bytes
        .get(*cursor)
        .copied()
        .ok_or(DeckstringError::UnexpectedEof)?;
    *cursor += 1;
    Ok(byte)
}

fn deck_format_code(format: Option<&str>) -> u64 {
    let format = format.unwrap_or_default().to_ascii_lowercase();
    if format.contains("standard") {
        FORMAT_STANDARD
    } else if format.contains("classic") {
        FORMAT_CLASSIC
    } else if format.contains("twist") {
        FORMAT_TWIST
    } else {
        FORMAT_WILD
    }
}

fn deck_format_name(format: u64) -> Result<&'static str, DeckstringError> {
    match format {
        FORMAT_WILD => Ok("wild"),
        FORMAT_STANDARD => Ok("standard"),
        FORMAT_CLASSIC => Ok("classic"),
        FORMAT_TWIST => Ok("twist"),
        _ => Err(DeckstringError::UnsupportedFormat(format)),
    }
}

fn class_hero_dbf_id(class: &str) -> Option<u64> {
    Some(match class {
        "warrior" => 7,
        "shaman" => 1066,
        "rogue" => 930,
        "paladin" => 671,
        "hunter" => 31,
        "druid" => 274,
        "warlock" => 893,
        "mage" => 637,
        "priest" => 813,
        "demon_hunter" => 56550,
        "death_knight" => 78065,
        _ => return None,
    })
}

fn hero_class(hero: u64) -> Option<&'static str> {
    Some(match hero {
        7 => "warrior",
        1066 => "shaman",
        930 => "rogue",
        671 => "paladin",
        31 => "hunter",
        274 => "druid",
        893 => "warlock",
        637 => "mage",
        813 => "priest",
        56550 => "demon_hunter",
        78065 => "death_knight",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use hearth_core::Locale;

    use super::*;

    #[test]
    fn canonical_fixture_matches_the_documented_deckstring_layout() {
        let code = encode_raw(&RawDeckstring {
            format: FORMAT_WILD,
            hero: 7,
            cards: vec![(1, 2), (2, 2), (3, 2), (4, 1)],
            sideboards: Vec::new(),
        })
        .unwrap();
        assert_eq!(code, "AAEBAQcBBAMBAgMAAA==");
        assert_eq!(
            decode_raw(&code).unwrap(),
            RawDeckstring {
                format: FORMAT_WILD,
                hero: 7,
                cards: vec![(1, 2), (2, 2), (3, 2), (4, 1)],
                sideboards: Vec::new(),
            }
        );
    }

    #[test]
    fn canonical_sideboard_fixture_matches_hearthsim() {
        let deck = RawDeckstring {
            format: FORMAT_WILD,
            hero: 7,
            cards: vec![(1, 2), (2, 2), (3, 2), (4, 1)],
            sideboards: vec![(5, 1, 90_749)],
        };
        let code = encode_raw(&deck).unwrap();
        assert_eq!(code, "AAEBAQcBBAMBAgMAAQEF/cQFAAA=");
        assert_eq!(decode_raw(&code).unwrap(), deck);
    }

    #[test]
    fn repository_deck_round_trips_through_an_official_deck_code() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let stored = library
            .decks()
            .iter()
            .find(|stored| stored.path.ends_with("quest_rogue.json"))
            .expect("Quest Rogue fixture should be loaded");
        let code = export_deckstring(&library, &stored.deck).unwrap();
        let imported = import_deckstring(
            &library,
            &format!("### Imported Quest Rogue\n# Format: Standard\n{code}"),
            "Fallback",
        )
        .unwrap();
        let mut expected_cards = stored.deck.cards.clone();
        let mut imported_cards = imported.cards.clone();
        expected_cards.sort();
        imported_cards.sort();
        assert_eq!(imported.name, "Imported Quest Rogue");
        assert_eq!(imported.class, "rogue");
        assert_eq!(imported.format.as_deref(), Some("standard"));
        assert_eq!(imported_cards, expected_cards);
    }

    #[test]
    fn etc_sideboard_round_trips_through_the_real_catalog() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let mut deck = library
            .decks()
            .iter()
            .find(|stored| stored.path.ends_with("quest_rogue.json"))
            .unwrap()
            .deck
            .clone();
        deck.cards[0] = "ETC_080".to_owned();
        let members = library
            .cards()
            .iter()
            .filter(|card| {
                card.class == "neutral" && card.id != "ETC_080" && !deck.cards.contains(&card.id)
            })
            .take(3)
            .map(|card| card.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(members.len(), 3);
        deck.sideboards = vec![DeckSideboard {
            owner: "ETC_080".to_owned(),
            cards: members.clone(),
        }];

        let code = export_deckstring(&library, &deck).unwrap();
        let imported = import_deckstring(&library, &code, "E.T.C. Round Trip").unwrap();
        let mut imported_members = imported.sideboards[0].cards.clone();
        let mut expected_members = members;
        imported_members.sort();
        expected_members.sort();
        assert_eq!(imported.sideboards[0].owner, "ETC_080");
        assert_eq!(imported_members, expected_members);
    }

    #[test]
    fn forty_card_renathal_deck_round_trips_through_deckstring() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let mut deck = library
            .decks()
            .iter()
            .find(|stored| stored.path.ends_with("quest_rogue.json"))
            .unwrap()
            .deck
            .clone();
        deck.cards[0] = "REV_018".to_owned();
        let extras = library
            .cards()
            .iter()
            .filter(|card| {
                card.class == "neutral" && card.id != "REV_018" && !deck.cards.contains(&card.id)
            })
            .take(10)
            .map(|card| card.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(extras.len(), 10);
        deck.cards.extend(extras);
        assert_eq!(library.required_deck_size(&deck), 40);

        let code = export_deckstring(&library, &deck).unwrap();
        let imported = import_deckstring(&library, &code, "Renathal Round Trip").unwrap();
        let mut expected = deck.cards;
        let mut actual = imported.cards;
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn death_knight_rune_deck_round_trips_and_conflicts_are_rejected() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let mut cards = vec!["RLK_067".to_owned(), "RLK_048".to_owned()];
        cards.extend(
            library
                .cards()
                .iter()
                .filter(|card| card.class == "neutral" && card.deck_size.is_none())
                .take(28)
                .map(|card| card.id.clone()),
        );
        assert_eq!(cards.len(), 30);
        let mut deck = DeckList {
            name: "Blood Unholy".to_owned(),
            format: Some("wild".to_owned()),
            class: "death_knight".to_owned(),
            cards,
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };

        let code = export_deckstring(&library, &deck).unwrap();
        let imported = import_deckstring(&library, &code, "Imported Runes").unwrap();
        assert_eq!(imported.class, "death_knight");
        assert_eq!(
            library.deck_rune_cost(&imported),
            hearth_core::RuneCost {
                blood: 2,
                frost: 0,
                unholy: 1,
            }
        );

        deck.cards[1] = "RLK_063".to_owned();
        assert!(matches!(
            export_deckstring(&library, &deck),
            Err(DeckstringError::InvalidDeck(message)) if message.contains("5 slots")
        ));
    }

    #[test]
    fn export_requires_a_complete_constructed_deck() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let mut deck = library
            .decks()
            .iter()
            .find(|stored| stored.path.ends_with("quest_rogue.json"))
            .unwrap()
            .deck
            .clone();
        deck.cards.pop();

        assert!(matches!(
            export_deckstring(&library, &deck),
            Err(DeckstringError::InvalidDeck(message)) if message.contains("29")
        ));
    }

    #[test]
    fn import_rejects_unknown_card_metadata_before_creating_a_deck() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
        let code = encode_raw(&RawDeckstring {
            format: FORMAT_STANDARD,
            hero: 930,
            cards: vec![(u64::from(u32::MAX), 30)],
            sideboards: Vec::new(),
        })
        .unwrap();

        assert!(matches!(
            import_deckstring(&library, &code, "Unknown"),
            Err(DeckstringError::UnknownCardDbfId(id)) if id == u64::from(u32::MAX)
        ));
    }
}
