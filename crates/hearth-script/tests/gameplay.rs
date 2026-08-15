use std::path::PathBuf;

use hearth_core::{
    CardKind, CardRuntime, ChoiceOptionValueView, ChoiceValue, DEFAULT_HERO_POWER, Game, GameError,
    GameEvent, Locale, PlayerCommand, PlayerId, PublicEvent, Zone,
};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn repeated(card: &str) -> Vec<String> {
    std::iter::repeat_n(card.to_owned(), 20).collect()
}

fn mixed(cards: &[&str]) -> Vec<String> {
    cards
        .iter()
        .cycle()
        .take(20)
        .map(|card| (*card).to_owned())
        .collect()
}

fn game(deck_one: &str, deck_two: &str) -> Game<LuaCardRuntime> {
    game_with_decks(repeated(deck_one), repeated(deck_two))
}

fn game_with_decks(deck_one: Vec<String>, deck_two: Vec<String>) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        deck_one,
        deck_two,
        7,
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn game_with_locale(card: &str, locale: Locale) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted(
        LuaCardRuntime::load_dir_with_locale(data_path(), locale).unwrap(),
        repeated(card),
        repeated("CS2_120"),
        7,
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn game_with_classes(
    deck_one: Vec<String>,
    deck_two: Vec<String>,
    classes: [&str; 2],
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        deck_one,
        deck_two,
        7,
        [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()],
        classes.map(str::to_owned),
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn game_with_hero_powers(
    deck_one: Vec<String>,
    deck_two: Vec<String>,
    hero_powers: [&str; 2],
    classes: [&str; 2],
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        deck_one,
        deck_two,
        7,
        hero_powers.map(str::to_owned),
        classes.map(str::to_owned),
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

#[test]
fn player_view_hides_opponent_hand_deck_order_and_secret_identity() {
    let mut game = game_with_decks(repeated("CS2_120"), repeated("CFM_800"));
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    let secret_play = game
        .legal_actions()
        .unwrap()
        .into_iter()
        .find(|command| {
            matches!(
                command,
                PlayerCommand::PlayCard { card, .. }
                    if game.state().entity(*card).unwrap().card_id == "CFM_800"
            )
        })
        .unwrap();
    game.dispatch(secret_play).unwrap();

    let player_one = game.state().player_view(PlayerId::ONE);
    let opponent = player_one.player(PlayerId::TWO);
    assert_eq!(
        opponent.hand_size,
        game.state().player(PlayerId::TWO).hand.len()
    );
    assert!(opponent.hand.is_empty());
    assert_eq!(opponent.secrets_count, 1);
    assert!(opponent.secrets.is_empty());
    assert!(
        player_one
            .history
            .windows(2)
            .all(|events| events[0].sequence < events[1].sequence)
    );
    assert!(player_one.history.iter().any(|record| matches!(
        &record.event,
        PublicEvent::SecretPlayed {
            player: PlayerId::TWO,
            secret: None,
        }
    )));
    assert!(
        !serde_json::to_string(player_one.history.as_slice())
            .unwrap()
            .contains("CFM_800")
    );
    for hidden in game
        .state()
        .player(PlayerId::TWO)
        .hand
        .iter()
        .chain(game.state().player(PlayerId::TWO).deck.iter())
        .chain(game.state().player(PlayerId::TWO).secrets.iter())
    {
        assert!(!player_one.entities.contains_key(hidden));
    }

    let player_two = game.state().player_view(PlayerId::TWO);
    assert_eq!(
        player_two.player(PlayerId::TWO).hand,
        game.state().player(PlayerId::TWO).hand
    );
    assert_eq!(
        player_two.player(PlayerId::TWO).secrets,
        game.state().player(PlayerId::TWO).secrets
    );
    assert!(player_two.history.iter().any(|record| matches!(
        &record.event,
        PublicEvent::SecretPlayed {
            player: PlayerId::TWO,
            secret: Some(secret),
        } if secret.card_id == "CFM_800"
    )));
    assert!(
        game.state()
            .player(PlayerId::TWO)
            .deck
            .iter()
            .all(|entity| !player_two.entities.contains_key(entity))
    );

    let mut quest_game = game_with_decks(repeated("CS2_120"), repeated("UNG_067"));
    quest_game.dispatch(PlayerCommand::EndTurn).unwrap();
    let quest_play = quest_game
        .legal_actions()
        .unwrap()
        .into_iter()
        .find(|command| matches!(command, PlayerCommand::PlayCard { .. }))
        .unwrap();
    quest_game.dispatch(quest_play).unwrap();
    let quest = quest_game.state().player(PlayerId::TWO).secrets[0];
    let opponent_view = quest_game.state().player_view(PlayerId::ONE);
    assert_eq!(opponent_view.player(PlayerId::TWO).secrets_count, 0);
    assert_eq!(
        opponent_view.player(PlayerId::TWO).public_objectives,
        vec![quest]
    );
    assert_eq!(opponent_view.entity(quest).unwrap().card_id, "UNG_067");
}

fn end_turn(game: &mut Game<LuaCardRuntime>) {
    game.dispatch(PlayerCommand::EndTurn).unwrap();
}

fn advance_to_mana(game: &mut Game<LuaCardRuntime>, player: PlayerId, mana: u8) {
    while game.state().active_player != player || game.state().player(player).max_mana < mana {
        end_turn(game);
    }
}

fn hand_card(
    game: &Game<LuaCardRuntime>,
    player: PlayerId,
    card_id: &str,
) -> hearth_core::EntityId {
    game.state()
        .player(player)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == card_id)
        .unwrap_or_else(|| panic!("{player} has no {card_id} in hand"))
}

fn play(
    game: &mut Game<LuaCardRuntime>,
    player: PlayerId,
    card_id: &str,
    target: Option<hearth_core::EntityId>,
) -> hearth_core::EntityId {
    assert_eq!(game.state().active_player, player);
    let card = hand_card(game, player, card_id);
    game.dispatch(PlayerCommand::PlayCard { card, target })
        .unwrap();
    card
}

fn deck_ids(game: &Game<LuaCardRuntime>, player: PlayerId) -> Vec<String> {
    game.state()
        .player(player)
        .deck
        .iter()
        .map(|entity| game.state().entity(*entity).unwrap().card_id.clone())
        .collect()
}

#[test]
fn lootapalooza_special_cards_use_generic_lua_primitives() {
    let mut ripper = game_with_decks(mixed(&["LOOT_529", "CS2_120"]), repeated("CS2_120"));
    while !ripper
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .any(|entity| ripper.state().entity(*entity).unwrap().card_id == "LOOT_529")
        || !ripper
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| ripper.state().entity(*entity).unwrap().card_id == "CS2_120")
    {
        end_turn(&mut ripper);
    }
    advance_to_mana(&mut ripper, PlayerId::ONE, 5);
    let crocolisk = play(&mut ripper, PlayerId::ONE, "CS2_120", None);
    play(&mut ripper, PlayerId::ONE, "LOOT_529", None);
    let crocolisk = ripper.state().entity(crocolisk).unwrap();
    assert_eq!((crocolisk.attack, crocolisk.health()), (3, 2));

    let mut togwaggle = game_with_decks(mixed(&["LOOT_541", "CS2_120"]), repeated("CS2_171"));
    advance_to_mana(&mut togwaggle, PlayerId::ONE, 8);
    let first_deck = deck_ids(&togwaggle, PlayerId::ONE);
    let second_deck = deck_ids(&togwaggle, PlayerId::TWO);
    play(&mut togwaggle, PlayerId::ONE, "LOOT_541", None);
    assert_eq!(deck_ids(&togwaggle, PlayerId::ONE), second_deck);
    assert_eq!(deck_ids(&togwaggle, PlayerId::TWO), first_deck);
}

#[test]
fn catalog_contains_only_traceable_official_cards() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let definitions = runtime.definitions().collect::<Vec<_>>();
    assert!(definitions.len() >= 70);
    assert!(definitions.iter().all(|card| !card.set.is_empty()));
    let source_path = data_path().join("hearthstonejson/selected.enUS.json");
    let source: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(source_path).unwrap()).unwrap();
    let source_ids = source
        .iter()
        .map(|card| card["id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    let mut manifested_keywords = std::collections::BTreeMap::<String, String>::new();
    for file in [
        "group_a.json",
        "group_b.json",
        "group_c.json",
        "group_d_basic.json",
        "group_d_existing.json",
        "group_d_hard.json",
    ] {
        let examples: Vec<serde_json::Value> = serde_json::from_str(
            &std::fs::read_to_string(data_path().join("keyword_examples").join(file)).unwrap(),
        )
        .unwrap();
        for example in examples {
            manifested_keywords.insert(
                example["card_id"].as_str().unwrap().to_owned(),
                example["keyword"].as_str().unwrap().to_owned(),
            );
        }
    }
    let definition_ids = definitions
        .iter()
        .map(|definition| definition.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        source_ids, definition_ids,
        "official source IDs must exactly match runtime definitions"
    );
    assert!(
        definitions
            .iter()
            .all(|card| source_ids.contains(card.id.as_str()))
    );
    let mut metadata_mismatches = Vec::new();
    for definition in &definitions {
        let record = source
            .iter()
            .find(|record| record["id"].as_str() == Some(definition.id.as_str()))
            .unwrap();
        let expected_text = record["text"].as_str().unwrap_or_default();
        let expected_attack = record["attack"].as_i64().unwrap_or(0) as i32;
        let expected_health = record["health"]
            .as_i64()
            .or_else(|| record["durability"].as_i64())
            .unwrap_or(0) as i32;
        for (field, actual, expected) in [
            (
                "name",
                definition.name.as_str(),
                record["name"].as_str().unwrap(),
            ),
            ("text", definition.text.as_str(), expected_text),
            (
                "set",
                definition.set.as_str(),
                record["set"].as_str().unwrap(),
            ),
        ] {
            if actual != expected {
                metadata_mismatches.push(format!(
                    "{} {field}: {:?} != {:?}",
                    definition.id, actual, expected
                ));
            }
        }
        if i64::from(definition.cost) != record["cost"].as_i64().unwrap_or(0) {
            metadata_mismatches.push(format!("{} cost", definition.id));
        }
        if definition.attack != expected_attack {
            metadata_mismatches.push(format!("{} attack", definition.id));
        }
        // A Starship is a 0/0 client-side placeholder whose accumulated pieces
        // supply its playable stats. Runtime minions require positive base Health.
        if definition.health != expected_health
            && !(definition.id == "GDB_100t2" && definition.health == 1)
        {
            metadata_mismatches.push(format!("{} health/durability", definition.id));
        }
        let expected_spell_damage = if definition.id == "LOE_051" {
            // Client data carries the legacy default value 1, but the card's
            // authoritative text is a symmetric player aura of +2.
            0
        } else if record["text"]
            .as_str()
            .is_some_and(|text| text.starts_with("<b>Spell Damage +2</b>"))
        {
            // A few current client records retain the legacy numeric tag 1
            // after their authoritative displayed text was buffed to +2.
            2
        } else {
            record["spellDamage"].as_i64().unwrap_or_else(|| {
                record["text"]
                    .as_str()
                    .filter(|text| text.starts_with("<b>Spell Damage +1</b>"))
                    .map(|_| 1)
                    .unwrap_or(0)
            })
        };
        let actual_spell_damage = definition
            .keyword_params
            .get("spell_damage")
            .copied()
            .unwrap_or(0);
        if actual_spell_damage != expected_spell_damage {
            metadata_mismatches.push(format!(
                "{} spell damage keyword parameter: {} != {}",
                definition.id, actual_spell_damage, expected_spell_damage
            ));
        }
        if definition.collectible != record["collectible"].as_bool().unwrap_or(false) {
            metadata_mismatches.push(format!("{} collectible", definition.id));
        }
        let expected_kind = match record["type"].as_str().unwrap() {
            "MINION" => CardKind::Minion,
            "SPELL" => CardKind::Spell,
            "WEAPON" => CardKind::Weapon,
            "LOCATION" => CardKind::Location,
            "HERO" => CardKind::Hero,
            "HERO_POWER" => CardKind::HeroPower,
            other => panic!("unsupported official card type {other}"),
        };
        if definition.kind != expected_kind {
            metadata_mismatches.push(format!(
                "{} type: {:?} != {:?}",
                definition.id, definition.kind, expected_kind
            ));
        }
        let expected_class = match record["cardClass"].as_str().unwrap() {
            "DEATHKNIGHT" => "death_knight".to_owned(),
            "DEMONHUNTER" => "demon_hunter".to_owned(),
            value => value.to_ascii_lowercase(),
        };
        if definition.class != expected_class {
            metadata_mismatches.push(format!(
                "{} class: {:?} != {:?}",
                definition.id, definition.class, expected_class
            ));
        }
        let expected_tags = record["races"]
            .as_array()
            .map(|races| {
                races
                    .iter()
                    .filter_map(|race| race.as_str())
                    .collect::<Vec<_>>()
            })
            .filter(|races| !races.is_empty())
            .or_else(|| record["race"].as_str().map(|race| vec![race]))
            .unwrap_or_default()
            .into_iter()
            .map(|race| match race {
                "MECHANICAL" => "mech".to_owned(),
                value => value.to_ascii_lowercase(),
            })
            .collect::<std::collections::BTreeSet<_>>();
        let actual_tags = definition
            .tags
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        if !expected_tags.is_subset(&actual_tags) {
            metadata_mismatches.push(format!(
                "{} tags: {:?} != {:?}",
                definition.id, actual_tags, expected_tags
            ));
        }
        let mechanics = record["mechanics"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|mechanic| mechanic.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let keyword_mapping = [
            ("BATTLECRY", "battlecry"),
            ("CHARGE", "charge"),
            ("CHOOSE_ONE", "choose_one"),
            ("COMBO", "combo"),
            ("DEATHRATTLE", "deathrattle"),
            ("DIVINE_SHIELD", "divine_shield"),
            ("DISCOVER", "discover"),
            ("ELUSIVE", "elusive"),
            ("FORGE", "forge"),
            ("INSPIRE", "inspire"),
            ("LIFESTEAL", "lifesteal"),
            ("MAGNETIC", "magnetic"),
            ("OVERLOAD", "overload"),
            ("POISONOUS", "poisonous"),
            ("QUEST", "quest"),
            ("REBORN", "reborn"),
            ("RUSH", "rush"),
            ("SECRET", "secret"),
            ("SPELLPOWER", "spell_damage"),
            ("SPELLBURST", "spellburst"),
            ("STEALTH", "stealth"),
            ("TAUNT", "taunt"),
            ("TRADEABLE", "tradeable"),
            ("UNTOUCHABLE", "dormant"),
            ("WINDFURY", "windfury"),
        ];
        let expected_keywords = keyword_mapping
            .iter()
            .filter(|(mechanic, _)| mechanics.contains(mechanic))
            .map(|(_, keyword)| (*keyword).to_owned())
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.contains("<b>Finale:</b>"))
                    .map(|_| "finale".to_owned()),
            )
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.contains("<b>Overheal:</b>"))
                    .map(|_| "overheal".to_owned()),
            )
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.contains("<b>Prepare</b>"))
                    .map(|_| "prepare".to_owned()),
            )
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.contains("<b>Casts When Drawn</b>"))
                    .map(|_| "casts_when_drawn".to_owned()),
            )
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.contains("<b>Mega-Windfury</b>"))
                    .map(|_| "mega_windfury".to_owned()),
            )
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.starts_with("<b>Spell Damage +1</b>"))
                    .map(|_| "spell_damage".to_owned()),
            )
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.starts_with("<b>Taunt</b>"))
                    .map(|_| "taunt".to_owned()),
            )
            .chain((definition.id == "CS2_146").then(|| "conditional_charge".to_owned()))
            .chain((definition.id == "EX1_287").then(|| "counter".to_owned()))
            .chain((definition.id == "ICC_827p").then(|| "passive".to_owned()))
            .chain((definition.id == "ICC_833t").then(|| "freeze".to_owned()))
            .chain(
                manifested_keywords
                    .get(&definition.id)
                    .map(ToOwned::to_owned),
            )
            .chain(
                (definition.id == "GDB_100t2")
                    .then(|| ["deathrattle", "taunt"])
                    .into_iter()
                    .flatten()
                    .map(str::to_owned),
            )
            .collect::<std::collections::BTreeSet<_>>();
        let actual_keywords = definition
            .keywords
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        if !actual_keywords.is_subset(&expected_keywords) {
            metadata_mismatches.push(format!(
                "{} keywords: {:?} != {:?}",
                definition.id, actual_keywords, expected_keywords
            ));
        }
        let expected_secret = record["mechanics"]
            .as_array()
            .is_some_and(|mechanics| mechanics.iter().any(|value| value == "SECRET"));
        if (definition.secret
            || definition
                .keywords
                .iter()
                .any(|keyword| keyword == "secret"))
            != expected_secret
        {
            metadata_mismatches.push(format!("{} secret", definition.id));
        }
    }
    assert!(
        metadata_mismatches.is_empty(),
        "Lua metadata differs from source:\n{}",
        metadata_mismatches.join("\n")
    );

    let fireball = runtime.definition("CS2_029").unwrap();
    assert_eq!(fireball.name, "Fireball");
    assert_eq!(fireball.text, "Deal $6 damage.");
    assert_eq!(fireball.set, "LEGACY");
    assert_eq!(runtime.definition("FP1_002").unwrap().set, "NAXX");
    assert_eq!(runtime.definition("WW_376").unwrap().set, "WILD_WEST");
    assert!(!runtime.definition("GAME_005").unwrap().collectible);
    assert_eq!(
        runtime.definition("EX1_238").unwrap().keyword_params["overload"],
        1
    );
    assert_eq!(
        runtime.definition("EX1_250").unwrap().keyword_params["overload"],
        2
    );

    let implemented_sets = definitions
        .iter()
        .map(|card| card.set.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for set in [
        "CORE",
        "LEGACY",
        "EXPERT1",
        "NAXX",
        "GVG",
        "BRM",
        "TGT",
        "LOE",
        "OG",
        "KARA",
        "GANGS",
        "UNGORO",
        "ICECROWN",
        "LOOTAPALOOZA",
        "GILNEAS",
        "BOOMSDAY",
        "TROLL",
        "DALARAN",
        "ULDUM",
        "DRAGONS",
        "DEMON_HUNTER_INITIATE",
        "BLACK_TEMPLE",
        "SCHOLOMANCE",
        "DARKMOON_FAIRE",
        "THE_BARRENS",
        "STORMWIND",
        "ALTERAC_VALLEY",
        "THE_SUNKEN_CITY",
        "REVENDRETH",
        "PATH_OF_ARTHAS",
        "RETURN_OF_THE_LICH_KING",
        "BATTLE_OF_THE_BANDS",
        "TITANS",
        "WILD_WEST",
        "WONDERS",
        "WHIZBANGS_WORKSHOP",
        "ISLAND_VACATION",
        "SPACE",
        "EMERALD_DREAM",
        "THE_LOST_CITY",
        "TIME_TRAVEL",
        "CATACLYSM",
        "ESCAPEFROM_VIOLET_HOLD",
    ] {
        assert!(
            implemented_sets.contains(set),
            "missing representative set {set}"
        );
    }
}

