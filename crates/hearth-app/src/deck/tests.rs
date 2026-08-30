use std::fs;
use std::path::{Path, PathBuf};

use hearth_core::{CardKind, Locale, RuneCost};

use super::*;
use crate::AppError;

#[test]
fn deck_library_discovers_repository_decks_and_collectible_cards() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let library = DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
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
    let library = DeckLibrary::load(root.join("decks"), root.join("data"), Locale::EnUs).unwrap();
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