#[test]
fn core_game_construction_rejects_off_class_decks() {
    let valid = Game::new(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        mixed(&["CS2_029", "CS2_120"]),
        repeated("CS2_120"),
        101,
    );
    assert!(valid.is_ok(), "Mage and Neutral cards should be legal");

    let invalid = Game::new(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        repeated("EX1_238"),
        repeated("CS2_120"),
        103,
    );
    assert!(matches!(
        invalid,
        Err(GameError::InvalidDeckClassCard {
            player: PlayerId::ONE,
            class,
            card,
            ..
        }) if class == "mage" && card == "EX1_238"
    ));
}

#[test]
fn keyword_catalog_matches_the_constructed_hearthstone_glossary() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let mut actual = runtime
        .keyword_ids()
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        actual.remove("conditional_charge"),
        "the official Southsea Deckhand helper must remain available"
    );
    assert!(
        actual.remove("deathrattle_repeater"),
        "the generic Baron Rivendare helper must remain available"
    );
    assert!(
        actual.remove("hero_power_can_target_minions"),
        "the generic Hero Power target-extension helper must remain available"
    );
    assert!(
        actual.remove("dragon_consort_discount"),
        "the generic player-owned Dragon discount helper must remain available"
    );
    assert!(actual.remove("hero_power_twice_per_turn"));
    assert!(actual.remove("hero_power_unlimited"));
    assert!(actual.remove("cannot_be_attacked_by_icehowl"));
    assert!(actual.remove("hero_power_next_turn_surcharge"));
    assert!(actual.remove("next_hero_power_discount"));
    assert!(actual.remove("power_word_glory"));
    assert!(actual.remove("battlecry_repeater"));
    assert!(actual.remove("costs_health_instead_of_mana"));
    assert!(actual.remove("cthun_buffs"));
    assert!(actual.remove("cthun_taunt"));
    assert!(actual.remove("healing_becomes_damage"));
    assert!(actual.remove("fools_bane_unlimited_attacks"));
    assert!(actual.remove("randomize_targets"));
    assert!(actual.remove("cannot_be_attacked_by_fools_bane"));
    assert!(actual.remove("raza_hero_power_zero"));
    assert!(actual.remove("next_secret_cost_one_this_turn"));
    assert!(actual.remove("next_spell_cost_zero_this_turn"));
    assert!(actual.remove("next_murloc_costs_health"));
    assert!(actual.remove("radiant_elemental_minimum_cost"));
    assert!(actual.remove("cannot_be_attacked_by_charged_devilsaur"));
    assert!(actual.remove("corrupting_mist_curse"));
    assert!(actual.remove("next_spell_costs_health"));
    assert!(actual.remove("weapon_durability_immune"));
    assert!(actual.remove("hero_power_disabled"));
    assert!(actual.remove("end_of_turn_repeater"));
    let expected = [
        "adapt",
        "battlecry",
        "casts_when_drawn",
        "charge",
        "choose_multiple",
        "choose_one",
        "colossal",
        "combo",
        "corrupt",
        "counter",
        "deathrattle",
        "discover",
        "divine_shield",
        "dormant",
        "dredge",
        "echo",
        "elusive",
        "excavate",
        "fabled",
        "finale",
        "forge",
        "freeze",
        "frenzy",
        "gigantify",
        "herald",
        "honorable_kill",
        "imbue",
        "immune",
        "infuse",
        "inspire",
        "invoke",
        "kindred",
        "lifesteal",
        "magnetic",
        "manathirst",
        "mega_windfury",
        "miniaturize",
        "outcast",
        "overheal",
        "overkill",
        "overload",
        "passive",
        "poisonous",
        "prepare",
        "quest",
        "questline",
        "quickdraw",
        "reborn",
        "recruit",
        "rewind",
        "rush",
        "secret",
        "shatter",
        "sidequest",
        "silence",
        "spell_damage",
        "spellburst",
        "starship",
        "start_of_game",
        "stealth",
        "summoned_when_drawn",
        "taunt",
        "temporary",
        "titan",
        "tourist",
        "tradeable",
        "twinspell",
        "windfury",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(expected.len(), 68);
    assert_eq!(actual, expected);
}

#[test]
fn forge_and_prepare_are_generic_replayable_card_actions() {
    let deck = ["TTN_724", "CATA_EVENT_401"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 3);

    let giant = hand_card(&game, PlayerId::ONE, "TTN_724");
    assert!(
        game.legal_actions()
            .unwrap()
            .contains(&PlayerCommand::UseCardAction {
                card: giant,
                action: "forge".to_owned(),
                target: None,
            })
    );
    game.dispatch(PlayerCommand::UseCardAction {
        card: giant,
        action: "forge".to_owned(),
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().entity(giant).unwrap().cost, 6);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 1);

    let geomancer = hand_card(&game, PlayerId::ONE, "CATA_EVENT_401");
    game.dispatch(PlayerCommand::UseCardAction {
        card: geomancer,
        action: "prepare".to_owned(),
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().player(PlayerId::ONE).mana, 0);
    assert_eq!(game.state().entity(geomancer).unwrap().cost, 1);
    assert!(!game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard { card, .. } if *card == geomancer
    )));

    end_turn(&mut game);
    end_turn(&mut game);
    assert!(game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard { card, .. } if *card == geomancer
    )));

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn forge_prepare_state_machine_walks_preserve_invariants() {
    let deck = ["TTN_724", "CATA_EVENT_401", "CS2_120", "CS2_029"]
        .into_iter()
        .cycle()
        .take(24)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for seed in 100_u64..108 {
        let mut game = Game::new_unrestricted(
            LuaCardRuntime::load_dir(data_path()).unwrap(),
            deck.clone(),
            deck.clone(),
            seed,
        )
        .unwrap();
        for step in 0_usize..100 {
            if game.state().outcome.is_some() {
                break;
            }
            let actions = game.legal_actions().unwrap();
            assert!(!actions.is_empty(), "seed {seed}, step {step}");
            let index = ((seed as usize).wrapping_mul(29) + step.wrapping_mul(31)) % actions.len();
            let command = actions[index].clone();
            game.dispatch(command.clone()).unwrap_or_else(|error| {
                panic!("legal command {command:?} failed for seed {seed}, step {step}: {error}")
            });
            game.state()
                .validate()
                .unwrap_or_else(|error| panic!("seed {seed}, step {step}: {error}"));
        }
        if seed == 100 {
            let replayed = Game::from_replay(
                LuaCardRuntime::load_dir(data_path()).unwrap(),
                &game.replay(),
            )
            .unwrap();
            assert_eq!(replayed.state(), game.state());
        }
    }
}

#[test]
fn all_eleven_basic_hero_powers_are_standalone_lua_modules() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let expected = [
        ("HERO_01bp", "warrior", 2),
        ("HERO_02bp", "shaman", 2),
        ("HERO_03bp", "rogue", 2),
        ("HERO_04bp", "paladin", 2),
        ("HERO_05bp", "hunter", 2),
        ("HERO_06bp", "druid", 2),
        ("HERO_07bp", "warlock", 2),
        ("HERO_08bp", "mage", 2),
        ("HERO_09bp", "priest", 2),
        ("HERO_10bp", "demon_hunter", 1),
        ("HERO_11bp", "death_knight", 2),
    ];
    for (id, class, cost) in expected {
        let definition = runtime
            .definition(id)
            .unwrap_or_else(|| panic!("missing basic Hero Power {id}"));
        assert_eq!(definition.kind, CardKind::HeroPower, "{id}");
        assert!(!definition.collectible, "{id}");
        assert_eq!(definition.class, class, "{id}");
        assert_eq!(definition.cost, cost, "{id}");
    }
    assert!(
        data_path()
            .join("hero_powers/basic/dagger_mastery.lua")
            .is_file()
    );
    assert!(!data_path().join("sets/legacy/dagger_mastery.lua").exists());
}

#[test]
fn basic_hero_power_lua_effects_execute_through_the_shared_runtime() {
    for power in [
        "HERO_01bp",
        "HERO_02bp",
        "HERO_03bp",
        "HERO_04bp",
        "HERO_05bp",
        "HERO_06bp",
        "HERO_07bp",
        "HERO_08bp",
        "HERO_09bp",
        "HERO_10bp",
        "HERO_11bp",
    ] {
        let mut game = game_with_hero_powers(
            repeated("CS2_120"),
            repeated("CS2_120"),
            [power, "HERO_08bp"],
            ["neutral", "mage"],
        );
        let cost = game.runtime().definition(power).unwrap().cost;
        advance_to_mana(&mut game, PlayerId::ONE, cost);
        let target = matches!(power, "HERO_08bp" | "HERO_09bp")
            .then_some(game.state().player(PlayerId::TWO).hero);
        game.dispatch(PlayerCommand::UseHeroPower { target })
            .unwrap();
        assert!(
            game.state().player(PlayerId::ONE).hero_power_used,
            "{power}"
        );

        match power {
            "HERO_01bp" => assert_eq!(game.state().hero(PlayerId::ONE).armor, 2),
            "HERO_02bp" => assert_eq!(game.state().player(PlayerId::ONE).board.len(), 1),
            "HERO_03bp" => assert!(game.state().player(PlayerId::ONE).weapon.is_some()),
            "HERO_04bp" => assert_eq!(
                game.state().entities[&game.state().player(PlayerId::ONE).board[0]].card_id,
                "CS2_101t"
            ),
            "HERO_05bp" => assert_eq!(game.state().hero(PlayerId::TWO).damage, 2),
            "HERO_06bp" => {
                assert_eq!(game.state().hero(PlayerId::ONE).attack, 1);
                assert_eq!(game.state().hero(PlayerId::ONE).armor, 1);
            }
            "HERO_07bp" => assert_eq!(game.state().hero(PlayerId::ONE).damage, 2),
            "HERO_08bp" => assert_eq!(game.state().hero(PlayerId::TWO).damage, 1),
            "HERO_09bp" => assert_eq!(game.state().hero(PlayerId::TWO).damage, 0),
            "HERO_10bp" => assert_eq!(game.state().hero(PlayerId::ONE).attack, 1),
            "HERO_11bp" => {
                let ghoul = game.state().player(PlayerId::ONE).board[0];
                assert_eq!(game.state().entities[&ghoul].card_id, "HERO_11bpt");
                game.dispatch(PlayerCommand::EndTurn).unwrap();
                assert_eq!(game.state().entities[&ghoul].zone, Zone::Graveyard);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn frozen_throne_has_all_nine_hero_cards_and_their_own_power_modules() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    for (hero, power, class, cost) in [
        ("ICC_481", "ICC_481p", "shaman", 5),
        ("ICC_827", "ICC_827p", "rogue", 9),
        ("ICC_828", "ICC_828p", "hunter", 6),
        ("ICC_829", "ICC_829p", "paladin", 9),
        ("ICC_830", "ICC_830p", "priest", 8),
        ("ICC_831", "ICC_831p", "warlock", 10),
        ("ICC_832", "ICC_832p", "druid", 7),
        ("ICC_833", "ICC_833h", "mage", 9),
        ("ICC_834", "ICC_834h", "warrior", 8),
    ] {
        let definition = runtime.definition(hero).unwrap();
        assert_eq!(definition.kind, CardKind::Hero, "{hero}");
        assert_eq!(definition.hero_power.as_deref(), Some(power), "{hero}");
        assert_eq!(definition.armor, 5, "{hero}");
        assert_eq!(definition.class, class, "{hero}");
        assert_eq!(definition.cost, cost, "{hero}");
        assert_eq!(runtime.definition(power).unwrap().kind, CardKind::HeroPower);
    }
}

#[test]
fn playing_a_hero_card_preserves_health_and_replaces_hero_armor_and_power() {
    let mut game = game_with_decks(repeated("ICC_828"), repeated("CS2_120"));
    let old_hero = game.state().player(PlayerId::ONE).hero;

    advance_to_mana(&mut game, PlayerId::TWO, 2);
    play(&mut game, PlayerId::TWO, "CS2_120", None);
    let enemy_minion = game.state().player(PlayerId::TWO).board[0];
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let new_hero = play(&mut game, PlayerId::ONE, "ICC_828", None);

    let state = game.state();
    assert_eq!(state.player(PlayerId::ONE).hero, new_hero);
    assert_eq!(state.entities[&old_hero].zone, Zone::Removed);
    assert_eq!(state.entities[&new_hero].zone, Zone::Hero);
    assert_eq!(state.entities[&new_hero].health(), 30);
    assert_eq!(state.entities[&new_hero].armor, 5);
    assert_eq!(
        state.entities[&state.player(PlayerId::ONE).hero_power].card_id,
        "ICC_828p"
    );
    assert_eq!(state.entities[&enemy_minion].damage, 2);
    assert!(state.log.iter().any(|event| matches!(
        event,
        GameEvent::HeroReplaced { old, new, .. } if *old == old_hero && *new == new_hero
    )));
}

#[test]
fn every_frozen_throne_hero_card_resolves_its_lua_battlecry_or_choice() {
    for (hero, power, cost) in [
        ("ICC_481", "ICC_481p", 5),
        ("ICC_827", "ICC_827p", 9),
        ("ICC_828", "ICC_828p", 6),
        ("ICC_829", "ICC_829p", 9),
        ("ICC_830", "ICC_830p", 8),
        ("ICC_831", "ICC_831p", 10),
        ("ICC_832", "ICC_832p", 7),
        ("ICC_833", "ICC_833h", 9),
        ("ICC_834", "ICC_834h", 8),
    ] {
        let mut game = game_with_decks(repeated(hero), repeated("CS2_120"));
        advance_to_mana(&mut game, PlayerId::ONE, cost);
        let hero_entity = play(&mut game, PlayerId::ONE, hero, None);
        if game.state().pending_input.is_some() {
            game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
        }
        let state = game.state();
        assert_eq!(state.player(PlayerId::ONE).hero, hero_entity, "{hero}");
        assert_eq!(state.entities[&hero_entity].card_id, hero, "{hero}");
        assert_eq!(
            state.entities[&state.player(PlayerId::ONE).hero_power].card_id,
            power,
            "{hero}"
        );
        assert_eq!(state.entities[&hero_entity].armor, 5, "{hero}");

        match hero {
            "ICC_827" => assert!(
                state
                    .player(PlayerId::ONE)
                    .hand
                    .iter()
                    .any(|entity| { state.entities[entity].card_id == "ICC_827t" })
            ),
            "ICC_829" => assert_eq!(
                state.entities[&state.player(PlayerId::ONE).weapon.unwrap()].card_id,
                "ICC_829t"
            ),
            "ICC_832" => assert_eq!(state.player(PlayerId::ONE).board.len(), 2),
            "ICC_833" => assert_eq!(
                state.entities[&state.player(PlayerId::ONE).board[0]].card_id,
                "ICC_833t"
            ),
            "ICC_834" => assert_eq!(
                state.entities[&state.player(PlayerId::ONE).weapon.unwrap()].card_id,
                "ICC_834w"
            ),
            _ => {}
        }
    }
}

#[test]
fn frozen_throne_active_hero_powers_resolve_choices_refresh_and_lifesteal() {
    let mut plague = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_120"),
        ["ICC_832p", "HERO_08bp"],
        ["druid", "mage"],
    );
    advance_to_mana(&mut plague, PlayerId::ONE, 2);
    plague
        .dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    plague.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert_eq!(plague.state().hero(PlayerId::ONE).attack, 3);

    let mut voidform = game_with_hero_powers(
        repeated("EX1_169"),
        repeated("CS2_120"),
        ["ICC_830p", "HERO_08bp"],
        ["priest", "mage"],
    );
    advance_to_mana(&mut voidform, PlayerId::ONE, 2);
    let enemy_hero = voidform.state().player(PlayerId::TWO).hero;
    voidform
        .dispatch(PlayerCommand::UseHeroPower {
            target: Some(enemy_hero),
        })
        .unwrap();
    play(&mut voidform, PlayerId::ONE, "EX1_169", None);
    assert!(!voidform.state().player(PlayerId::ONE).hero_power_used);

    let mut siphon = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_120"),
        ["ICC_831p", "HERO_08bp"],
        ["warlock", "mage"],
    );
    advance_to_mana(&mut siphon, PlayerId::TWO, 2);
    let friendly_hero = siphon.state().player(PlayerId::ONE).hero;
    siphon
        .dispatch(PlayerCommand::UseHeroPower {
            target: Some(friendly_hero),
        })
        .unwrap();
    advance_to_mana(&mut siphon, PlayerId::ONE, 2);
    let enemy_hero = siphon.state().player(PlayerId::TWO).hero;
    siphon
        .dispatch(PlayerCommand::UseHeroPower {
            target: Some(enemy_hero),
        })
        .unwrap();
    assert_eq!(siphon.state().hero(PlayerId::ONE).damage, 0);
    assert_eq!(siphon.state().hero(PlayerId::TWO).damage, 3);
}

#[test]
fn frozen_throne_generated_and_transforming_hero_powers_are_replayable() {
    let shadow = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_120"),
        ["ICC_827p", "HERO_08bp"],
        ["rogue", "mage"],
    );
    assert!(
        shadow
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { shadow.state().entities[entity].card_id == "ICC_827t" })
    );

    let mut icy = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_171"),
        ["ICC_833h", "HERO_08bp"],
        ["mage", "mage"],
    );
    advance_to_mana(&mut icy, PlayerId::TWO, 1);
    let boar = play(&mut icy, PlayerId::TWO, "CS2_171", None);
    advance_to_mana(&mut icy, PlayerId::ONE, 2);
    icy.dispatch(PlayerCommand::UseHeroPower { target: Some(boar) })
        .unwrap();
    assert_eq!(icy.state().entities[&boar].zone, Zone::Graveyard);
    assert!(
        icy.state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .any(|entity| { icy.state().entities[entity].card_id == "ICC_833t" })
    );

    let mut transmute = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_120"),
        ["ICC_481p", "HERO_08bp"],
        ["shaman", "mage"],
    );
    advance_to_mana(&mut transmute, PlayerId::ONE, 2);
    let minion = play(&mut transmute, PlayerId::ONE, "CS2_120", None);
    advance_to_mana(&mut transmute, PlayerId::ONE, 3);
    transmute
        .dispatch(PlayerCommand::UseHeroPower {
            target: Some(minion),
        })
        .unwrap();
    assert_eq!(transmute.state().entities[&minion].cost, 3);

    let mut builder = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_120"),
        ["ICC_828p", "HERO_08bp"],
        ["hunter", "mage"],
    );
    advance_to_mana(&mut builder, PlayerId::ONE, 2);
    builder
        .dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    builder
        .dispatch(PlayerCommand::Choose { index: 0 })
        .unwrap();
    builder
        .dispatch(PlayerCommand::Choose { index: 0 })
        .unwrap();
    assert!(
        builder
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| {
                let entity = &builder.state().entities[entity];
                entity.card_id == "ICC_828t" && entity.attached_cards.len() == 2
            })
    );
}

#[test]
fn the_four_horsemen_power_wins_only_after_four_distinct_horsemen() {
    let mut game = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_120"),
        ["ICC_829p", "HERO_08bp"],
        ["paladin", "mage"],
    );
    for count in 1..=4 {
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        game.dispatch(PlayerCommand::UseHeroPower { target: None })
            .unwrap();
        assert_eq!(game.state().player(PlayerId::ONE).board.len(), count);
        if count < 4 {
            assert!(game.state().outcome.is_none());
            game.dispatch(PlayerCommand::EndTurn).unwrap();
        }
    }
    assert_eq!(
        game.state().outcome,
        Some(hearth_core::GameOutcome::Winner(PlayerId::ONE))
    );
}

#[test]
fn magnetic_merges_stats_scripts_and_keywords_without_summoning_a_second_minion() {
    let deck = ["GVG_085", "BOT_563", "EX1_332"]
        .into_iter()
        .cycle()
        .take(24)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let target = play(&mut game, PlayerId::ONE, "GVG_085", None);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let wargear = hand_card(&game, PlayerId::ONE, "BOT_563");
    let summons_before = game
        .state()
        .log
        .iter()
        .filter(|event| matches!(event, GameEvent::MinionSummoned { .. }))
        .count();
    game.dispatch(PlayerCommand::PlayCardAt {
        card: wargear,
        target: None,
        position: 0,
    })
    .unwrap();

    let merged = game.state().entity(target).unwrap();
    assert_eq!((merged.attack, merged.max_health), (7, 7));
    assert_eq!(merged.attached_cards, vec!["BOT_563"]);
    assert_eq!(game.state().entity(wargear).unwrap().zone, Zone::Removed);
    assert_eq!(game.state().player(PlayerId::ONE).board, vec![target]);
    assert_eq!(
        game.state()
            .log
            .iter()
            .filter(|event| matches!(event, GameEvent::MinionSummoned { .. }))
            .count(),
        summons_before
    );
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::Magnetized { attachment, target: event_target, .. }
            if *attachment == wargear && *event_target == target
    )));

    let silence = hand_card(&game, PlayerId::ONE, "EX1_332");
    game.dispatch(PlayerCommand::PlayCard {
        card: silence,
        target: Some(target),
    })
    .unwrap();
    let silenced = game.state().entity(target).unwrap();
    assert_eq!((silenced.attack, silenced.max_health), (1, 2));
}

#[test]
fn every_card_has_all_three_official_locales() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    for definition in runtime.definitions() {
        let english = definition
            .localizations
            .get(&Locale::EnUs)
            .unwrap_or_else(|| panic!("{} is missing enUS", definition.id));
        assert_eq!(
            definition.name, english.name,
            "{} fallback name",
            definition.id
        );
        assert_eq!(
            definition.text, english.text,
            "{} fallback text",
            definition.id
        );
        for locale in Locale::ALL {
            let text = definition
                .localizations
                .get(&locale)
                .unwrap_or_else(|| panic!("{} is missing {}", definition.id, locale.code()));
            assert!(
                !text.name.trim().is_empty(),
                "{} {} has no name",
                definition.id,
                locale.code()
            );
        }
    }

    let caverns = runtime.definition("UNG_067").unwrap();
    assert_eq!(caverns.name, "The Caverns Below");
    assert_eq!(caverns.text, caverns.localized(Locale::EnUs).text);
    assert_eq!(caverns.localized(Locale::EnUs).name, "The Caverns Below");
    assert_eq!(caverns.localized(Locale::ZhCn).name, "探索地下洞穴");
    assert_eq!(caverns.localized(Locale::ZhTw).name, "洞穴歷險");

    let english = LuaCardRuntime::load_dir_with_locale(data_path(), Locale::EnUs).unwrap();
    assert_eq!(
        english.definition("UNG_067").unwrap().name,
        "The Caverns Below"
    );
    let traditional = LuaCardRuntime::load_dir_with_locale(data_path(), Locale::ZhTw).unwrap();
    assert_eq!(traditional.definition("UNG_067").unwrap().name, "洞穴歷險");
    assert_ne!(english.pack_hash(), runtime.pack_hash());
}

#[test]
fn lua_dynamic_prompts_follow_the_runtime_locale() {
    for (locale, expected) in [
        (Locale::EnUs, "Discover a spell"),
        (Locale::ZhCn, "发现一张法术牌"),
        (Locale::ZhTw, "發現一張法術牌"),
    ] {
        let mut game = game_with_locale("BAR_065", locale);
        advance_to_mana(&mut game, PlayerId::ONE, 3);
        play(&mut game, PlayerId::ONE, "BAR_065", None);
        assert_eq!(
            game.state().pending_input.as_ref().unwrap().prompt,
            expected
        );
    }
}

#[test]
fn published_dog_quest_rogue_deck_is_complete_and_playable() {
    let source = std::fs::read_to_string(data_path().join("../decks/quest_rogue.json")).unwrap();
    let deck: serde_json::Value = serde_json::from_str(&source).unwrap();
    let cards = deck["cards"].as_array().unwrap();
    assert_eq!(cards.len(), 30);
    assert_eq!(deck["name"], "Classic Caverns Quest Rogue (Dog, 2017)");
    assert!(deck.get("localized_names").is_none());
    assert_eq!(deck["class"], "rogue");
    assert_eq!(deck["hero_power"], "HERO_03bp");

    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let mut counts = std::collections::BTreeMap::new();
    for card in cards {
        let id = card.as_str().unwrap();
        *counts.entry(id).or_insert(0_usize) += 1;
        assert!(
            runtime
                .definition(id)
                .is_some_and(|definition| definition.collectible),
            "deck contains unavailable card {id}"
        );
    }
    let expected = [
        ("EX1_613", 1),
        ("EX1_145", 2),
        ("EX1_129", 2),
        ("EX1_124", 2),
        ("CS2_072", 2),
        ("EX1_144", 2),
        ("KAR_069", 2),
        ("CFM_693", 2),
        ("UNG_067", 1),
        ("UNG_060", 2),
        ("CFM_637", 1),
        ("EX1_015", 2),
        ("CS2_171", 2),
        ("CS2_146", 2),
        ("EX1_049", 2),
        ("KAR_044", 1),
        ("NEW1_026", 2),
    ]
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(counts, expected);
}

#[test]
fn quest_starts_in_hand_completes_once_and_crystal_core_persists() {
    let mut deck = vec!["UNG_067".to_owned()];
    deck.extend(std::iter::repeat_n("KAR_069".to_owned(), 19));
    let opponent = ["EX1_332", "CS1_113"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_classes(deck, opponent, ["rogue", "priest"]);

    let quest = hand_card(&game, PlayerId::ONE, "UNG_067");
    assert_eq!(game.state().entity(quest).unwrap().zone, Zone::Hand);
    play(&mut game, PlayerId::ONE, "UNG_067", None);
    assert_eq!(game.state().entity(quest).unwrap().zone, Zone::Secret);

    for _ in 0..4 {
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        play(&mut game, PlayerId::ONE, "KAR_069", None);
        if game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().card_id != "UNG_067t1")
        {
            end_turn(&mut game);
        }
    }

    let rewards = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "UNG_067t1")
        .count();
    assert_eq!(rewards, 1);

    end_turn(&mut game);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "UNG_067t1", None);
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .all(|entity| {
                let entity = game.state().entity(*entity).unwrap();
                entity.attack == 5 && entity.max_health == 5
            })
    );

    let new_minion = play(&mut game, PlayerId::ONE, "KAR_069", None);
    assert_eq!(game.state().entity(new_minion).unwrap().attack, 5);
    assert_eq!(game.state().entity(new_minion).unwrap().max_health, 5);

    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "EX1_332", Some(new_minion));
    assert_eq!(game.state().entity(new_minion).unwrap().attack, 5);
    assert_eq!(game.state().entity(new_minion).unwrap().max_health, 5);

    advance_to_mana(&mut game, PlayerId::TWO, 9);
    play(&mut game, PlayerId::TWO, "CS1_113", Some(new_minion));
    let stolen = game.state().entity(new_minion).unwrap();
    assert_eq!(stolen.controller, PlayerId::TWO);
    assert_eq!(stolen.attack, 1);
    assert_eq!(stolen.max_health, 2);

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn preparation_reduces_exactly_the_next_spell_and_expires() {
    let deck = ["EX1_145", "CS2_029"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "EX1_145", None);
    let fireball = hand_card(&game, PlayerId::ONE, "CS2_029");
    assert_eq!(game.state().entity(fireball).unwrap().cost, 2);
    let enemy = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "CS2_029", Some(enemy));
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .filter_map(|id| {
                let entity = game.state().entity(*id).unwrap();
                (entity.card_id == "CS2_029").then_some(entity.cost)
            })
            .all(|cost| cost == 4)
    );

    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "EX1_145", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let fireball = hand_card(&game, PlayerId::ONE, "CS2_029");
    assert_eq!(game.state().entity(fireball).unwrap().cost, 4);
}

#[test]
fn countered_spell_still_consumes_preparation_discount() {
    let deck = ["EX1_145", "CS2_029"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("EX1_287"));
    advance_to_mana(&mut game, PlayerId::TWO, 3);
    play(&mut game, PlayerId::TWO, "EX1_287", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "EX1_145", None);
    let enemy = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "CS2_029", Some(enemy));
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::CardCountered {
            player: PlayerId::ONE,
            ..
        }
    )));
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .filter_map(|id| {
                let entity = game.state().entity(*id).unwrap();
                (entity.card_id == "CS2_029").then_some(entity.cost)
            })
            .all(|cost| cost == 4)
    );
}

#[test]
fn shadowstep_returns_a_minion_and_applies_cost_after_hidden_zone_reset() {
    let deck = ["EX1_144", "EX1_015"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let engineer = play(&mut game, PlayerId::ONE, "EX1_015", None);
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "EX1_144", Some(engineer));
    let returned = game.state().entity(engineer).unwrap();
    assert_eq!(returned.zone, Zone::Hand);
    assert_eq!(returned.cost, 0);
}

#[test]
fn patches_recruits_only_when_still_in_the_deck() {
    let mut deck = vec!["CFM_637".to_owned()];
    deck.extend(std::iter::repeat_n("KAR_069".to_owned(), 19));
    let mut game = Game::new_unrestricted(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        deck,
        repeated("CS2_120"),
        11,
    )
    .unwrap();
    let patches_in_hand = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "CFM_637");
    game.dispatch(PlayerCommand::Mulligan {
        replace: patches_in_hand.into_iter().collect(),
    })
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();

    let pirate = play(&mut game, PlayerId::ONE, "KAR_069", None);
    let patches = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "CFM_637")
        .expect("Patches should be recruited");
    assert_ne!(patches, pirate);
    assert!(!game.state().player(PlayerId::ONE).deck.contains(&patches));

    end_turn(&mut game);
    end_turn(&mut game);
    play(&mut game, PlayerId::ONE, "KAR_069", None);
    assert_eq!(
        game.state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .filter(|entity| { game.state().entity(**entity).unwrap().card_id == "CFM_637" })
            .count(),
        1
    );
}

#[test]
fn rogue_hero_power_equips_and_replaces_the_generated_dagger() {
    let mut game = game_with_hero_powers(
        repeated("CS2_120"),
        repeated("CS2_120"),
        ["HERO_03bp", DEFAULT_HERO_POWER],
        ["rogue", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    game.dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    let first = game.state().player(PlayerId::ONE).weapon.unwrap();
    assert_eq!(game.state().entity(first).unwrap().card_id, "CS2_082");

    end_turn(&mut game);
    end_turn(&mut game);
    game.dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    let second = game.state().player(PlayerId::ONE).weapon.unwrap();
    assert_ne!(first, second);
    assert_eq!(game.state().entity(first).unwrap().zone, Zone::Graveyard);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::WeaponDestroyed { weapon, .. } if *weapon == first
    )));
}

#[test]
fn token_generators_do_nothing_on_a_full_board() {
    let mut game = game("KAR_044", "CS2_120");
    for mana in 3..=5 {
        advance_to_mana(&mut game, PlayerId::ONE, mana);
        play(&mut game, PlayerId::ONE, "KAR_044", None);
        end_turn(&mut game);
    }
    assert_eq!(game.state().player(PlayerId::ONE).board.len(), 7);
    assert_eq!(
        game.state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .filter(|entity| { game.state().entity(**entity).unwrap().card_id == "KAR_044a" })
            .count(),
        4
    );
}

#[test]
fn moroes_and_violet_teacher_generate_their_official_tokens_at_the_right_time() {
    let mut moroes = game("KAR_044", "CS2_120");
    advance_to_mana(&mut moroes, PlayerId::ONE, 3);
    play(&mut moroes, PlayerId::ONE, "KAR_044", None);
    assert_eq!(moroes.state().player(PlayerId::ONE).board.len(), 1);
    end_turn(&mut moroes);
    assert!(
        moroes
            .state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .any(|entity| { moroes.state().entity(*entity).unwrap().card_id == "KAR_044a" })
    );

    let deck = ["NEW1_026", "EX1_145"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut teacher = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut teacher, PlayerId::ONE, 4);
    play(&mut teacher, PlayerId::ONE, "NEW1_026", None);
    assert_eq!(teacher.state().player(PlayerId::ONE).board.len(), 1);
    play(&mut teacher, PlayerId::ONE, "EX1_145", None);
    assert!(
        teacher
            .state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .any(|entity| { teacher.state().entity(*entity).unwrap().card_id == "NEW1_026t" })
    );
}

#[test]
fn fan_of_knives_damages_all_enemy_minions_simultaneously_and_draws() {
    let mut game = game("EX1_129", "CS2_120");
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let first = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let second = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);

    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    play(&mut game, PlayerId::ONE, "EX1_129", None);
    assert_eq!(game.state().entity(first).unwrap().health(), 2);
    assert_eq!(game.state().entity(second).unwrap().health(), 2);
    assert_eq!(game.state().player(PlayerId::ONE).hand.len(), hand_before);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::CardDrawn {
            player: PlayerId::ONE,
            ..
        }
    )));
}

#[test]
fn eviscerate_replaces_base_damage_when_combo_is_active() {
    let deck: Vec<String> = ["EX1_145", "EX1_124"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut normal = game_with_decks(deck.clone(), repeated("CS2_120"));
    advance_to_mana(&mut normal, PlayerId::ONE, 2);
    let enemy = normal.state().player(PlayerId::TWO).hero;
    play(&mut normal, PlayerId::ONE, "EX1_124", Some(enemy));
    assert_eq!(normal.state().entity(enemy).unwrap().health(), 28);

    let mut combo = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut combo, PlayerId::ONE, 2);
    play(&mut combo, PlayerId::ONE, "EX1_145", None);
    let enemy = combo.state().player(PlayerId::TWO).hero;
    play(&mut combo, PlayerId::ONE, "EX1_124", Some(enemy));
    assert_eq!(combo.state().entity(enemy).unwrap().health(), 26);
}

#[test]
fn backstab_stops_targeting_a_minion_after_any_damage() {
    let mut game = game("CS2_072", "CS2_200");
    advance_to_mana(&mut game, PlayerId::TWO, 6);
    let ogre = play(&mut game, PlayerId::TWO, "CS2_200", None);
    end_turn(&mut game);
    let first = hand_card(&game, PlayerId::ONE, "CS2_072");
    assert!(game.valid_targets(first).unwrap().contains(&ogre));
    play(&mut game, PlayerId::ONE, "CS2_072", Some(ogre));
    let second = hand_card(&game, PlayerId::ONE, "CS2_072");
    assert!(!game.valid_targets(second).unwrap().contains(&ogre));
    assert_eq!(game.state().entity(ogre).unwrap().health(), 5);
}

#[test]
fn mimic_pod_draws_the_top_card_and_creates_exactly_one_copy() {
    let deck = ["UNG_060", "CS2_029"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let top = game.state().player(PlayerId::ONE).deck[0];
    let expected = game.state().entity(top).unwrap().card_id.clone();
    let before = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == expected)
        .count();
    play(&mut game, PlayerId::ONE, "UNG_060", None);
    let after = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == expected)
        .count();
    let played_was_expected = usize::from(expected == "UNG_060");
    assert_eq!(after, before + 2 - played_was_expected);
}

#[test]
fn gadgetzan_ferryman_only_requests_and_returns_a_target_on_combo() {
    let deck: Vec<String> = ["CFM_693", "EX1_145", "EX1_015"]
        .into_iter()
        .cycle()
        .take(21)
        .map(str::to_owned)
        .collect();
    let mut no_combo = game_with_decks(deck.clone(), repeated("CS2_120"));
    advance_to_mana(&mut no_combo, PlayerId::ONE, 2);
    let ferryman = hand_card(&no_combo, PlayerId::ONE, "CFM_693");
    assert!(no_combo.valid_targets(ferryman).unwrap().is_empty());
    no_combo
        .dispatch(PlayerCommand::PlayCard {
            card: ferryman,
            target: None,
        })
        .unwrap();

    let mut combo = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut combo, PlayerId::ONE, 4);
    let engineer = play(&mut combo, PlayerId::ONE, "EX1_015", None);
    play(&mut combo, PlayerId::ONE, "EX1_145", None);
    let ferryman = hand_card(&combo, PlayerId::ONE, "CFM_693");
    assert!(combo.valid_targets(ferryman).unwrap().contains(&engineer));
    combo
        .dispatch(PlayerCommand::PlayCard {
            card: ferryman,
            target: Some(engineer),
        })
        .unwrap();
    assert_eq!(combo.state().entity(engineer).unwrap().zone, Zone::Hand);
}

#[test]
fn southsea_deckhand_has_charge_only_while_a_weapon_is_equipped() {
    let mut without_weapon = game("CS2_146", "CS2_120");
    let deckhand = play(&mut without_weapon, PlayerId::ONE, "CS2_146", None);
    let enemy = without_weapon.state().player(PlayerId::TWO).hero;
    assert!(
        !without_weapon
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: deckhand,
                defender: enemy,
            })
    );

    let mut with_weapon = game_with_hero_powers(
        repeated("CS2_146"),
        repeated("CS2_120"),
        ["HERO_03bp", DEFAULT_HERO_POWER],
        ["rogue", "mage"],
    );
    advance_to_mana(&mut with_weapon, PlayerId::ONE, 2);
    with_weapon
        .dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    end_turn(&mut with_weapon);
    end_turn(&mut with_weapon);
    let deckhand = play(&mut with_weapon, PlayerId::ONE, "CS2_146", None);
    let enemy = with_weapon.state().player(PlayerId::TWO).hero;
    assert!(
        with_weapon
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: deckhand,
                defender: enemy,
            })
    );
}

#[test]
fn swashburglar_generates_only_a_card_from_another_class() {
    let mut game = game_with_classes(repeated("KAR_069"), repeated("CS2_120"), ["rogue", "mage"]);
    let before = game.state().player(PlayerId::ONE).hand.clone();
    play(&mut game, PlayerId::ONE, "KAR_069", None);
    let generated = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .find(|entity| {
            !before.contains(entity) && game.state().entity(*entity).unwrap().card_id != "KAR_069"
        })
        .expect("battlecry should create a card");
    let card_id = &game.state().entity(generated).unwrap().card_id;
    let definition = game.runtime().definition(card_id).unwrap();
    assert_ne!(definition.class, "neutral");
    assert_ne!(definition.class, "rogue");
}

#[test]
fn quest_rogue_legal_action_walks_preserve_all_state_invariants() {
    let source = std::fs::read_to_string(data_path().join("../decks/quest_rogue.json")).unwrap();
    let document: serde_json::Value = serde_json::from_str(&source).unwrap();
    let deck = document["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|card| card.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();

    for seed in 0_u64..16 {
        let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
            LuaCardRuntime::load_dir(data_path()).unwrap(),
            deck.clone(),
            deck.clone(),
            seed,
            ["HERO_03bp".to_owned(), "HERO_03bp".to_owned()],
            ["rogue".to_owned(), "rogue".to_owned()],
        )
        .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();

        for step in 0..120_usize {
            if game.state().outcome.is_some() {
                break;
            }
            let actions = game
                .legal_actions()
                .unwrap()
                .into_iter()
                .filter(|action| !matches!(action, PlayerCommand::Concede))
                .collect::<Vec<_>>();
            assert!(!actions.is_empty(), "seed {seed}, step {step}");
            let index = ((seed as usize).wrapping_mul(37) + step.wrapping_mul(17)) % actions.len();
            let command = actions[index].clone();
            game.dispatch(command.clone()).unwrap_or_else(|error| {
                panic!("legal command {command:?} failed for seed {seed}, step {step}: {error}")
            });
            game.state()
                .validate()
                .unwrap_or_else(|error| panic!("seed {seed}, step {step}: {error}"));
        }

        if seed == 0 {
            let replay = game.replay();
            let restored =
                Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
            assert_eq!(restored.state(), game.state());
        }
    }
}

#[test]
fn opening_coin_uses_the_official_id_and_replay_is_exact() {
    let mut game = game("CS2_120", "CS2_120");
    let coin = hand_card(&game, PlayerId::TWO, "GAME_005");
    end_turn(&mut game);
    game.dispatch(PlayerCommand::PlayCard {
        card: coin,
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().player(PlayerId::TWO).mana, 2);
    assert_eq!(game.state().player(PlayerId::TWO).temporary_mana, 1);

    let replay = game.replay();
    let restored =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn taunt_lua_module_controls_attack_priority() {
    let mut game = game("CS2_120", "CS2_125");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let attacker = play(&mut game, PlayerId::ONE, "CS2_120", None);
    end_turn(&mut game);
    let coin = play(&mut game, PlayerId::TWO, "GAME_005", None);
    assert_eq!(game.state().entity(coin).unwrap().zone, Zone::Graveyard);
    let taunt = play(&mut game, PlayerId::TWO, "CS2_125", None);
    end_turn(&mut game);

    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    let legal = game.legal_actions().unwrap();
    assert!(legal.contains(&PlayerCommand::Attack {
        attacker,
        defender: taunt,
    }));
    assert!(!legal.contains(&PlayerCommand::Attack {
        attacker,
        defender: enemy_hero,
    }));
}

#[test]
fn overload_lua_module_queues_its_parameter_and_locks_only_the_next_turn() {
    let mut game = game("EX1_250", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let earth_elemental = play(&mut game, PlayerId::ONE, "EX1_250", None);

    assert_eq!(game.state().player(PlayerId::ONE).overload_pending, 2);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::OverloadQueued {
            source,
            player: PlayerId::ONE,
            amount: 2,
        } if *source == earth_elemental
    )));

    let snapshot = game.snapshot();
    let encoded = serde_json::to_string(&snapshot).unwrap();
    let portable: hearth_core::GameSnapshot = serde_json::from_str(&encoded).unwrap();
    let restored =
        Game::from_snapshot(LuaCardRuntime::load_dir(data_path()).unwrap(), &portable).unwrap();
    assert_eq!(restored.state(), game.state());
    assert_eq!(
        restored.state().public_history(PlayerId::ONE),
        game.state().public_history(PlayerId::ONE)
    );

    end_turn(&mut game);
    end_turn(&mut game);
    let locked_turn = game.state().player(PlayerId::ONE);
    assert_eq!(locked_turn.max_mana, 6);
    assert_eq!(locked_turn.overload_pending, 0);
    assert_eq!(locked_turn.overloaded_mana, 2);
    assert_eq!(locked_turn.mana, 4);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::ManaLocked {
            player: PlayerId::ONE,
            amount: 2,
        }
    )));

    end_turn(&mut game);
    end_turn(&mut game);
    let following_turn = game.state().player(PlayerId::ONE);
    assert_eq!(following_turn.max_mana, 7);
    assert_eq!(following_turn.overloaded_mana, 0);
    assert_eq!(following_turn.mana, 7);

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn lava_shock_clears_current_and_pending_overload() {
    let deck_one = ["EX1_250", "EX1_238", "BRM_011"]
        .into_iter()
        .cycle()
        .take(30)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck_one, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "EX1_250", None);
    end_turn(&mut game);
    end_turn(&mut game);

    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "EX1_238", Some(enemy_hero));
    assert_eq!(game.state().player(PlayerId::ONE).overload_pending, 1);
    assert_eq!(game.state().player(PlayerId::ONE).overloaded_mana, 2);
    let lava_shock = play(&mut game, PlayerId::ONE, "BRM_011", Some(enemy_hero));

    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.overload_pending, 0);
    assert_eq!(player.overloaded_mana, 0);
    assert_eq!(player.mana, 3);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::OverloadCleared {
            source,
            player: PlayerId::ONE,
            pending: 1,
            locked: 2,
        } if *source == lava_shock
    )));

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn divine_shield_is_consumed_by_its_lua_trigger() {
    let mut game = game("EX1_008", "CS2_120");
    let squire = play(&mut game, PlayerId::ONE, "EX1_008", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);

    game.dispatch(PlayerCommand::Attack {
        attacker: squire,
        defender: crocolisk,
    })
    .unwrap();
    let squire_state = game.state().entity(squire).unwrap();
    assert_eq!(squire_state.health(), 1);
    assert!(!squire_state.has_keyword("divine_shield"));
    assert_eq!(game.state().entity(crocolisk).unwrap().health(), 2);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::KeywordDisabled { target, keyword, .. }
            if *target == squire && keyword == "divine_shield"
    )));
}

#[test]
fn charge_lua_module_readies_a_new_minion() {
    let mut game = game("AT_087", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let rider = play(&mut game, PlayerId::ONE, "AT_087", None);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    assert!(
        game.legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: rider,
                defender: enemy_hero,
            })
    );
}

#[test]
fn targeted_battlecry_can_be_played_without_a_target_only_when_none_exist() {
    let mut empty_board = game("EDR_468", "CS2_029");
    advance_to_mana(&mut empty_board, PlayerId::ONE, 4);
    let eggbasher = hand_card(&empty_board, PlayerId::ONE, "EDR_468");
    assert!(
        empty_board
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::PlayCard {
                card: eggbasher,
                target: None,
            })
    );
    empty_board
        .dispatch(PlayerCommand::PlayCard {
            card: eggbasher,
            target: None,
        })
        .unwrap();
    assert_eq!(
        empty_board.state().entity(eggbasher).unwrap().zone,
        Zone::Board
    );

    let mut with_target = game("EDR_468", "CS2_120");
    end_turn(&mut with_target);
    play(&mut with_target, PlayerId::TWO, "GAME_005", None);
    let crocolisk = play(&mut with_target, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut with_target, PlayerId::ONE, 4);
    let eggbasher = hand_card(&with_target, PlayerId::ONE, "EDR_468");
    assert!(
        !with_target
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::PlayCard {
                card: eggbasher,
                target: None,
            })
    );
    assert!(
        with_target
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::PlayCard {
                card: eggbasher,
                target: Some(crocolisk),
            })
    );
    assert!(
        with_target
            .dispatch(PlayerCommand::PlayCard {
                card: eggbasher,
                target: None,
            })
            .is_err()
    );
    with_target
        .dispatch(PlayerCommand::PlayCard {
            card: eggbasher,
            target: Some(crocolisk),
        })
        .unwrap();
}

#[test]
fn declared_battlecry_target_is_not_revalidated_after_hand_aura_changes() {
    let mut deck_one = vec!["EX1_005".to_owned()];
    deck_one.extend(std::iter::repeat_n("EX1_008".to_owned(), 6));
    let mut game = game_with_decks(deck_one, vec!["EX1_162".to_owned(), "GVG_095".to_owned()]);

    play(&mut game, PlayerId::ONE, "EX1_008", None);
    end_turn(&mut game);
    end_turn(&mut game);
    end_turn(&mut game);
    let wolf = play(&mut game, PlayerId::TWO, "EX1_162", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let sapper = play(&mut game, PlayerId::TWO, "GVG_095", None);
    end_turn(&mut game);

    assert_eq!(game.state().player(PlayerId::ONE).hand.len(), 6);
    assert_eq!(game.state().entity(sapper).unwrap().attack, 7);
    let hunter = hand_card(&game, PlayerId::ONE, "EX1_005");
    assert!(game.valid_targets(hunter).unwrap().contains(&sapper));
    game.dispatch(PlayerCommand::PlayCardAt {
        card: hunter,
        target: Some(sapper),
        position: 0,
    })
    .unwrap();

    assert_eq!(game.state().entity(wolf).unwrap().zone, Zone::Board);
    assert_eq!(game.state().entity(hunter).unwrap().zone, Zone::Board);
    assert_eq!(game.state().entity(sapper).unwrap().zone, Zone::Graveyard);
    assert!(
        !game
            .state()
            .log
            .iter()
            .any(|event| matches!(event, GameEvent::CardCountered { card, .. } if *card == hunter))
    );
}

#[test]
fn tradeable_keyword_adds_a_deterministic_non_play_action() {
    let mut deck = std::iter::repeat_n("CFM_305".to_owned(), 3).collect::<Vec<_>>();
    deck.extend(std::iter::repeat_n("EX1_005".to_owned(), 3));
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    play(&mut game, PlayerId::ONE, "CFM_305", None);
    let hunter = hand_card(&game, PlayerId::ONE, "EX1_005");
    assert_eq!(game.state().entity(hunter).unwrap().attack, 5);
    assert_eq!(game.state().entity(hunter).unwrap().health(), 3);
    assert_eq!(game.state().entity(hunter).unwrap().enchantments.len(), 1);
    assert_eq!(
        game.dispatch(PlayerCommand::TradeCard { card: hunter }),
        Err(GameError::NotEnoughMana {
            needed: 1,
            available: 0,
        })
    );
    end_turn(&mut game);
    end_turn(&mut game);
    assert!(
        game.legal_actions()
            .unwrap()
            .contains(&PlayerCommand::TradeCard { card: hunter })
    );
    let hand_size = game.state().player(PlayerId::ONE).hand.len();
    let deck_size = game.state().player(PlayerId::ONE).deck.len();
    let mana = game.state().player(PlayerId::ONE).mana;
    let random_counter = game.state().random_counter;
    let log_start = game.state().log.len();

    game.dispatch(PlayerCommand::TradeCard { card: hunter })
        .unwrap();

    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.mana, mana - 1);
    assert_eq!(player.cards_played_this_turn, 0);
    assert_eq!(player.hand.len(), hand_size);
    assert_eq!(player.deck.len(), deck_size);
    assert!(!player.hand.contains(&hunter));
    assert!(player.deck.contains(&hunter));
    let traded = game.state().entity(hunter).unwrap();
    assert_eq!(traded.zone, Zone::Deck);
    assert_eq!(traded.attack, 5);
    assert_eq!(traded.health(), 3);
    assert_eq!(traded.enchantments.len(), 1);
    assert_eq!(game.state().random_counter, random_counter + 1);

    let events = &game.state().log[log_start..];
    let drawn = events
        .iter()
        .position(|event| matches!(event, GameEvent::CardDrawn { .. }))
        .unwrap();
    let traded = events
        .iter()
        .position(|event| matches!(event, GameEvent::CardTraded { card, .. } if *card == hunter))
        .unwrap();
    assert!(drawn < traded);

    let replay = game.replay();
    let restored =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn trade_requires_a_lua_rule_mana_and_a_nonempty_deck() {
    let mut ordinary = game("CS2_120", "CS2_120");
    let crocolisk = hand_card(&ordinary, PlayerId::ONE, "CS2_120");
    assert!(
        !ordinary
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::TradeCard { card: crocolisk })
    );
    assert_eq!(
        ordinary.dispatch(PlayerCommand::TradeCard { card: crocolisk }),
        Err(GameError::CardNotTradeable(crocolisk))
    );

    let mut empty = game_with_decks(vec!["EX1_005".to_owned()], repeated("CS2_120"));
    let hunter = hand_card(&empty, PlayerId::ONE, "EX1_005");
    assert!(
        !empty
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::TradeCard { card: hunter })
    );
    assert_eq!(
        empty.dispatch(PlayerCommand::TradeCard { card: hunter }),
        Err(GameError::EmptyDeck(PlayerId::ONE))
    );
}

#[test]
fn auctioneer_jaxon_replaces_trade_draw_entirely_from_lua() {
    let mut deck = std::iter::repeat_n("SW_045".to_owned(), 3).collect::<Vec<_>>();
    deck.extend(std::iter::repeat_n("EX1_005".to_owned(), 3));
    deck.extend(
        ["CS2_120", "EX1_008", "CS2_125", "EX1_096"]
            .into_iter()
            .map(str::to_owned),
    );
    let mut game = game_with_decks(deck, repeated("CS2_120"));

    end_turn(&mut game);
    end_turn(&mut game);
    let jaxon = play(&mut game, PlayerId::ONE, "SW_045", None);
    end_turn(&mut game);
    end_turn(&mut game);

    let hunter = hand_card(&game, PlayerId::ONE, "EX1_005");
    let original_top = *game.state().player(PlayerId::ONE).deck.front().unwrap();
    let log_start = game.state().log.len();
    game.dispatch(PlayerCommand::TradeCard { card: hunter })
        .unwrap();

    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(pending.source, jaxon);
    assert_eq!(pending.player, PlayerId::ONE);
    assert!((1..=3).contains(&pending.options.len()));
    let option_count = pending.options.len();
    let option_card_ids = pending
        .options
        .iter()
        .filter_map(|option| match &option.value {
            ChoiceValue::Entity(entity) => {
                Some(game.state().entity(*entity).unwrap().card_id.as_str())
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(option_card_ids.len(), option_count);
    let public_choice = game
        .state()
        .player_view(PlayerId::ONE)
        .pending_input
        .unwrap();
    assert_eq!(public_choice.options.len(), option_count);
    assert!(public_choice.options.iter().all(|option| {
        option.to_string() == option.label
            && matches!(option.value, ChoiceOptionValueView::Entity(_))
    }));
    assert!(
        public_choice
            .options
            .iter()
            .all(|option| !option.label.contains('['))
    );
    let (choice_index, selected) = pending
        .options
        .iter()
        .enumerate()
        .find_map(|(index, option)| match &option.value {
            ChoiceValue::Entity(entity) if *entity != original_top => Some((index, *entity)),
            _ => None,
        })
        .unwrap();
    assert_eq!(game.state().entity(hunter).unwrap().zone, Zone::SetAside);
    let snapshot = game.snapshot();
    let restored_pending =
        Game::from_snapshot(LuaCardRuntime::load_dir(data_path()).unwrap(), &snapshot).unwrap();
    assert_eq!(restored_pending.state(), game.state());

    game.dispatch(PlayerCommand::Choose {
        index: choice_index,
    })
    .unwrap();

    let player = game.state().player(PlayerId::ONE);
    assert!(game.state().pending_input.is_none());
    assert!(player.hand.contains(&selected));
    assert!(player.deck.contains(&hunter));
    assert!(player.deck.contains(&original_top));
    assert_eq!(game.state().entity(selected).unwrap().zone, Zone::Hand);
    assert_eq!(game.state().entity(hunter).unwrap().zone, Zone::Deck);
    let events = &game.state().log[log_start..];
    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::RandomEntitiesSampled {
            source,
            entities,
            population: 4,
        } if *source == jaxon && entities.len() == option_count
    )));
    let drawn = events
        .iter()
        .position(|event| matches!(event, GameEvent::CardDrawn { card, .. } if *card == selected))
        .unwrap();
    let trade_draw = events
        .iter()
        .position(|event| {
            matches!(
                event,
                GameEvent::TradeDraw {
                    replacement: Some(entity),
                    ..
                } if *entity == selected
            )
        })
        .unwrap();
    let traded = events
        .iter()
        .position(|event| matches!(event, GameEvent::CardTraded { card, .. } if *card == hunter))
        .unwrap();
    assert!(drawn < trade_draw && trade_draw < traded);

    let replay = game.replay();
    let restored =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn rush_lua_module_allows_only_minion_targets_on_the_first_turn() {
    let mut game = game("GIL_143", "CS2_120");
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let enemy = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    let scalehide = play(&mut game, PlayerId::ONE, "GIL_143", None);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    let legal = game.legal_actions().unwrap();
    assert!(legal.contains(&PlayerCommand::Attack {
        attacker: scalehide,
        defender: enemy,
    }));
    assert!(!legal.contains(&PlayerCommand::Attack {
        attacker: scalehide,
        defender: enemy_hero,
    }));
}

#[test]
fn stealth_lua_module_filters_targets_and_breaks_after_attack() {
    let mut game = game("EX1_010", "CS2_029");
    let infiltrator = play(&mut game, PlayerId::ONE, "EX1_010", None);
    end_turn(&mut game);
    let fireball = hand_card(&game, PlayerId::TWO, "CS2_029");
    assert!(!game.valid_targets(fireball).unwrap().contains(&infiltrator));
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: infiltrator,
        defender: enemy_hero,
    })
    .unwrap();
    assert!(
        !game
            .state()
            .entity(infiltrator)
            .unwrap()
            .has_keyword("stealth")
    );
}

#[test]
fn lifesteal_lua_trigger_heals_after_damage() {
    let mut game = game("GIL_143", "CS2_120");
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    let scalehide = play(&mut game, PlayerId::ONE, "GIL_143", None);
    game.dispatch(PlayerCommand::Attack {
        attacker: scalehide,
        defender: crocolisk,
    })
    .unwrap();
    end_turn(&mut game);
    let hero = game.state().player(PlayerId::ONE).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: crocolisk,
        defender: hero,
    })
    .unwrap();
    assert_eq!(game.state().entity(hero).unwrap().health(), 28);
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: scalehide,
        defender: enemy_hero,
    })
    .unwrap();
    assert_eq!(game.state().entity(hero).unwrap().health(), 29);
}

#[test]
fn windfury_lua_rule_allows_exactly_two_attacks() {
    let mut game = game("EX1_033", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let harpy = play(&mut game, PlayerId::ONE, "EX1_033", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    for _ in 0..2 {
        game.dispatch(PlayerCommand::Attack {
            attacker: harpy,
            defender: enemy_hero,
        })
        .unwrap();
    }
    assert!(
        !game
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: harpy,
                defender: enemy_hero,
            })
    );
}

#[test]
fn poisonous_lua_trigger_destroys_a_damaged_minion() {
    let mut game = game("WW_376", "BOT_309");
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let framebot = play(&mut game, PlayerId::TWO, "BOT_309", None);
    end_turn(&mut game);
    let cactus = play(&mut game, PlayerId::ONE, "WW_376", None);
    end_turn(&mut game);
    end_turn(&mut game);
    game.dispatch(PlayerCommand::Attack {
        attacker: cactus,
        defender: framebot,
    })
    .unwrap();
    assert_eq!(game.state().entity(framebot).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().entity(cactus).unwrap().zone, Zone::Graveyard);
}

#[test]
fn reborn_is_entirely_driven_by_the_keyword_lua_file() {
    let mut game = game("ULD_208", "CS2_029");
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let original = play(&mut game, PlayerId::ONE, "ULD_208", None);
    end_turn(&mut game);
    let fireball = hand_card(&game, PlayerId::TWO, "CS2_029");
    game.dispatch(PlayerCommand::PlayCard {
        card: fireball,
        target: Some(original),
    })
    .unwrap();

    assert_eq!(game.state().entity(original).unwrap().zone, Zone::Graveyard);
    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 1);
    let reborn = game.state().entity(board[0]).unwrap();
    assert_eq!(reborn.card_id, "ULD_208");
    assert_eq!(reborn.health(), 1);
    assert!(!reborn.has_keyword("reborn"));
}

#[test]
fn elusive_lua_rule_filters_spell_targets() {
    let deck = ["DRG_079", "EX1_332"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let wyrm = play(&mut game, PlayerId::ONE, "DRG_079", None);
    let friendly_silence = hand_card(&game, PlayerId::ONE, "EX1_332");
    assert!(
        !game
            .valid_targets(friendly_silence)
            .unwrap()
            .contains(&wyrm)
    );
    end_turn(&mut game);
    let fireball = hand_card(&game, PlayerId::TWO, "CS2_029");
    assert!(!game.valid_targets(fireball).unwrap().contains(&wyrm));
}

#[test]
fn freeze_survives_until_the_frozen_characters_next_turn_ends() {
    let mut game = game("CS2_120", "BT_714");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let crocolisk = play(&mut game, PlayerId::ONE, "CS2_120", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: crocolisk,
        defender: enemy_hero,
    })
    .unwrap();
    end_turn(&mut game);
    advance_to_mana(&mut game, PlayerId::TWO, 3);
    play(&mut game, PlayerId::TWO, "BT_714", Some(crocolisk));
    assert!(game.state().entity(crocolisk).unwrap().frozen);
    end_turn(&mut game);
    assert!(!game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::Attack { attacker, .. } if *attacker == crocolisk
    )));
    end_turn(&mut game);
    end_turn(&mut game);
    assert!(game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::Attack { attacker, .. } if *attacker == crocolisk
    )));
}

#[test]
fn official_battlecry_and_token_ids_are_used() {
    let mut game = game("OG_156", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let tidehunter = play(&mut game, PlayerId::ONE, "OG_156", None);
    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 2);
    let ooze = game.state().entity(board[1]).unwrap();
    assert_eq!(ooze.card_id, "OG_156a");
    assert!(ooze.has_keyword("taunt"));
    assert!(
        game.state()
            .entity(tidehunter)
            .unwrap()
            .has_keyword("battlecry")
    );

    let ooze_summoned = game
        .state()
        .log
        .iter()
        .position(
            |event| matches!(event, GameEvent::MinionSummoned { entity, .. } if *entity == ooze.id),
        )
        .unwrap();
    let tidehunter_played = game
        .state()
        .log
        .iter()
        .position(
            |event| matches!(event, GameEvent::CardPlayed { card, .. } if *card == tidehunter),
        )
        .unwrap();
    let tidehunter_summoned = game
        .state()
        .log
        .iter()
        .position(|event| matches!(event, GameEvent::MinionSummoned { entity, .. } if *entity == tidehunter))
        .unwrap();
    assert!(ooze_summoned < tidehunter_played);
    assert!(tidehunter_played < tidehunter_summoned);
}

#[test]
fn battlecry_choice_pauses_before_after_play_and_survives_snapshot() {
    let mut game = game("BAR_065", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let scorpid = play(&mut game, PlayerId::ONE, "BAR_065", None);

    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(pending.source, scorpid);
    assert_eq!(pending.player, PlayerId::ONE);
    assert_eq!(pending.options.len(), 3);
    assert!(
        !game
            .state()
            .log
            .iter()
            .any(|event| matches!(event, GameEvent::CardPlayed { card, .. } if *card == scorpid))
    );
    assert!(!game.state().log.iter().any(
        |event| matches!(event, GameEvent::MinionSummoned { entity, .. } if *entity == scorpid)
    ));

    let snapshot = game.snapshot();
    let restored_pending =
        Game::from_snapshot(LuaCardRuntime::load_dir(data_path()).unwrap(), &snapshot).unwrap();
    assert_eq!(restored_pending.state(), game.state());

    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert!(game.state().pending_input.is_none());
    let card_played = game
        .state()
        .log
        .iter()
        .position(|event| matches!(event, GameEvent::CardPlayed { card, .. } if *card == scorpid))
        .unwrap();
    let minion_summoned = game
        .state()
        .log
        .iter()
        .position(
            |event| matches!(event, GameEvent::MinionSummoned { entity, .. } if *entity == scorpid),
        )
        .unwrap();
    assert!(card_played < minion_summoned);

    let replay = game.replay();
    let restored =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn finale_keyword_triggers_only_after_spending_all_remaining_mana() {
    let mut exact_mana = game("ETC_088", "CS2_120");
    advance_to_mana(&mut exact_mana, PlayerId::ONE, 5);
    let hand_before = exact_mana.state().player(PlayerId::ONE).hand.len();
    let writer = play(&mut exact_mana, PlayerId::ONE, "ETC_088", None);
    assert!(
        exact_mana
            .state()
            .entity(writer)
            .unwrap()
            .has_keyword("finale")
    );
    assert_eq!(exact_mana.state().player(PlayerId::ONE).mana, 0);

    let first = exact_mana.state().pending_input.as_ref().unwrap();
    assert_eq!(first.source, writer);
    assert_eq!(first.prompt, "Discover a spell");
    assert_eq!(first.options.len(), 3);
    exact_mana
        .dispatch(PlayerCommand::Choose { index: 0 })
        .unwrap();

    let second = exact_mana.state().pending_input.as_ref().unwrap();
    assert_eq!(second.source, writer);
    assert_eq!(second.prompt, "Finale: Discover another spell");
    assert_eq!(second.options.len(), 3);
    let snapshot = exact_mana.snapshot();
    let restored =
        Game::from_snapshot(LuaCardRuntime::load_dir(data_path()).unwrap(), &snapshot).unwrap();
    assert_eq!(restored.state(), exact_mana.state());

    exact_mana
        .dispatch(PlayerCommand::Choose { index: 0 })
        .unwrap();
    assert!(exact_mana.state().pending_input.is_none());
    assert_eq!(
        exact_mana
            .state()
            .log
            .iter()
            .filter(
                |event| matches!(event, GameEvent::CardCreated { source, .. } if *source == writer)
            )
            .count(),
        2
    );
    assert!(exact_mana.state().player(PlayerId::ONE).hand.len() > hand_before);
    let replay = exact_mana.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), exact_mana.state());

    let mut spare_mana = game("ETC_088", "CS2_120");
    advance_to_mana(&mut spare_mana, PlayerId::ONE, 6);
    play(&mut spare_mana, PlayerId::ONE, "ETC_088", None);
    assert_eq!(spare_mana.state().player(PlayerId::ONE).mana, 1);
    assert_eq!(
        spare_mana.state().pending_input.as_ref().unwrap().prompt,
        "Discover a spell"
    );
    spare_mana
        .dispatch(PlayerCommand::Choose { index: 0 })
        .unwrap();
    assert!(spare_mana.state().pending_input.is_none());
}

#[test]
fn discover_spell_pool_is_filtered_by_replayable_player_class_in_lua() {
    let mut game = game_with_classes(repeated("ETC_088"), repeated("CS2_120"), ["mage", "shaman"]);
    assert_eq!(game.state().player(PlayerId::ONE).class, "mage");
    assert_eq!(game.state().player(PlayerId::TWO).class, "shaman");
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let writer = play(&mut game, PlayerId::ONE, "ETC_088", None);

    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(pending.source, writer);
    assert_eq!(pending.options.len(), 3);
    for option in &pending.options {
        let ChoiceValue::Card(card_id) = &option.value else {
            panic!("spell discovery returned a non-card option");
        };
        let definition = game.runtime().definition(card_id).unwrap();
        assert_eq!(definition.kind, CardKind::Spell);
        assert!(matches!(definition.class.as_str(), "mage" | "neutral"));
        assert_ne!(card_id, "EX1_238");
    }

    let snapshot = game.snapshot();
    assert_eq!(snapshot.replay.classes, ["mage", "shaman"]);
    let restored =
        Game::from_snapshot(LuaCardRuntime::load_dir(data_path()).unwrap(), &snapshot).unwrap();
    assert_eq!(restored.state(), game.state());

    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert_eq!(
        game.state().pending_input.as_ref().unwrap().prompt,
        "Finale: Discover another spell"
    );
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    let replay = game.replay();
    assert_eq!(replay.classes, ["mage", "shaman"]);
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn wild_growth_creates_official_excess_mana_at_the_crystal_cap() {
    let mut game = game("CS2_013", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    play(&mut game, PlayerId::ONE, "CS2_013", None);

    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.max_mana, 10);
    assert_eq!(player.hand.len(), hand_before);
    let excess = hand_card(&game, PlayerId::ONE, "CS2_013t");
    assert!(
        !LuaCardRuntime::load_dir(data_path())
            .unwrap()
            .definition("CS2_013t")
            .unwrap()
            .collectible
    );

    let deck_before = game.state().player(PlayerId::ONE).deck.len();
    game.dispatch(PlayerCommand::PlayCard {
        card: excess,
        target: None,
    })
    .unwrap();
    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.deck.len(), deck_before - 1);
    assert_eq!(player.hand.len(), hand_before);
    assert_eq!(game.state().entity(excess).unwrap().zone, Zone::Graveyard);

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn combo_keyword_requires_an_earlier_card_and_routes_the_declared_target() {
    let mut game = game("EX1_134", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    let starting_health = game.state().entity(enemy_hero).unwrap().health();

    let first = hand_card(&game, PlayerId::ONE, "EX1_134");
    assert!(game.valid_targets(first).unwrap().is_empty());
    assert!(
        game.dispatch(PlayerCommand::PlayCard {
            card: first,
            target: Some(enemy_hero),
        })
        .is_err()
    );
    game.dispatch(PlayerCommand::PlayCard {
        card: first,
        target: None,
    })
    .unwrap();
    assert_eq!(
        game.state().entity(enemy_hero).unwrap().health(),
        starting_health
    );
    assert_eq!(game.state().entity(first).unwrap().cards_played_before, 0);

    let second = hand_card(&game, PlayerId::ONE, "EX1_134");
    assert!(game.valid_targets(second).unwrap().contains(&enemy_hero));
    assert!(
        game.dispatch(PlayerCommand::PlayCard {
            card: second,
            target: None,
        })
        .is_err()
    );
    let snapshot = game.snapshot();
    let restored =
        Game::from_snapshot(LuaCardRuntime::load_dir(data_path()).unwrap(), &snapshot).unwrap();
    assert_eq!(restored.state(), game.state());

    game.dispatch(PlayerCommand::PlayCard {
        card: second,
        target: Some(enemy_hero),
    })
    .unwrap();
    assert_eq!(
        game.state().entity(enemy_hero).unwrap().health(),
        starting_health - 3
    );
    assert_eq!(game.state().entity(second).unwrap().cards_played_before, 1);

    let replay = game.replay();
    let restored =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn edwin_combo_scales_from_the_frozen_pre_play_count() {
    let mut game = game("EX1_613", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 9);

    let first = play(&mut game, PlayerId::ONE, "EX1_613", None);
    let second = play(&mut game, PlayerId::ONE, "EX1_613", None);
    let third = play(&mut game, PlayerId::ONE, "EX1_613", None);

    for (entity, cards_before, attack, health) in
        [(first, 0, 2, 2), (second, 1, 4, 4), (third, 2, 6, 6)]
    {
        let entity = game.state().entity(entity).unwrap();
        assert_eq!(entity.cards_played_before, cards_before);
        assert_eq!(entity.attack, attack);
        assert_eq!(entity.health(), health);
        assert!(entity.has_keyword("combo"));
    }

    let replay = game.replay();
    let restored =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn battlecry_summon_is_inserted_immediately_to_the_right() {
    let mut game = game("OG_156", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let first = play(&mut game, PlayerId::ONE, "OG_156", None);
    let first_ooze = game.state().player(PlayerId::ONE).board[1];
    end_turn(&mut game);
    end_turn(&mut game);
    let second = hand_card(&game, PlayerId::ONE, "OG_156");
    game.dispatch(PlayerCommand::PlayCardAt {
        card: second,
        target: None,
        position: 0,
    })
    .unwrap();
    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board[0], second);
    assert_eq!(game.state().entity(board[1]).unwrap().card_id, "OG_156a");
    assert_eq!(board[2], first);
    assert_eq!(board[3], first_ooze);
}

#[test]
fn deathrattle_keyword_calls_card_lua_and_summons_official_tokens() {
    let mut game = game("FP1_002", "CS2_029");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let creeper = play(&mut game, PlayerId::ONE, "FP1_002", None);
    assert!(
        game.state()
            .entity(creeper)
            .unwrap()
            .has_keyword("deathrattle")
    );
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(creeper));
    let spiders = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "FP1_002t")
        .count();
    assert_eq!(spiders, 2);
}

#[test]
fn deathrattle_tokens_use_the_remembered_death_position() {
    let mut game = game("FP1_002", "CS2_029");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let first = play(&mut game, PlayerId::ONE, "FP1_002", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let second = play(&mut game, PlayerId::ONE, "FP1_002", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(first));

    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 3);
    assert_eq!(game.state().entity(board[0]).unwrap().card_id, "FP1_002t");
    assert_eq!(game.state().entity(board[1]).unwrap().card_id, "FP1_002t");
    assert_eq!(board[2], second);
}

#[test]
fn spellburst_keyword_owns_the_one_shot_trigger_and_calls_card_lua() {
    let mut game = game("CS2_120", "SCH_231");
    end_turn(&mut game);
    let initiate = play(&mut game, PlayerId::TWO, "SCH_231", None);
    assert!(
        game.state()
            .entity(initiate)
            .unwrap()
            .has_keyword("spellburst")
    );
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    assert_eq!(game.state().entity(initiate).unwrap().attack, 3);
    assert!(
        !game
            .state()
            .entity(initiate)
            .unwrap()
            .has_keyword("spellburst")
    );
    assert!(
        game.state()
            .entity(initiate)
            .unwrap()
            .script_data
            .is_empty()
    );
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::KeywordDisabled { target, keyword, .. }
            if *target == initiate && keyword == "spellburst"
    )));
}

#[test]
fn spell_damage_keyword_adds_to_spells_and_silence_removes_it() {
    let deck_one = ["ICC_913", "CS2_029"]
        .into_iter()
        .cycle()
        .take(30)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck_one, repeated("EX1_332"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let zealot = play(&mut game, PlayerId::ONE, "ICC_913", None);
    assert!(
        game.state()
            .entity(zealot)
            .unwrap()
            .has_keyword("spell_damage")
    );
    assert_eq!(game.state().entity(zealot).unwrap().spell_damage, 1);

    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "CS2_029", Some(enemy_hero));
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 23);

    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "EX1_332", Some(zealot));
    assert!(
        !game
            .state()
            .entity(zealot)
            .unwrap()
            .has_keyword("spell_damage")
    );
    assert_eq!(game.state().entity(zealot).unwrap().spell_damage, 0);
    end_turn(&mut game);
    play(&mut game, PlayerId::ONE, "CS2_029", Some(enemy_hero));
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 17);

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn pen_flinger_spellburst_returns_to_hand_and_refreshes_the_keyword() {
    let deck_one = (0..20)
        .map(|index| {
            if index % 2 == 0 {
                "SCH_248".to_owned()
            } else {
                "EX1_169".to_owned()
            }
        })
        .collect();
    let mut game = game_with_decks(deck_one, repeated("CS2_120"));
    let flinger = play(&mut game, PlayerId::ONE, "SCH_248", None);
    assert_eq!(game.state().entity(flinger).unwrap().zone, Zone::Board);

    play(&mut game, PlayerId::ONE, "EX1_169", None);

    let flinger = game.state().entity(flinger).unwrap();
    assert_eq!(flinger.zone, Zone::Hand);
    assert!(flinger.has_keyword("spellburst"));
    assert!(flinger.disabled_keywords.is_empty());
    assert!(flinger.script_data.is_empty());
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .contains(&flinger.id)
    );

    let replay = game.replay();
    let restored =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn counterspell_secret_and_counter_rules_come_from_lua_keywords() {
    let mut game = game("EX1_287", "CS2_029");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let secret = play(&mut game, PlayerId::ONE, "EX1_287", None);
    assert_eq!(game.state().entity(secret).unwrap().zone, Zone::Secret);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let hero = game.state().player(PlayerId::ONE).hero;
    let health = game.state().entity(hero).unwrap().health();
    play(&mut game, PlayerId::TWO, "CS2_029", Some(hero));
    assert_eq!(game.state().entity(hero).unwrap().health(), health);
    assert_eq!(game.state().entity(secret).unwrap().zone, Zone::Graveyard);
}

#[test]
fn duplicate_persistent_cards_cannot_enter_the_secret_zone_from_hand() {
    let mut game = game("EX1_287", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let first = play(&mut game, PlayerId::ONE, "EX1_287", None);
    assert_eq!(game.state().entity(first).unwrap().zone, Zone::Secret);
    let duplicate = hand_card(&game, PlayerId::ONE, "EX1_287");
    assert!(!game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard { card, .. } | PlayerCommand::PlayCardAt { card, .. }
            if *card == duplicate
    )));

    let before = game.state().clone();
    assert_eq!(
        game.dispatch(PlayerCommand::PlayCard {
            card: duplicate,
            target: None,
        }),
        Err(GameError::CardCannotBePlayed(duplicate))
    );
    assert_eq!(game.state(), &before);
}
