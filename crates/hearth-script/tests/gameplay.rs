use std::collections::BTreeMap;
use std::path::PathBuf;

use hearth_core::{
    CardKind, CardRuntime, ChoiceOptionValueView, ChoiceValue, DEFAULT_HERO_POWER, Game, GameError,
    GameEvent, Locale, PlayerCommand, PlayerId, PublicEvent, RuneCost, Zone,
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
fn etc_band_manager_consumes_a_private_replayable_sideboard_card() {
    let sideboard = BTreeMap::from([(
        "ETC_080".to_owned(),
        ["EX1_008", "CS2_171", "CS2_120"]
            .map(str::to_owned)
            .to_vec(),
    )]);
    let mut game = Game::new_with_sideboards_hero_powers_classes_and_starting_player(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        repeated("ETC_080"),
        repeated("CS2_120"),
        [sideboard.clone(), BTreeMap::new()],
        7,
        [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()],
        ["mage".to_owned(), "mage".to_owned()],
        PlayerId::ONE,
        true,
    )
    .unwrap();

    assert_eq!(
        game.state()
            .player_view(PlayerId::ONE)
            .player(PlayerId::ONE)
            .sideboards,
        sideboard
    );
    assert!(
        game.state()
            .player_view(PlayerId::TWO)
            .player(PlayerId::ONE)
            .sideboards
            .is_empty()
    );

    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "ETC_080", None);
    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(pending.options.len(), 3);
    let index = pending
        .options
        .iter()
        .position(
            |option| matches!(&option.value, ChoiceValue::Card(card_id) if card_id == "EX1_008"),
        )
        .unwrap();
    game.dispatch(PlayerCommand::Choose { index }).unwrap();

    let argent_squire = hand_card(&game, PlayerId::ONE, "EX1_008");
    assert!(game.state().entity(argent_squire).unwrap().started_in_deck);
    assert_eq!(
        game.state().player(PlayerId::ONE).sideboards["ETC_080"],
        ["CS2_171", "CS2_120"].map(str::to_owned)
    );

    let replay = game.replay();
    assert_eq!(replay.sideboards[0], sideboard);
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn prince_renathal_grants_forty_card_capacity_and_starting_health() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let Err(too_large) = Game::new_unrestricted(
        runtime,
        std::iter::repeat_n("CS2_120".to_owned(), 31).collect(),
        repeated("CS2_120"),
        7,
    ) else {
        panic!("a 31-card deck without a modifier must be rejected");
    };
    assert!(matches!(
        too_large,
        GameError::DeckTooLarge {
            player: PlayerId::ONE,
            cards: 31,
            maximum: 30,
        }
    ));

    let mut game = Game::new_unrestricted(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        std::iter::repeat_n("REV_018".to_owned(), 40).collect(),
        repeated("CS2_120"),
        7,
    )
    .unwrap();
    let hero = game.state().player(PlayerId::ONE).hero;
    assert_eq!(game.state().entity(hero).unwrap().health(), 40);
    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.starting_deck.len(), 40);
    assert_eq!(player.deck.len() + player.hand.len(), 40);

    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    assert_eq!(game.state().entity(hero).unwrap().health(), 40);

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
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
fn the_darkness_keeps_its_battlecry_owner_when_summon_transforms_it() {
    let mut game = game_with_decks(mixed(&["LOOT_526", "CS2_120"]), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 4);

    let darkness = play(&mut game, PlayerId::ONE, "LOOT_526", None);

    assert_eq!(game.state().entity(darkness).unwrap().card_id, "LOOT_526d");
    assert_eq!(
        deck_ids(&game, PlayerId::TWO)
            .iter()
            .filter(|card_id| card_id.as_str() == "LOOT_526t")
            .count(),
        3
    );
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
    manifested_keywords.insert("RLK_008t".to_owned(), "no_corpse".to_owned());
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
        let displayed_spell_damage = record["text"]
            .as_str()
            .and_then(|text| text.strip_prefix("<b>Spell Damage +"))
            .and_then(|text| text.split("</b>").next())
            .and_then(|amount| amount.parse::<i64>().ok());
        let expected_spell_damage = if definition.id == "LOE_051" {
            // Client data carries the legacy default value 1, but the card's
            // authoritative text is a symmetric player aura of +2.
            0
        } else if let Some(amount) = displayed_spell_damage {
            // Some current client records retain the legacy numeric tag 1
            // after their authoritative displayed text was changed.
            amount
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
        let expected_rarity = record["rarity"].as_str().map(str::to_ascii_lowercase);
        if definition.rarity != expected_rarity {
            metadata_mismatches.push(format!(
                "{} rarity: {:?} != {:?}",
                definition.id, definition.rarity, expected_rarity
            ));
        }
        let expected_spell_school = record["spellSchool"].as_str().map(str::to_ascii_lowercase);
        if definition.spell_school != expected_spell_school {
            metadata_mismatches.push(format!(
                "{} spell school: {:?} != {:?}",
                definition.id, definition.spell_school, expected_spell_school
            ));
        }
        let expected_rune_cost = RuneCost {
            blood: record["runeCost"]["blood"].as_u64().unwrap_or(0) as u8,
            frost: record["runeCost"]["frost"].as_u64().unwrap_or(0) as u8,
            unholy: record["runeCost"]["unholy"].as_u64().unwrap_or(0) as u8,
        };
        if definition.rune_cost != expected_rune_cost {
            metadata_mismatches.push(format!(
                "{} rune_cost: {:?} != {:?}",
                definition.id, definition.rune_cost, expected_rune_cost
            ));
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
            ("COLOSSAL", "colossal"),
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
            ("TITAN", "titan"),
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
            .chain(
                record["text"]
                    .as_str()
                    .filter(|text| text.contains("Doesn't leave a <b>Corpse</b>"))
                    .map(|_| "no_corpse".to_owned()),
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
            .filter(|keyword| keyword.as_str() != "death_knight_corpses")
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
fn core_game_construction_enforces_three_death_knight_rune_slots() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let hero_powers = [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()];
    let classes = ["death_knight".to_owned(), "mage".to_owned()];
    let valid = Game::new_with_hero_powers_and_classes(
        runtime,
        mixed(&["RLK_067", "RLK_048"]),
        repeated("CS2_120"),
        107,
        hero_powers.clone(),
        classes.clone(),
    );
    assert!(valid.is_ok(), "two Blood plus one Unholy is legal");

    let invalid = Game::new_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        mixed(&["RLK_067", "RLK_063"]),
        repeated("CS2_120"),
        109,
        hero_powers.clone(),
        classes.clone(),
    );
    assert!(matches!(
        invalid,
        Err(GameError::InvalidDeckRunes {
            player: PlayerId::ONE,
            total: 5,
            blood: 2,
            frost: 3,
            unholy: 0,
        })
    ));

    let sideboard_invalid = Game::new_with_sideboards_hero_powers_classes_and_starting_player(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        mixed(&["ETC_080", "RLK_067"]),
        repeated("CS2_120"),
        [
            BTreeMap::from([(
                "ETC_080".to_owned(),
                ["RLK_063", "CS2_120", "CS2_171"]
                    .map(str::to_owned)
                    .to_vec(),
            )]),
            BTreeMap::new(),
        ],
        109,
        hero_powers.clone(),
        classes.clone(),
        PlayerId::ONE,
        false,
    );
    assert!(matches!(
        sideboard_invalid,
        Err(GameError::InvalidDeckRunes {
            player: PlayerId::ONE,
            total: 5,
            ..
        })
    ));

    let unrestricted = Game::new_unrestricted_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        mixed(&["RLK_067", "RLK_063"]),
        repeated("CS2_120"),
        109,
        hero_powers,
        classes,
    );
    assert!(
        unrestricted.is_ok(),
        "mechanics sandboxes explicitly bypass constructed rune limits"
    );
}

#[test]
fn death_knight_rune_cards_apply_their_printed_gameplay() {
    let mut shell_game = game_with_decks(mixed(&["CS2_120", "RLK_048"]), repeated("CS2_120"));
    advance_to_mana(&mut shell_game, PlayerId::ONE, 5);
    let minion = play(&mut shell_game, PlayerId::ONE, "CS2_120", None);
    play(&mut shell_game, PlayerId::ONE, "RLK_048", None);
    let minion_state = shell_game.state().entity(minion).unwrap();
    assert_eq!((minion_state.attack, minion_state.health()), (3, 4));
    assert!(minion_state.has_keyword("elusive"));

    let mut strike_game = game("RLK_024", "CS2_120");
    advance_to_mana(&mut strike_game, PlayerId::TWO, 2);
    let victim = play(&mut strike_game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut strike_game, PlayerId::ONE, 4);
    play(&mut strike_game, PlayerId::ONE, "RLK_024", Some(victim));
    assert_eq!(
        strike_game.state().entity(victim).unwrap().zone,
        Zone::Graveyard
    );

    let mut weapon_game = game("RLK_067", "CS2_120");
    advance_to_mana(&mut weapon_game, PlayerId::ONE, 6);
    let ashbringer = play(&mut weapon_game, PlayerId::ONE, "RLK_067", None);
    assert_eq!(
        weapon_game.state().player(PlayerId::ONE).weapon,
        Some(ashbringer)
    );
    assert!(
        weapon_game
            .state()
            .entity(ashbringer)
            .unwrap()
            .has_keyword("lifesteal")
    );

    let mut fury_game = game("RLK_063", "CS2_120");
    advance_to_mana(&mut fury_game, PlayerId::TWO, 4);
    let first = play(&mut fury_game, PlayerId::TWO, "CS2_120", None);
    let second = play(&mut fury_game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut fury_game, PlayerId::ONE, 7);
    let enemy_hero = fury_game.state().player(PlayerId::TWO).hero;
    play(&mut fury_game, PlayerId::ONE, "RLK_063", Some(enemy_hero));
    assert_eq!(fury_game.state().entity(enemy_hero).unwrap().health(), 25);
    assert!(fury_game.state().entity(first).unwrap().frozen);
    assert!(fury_game.state().entity(second).unwrap().frozen);
    assert!(
        fury_game
            .state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .any(|entity| fury_game.state().entity(*entity).unwrap().card_id == "RLK_063t")
    );

    let replay = fury_game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), fury_game.state());
}

#[test]
fn death_knight_corpses_cover_deaths_public_views_spending_tokens_and_replay() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_060"]),
        repeated("CS2_029"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let bagger = play(&mut game, PlayerId::ONE, "RLK_503", None);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 1);

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(bagger));
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 2);
    for viewer in [PlayerId::ONE, PlayerId::TWO] {
        let view = game.state().player_view(viewer);
        assert_eq!(view.player(PlayerId::ONE).resource("corpses"), 2);
        assert_eq!(view.player(PlayerId::ONE).resource_spent("corpses"), 0);
    }
    assert!(
        game.state()
            .public_history(PlayerId::TWO)
            .iter()
            .any(|record| {
                matches!(
                    &record.event,
                    PublicEvent::PlayerResourceGained {
                        player: PlayerId::ONE,
                        resource,
                        amount: 1,
                        ..
                    } if resource == "corpses"
                )
            })
    );

    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "RLK_060", None);
    let risen = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .filter(|entity| game.state().entity(*entity).unwrap().card_id == "RLK_008t")
        .collect::<Vec<_>>();
    assert_eq!(risen.len(), 2);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        2
    );

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(risen[0]));
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);

    let snapshot = game.snapshot();
    let encoded = serde_json::to_string(&snapshot).unwrap();
    let portable = serde_json::from_str(&encoded).unwrap();
    let restored =
        Game::from_snapshot(LuaCardRuntime::load_dir(data_path()).unwrap(), &portable).unwrap();
    assert_eq!(restored.state(), game.state());

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}

fn plague_count(game: &Game<LuaCardRuntime>, player: PlayerId) -> usize {
    deck_ids(game, player)
        .iter()
        .filter(|card_id| matches!(card_id.as_str(), "TTN_450t" | "TTN_450t2" | "TTN_450t3"))
        .count()
}

fn plague_spells_cast(game: &Game<LuaCardRuntime>, player: PlayerId) -> Vec<String> {
    game.state()
        .log
        .iter()
        .filter_map(|event| {
            let GameEvent::SpellCast {
                player: caster,
                spell,
                ..
            } = event
            else {
                return None;
            };
            if *caster != player {
                return None;
            }
            let card_id = &game.state().entity(*spell)?.card_id;
            matches!(card_id.as_str(), "TTN_450t" | "TTN_450t2" | "TTN_450t3")
                .then(|| card_id.clone())
        })
        .collect()
}

#[test]
fn plague_generators_shuffle_exactly_two_replayable_random_plagues() {
    let mut spell = game_with_classes(
        mixed(&["TTN_454", "CS2_120"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut spell, PlayerId::ONE, 2);
    let enemy_hero = spell.state().player(PlayerId::TWO).hero;
    play(&mut spell, PlayerId::ONE, "TTN_454", Some(enemy_hero));
    assert_eq!(
        plague_count(&spell, PlayerId::TWO),
        2,
        "deck={:?}, source_data={:?}",
        deck_ids(&spell, PlayerId::TWO),
        spell.state().player(PlayerId::ONE).script_data
    );
    assert_eq!(
        spell
            .state()
            .player(PlayerId::ONE)
            .script_data
            .get("plagues_shuffled_into_enemy"),
        Some(&2)
    );
    assert_eq!(
        spell
            .state()
            .player(PlayerId::TWO)
            .script_data
            .get("plague_source_player"),
        Some(&1)
    );
    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &spell.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), spell.state());

    let mut deathrattle = game_with_classes(
        mixed(&["TTN_450", "CS2_120"]),
        repeated("CS2_029"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut deathrattle, PlayerId::ONE, 2);
    let kvaldir = play(&mut deathrattle, PlayerId::ONE, "TTN_450", None);
    advance_to_mana(&mut deathrattle, PlayerId::TWO, 4);
    play(&mut deathrattle, PlayerId::TWO, "CS2_029", Some(kvaldir));
    assert_eq!(
        plague_count(&deathrattle, PlayerId::TWO),
        2,
        "zone={:?}, entity_data={:?}, player_data={:?}, recent={:?}",
        deathrattle.state().entity(kvaldir).unwrap().zone,
        deathrattle.state().entity(kvaldir).unwrap().script_data,
        deathrattle.state().player(PlayerId::ONE).script_data,
        deathrattle
            .state()
            .log
            .iter()
            .rev()
            .take(12)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        deathrattle
            .state()
            .player(PlayerId::ONE)
            .script_data
            .get("plagues_shuffled_into_enemy"),
        Some(&2)
    );
}

#[test]
fn tomb_traitor_consumes_one_plague_before_damaging_all_enemy_minions() {
    let mut game = game_with_classes(
        mixed(&["TTN_850", "TTN_455"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "TTN_850", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_120", None);
    play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "TTN_455", None);

    assert_eq!(plague_count(&game, PlayerId::TWO), 2);
    assert!(game.state().player(PlayerId::TWO).board.is_empty());
    assert_eq!(
        game.state()
            .player(PlayerId::ONE)
            .script_data
            .get("plagues_shuffled_into_enemy"),
        Some(&3),
        "destroying a Plague must not rewrite the historical shuffle count"
    );
    assert_eq!(
        game.state()
            .entities
            .values()
            .filter(|entity| {
                matches!(
                    entity.card_id.as_str(),
                    "TTN_450t" | "TTN_450t2" | "TTN_450t3"
                ) && entity.zone == Zone::Removed
            })
            .count(),
        1
    );

    let mut no_plague = game_with_classes(
        repeated("TTN_455"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut no_plague, PlayerId::TWO, 2);
    let croc = play(&mut no_plague, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut no_plague, PlayerId::ONE, 4);
    play(&mut no_plague, PlayerId::ONE, "TTN_455", None);
    assert_eq!(no_plague.state().entity(croc).unwrap().health(), 3);
}

#[test]
fn staff_of_the_primus_shuffles_after_every_attack_including_final_durability() {
    let mut game = game_with_classes(
        repeated("TTN_736"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let staff = play(&mut game, PlayerId::ONE, "TTN_736", None);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;

    for expected in 1..=3 {
        let own_hero = game.state().player(PlayerId::ONE).hero;
        game.dispatch(PlayerCommand::Attack {
            attacker: own_hero,
            defender: enemy_hero,
        })
        .unwrap();
        assert_eq!(plague_count(&game, PlayerId::TWO), expected);
        assert_eq!(
            game.state()
                .player(PlayerId::ONE)
                .script_data
                .get("plagues_shuffled_into_enemy"),
            Some(&(expected as i64))
        );
        if expected < 3 {
            end_turn(&mut game);
            end_turn(&mut game);
        }
    }

    assert_eq!(game.state().entity(staff).unwrap().zone, Zone::Graveyard);
    assert!(game.state().player(PlayerId::ONE).weapon.is_none());
    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn chained_guardian_tracks_initial_and_unending_plague_shuffles_then_rushes_and_reborns() {
    let mut game = game_with_classes(
        mixed(&["TTN_850", "TTN_459"]),
        mixed(&["EX1_169", "CS2_200"]),
        ["death_knight", "druid"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "TTN_850", None);

    let guardian = hand_card(&game, PlayerId::ONE, "TTN_459");
    assert_eq!(game.state().entity(guardian).unwrap().cost, 8);

    let mut ogre = None;
    for _ in 0..50 {
        assert!(game.state().outcome.is_none());
        if game.state().active_player == PlayerId::TWO {
            while let Some(innervate) = game.legal_actions().unwrap().into_iter().find(|action| {
                matches!(action, PlayerCommand::PlayCard { card, target: None }
                    if game.state().entity(*card).unwrap().card_id == "EX1_169")
            }) {
                game.dispatch(innervate).unwrap();
            }
            let shuffled = game
                .state()
                .player(PlayerId::ONE)
                .script_data
                .get("plagues_shuffled_into_enemy")
                .copied()
                .unwrap_or(0);
            if shuffled >= 4 && game.state().player(PlayerId::TWO).max_mana >= 6 {
                if let Some(action) = game.legal_actions().unwrap().into_iter().find(|action| {
                    matches!(action, PlayerCommand::PlayCard { card, target: None }
                        if game.state().entity(*card).unwrap().card_id == "CS2_200")
                }) {
                    let PlayerCommand::PlayCard { card, .. } = action else {
                        unreachable!()
                    };
                    game.dispatch(action).unwrap();
                    ogre = Some(card);
                    end_turn(&mut game);
                    break;
                }
            }
        }
        end_turn(&mut game);
    }

    let ogre = ogre.expect("an enemy Boulderfist Ogre should be available after an unending draw");
    let shuffled = *game
        .state()
        .player(PlayerId::ONE)
        .script_data
        .get("plagues_shuffled_into_enemy")
        .unwrap();
    assert!(
        shuffled >= 4,
        "an unending reshuffle must count toward the discount"
    );
    assert_eq!(
        game.state().entity(guardian).unwrap().cost,
        (11 - shuffled).max(0) as u8
    );

    play(&mut game, PlayerId::ONE, "TTN_459", None);
    assert!(
        game.legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: guardian,
                defender: ogre,
            })
    );
    game.dispatch(PlayerCommand::Attack {
        attacker: guardian,
        defender: ogre,
    })
    .unwrap();
    assert_eq!(game.state().entity(guardian).unwrap().zone, Zone::Graveyard);
    let reborn = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "TTN_459")
        .expect("Chained Guardian should return with Reborn");
    assert_eq!(game.state().entity(reborn).unwrap().health(), 1);
    assert!(
        !game
            .state()
            .entity(reborn)
            .unwrap()
            .keywords
            .contains(&"reborn".to_owned())
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn helya_plagues_are_public_unending_and_apply_all_draw_effects() {
    let mut game = game_with_classes(
        mixed(&["TTN_850", "CS2_062"]),
        mixed(&["AT_055", "NEW1_030"]),
        ["death_knight", "priest"],
    );

    let helya_hero = game.state().player(PlayerId::ONE).hero;
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "CS2_062", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "TTN_850", None);

    assert_eq!(plague_count(&game, PlayerId::TWO), 3);
    assert_eq!(
        game.state()
            .player(PlayerId::ONE)
            .script_data
            .get("plagues_shuffled_into_enemy"),
        Some(&3)
    );
    assert!(
        game.state()
            .player(PlayerId::TWO)
            .keywords
            .contains(&"unending_plagues".to_owned())
    );
    for viewer in [PlayerId::ONE, PlayerId::TWO] {
        assert_eq!(
            game.state()
                .player_view(viewer)
                .player(PlayerId::TWO)
                .public_statuses,
            vec!["unending_plagues".to_owned()]
        );
    }

    let mut turns = 0;
    while plague_spells_cast(&game, PlayerId::TWO)
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        < 3
    {
        assert!(
            game.state().outcome.is_none(),
            "game ended before all Plagues were drawn: casts={:?}, deck={:?}, health={}, victim={:?}, source={:?}",
            plague_spells_cast(&game, PlayerId::TWO),
            deck_ids(&game, PlayerId::TWO),
            game.state()
                .entity(game.state().player(PlayerId::TWO).hero)
                .unwrap()
                .health(),
            game.state().player(PlayerId::TWO).script_data,
            game.state().player(PlayerId::ONE).script_data
        );
        end_turn(&mut game);
        turns += 1;
        assert!(
            turns < 80,
            "all three Plagues were not drawn deterministically"
        );

        if game.state().active_player == PlayerId::TWO {
            let own_hero = game.state().player(PlayerId::TWO).hero;
            if let Some(heal) = game.legal_actions().unwrap().into_iter().find(|action| {
                matches!(action, PlayerCommand::PlayCard { card, target: Some(target) }
                    if *target == own_hero
                        && game.state().entity(*card).unwrap().card_id == "AT_055")
            }) {
                game.dispatch(heal).unwrap();
            }
        }
    }

    let cast = plague_spells_cast(&game, PlayerId::TWO)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        cast,
        ["TTN_450t", "TTN_450t2", "TTN_450t3"]
            .map(str::to_owned)
            .into_iter()
            .collect()
    );
    assert_eq!(plague_count(&game, PlayerId::TWO), 3);
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .script_data
            .get("plagues_shuffled_into_enemy")
            .is_some_and(|count| *count > 3)
    );
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "RLK_070t" })
    );
    assert!(game.state().log.iter().any(|event| {
        matches!(event, GameEvent::Healed { source, target, .. }
            if *target == helya_hero
                && game.state().entity(*source).unwrap().card_id == "TTN_450t")
    }));

    let snapshot = game.snapshot();
    let restored = Game::from_snapshot(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &serde_json::from_str(&serde_json::to_string(&snapshot).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(restored.state(), game.state());
    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn frost_plague_stacks_caps_at_ten_and_is_consumed_by_the_next_card() {
    let mut game = game_with_classes(
        mixed(&["TTN_850", "CS2_120"]),
        mixed(&["AT_055", "NEW1_030"]),
        ["death_knight", "priest"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "TTN_850", None);

    let mut turns = 0;
    loop {
        end_turn(&mut game);
        turns += 1;
        assert!(turns < 50, "Frost Plague was not drawn deterministically");
        assert!(game.state().outcome.is_none());
        if game.state().active_player != PlayerId::TWO {
            continue;
        }
        let layers = game
            .state()
            .player(PlayerId::TWO)
            .script_data
            .get("frost_plague_surcharge")
            .copied()
            .unwrap_or(0);
        if layers > 0 {
            break;
        }
        let own_hero = game.state().player(PlayerId::TWO).hero;
        let action = game
            .legal_actions()
            .unwrap()
            .into_iter()
            .find(|action| {
                matches!(action, PlayerCommand::PlayCard { card, target: Some(target) }
                    if *target == own_hero
                        && game.state().entity(*card).unwrap().card_id == "AT_055")
            })
            .expect("the victim should keep hand space with Flash Heal");
        game.dispatch(action).unwrap();
    }

    let layers = *game
        .state()
        .player(PlayerId::TWO)
        .script_data
        .get("frost_plague_surcharge")
        .unwrap();
    assert!(layers > 0);
    let capped = game
        .state()
        .player(PlayerId::TWO)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "NEW1_030")
        .expect("the victim should retain a 10-Cost card for the surcharge cap check");
    assert_eq!(game.state().entity(capped).unwrap().cost, 10);

    let own_hero = game.state().player(PlayerId::TWO).hero;
    let (card, action) = game
        .legal_actions()
        .unwrap()
        .into_iter()
        .find_map(|action| match action {
            PlayerCommand::PlayCard {
                card,
                target: Some(target),
            } if target == own_hero && game.state().entity(card).unwrap().card_id == "AT_055" => {
                Some((card, action))
            }
            _ => None,
        })
        .expect("the surcharged Flash Heal should remain playable");
    assert_eq!(
        game.state().entity(card).unwrap().cost,
        (1 + layers).min(10) as u8
    );
    game.dispatch(action).unwrap();
    assert_eq!(
        game.state()
            .player(PlayerId::TWO)
            .script_data
            .get("frost_plague_surcharge"),
        Some(&0)
    );
    assert!(
        !game
            .state()
            .player(PlayerId::TWO)
            .keywords
            .contains(&"frost_plague_surcharge".to_owned())
    );
    assert_eq!(game.state().entity(capped).unwrap().cost, 10);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn corpse_generation_batches_simultaneous_deaths_and_ignores_transform() {
    let mut deaths = game_with_classes(
        repeated("CS2_120"),
        repeated("CS2_062"),
        ["death_knight", "warlock"],
    );
    advance_to_mana(&mut deaths, PlayerId::ONE, 4);
    play(&mut deaths, PlayerId::ONE, "CS2_120", None);
    play(&mut deaths, PlayerId::ONE, "CS2_120", None);
    advance_to_mana(&mut deaths, PlayerId::TWO, 4);
    play(&mut deaths, PlayerId::TWO, "CS2_062", None);
    assert_eq!(deaths.state().player(PlayerId::ONE).resource("corpses"), 2);
    assert_eq!(
        deaths
            .state()
            .log
            .iter()
            .filter(|event| matches!(
                event,
                GameEvent::PlayerResourceGained {
                    player: PlayerId::ONE,
                    resource,
                    amount: 1,
                    ..
                } if resource == "corpses"
            ))
            .count(),
        2
    );

    let mut transformed = game_with_classes(
        repeated("CS2_120"),
        repeated("CS2_022"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut transformed, PlayerId::ONE, 2);
    let minion = play(&mut transformed, PlayerId::ONE, "CS2_120", None);
    advance_to_mana(&mut transformed, PlayerId::TWO, 4);
    play(&mut transformed, PlayerId::TWO, "CS2_022", Some(minion));
    assert_eq!(
        transformed
            .state()
            .player(PlayerId::ONE)
            .resource("corpses"),
        0
    );
    assert_eq!(
        transformed.state().entity(minion).unwrap().zone,
        Zone::Board
    );
}

#[test]
fn reborn_minions_leave_a_corpse_each_time_they_die() {
    let mut game = game_with_classes(
        repeated("ULD_208"),
        repeated("CS2_029"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let original = play(&mut game, PlayerId::ONE, "ULD_208", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(original));
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 1);
    let reborn = game.state().player(PlayerId::ONE).board[0];

    end_turn(&mut game);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(reborn));
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 2);
}

#[test]
fn exact_corpse_spending_is_atomic_when_defrost_cannot_afford_it() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_101"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    play(&mut game, PlayerId::ONE, "RLK_503", None);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 1);

    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    play(&mut game, PlayerId::ONE, "RLK_101", None);
    assert_eq!(game.state().player(PlayerId::ONE).hand.len(), hand_before);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 1);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        0
    );
    assert!(!game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::PlayerResourceSpent { resource, .. } if resource == "corpses"
    )));

    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "RLK_503", None);
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    play(&mut game, PlayerId::ONE, "RLK_101", None);
    assert_eq!(
        game.state().player(PlayerId::ONE).hand.len(),
        hand_before + 1
    );
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        2
    );
}

#[test]
fn eulogizer_forge_gains_corpses_while_the_base_card_spends_them_atomically() {
    let mut game = game_with_classes(
        repeated("TTN_457"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let forged = hand_card(&game, PlayerId::ONE, "TTN_457");
    game.dispatch(PlayerCommand::UseCardAction {
        card: forged,
        action: "forge".to_owned(),
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().entity(forged).unwrap().card_id, "TTN_457t");
    assert_eq!(game.state().player(PlayerId::ONE).mana, 3);
    assert!(!game.legal_actions().unwrap().iter().any(|action| {
        matches!(action, PlayerCommand::UseCardAction { card, action, .. }
            if *card == forged && action == "forge")
    }));

    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::PlayCard {
        card: forged,
        target: Some(enemy_hero),
    })
    .unwrap();
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 27);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 3);

    advance_to_mana(&mut game, PlayerId::ONE, 6);
    play(&mut game, PlayerId::ONE, "TTN_457", Some(enemy_hero));
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 24);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        3
    );

    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "TTN_457", Some(enemy_hero));
    assert_eq!(
        game.state().entity(enemy_hero).unwrap().health(),
        24,
        "the un-forged Battlecry must not deal damage when three Corpses cannot be spent"
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

fn choose_deck_entity(game: &mut Game<LuaCardRuntime>, card_id: &str) -> hearth_core::EntityId {
    let (index, entity) = game
        .state()
        .pending_input
        .as_ref()
        .expect("a deck Discover should be pending")
        .options
        .iter()
        .enumerate()
        .find_map(|(index, option)| match &option.value {
            ChoiceValue::Entity(entity)
                if game.state().entity(*entity).unwrap().card_id == card_id =>
            {
                Some((index, *entity))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("Discover did not offer {card_id}"));
    game.dispatch(PlayerCommand::Choose { index }).unwrap();
    entity
}

#[test]
fn northern_navigation_draws_the_discovered_deck_spell_and_only_frost_freezes() {
    let mut frost = game_with_classes(
        mixed(&["TTN_735", "CS2_024"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut frost, PlayerId::TWO, 2);
    let enemy = play(&mut frost, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut frost, PlayerId::ONE, 3);
    play(&mut frost, PlayerId::ONE, "TTN_735", None);
    let selected = choose_deck_entity(&mut frost, "CS2_024");
    assert_eq!(frost.state().entity(selected).unwrap().zone, Zone::Hand);
    assert!(frost.state().entity(enemy).unwrap().frozen);

    let mut fire = game_with_classes(
        mixed(&["TTN_735", "CS2_029"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut fire, PlayerId::TWO, 2);
    let enemy = play(&mut fire, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut fire, PlayerId::ONE, 3);
    play(&mut fire, PlayerId::ONE, "TTN_735", None);
    let selected = choose_deck_entity(&mut fire, "CS2_029");
    assert_eq!(fire.state().entity(selected).unwrap().zone, Zone::Hand);
    assert!(!fire.state().entity(enemy).unwrap().frozen);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &frost.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), frost.state());
}

#[test]
fn frozen_over_locks_only_the_opponents_direct_draws_for_their_next_turn() {
    let mut game = game_with_classes(
        mixed(&["TTN_744", "CS2_120"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let log_start = game.state().log.len();
    let frozen_over = play(&mut game, PlayerId::ONE, "TTN_744", None);

    let drawn_for = |player| {
        game.state().log[log_start..]
            .iter()
            .filter_map(|event| match event {
                GameEvent::CardDrawn {
                    player: drawing_player,
                    card,
                    source: Some(source),
                } if *drawing_player == player && *source == frozen_over => Some(*card),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let own_draws = drawn_for(PlayerId::ONE);
    let locked = drawn_for(PlayerId::TWO);
    assert_eq!(own_draws.len(), 2);
    assert_eq!(locked.len(), 2);
    assert!(own_draws.iter().all(|entity| {
        !game
            .state()
            .entity(*entity)
            .unwrap()
            .keywords
            .contains(&"frozen_solid".to_owned())
    }));
    assert!(locked.iter().all(|entity| {
        game.state()
            .entity(*entity)
            .unwrap()
            .keywords
            .contains(&"frozen_solid".to_owned())
    }));

    end_turn(&mut game);
    let next_turn_actions = game.legal_actions().unwrap();
    assert!(locked.iter().all(|entity| {
        !next_turn_actions
            .iter()
            .any(|action| matches!(action, PlayerCommand::PlayCard { card, .. } if card == entity))
    }));

    end_turn(&mut game);
    end_turn(&mut game);
    let following_turn_actions = game.legal_actions().unwrap();
    assert!(locked.iter().all(|entity| {
        following_turn_actions
            .iter()
            .any(|action| matches!(action, PlayerCommand::PlayCard { card, .. } if card == entity))
    }));

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

fn choose_primus_rune_card(game: &mut Game<LuaCardRuntime>, rune: &str) -> String {
    let pending = game
        .state()
        .pending_input
        .as_ref()
        .expect("The Primus should Discover after its ability");
    assert!((1..=3).contains(&pending.options.len()));
    assert!(pending.prompt.contains(match rune {
        "blood" => "Blood",
        "frost" => "Frost",
        "unholy" => "Unholy",
        other => panic!("unsupported test Rune {other}"),
    }));
    let cards = pending
        .options
        .iter()
        .map(|option| match &option.value {
            ChoiceValue::Card(card_id) => card_id.clone(),
            other => panic!("Rune Discover returned {other:?}"),
        })
        .collect::<Vec<_>>();
    for card_id in &cards {
        let definition = game.runtime().definition(card_id).unwrap();
        assert_eq!(definition.class, "death_knight");
        let requirement = match rune {
            "blood" => definition.rune_cost.blood,
            "frost" => definition.rune_cost.frost,
            "unholy" => definition.rune_cost.unholy,
            _ => unreachable!(),
        };
        assert!(
            requirement > 0,
            "{card_id} lacks the discovered {rune} Rune"
        );
    }
    let selected = cards[0].clone();
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == selected })
    );
    selected
}

#[test]
fn the_primus_resolves_all_three_runes_and_discovers_from_the_matching_pool() {
    let mut blood = game_with_classes(
        repeated("TTN_737"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut blood, PlayerId::TWO, 2);
    let victim = play(&mut blood, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut blood, PlayerId::ONE, 8);
    let primus = play(&mut blood, PlayerId::ONE, "TTN_737", None);
    blood
        .dispatch(PlayerCommand::UseCardAction {
            card: primus,
            action: "titan_1".to_owned(),
            target: Some(victim),
        })
        .unwrap();
    assert_eq!(blood.state().entity(victim).unwrap().zone, Zone::Graveyard);
    assert_eq!(blood.state().entity(primus).unwrap().max_health, 12);
    let hero = blood.state().player(PlayerId::ONE).hero;
    assert_eq!(blood.state().entity(hero).unwrap().max_health, 33);
    assert_eq!(blood.state().entity(hero).unwrap().health(), 33);
    choose_primus_rune_card(&mut blood, "blood");
    assert!(
        !blood.legal_actions().unwrap().iter().any(|action| matches!(
            action,
            PlayerCommand::UseCardAction { card, .. } if *card == primus
        ))
    );

    let mut unholy = game_with_classes(
        repeated("TTN_737"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut unholy, PlayerId::ONE, 8);
    let primus = play(&mut unholy, PlayerId::ONE, "TTN_737", None);
    unholy
        .dispatch(PlayerCommand::UseCardAction {
            card: primus,
            action: "titan_2".to_owned(),
            target: None,
        })
        .unwrap();
    let servants = unholy
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter_map(|entity| {
            (unholy.state().entity(*entity).unwrap().card_id == "TTN_737t2").then_some(*entity)
        })
        .collect::<Vec<_>>();
    assert_eq!(servants.len(), 2);
    assert!(servants.iter().all(|entity| {
        let servant = unholy.state().entity(*entity).unwrap();
        servant.has_keyword("taunt")
            && servant.has_keyword("reborn")
            && unholy
                .runtime()
                .definition(&servant.card_id)
                .unwrap()
                .tags
                .contains(&"undead".to_owned())
    }));
    choose_primus_rune_card(&mut unholy, "unholy");

    let mut frost = game_with_classes(
        mixed(&["TTN_737", "CS2_024"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut frost, PlayerId::ONE, 8);
    let primus = play(&mut frost, PlayerId::ONE, "TTN_737", None);
    frost
        .dispatch(PlayerCommand::UseCardAction {
            card: primus,
            action: "titan_3".to_owned(),
            target: None,
        })
        .unwrap();
    choose_primus_rune_card(&mut frost, "frost");
    let first_frostbolt = hand_card(&frost, PlayerId::ONE, "CS2_024");
    assert_eq!(frost.state().entity(first_frostbolt).unwrap().cost, 0);
    assert_eq!(
        frost
            .state()
            .entity(frost.state().player(PlayerId::ONE).hero)
            .unwrap()
            .spell_damage,
        3
    );
    let enemy_hero = frost.state().player(PlayerId::TWO).hero;
    frost
        .dispatch(PlayerCommand::PlayCard {
            card: first_frostbolt,
            target: Some(enemy_hero),
        })
        .unwrap();
    assert_eq!(frost.state().entity(enemy_hero).unwrap().health(), 24);
    let second_frostbolt = hand_card(&frost, PlayerId::ONE, "CS2_024");
    assert_eq!(frost.state().entity(second_frostbolt).unwrap().cost, 2);
    assert_eq!(
        frost
            .state()
            .entity(frost.state().player(PlayerId::ONE).hero)
            .unwrap()
            .spell_damage,
        0
    );

    for completed in [&blood, &unholy, &frost] {
        let replayed = Game::from_replay(
            LuaCardRuntime::load_dir(data_path()).unwrap(),
            &completed.replay(),
        )
        .unwrap();
        assert_eq!(replayed.state(), completed.state());
    }
}

fn gain_body_bagger_corpses(game: &mut Game<LuaCardRuntime>, amount: u32) {
    let mut turns = 0;
    while game.state().player(PlayerId::ONE).resource("corpses") < amount {
        if game.state().active_player == PlayerId::ONE
            && let Some(action) = game.legal_actions().unwrap().into_iter().find(|action| {
                matches!(action, PlayerCommand::PlayCard { card, target: None }
                    if game.state().entity(*card).unwrap().card_id == "RLK_503")
            })
        {
            game.dispatch(action).unwrap();
            continue;
        }
        end_turn(game);
        turns += 1;
        assert!(turns < 30, "Body Baggers did not generate enough Corpses");
    }
}

fn choose_created_weapon(
    game: &mut Game<LuaCardRuntime>,
    source: hearth_core::EntityId,
) -> hearth_core::EntityId {
    let pending = game
        .state()
        .pending_input
        .as_ref()
        .expect("Runes of Darkness should Discover a weapon");
    assert_eq!(pending.prompt, "Discover a weapon");
    assert!((1..=3).contains(&pending.options.len()));
    for option in &pending.options {
        let ChoiceValue::Card(card_id) = &option.value else {
            panic!("weapon Discover returned a non-card option")
        };
        let definition = game.runtime().definition(card_id).unwrap();
        assert_eq!(definition.kind, CardKind::Weapon);
        assert!(matches!(
            definition.class.as_str(),
            "death_knight" | "neutral"
        ));
    }
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    game.state()
        .log
        .iter()
        .rev()
        .find_map(|event| match event {
            GameEvent::CardCreated {
                source: event_source,
                player: PlayerId::ONE,
                card,
            } if *event_source == source => Some(*card),
            _ => None,
        })
        .expect("the discovered weapon should be created by Runes of Darkness")
}

#[test]
fn runes_of_darkness_discovers_a_legal_weapon_and_spends_only_for_the_buff() {
    let mut without_corpses = game_with_classes(
        repeated("YOG_511"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut without_corpses, PlayerId::ONE, 1);
    let runes = play(&mut without_corpses, PlayerId::ONE, "YOG_511", None);
    let weapon = choose_created_weapon(&mut without_corpses, runes);
    let definition = without_corpses
        .runtime()
        .definition(&without_corpses.state().entity(weapon).unwrap().card_id)
        .unwrap();
    assert_eq!(
        without_corpses.state().entity(weapon).unwrap().attack,
        definition.attack
    );
    assert_eq!(
        without_corpses.state().entity(weapon).unwrap().max_health,
        definition.health
    );
    assert_eq!(
        without_corpses
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        0
    );

    let mut buffed = game_with_classes(
        mixed(&["RLK_503", "YOG_511"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut buffed, 3);
    end_turn(&mut buffed);
    advance_to_mana(&mut buffed, PlayerId::ONE, 1);
    let runes = play(&mut buffed, PlayerId::ONE, "YOG_511", None);
    let weapon = choose_created_weapon(&mut buffed, runes);
    let definition = buffed
        .runtime()
        .definition(&buffed.state().entity(weapon).unwrap().card_id)
        .unwrap();
    assert_eq!(
        buffed.state().entity(weapon).unwrap().attack,
        definition.attack + 1
    );
    assert_eq!(
        buffed.state().entity(weapon).unwrap().max_health,
        definition.health + 1
    );
    assert_eq!(buffed.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        buffed
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        3
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &buffed.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), buffed.state());
}

#[test]
fn sickly_grimewalker_marks_later_friendly_undead_but_not_itself_or_other_tribes() {
    let mut game = game_with_classes(
        mixed(&["YOG_512", "RLK_503", "CS2_120"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let grimewalker = play(&mut game, PlayerId::ONE, "YOG_512", None);
    assert!(
        !game
            .state()
            .entity(grimewalker)
            .unwrap()
            .has_keyword("poisonous")
    );
    let undead = play(&mut game, PlayerId::ONE, "RLK_503", None);
    assert!(
        game.state()
            .entity(undead)
            .unwrap()
            .has_keyword("poisonous")
    );

    end_turn(&mut game);
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let crocolisk = play(&mut game, PlayerId::ONE, "CS2_120", None);
    assert!(
        !game
            .state()
            .entity(crocolisk)
            .unwrap()
            .has_keyword("poisonous")
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn sinister_soulcage_always_buffs_and_copies_only_after_spending_five_corpses() {
    let mut insufficient = game_with_classes(
        mixed(&["RLK_503", "YOG_513"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut insufficient, PlayerId::ONE, 5);
    let target = play(&mut insufficient, PlayerId::ONE, "RLK_503", None);
    assert_eq!(
        insufficient
            .state()
            .player(PlayerId::ONE)
            .resource("corpses"),
        1
    );
    let board_before = insufficient.state().player(PlayerId::ONE).board.len();
    play(&mut insufficient, PlayerId::ONE, "YOG_513", Some(target));
    assert_eq!(insufficient.state().entity(target).unwrap().attack, 3);
    assert_eq!(insufficient.state().entity(target).unwrap().max_health, 5);
    assert_eq!(
        insufficient.state().player(PlayerId::ONE).board.len(),
        board_before
    );
    assert_eq!(
        insufficient
            .state()
            .player(PlayerId::ONE)
            .resource("corpses"),
        1
    );
    assert_eq!(
        insufficient
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        0
    );

    let mut copied = game_with_classes(
        mixed(&["RLK_503", "YOG_513"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut copied, 5);
    let target = copied.state().player(PlayerId::ONE).board[0];
    let board_before = copied.state().player(PlayerId::ONE).board.len();
    end_turn(&mut copied);
    advance_to_mana(&mut copied, PlayerId::ONE, 4);
    play(&mut copied, PlayerId::ONE, "YOG_513", Some(target));
    assert_eq!(
        copied.state().player(PlayerId::ONE).board.len(),
        board_before + 1
    );
    let copies = copied
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter_map(|entity| {
            let entity_state = copied.state().entity(*entity).unwrap();
            (entity_state.card_id == "RLK_503"
                && entity_state.attack == 3
                && entity_state.max_health == 5)
                .then_some(*entity)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        copies.len(),
        2,
        "the summon should copy the post-buff state"
    );
    assert_eq!(copied.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        copied
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        5
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &copied.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), copied.state());
}

#[test]
fn howling_blast_hits_all_enemies_in_one_spell_damage_batch_and_freezes_only_its_target() {
    let mut game = game_with_classes(
        mixed(&["RLK_015", "CS2_142"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let geomancer = play(&mut game, PlayerId::ONE, "CS2_142", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let first = play(&mut game, PlayerId::TWO, "CS2_120", None);
    let second = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 3);

    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    let friendly_hero = game.state().player(PlayerId::ONE).hero;
    play(&mut game, PlayerId::ONE, "RLK_015", Some(enemy_hero));

    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 26);
    assert!(game.state().entity(enemy_hero).unwrap().frozen);
    assert_eq!(game.state().entity(first).unwrap().health(), 1);
    assert_eq!(game.state().entity(second).unwrap().health(), 1);
    assert!(!game.state().entity(first).unwrap().frozen);
    assert!(!game.state().entity(second).unwrap().frozen);
    assert_eq!(game.state().entity(friendly_hero).unwrap().health(), 30);
    assert_eq!(game.state().entity(geomancer).unwrap().health(), 2);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn horn_of_winter_refreshes_at_most_two_existing_mana_crystals() {
    let mut game = game_with_classes(
        mixed(&["RLK_042", "CS2_125"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "CS2_125", None);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 1);

    play(&mut game, PlayerId::ONE, "RLK_042", None);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 3);
    play(&mut game, PlayerId::ONE, "RLK_042", None);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 4);
    assert_eq!(game.state().player(PlayerId::ONE).max_mana, 4);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn glacial_advance_discounts_only_the_next_spell_this_turn() {
    let mut consumed = game_with_classes(
        mixed(&["RLK_512", "CS2_029"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut consumed, PlayerId::ONE, 7);
    let enemy_hero = consumed.state().player(PlayerId::TWO).hero;
    play(&mut consumed, PlayerId::ONE, "RLK_512", Some(enemy_hero));
    assert_eq!(consumed.state().entity(enemy_hero).unwrap().health(), 26);

    let fireball = hand_card(&consumed, PlayerId::ONE, "CS2_029");
    assert_eq!(consumed.state().entity(fireball).unwrap().cost, 2);
    consumed
        .dispatch(PlayerCommand::PlayCard {
            card: fireball,
            target: Some(enemy_hero),
        })
        .unwrap();
    assert_eq!(consumed.state().player(PlayerId::ONE).mana, 2);
    assert_eq!(consumed.state().entity(enemy_hero).unwrap().health(), 20);
    let next_fireball = hand_card(&consumed, PlayerId::ONE, "CS2_029");
    assert_eq!(consumed.state().entity(next_fireball).unwrap().cost, 4);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &consumed.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), consumed.state());

    let mut expired = game_with_classes(
        mixed(&["RLK_512", "CS2_029"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut expired, PlayerId::ONE, 3);
    let enemy_hero = expired.state().player(PlayerId::TWO).hero;
    play(&mut expired, PlayerId::ONE, "RLK_512", Some(enemy_hero));
    assert_eq!(
        expired
            .state()
            .entity(hand_card(&expired, PlayerId::ONE, "CS2_029"))
            .unwrap()
            .cost,
        2
    );
    end_turn(&mut expired);
    end_turn(&mut expired);
    assert_eq!(
        expired
            .state()
            .entity(hand_card(&expired, PlayerId::ONE, "CS2_029"))
            .unwrap()
            .cost,
        4
    );
}

#[test]
fn deathchiller_fires_twice_only_after_its_controllers_player_cast_spell() {
    let mut game = game_with_classes(
        mixed(&["RLK_083", "RLK_042"]),
        repeated("CS2_029"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "RLK_083", None);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "RLK_042", None);
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 28);

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let friendly_hero = game.state().player(PlayerId::ONE).hero;
    play(&mut game, PlayerId::TWO, "CS2_029", Some(friendly_hero));
    assert_eq!(game.state().entity(friendly_hero).unwrap().health(), 24);
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 28);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn ymirjar_frostbreaker_counts_only_frost_spells_remaining_in_hand() {
    let mut game = game_with_classes(
        mixed(&["RLK_110", "RLK_015", "CS2_029"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    let mut turns = 0;
    while game.state().active_player != PlayerId::ONE
        || !game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| game.state().entity(*entity).unwrap().card_id == "RLK_110")
        || !game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| game.state().entity(*entity).unwrap().card_id == "RLK_015")
        || !game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| game.state().entity(*entity).unwrap().card_id == "CS2_029")
    {
        end_turn(&mut game);
        turns += 1;
        assert!(turns < 20, "the mixed deck did not expose all test cards");
    }
    let ymirjar = hand_card(&game, PlayerId::ONE, "RLK_110");
    let frost_spells = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| {
            let entity = game.state().entity(**entity).unwrap();
            let definition = game.runtime().definition(&entity.card_id).unwrap();
            definition.kind == CardKind::Spell
                && definition.spell_school.as_deref() == Some("frost")
        })
        .count() as i32;
    assert!(frost_spells > 0);
    let all_spells = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| {
            let card_id = &game.state().entity(**entity).unwrap().card_id;
            game.runtime().definition(card_id).unwrap().kind == CardKind::Spell
        })
        .count() as i32;
    assert!(all_spells > frost_spells);

    game.dispatch(PlayerCommand::PlayCard {
        card: ymirjar,
        target: None,
    })
    .unwrap();
    assert_eq!(
        game.state().entity(ymirjar).unwrap().attack,
        1 + frost_spells
    );
    assert_eq!(game.state().entity(ymirjar).unwrap().max_health, 2);
}

#[test]
fn marrow_manipulator_spends_up_to_five_corpses_and_fires_once_for_each() {
    let mut partial = game_with_classes(
        mixed(&["RLK_503", "RLK_505"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut partial, 3);
    advance_to_mana(&mut partial, PlayerId::ONE, 6);
    let enemy_hero = partial.state().player(PlayerId::TWO).hero;
    play(&mut partial, PlayerId::ONE, "RLK_505", None);
    assert_eq!(partial.state().entity(enemy_hero).unwrap().health(), 24);
    assert_eq!(partial.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        partial
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        3
    );

    let mut capped = game_with_classes(
        mixed(&["RLK_503", "RLK_505"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut capped, 5);
    advance_to_mana(&mut capped, PlayerId::ONE, 6);
    let enemy_hero = capped.state().player(PlayerId::TWO).hero;
    play(&mut capped, PlayerId::ONE, "RLK_505", None);
    assert_eq!(capped.state().entity(enemy_hero).unwrap().health(), 20);
    assert_eq!(capped.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        capped
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        5
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &capped.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), capped.state());
}

#[test]
fn bone_breaker_burns_the_enemy_hero_after_minion_attacks_even_on_final_durability() {
    let mut game = game_with_classes(
        repeated("RLK_516"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let weapon = play(&mut game, PlayerId::ONE, "RLK_516", None);
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let first = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);

    let hero = game.state().player(PlayerId::ONE).hero;
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender: first,
    })
    .unwrap();
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 28);
    assert_eq!(game.state().entity(weapon).unwrap().health(), 1);

    end_turn(&mut game);
    let second = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender: second,
    })
    .unwrap();
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 26);
    assert_eq!(game.state().entity(weapon).unwrap().zone, Zone::Graveyard);
    assert!(game.state().player(PlayerId::ONE).weapon.is_none());

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn lady_deathwhisper_copies_every_frost_spell_but_no_other_hand_card() {
    let mut game = game_with_classes(
        mixed(&["RLK_713", "RLK_015", "CS2_029"]),
        repeated("CS2_029"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let lady = play(&mut game, PlayerId::ONE, "RLK_713", None);
    let is_frost_spell = |game: &Game<LuaCardRuntime>, entity: &hearth_core::EntityId| {
        let card_id = &game.state().entity(*entity).unwrap().card_id;
        let definition = game.runtime().definition(card_id).unwrap();
        definition.kind == CardKind::Spell && definition.spell_school.as_deref() == Some("frost")
    };
    let frost_before = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| is_frost_spell(&game, entity))
        .count();
    let fireballs_before = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "CS2_029")
        .count();
    assert!(frost_before > 0);
    assert!(fireballs_before > 0);

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(lady));
    let frost_after = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| is_frost_spell(&game, entity))
        .count();
    let fireballs_after = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "CS2_029")
        .count();
    assert_eq!(frost_after, frost_before * 2);
    assert_eq!(fireballs_after, fireballs_before);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn might_of_menethil_spends_only_for_distinct_enemy_minions_it_can_freeze() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_740"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut game, 3);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let first = play(&mut game, PlayerId::TWO, "CS2_120", None);
    let second = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "RLK_740", None);

    assert!(game.state().entity(first).unwrap().frozen);
    assert!(game.state().entity(second).unwrap().frozen);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 1);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        2
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn hematurge_spends_one_corpse_before_discovering_only_blood_rune_cards() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_066"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut game, 1);
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let hematurge = play(&mut game, PlayerId::ONE, "RLK_066", None);
    let pending = game
        .state()
        .pending_input
        .as_ref()
        .expect("Hematurge should Discover after spending its Corpse");
    assert_eq!(pending.source, hematurge);
    assert_eq!(pending.prompt, "Discover a Blood Rune card");
    assert!((1..=3).contains(&pending.options.len()));
    for option in &pending.options {
        let ChoiceValue::Card(card_id) = &option.value else {
            panic!("Hematurge returned a non-card option")
        };
        let definition = game.runtime().definition(card_id).unwrap();
        assert_eq!(definition.class, "death_knight");
        assert!(definition.rune_cost.blood > 0, "{card_id}");
    }
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        1
    );
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());

    let mut empty = game_with_classes(
        repeated("RLK_066"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut empty, PlayerId::ONE, 2);
    play(&mut empty, PlayerId::ONE, "RLK_066", None);
    assert!(empty.state().pending_input.is_none());
    assert_eq!(
        empty
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        0
    );
}

#[test]
fn vicious_bloodworm_targets_a_real_minion_entity_in_its_own_hand() {
    let mut game = game_with_classes(
        mixed(&["RLK_711", "CS2_120"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let bloodworm = hand_card(&game, PlayerId::ONE, "RLK_711");
    let target = hand_card(&game, PlayerId::ONE, "CS2_120");
    assert!(game.legal_actions().unwrap().iter().any(|action| {
        matches!(
            action,
            PlayerCommand::PlayCard {
                card,
                target: Some(candidate),
            } | PlayerCommand::PlayCardAt {
                card,
                target: Some(candidate),
                ..
            } if *card == bloodworm && *candidate == target
        )
    }));
    game.dispatch(PlayerCommand::PlayCard {
        card: bloodworm,
        target: Some(target),
    })
    .unwrap();
    assert_eq!(game.state().entity(target).unwrap().zone, Zone::Hand);
    assert_eq!(game.state().entity(target).unwrap().attack, 5);
    assert_eq!(game.state().entity(target).unwrap().max_health, 3);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

fn hand_minion_stats(
    game: &Game<LuaCardRuntime>,
    player: PlayerId,
) -> BTreeMap<hearth_core::EntityId, (i32, i32)> {
    game.state()
        .player(player)
        .hand
        .iter()
        .filter_map(|entity| {
            let state = game.state().entity(*entity).unwrap();
            (state.kind == CardKind::Minion).then_some((*entity, (state.attack, state.max_health)))
        })
        .collect()
}

#[test]
fn blood_tap_buffs_every_hand_minion_and_doubles_only_after_spending_two_corpses() {
    let mut paid = game_with_classes(
        mixed(&["RLK_503", "RLK_712"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut paid, 2);
    end_turn(&mut paid);
    advance_to_mana(&mut paid, PlayerId::ONE, 2);
    let before = hand_minion_stats(&paid, PlayerId::ONE);
    assert!(!before.is_empty());
    play(&mut paid, PlayerId::ONE, "RLK_712", None);
    for (entity, (attack, health)) in before {
        let buffed = paid.state().entity(entity).unwrap();
        assert_eq!(buffed.attack, attack + 2, "{entity}");
        assert_eq!(buffed.max_health, health + 2, "{entity}");
    }
    assert_eq!(paid.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        paid.state().player(PlayerId::ONE).resource_spent("corpses"),
        2
    );

    let mut unpaid = game_with_classes(
        mixed(&["RLK_712", "CS2_120"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut unpaid, PlayerId::ONE, 2);
    let before = hand_minion_stats(&unpaid, PlayerId::ONE);
    play(&mut unpaid, PlayerId::ONE, "RLK_712", None);
    for (entity, (attack, health)) in before {
        let buffed = unpaid.state().entity(entity).unwrap();
        assert_eq!(buffed.attack, attack + 1, "{entity}");
        assert_eq!(buffed.max_health, health + 1, "{entity}");
    }
    assert_eq!(
        unpaid
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        0
    );
}

#[test]
fn darkfallen_neophyte_buffs_hand_attack_only_after_exact_corpse_spending() {
    let mut paid = game_with_classes(
        mixed(&["RLK_503", "RLK_731"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut paid, 2);
    end_turn(&mut paid);
    advance_to_mana(&mut paid, PlayerId::ONE, 3);
    let neophyte = hand_card(&paid, PlayerId::ONE, "RLK_731");
    let mut before = hand_minion_stats(&paid, PlayerId::ONE);
    before.remove(&neophyte);
    paid.dispatch(PlayerCommand::PlayCard {
        card: neophyte,
        target: None,
    })
    .unwrap();
    for (entity, (attack, health)) in before {
        let buffed = paid.state().entity(entity).unwrap();
        assert_eq!(buffed.attack, attack + 2, "{entity}");
        assert_eq!(buffed.max_health, health, "{entity}");
    }
    assert_eq!(paid.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        paid.state().player(PlayerId::ONE).resource_spent("corpses"),
        2
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &paid.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), paid.state());
}

#[test]
fn blood_boil_infections_tick_with_lifesteal_ignore_later_minions_and_are_silenciable() {
    let mut game = game_with_classes(
        mixed(&["RLK_730", "EX1_332"]),
        mixed(&["CS2_120", "CS2_029"]),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 8);
    let infected = play(&mut game, PlayerId::TWO, "CS2_120", None);
    let cleansed = play(&mut game, PlayerId::TWO, "CS2_120", None);
    let friendly_hero = game.state().player(PlayerId::ONE).hero;
    play(&mut game, PlayerId::TWO, "CS2_029", Some(friendly_hero));
    assert_eq!(game.state().entity(friendly_hero).unwrap().health(), 24);

    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let blood_boil = play(&mut game, PlayerId::ONE, "RLK_730", None);
    assert!(
        game.state()
            .entity(infected)
            .unwrap()
            .enchantments
            .iter()
            .any(|enchantment| enchantment.source == blood_boil)
    );
    play(&mut game, PlayerId::ONE, "EX1_332", Some(cleansed));
    assert!(
        game.state()
            .entity(cleansed)
            .unwrap()
            .enchantments
            .is_empty()
    );
    end_turn(&mut game);

    assert_eq!(game.state().entity(infected).unwrap().health(), 1);
    assert_eq!(game.state().entity(cleansed).unwrap().health(), 3);
    assert_eq!(game.state().entity(friendly_hero).unwrap().health(), 26);

    let later = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    end_turn(&mut game);
    assert_eq!(game.state().entity(infected).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().entity(cleansed).unwrap().health(), 3);
    assert_eq!(game.state().entity(later).unwrap().health(), 3);
    assert_eq!(game.state().entity(friendly_hero).unwrap().health(), 28);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn asphyxiate_destroys_exactly_one_random_member_of_the_highest_attack_tie() {
    let mut game = game_with_classes(
        repeated("RLK_087"),
        mixed(&["CS2_182", "CS2_120"]),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 10);
    let first_yeti = play(&mut game, PlayerId::TWO, "CS2_182", None);
    let second_yeti = play(&mut game, PlayerId::TWO, "CS2_182", None);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "RLK_087", None);

    let destroyed_yetis = [first_yeti, second_yeti]
        .into_iter()
        .filter(|entity| game.state().entity(*entity).unwrap().zone == Zone::Graveyard)
        .count();
    assert_eq!(destroyed_yetis, 1);
    assert_eq!(game.state().entity(crocolisk).unwrap().zone, Zone::Board);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn nerubian_swarmguard_summons_two_exact_copies_of_its_hand_buffed_state() {
    let mut game = game_with_classes(
        mixed(&["RLK_062", "RLK_712"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let swarmguard = hand_card(&game, PlayerId::ONE, "RLK_062");
    play(&mut game, PlayerId::ONE, "RLK_712", None);
    assert_eq!(game.state().entity(swarmguard).unwrap().attack, 2);
    assert_eq!(game.state().entity(swarmguard).unwrap().max_health, 4);
    game.dispatch(PlayerCommand::PlayCard {
        card: swarmguard,
        target: None,
    })
    .unwrap();

    let copies = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter_map(|entity| {
            let state = game.state().entity(*entity).unwrap();
            (state.card_id == "RLK_062").then_some(state)
        })
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 3);
    assert!(copies.iter().all(|entity| {
        entity.attack == 2
            && entity.max_health == 4
            && entity.has_keyword("taunt")
            && entity.has_keyword("battlecry")
    }));

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn plague_strike_summons_a_rushing_zombie_only_when_its_damage_kills() {
    let mut game = game_with_classes(
        repeated("RLK_018"),
        mixed(&["CS2_120", "CS2_182"]),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 6);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    let yeti = play(&mut game, PlayerId::TWO, "CS2_182", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);

    play(&mut game, PlayerId::ONE, "RLK_018", Some(crocolisk));
    assert_eq!(
        game.state().entity(crocolisk).unwrap().zone,
        Zone::Graveyard
    );
    let zombie = game.state().player(PlayerId::ONE).board[0];
    let zombie_state = game.state().entity(zombie).unwrap();
    assert_eq!(zombie_state.card_id, "RLK_018t");
    assert_eq!((zombie_state.attack, zombie_state.health()), (2, 2));
    assert!(zombie_state.has_keyword("rush"));

    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    let legal = game.legal_actions().unwrap();
    assert!(legal.contains(&PlayerCommand::Attack {
        attacker: zombie,
        defender: yeti,
    }));
    assert!(!legal.contains(&PlayerCommand::Attack {
        attacker: zombie,
        defender: enemy_hero,
    }));

    play(&mut game, PlayerId::ONE, "RLK_018", Some(yeti));
    assert_eq!(game.state().entity(yeti).unwrap().health(), 2);
    assert_eq!(game.state().player(PlayerId::ONE).board, vec![zombie]);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn dark_transformation_targets_any_undead_and_preserves_its_side_and_position() {
    let mut game = game_with_classes(
        repeated("RLK_057"),
        mixed(&["RLK_503", "CS2_120"]),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 3);
    let undead = play(&mut game, PlayerId::TWO, "RLK_503", None);
    let beast = play(&mut game, PlayerId::TWO, "CS2_120", None);
    assert_eq!(
        game.state().player(PlayerId::TWO).board,
        vec![undead, beast]
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);

    let transformation = hand_card(&game, PlayerId::ONE, "RLK_057");
    let legal = game.legal_actions().unwrap();
    assert!(legal.iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard {
            card,
            target: Some(target),
        } | PlayerCommand::PlayCardAt {
            card,
            target: Some(target),
            ..
        } if *card == transformation && *target == undead
    )));
    assert!(!legal.iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard {
            card,
            target: Some(target),
        } | PlayerCommand::PlayCardAt {
            card,
            target: Some(target),
            ..
        } if *card == transformation && *target == beast
    )));
    game.dispatch(PlayerCommand::PlayCard {
        card: transformation,
        target: Some(undead),
    })
    .unwrap();

    let transformed = game.state().entity(undead).unwrap();
    assert_eq!(transformed.card_id, "RLK_057t");
    assert_eq!(transformed.controller, PlayerId::TWO);
    assert_eq!((transformed.attack, transformed.health()), (4, 5));
    assert!(transformed.has_keyword("rush"));
    assert_eq!(
        game.state().player(PlayerId::TWO).board,
        vec![undead, beast]
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn tomb_guardians_spends_corpses_only_when_it_can_summon_and_grants_reborn() {
    let mut paid = game_with_classes(
        mixed(&["RLK_503", "RLK_118"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut paid, 4);
    advance_to_mana(&mut paid, PlayerId::ONE, 4);
    play(&mut paid, PlayerId::ONE, "RLK_118", None);
    let zombies = paid
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter_map(|entity| {
            let state = paid.state().entity(*entity).unwrap();
            (state.card_id == "RLK_118t3").then_some(state)
        })
        .collect::<Vec<_>>();
    assert_eq!(zombies.len(), 2);
    assert!(zombies.iter().all(|zombie| {
        zombie.attack == 2
            && zombie.health() == 2
            && zombie.has_keyword("taunt")
            && zombie.has_keyword("reborn")
    }));
    assert_eq!(paid.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        paid.state().player(PlayerId::ONE).resource_spent("corpses"),
        4
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &paid.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), paid.state());

    let mut unpaid = game_with_classes(
        repeated("RLK_118"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut unpaid, PlayerId::ONE, 4);
    play(&mut unpaid, PlayerId::ONE, "RLK_118", None);
    assert_eq!(unpaid.state().player(PlayerId::ONE).board.len(), 2);
    assert!(
        unpaid
            .state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .all(|entity| {
                let zombie = unpaid.state().entity(*entity).unwrap();
                zombie.has_keyword("taunt") && !zombie.has_keyword("reborn")
            })
    );
    assert_eq!(
        unpaid
            .state()
            .player(PlayerId::ONE)
            .resource_spent("corpses"),
        0
    );

    let mut one_space = game_with_classes(
        mixed(&["RLK_503", "CS2_120", "RLK_118"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut one_space, 4);
    advance_to_mana(&mut one_space, PlayerId::ONE, 10);
    play(&mut one_space, PlayerId::ONE, "CS2_120", None);
    play(&mut one_space, PlayerId::ONE, "CS2_120", None);
    assert_eq!(one_space.state().player(PlayerId::ONE).board.len(), 6);
    play(&mut one_space, PlayerId::ONE, "RLK_118", None);
    assert_eq!(one_space.state().player(PlayerId::ONE).board.len(), 7);
    let last = *one_space
        .state()
        .player(PlayerId::ONE)
        .board
        .last()
        .unwrap();
    assert_eq!(one_space.state().entity(last).unwrap().card_id, "RLK_118t3");
    assert!(
        one_space
            .state()
            .entity(last)
            .unwrap()
            .has_keyword("reborn")
    );
    assert_eq!(
        one_space.state().player(PlayerId::ONE).resource("corpses"),
        0
    );
}

#[test]
fn unholy_frenzy_attacks_left_to_right_and_resummons_friendly_minions_that_die() {
    let mut game = game_with_classes(
        mixed(&["CS2_120", "CS2_182", "RLK_056"]),
        repeated("CS2_182"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let crocolisk = play(&mut game, PlayerId::ONE, "CS2_120", None);
    let yeti = play(&mut game, PlayerId::ONE, "CS2_182", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let target = play(&mut game, PlayerId::TWO, "CS2_182", None);
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "RLK_056", Some(target));

    assert_eq!(game.state().entity(target).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.state().entity(crocolisk).unwrap().zone,
        Zone::Graveyard
    );
    assert_eq!(game.state().entity(yeti).unwrap().zone, Zone::Board);
    assert_eq!(game.state().entity(yeti).unwrap().health(), 1);
    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 2);
    let resummoned = board[0];
    assert_ne!(resummoned, crocolisk);
    assert_eq!(game.state().entity(resummoned).unwrap().card_id, "CS2_120");
    assert_eq!(
        (
            game.state().entity(resummoned).unwrap().attack,
            game.state().entity(resummoned).unwrap().health(),
        ),
        (2, 3)
    );
    assert_eq!(board[1], yeti);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn the_scourge_fills_every_open_board_slot_with_replayable_random_undead() {
    let mut game = game_with_classes(
        mixed(&["RLK_122", "CS2_120"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let existing = play(&mut game, PlayerId::ONE, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    play(&mut game, PlayerId::ONE, "RLK_122", None);

    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 7);
    assert_eq!(board[0], existing);
    for entity in &board[1..] {
        let state = game.state().entity(*entity).unwrap();
        let definition = game.runtime().definition(&state.card_id).unwrap();
        assert_eq!(definition.kind, CardKind::Minion);
        assert!(
            definition
                .tags
                .iter()
                .any(|tag| tag == "undead" || tag == "all"),
            "{} is not Undead",
            state.card_id
        );
    }

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn corpse_bride_spends_up_to_ten_corpses_and_raises_a_corpseless_scaled_groom() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_504"]),
        repeated("EX1_161"),
        ["death_knight", "druid"],
    );
    gain_body_bagger_corpses(&mut game, 3);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "RLK_504", None);
    let groom = *game.state().player(PlayerId::ONE).board.last().unwrap();
    let groom_state = game.state().entity(groom).unwrap();
    assert_eq!(groom_state.card_id, "RLK_506t");
    assert_eq!((groom_state.attack, groom_state.health()), (3, 3));
    assert!(groom_state.has_keyword("taunt"));
    assert!(groom_state.has_keyword("no_corpse"));
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        3
    );

    advance_to_mana(&mut game, PlayerId::TWO, 1);
    play(&mut game, PlayerId::TWO, "EX1_161", Some(groom));
    assert_eq!(game.state().entity(groom).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn malignant_horror_spends_once_per_existing_trigger_and_copies_its_current_state() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_745", "CS2_009"]),
        repeated("CS2_120"),
        ["death_knight", "druid"],
    );
    gain_body_bagger_corpses(&mut game, 4);
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let horror = play(&mut game, PlayerId::ONE, "RLK_745", None);
    play(&mut game, PlayerId::ONE, "CS2_009", Some(horror));
    assert_eq!(
        (
            game.state().entity(horror).unwrap().attack,
            game.state().entity(horror).unwrap().health(),
        ),
        (4, 7)
    );
    end_turn(&mut game);

    let horrors = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter_map(|entity| {
            let state = game.state().entity(*entity).unwrap();
            (state.card_id == "RLK_745").then_some(state)
        })
        .collect::<Vec<_>>();
    assert_eq!(horrors.len(), 2);
    assert!(horrors.iter().all(|horror| {
        horror.attack == 4
            && horror.health() == 7
            && horror.has_keyword("reborn")
            && horror.has_keyword("taunt")
    }));
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        4
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn competing_malignant_horror_triggers_cannot_share_one_corpse_payment() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_745", "RLK_745"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut game, 4);
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "RLK_745", None);
    play(&mut game, PlayerId::ONE, "RLK_745", None);

    end_turn(&mut game);

    let horrors = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "RLK_745")
        .count();
    assert_eq!(horrors, 3);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        4
    );
}

#[test]
fn explicit_concede_rejects_invalid_player_without_mutating_state() {
    let mut game = game("CS2_120", "CS2_120");
    let checkpoint = game.state().clone();
    let error = game
        .dispatch(PlayerCommand::ConcedePlayer {
            player: PlayerId(2),
        })
        .unwrap_err();
    assert!(matches!(
        error,
        hearth_core::GameError::InvalidCommandPlayer(PlayerId(2))
    ));
    assert_eq!(game.state(), &checkpoint);
}

#[test]
fn frostmourne_records_each_combat_kill_and_summons_them_after_final_durability() {
    let mut game = game_with_classes(
        repeated("RLK_086"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 6);
    let victims = [
        play(&mut game, PlayerId::TWO, "CS2_120", None),
        play(&mut game, PlayerId::TWO, "CS2_120", None),
        play(&mut game, PlayerId::TWO, "CS2_120", None),
    ];
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let weapon = play(&mut game, PlayerId::ONE, "RLK_086", None);
    let hero = game.state().player(PlayerId::ONE).hero;

    for (index, victim) in victims.into_iter().enumerate() {
        game.dispatch(PlayerCommand::Attack {
            attacker: hero,
            defender: victim,
        })
        .unwrap();
        assert_eq!(game.state().entity(victim).unwrap().zone, Zone::Graveyard);
        if index < 2 {
            assert_eq!(game.state().player(PlayerId::ONE).weapon, Some(weapon));
            end_turn(&mut game);
            end_turn(&mut game);
        }
    }

    assert_eq!(game.state().entity(weapon).unwrap().zone, Zone::Graveyard);
    assert!(game.state().player(PlayerId::ONE).weapon.is_none());
    let summoned = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .map(|entity| game.state().entity(*entity).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(summoned.len(), 3);
    assert!(summoned.iter().all(|minion| {
        minion.card_id == "CS2_120" && minion.attack == 2 && minion.health() == 3
    }));

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn path_of_arthas_catalog_contains_all_26_collectible_cards() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    assert_eq!(
        runtime
            .definitions()
            .filter(|card| card.set == "PATH_OF_ARTHAS" && card.collectible)
            .count(),
        26
    );
}

#[test]
fn return_of_the_lich_king_catalog_contains_all_13_death_knight_collectible_cards() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    assert_eq!(
        runtime
            .definitions()
            .filter(|card| {
                card.set == "RETURN_OF_THE_LICH_KING"
                    && card.collectible
                    && card.class == "death_knight"
            })
            .count(),
        13
    );
}

#[test]
fn soulbreaker_gains_corpses_for_combat_kills_including_final_durability() {
    let mut game = game_with_classes(
        repeated("RLK_012"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let victims = [
        play(&mut game, PlayerId::TWO, "CS2_120", None),
        play(&mut game, PlayerId::TWO, "CS2_120", None),
    ];
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let weapon = play(&mut game, PlayerId::ONE, "RLK_012", None);
    let hero = game.state().player(PlayerId::ONE).hero;
    for (index, victim) in victims.into_iter().enumerate() {
        game.dispatch(PlayerCommand::Attack {
            attacker: hero,
            defender: victim,
        })
        .unwrap();
        assert_eq!(
            game.state().player(PlayerId::ONE).resource("corpses"),
            2 * (index as u32 + 1)
        );
        if index == 0 {
            end_turn(&mut game);
            end_turn(&mut game);
        }
    }
    assert_eq!(game.state().entity(weapon).unwrap().zone, Zone::Graveyard);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn vampiric_blood_always_grants_five_health_and_doubles_only_after_exact_spending() {
    let mut paid = game_with_classes(
        mixed(&["RLK_503", "RLK_051"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    gain_body_bagger_corpses(&mut paid, 3);
    end_turn(&mut paid);
    advance_to_mana(&mut paid, PlayerId::ONE, 2);
    let hero = paid.state().player(PlayerId::ONE).hero;
    let deck_before = paid.state().player(PlayerId::ONE).deck.len();
    play(&mut paid, PlayerId::ONE, "RLK_051", None);
    let hero_state = paid.state().entity(hero).unwrap();
    assert_eq!((hero_state.health(), hero_state.max_health), (40, 40));
    assert_eq!(
        paid.state().player(PlayerId::ONE).deck.len(),
        deck_before - 1
    );
    assert_eq!(paid.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        paid.state().player(PlayerId::ONE).resource_spent("corpses"),
        3
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &paid.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), paid.state());

    let mut unpaid = game_with_classes(
        repeated("RLK_051"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut unpaid, PlayerId::ONE, 2);
    let hero = unpaid.state().player(PlayerId::ONE).hero;
    let deck_before = unpaid.state().player(PlayerId::ONE).deck.len();
    play(&mut unpaid, PlayerId::ONE, "RLK_051", None);
    let hero_state = unpaid.state().entity(hero).unwrap();
    assert_eq!((hero_state.health(), hero_state.max_health), (35, 35));
    assert_eq!(unpaid.state().player(PlayerId::ONE).deck.len(), deck_before);
}

#[test]
fn necrotic_mortician_discovers_only_after_a_recent_friendly_undead_death() {
    let setup = || {
        let mut game = game_with_classes(
            mixed(&["RLK_503", "RLK_116"]),
            repeated("CS2_029"),
            ["death_knight", "mage"],
        );
        advance_to_mana(&mut game, PlayerId::ONE, 1);
        let undead = play(&mut game, PlayerId::ONE, "RLK_503", None);
        advance_to_mana(&mut game, PlayerId::TWO, 4);
        play(&mut game, PlayerId::TWO, "CS2_029", Some(undead));
        assert_eq!(game.state().entity(undead).unwrap().zone, Zone::Graveyard);
        game
    };

    let mut recent = setup();
    advance_to_mana(&mut recent, PlayerId::ONE, 2);
    let mortician = play(&mut recent, PlayerId::ONE, "RLK_116", None);
    let pending = recent
        .state()
        .pending_input
        .as_ref()
        .expect("a recent Undead death should enable the Discover");
    assert_eq!(pending.source, mortician);
    assert_eq!(pending.prompt, "Discover an Unholy Rune card");
    for option in &pending.options {
        let ChoiceValue::Card(card_id) = &option.value else {
            panic!("Mortician returned a non-card option")
        };
        let definition = recent.runtime().definition(card_id).unwrap();
        assert_eq!(definition.class, "death_knight");
        assert!(definition.rune_cost.unholy > 0, "{card_id}");
    }
    recent.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &recent.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), recent.state());

    let mut stale = setup();
    advance_to_mana(&mut stale, PlayerId::ONE, 2);
    end_turn(&mut stale);
    end_turn(&mut stale);
    play(&mut stale, PlayerId::ONE, "RLK_116", None);
    assert!(stale.state().pending_input.is_none());
}

#[test]
fn meat_grinder_removes_one_random_deck_minion_and_gains_four_corpses() {
    let mut game = game_with_classes(
        mixed(&["RLK_120", "CS2_120", "CS2_029"]),
        repeated("CS2_120"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let eligible = game
        .state()
        .player(PlayerId::ONE)
        .deck
        .iter()
        .copied()
        .filter(|entity| game.state().entity(*entity).unwrap().kind == CardKind::Minion)
        .collect::<Vec<_>>();
    assert!(!eligible.is_empty());
    play(&mut game, PlayerId::ONE, "RLK_120", None);
    assert_eq!(
        eligible
            .iter()
            .filter(|entity| game.state().entity(**entity).unwrap().zone == Zone::Removed)
            .count(),
        1
    );
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 4);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn acolyte_of_death_draws_for_friendly_undead_deaths_only() {
    let mut game = game_with_classes(
        mixed(&["RLK_121", "RLK_503", "CS2_120"]),
        repeated("CS2_029"),
        ["death_knight", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    play(&mut game, PlayerId::ONE, "RLK_121", None);
    let undead = play(&mut game, PlayerId::ONE, "RLK_503", None);
    let beast = play(&mut game, PlayerId::ONE, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::TWO, 8);
    let deck_before = game.state().player(PlayerId::ONE).deck.len();
    play(&mut game, PlayerId::TWO, "CS2_029", Some(undead));
    assert_eq!(
        game.state().player(PlayerId::ONE).deck.len(),
        deck_before - 1
    );
    play(&mut game, PlayerId::TWO, "CS2_029", Some(beast));
    assert_eq!(
        game.state().player(PlayerId::ONE).deck.len(),
        deck_before - 1
    );
}

#[test]
fn boneguard_commander_spends_only_for_available_footman_slots() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "RLK_506"]),
        repeated("CS2_032"),
        ["death_knight", "mage"],
    );
    gain_body_bagger_corpses(&mut game, 6);
    advance_to_mana(&mut game, PlayerId::TWO, 7);
    play(&mut game, PlayerId::TWO, "CS2_032", None);
    assert!(game.state().player(PlayerId::ONE).board.is_empty());
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 12);
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    let commander = play(&mut game, PlayerId::ONE, "RLK_506", None);

    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 7);
    assert_eq!(board[0], commander);
    for footman in &board[1..] {
        let state = game.state().entity(*footman).unwrap();
        assert_eq!(state.card_id, "RLK_061t");
        assert_eq!((state.attack, state.health()), (1, 3));
        assert!(state.has_keyword("taunt"));
        assert!(state.has_keyword("no_corpse"));
    }
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 6);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        6
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn mograine_persists_after_death_is_public_and_stacks_once_per_battlecry() {
    let mut game = game_with_classes(
        repeated("RLK_706"),
        repeated("EX1_161"),
        ["death_knight", "druid"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    let first = play(&mut game, PlayerId::ONE, "RLK_706", None);
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 27);
    assert!(
        game.state()
            .player_view(PlayerId::TWO)
            .player(PlayerId::ONE)
            .public_statuses
            .contains(&"mograine".to_owned())
    );

    play(&mut game, PlayerId::TWO, "EX1_161", Some(first));
    assert_eq!(game.state().entity(first).unwrap().zone, Zone::Graveyard);
    end_turn(&mut game);
    let second = play(&mut game, PlayerId::ONE, "RLK_706", None);
    assert_ne!(first, second);
    end_turn(&mut game);
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 21);
    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["mograine_end_turn_damage"],
        6
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn soulstealer_destroys_all_others_and_gains_extra_corpses_for_enemies() {
    let mut game = game_with_classes(
        mixed(&["CS2_120", "RLK_741"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let friendly = play(&mut game, PlayerId::ONE, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let enemies = [
        play(&mut game, PlayerId::TWO, "CS2_120", None),
        play(&mut game, PlayerId::TWO, "CS2_120", None),
    ];
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    let soulstealer = play(&mut game, PlayerId::ONE, "RLK_741", None);

    assert_eq!(game.state().entity(friendly).unwrap().zone, Zone::Graveyard);
    assert!(
        enemies
            .iter()
            .all(|enemy| game.state().entity(*enemy).unwrap().zone == Zone::Graveyard)
    );
    assert_eq!(game.state().player(PlayerId::ONE).board, vec![soulstealer]);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 3);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        0
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn corpse_explosion_spends_per_wave_and_can_reuse_a_corpse_created_mid_cast() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "CS2_231", "RLK_035"]),
        repeated("CS2_182"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let bagger = play(&mut game, PlayerId::ONE, "RLK_503", None);
    let wisp = play(&mut game, PlayerId::ONE, "CS2_231", None);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 1);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let yeti = play(&mut game, PlayerId::TWO, "CS2_182", None);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "RLK_035", None);

    assert_eq!(game.state().entity(wisp).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().entity(bagger).unwrap().health(), 1);
    assert_eq!(game.state().entity(yeti).unwrap().health(), 3);
    assert_eq!(game.state().player(PlayerId::ONE).resource("corpses"), 0);
    assert_eq!(
        game.state().player(PlayerId::ONE).resource_spent("corpses"),
        2
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn blightfang_infections_summon_for_the_opponent_and_silence_removes_one() {
    let mut game = game_with_classes(
        mixed(&["RLK_225", "EX1_332"]),
        mixed(&["CS2_120", "EX1_161"]),
        ["death_knight", "druid"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let silenced = play(&mut game, PlayerId::TWO, "CS2_120", None);
    let infected = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "RLK_225", None);
    assert!(
        game.state()
            .entity(infected)
            .unwrap()
            .has_keyword("deathrattle")
    );
    play(&mut game, PlayerId::ONE, "EX1_332", Some(silenced));
    assert!(
        game.state()
            .entity(silenced)
            .unwrap()
            .scripts_for_hook("on_deathrattle")
            .is_empty()
    );

    advance_to_mana(&mut game, PlayerId::TWO, 2);
    play(&mut game, PlayerId::TWO, "EX1_161", Some(silenced));
    play(&mut game, PlayerId::TWO, "EX1_161", Some(infected));
    let zombies = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "RLK_118t3")
        .count();
    assert_eq!(zombies, 1);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn rimescale_siren_counts_player_cast_spells_while_held_and_freezes_three_distinct_minions() {
    let mut game = game_with_classes(
        mixed(&["NX2_035", "CS2_008"]),
        repeated("CS2_231"),
        ["death_knight", "druid"],
    );
    end_turn(&mut game);
    let enemy_minions = (0..4)
        .map(|_| play(&mut game, PlayerId::TWO, "CS2_231", None))
        .collect::<Vec<_>>();
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let siren = hand_card(&game, PlayerId::ONE, "NX2_035");
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    for _ in 0..3 {
        play(&mut game, PlayerId::ONE, "CS2_008", Some(enemy_hero));
    }
    assert_eq!(
        game.state().entity(siren).unwrap().script_data["rimescale_spells"],
        3
    );
    game.dispatch(PlayerCommand::PlayCard {
        card: siren,
        target: None,
    })
    .unwrap();
    assert_eq!(
        enemy_minions
            .iter()
            .filter(|minion| game.state().entity(**minion).unwrap().frozen)
            .count(),
        3
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn construct_quarter_uses_location_targeting_and_summons_the_current_four_five_horror() {
    let mut game = game_with_classes(
        mixed(&["RLK_503", "NX2_036"]),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let enemy = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let sacrifice = play(&mut game, PlayerId::ONE, "RLK_503", None);
    let location = play(&mut game, PlayerId::ONE, "NX2_036", None);
    assert!(
        game.legal_actions()
            .unwrap()
            .contains(&PlayerCommand::UseLocation {
                location,
                target: Some(sacrifice),
            })
    );
    game.dispatch(PlayerCommand::UseLocation {
        location,
        target: Some(sacrifice),
    })
    .unwrap();

    assert_eq!(
        game.state().entity(sacrifice).unwrap().zone,
        Zone::Graveyard
    );
    assert_eq!(game.state().entity(location).unwrap().health(), 2);
    let horror = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "NX2_036t")
        .unwrap();
    let state = game.state().entity(horror).unwrap();
    assert_eq!((state.attack, state.health()), (4, 5));
    assert!(state.has_keyword("rush"));
    assert!(
        game.legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: horror,
                defender: enemy,
            })
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn frost_queen_sindragosa_summons_both_wings_and_destroys_their_frozen_enemy() {
    let mut game = game_with_classes(
        repeated("NX2_037"),
        repeated("CS2_120"),
        ["death_knight", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let enemy = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    let sindragosa = play(&mut game, PlayerId::ONE, "NX2_037", None);
    let board = game.state().player(PlayerId::ONE).board.clone();
    assert_eq!(board.len(), 3);
    assert_eq!(board[1], sindragosa);
    assert_eq!(game.state().entity(board[0]).unwrap().card_id, "NX2_037t");
    assert_eq!(game.state().entity(board[2]).unwrap().card_id, "NX2_037t2");

    let wing = board[0];
    assert!(
        game.legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: wing,
                defender: enemy,
            })
    );
    game.dispatch(PlayerCommand::Attack {
        attacker: wing,
        defender: enemy,
    })
    .unwrap();
    assert_eq!(game.state().entity(wing).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().entity(enemy).unwrap().zone, Zone::Graveyard);

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
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
    assert!(
        actual.remove("death_knight_corpses"),
        "the Death Knight starting hero installs corpse rules through an internal helper"
    );
    assert!(
        actual.remove("no_corpse"),
        "printed tokens that do not leave Corpses use an internal marker"
    );
    assert!(
        actual.remove("unending_plagues"),
        "Helya's persistent public player rule uses an internal marker"
    );
    assert!(
        actual.remove("frost_plague_surcharge"),
        "Frost Plague's stacked next-card surcharge uses an internal marker"
    );
    assert!(
        actual.remove("frozen_solid"),
        "Frozen Over's temporary per-card play lock uses an internal marker"
    );
    assert!(
        actual.remove("primus_frost_runes"),
        "The Primus' next-spell discount and Spell Damage use an internal player rule"
    );
    assert!(
        actual.remove("mograine"),
        "Mograine's persistent end-turn damage uses a public player rule"
    );
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
fn all_eleven_basic_heroes_and_powers_are_official_standalone_definitions() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let expected = [
        ("HERO_01", "HERO_01bp", "warrior", 2),
        ("HERO_02", "HERO_02bp", "shaman", 2),
        ("HERO_03", "HERO_03bp", "rogue", 2),
        ("HERO_04", "HERO_04bp", "paladin", 2),
        ("HERO_05", "HERO_05bp", "hunter", 2),
        ("HERO_06", "HERO_06bp", "druid", 2),
        ("HERO_07", "HERO_07bp", "warlock", 2),
        ("HERO_08", "HERO_08bp", "mage", 2),
        ("HERO_09", "HERO_09bp", "priest", 2),
        ("HERO_10", "HERO_10bp", "demon_hunter", 1),
        ("HERO_11", "HERO_11bp", "death_knight", 2),
    ];
    for (hero_id, power_id, class, cost) in expected {
        let hero = runtime
            .definition(hero_id)
            .unwrap_or_else(|| panic!("missing basic Hero {hero_id}"));
        assert_eq!(hero.kind, CardKind::Hero, "{hero_id}");
        assert!(hero.collectible, "{hero_id}");
        assert!(!hero.is_deckable(), "{hero_id}");
        assert_eq!(hero.set, "HERO_SKINS", "{hero_id}");
        assert_eq!(hero.class, class, "{hero_id}");
        assert_eq!(hero.health, 30, "{hero_id}");
        assert_eq!(hero.hero_power.as_deref(), Some(power_id), "{hero_id}");

        let definition = runtime
            .definition(power_id)
            .unwrap_or_else(|| panic!("missing basic Hero Power {power_id}"));
        assert_eq!(definition.kind, CardKind::HeroPower, "{power_id}");
        assert!(!definition.collectible, "{power_id}");
        assert_eq!(definition.class, class, "{power_id}");
        assert_eq!(definition.cost, cost, "{power_id}");
    }
    assert!(
        data_path()
            .join("hero_powers/basic/dagger_mastery.lua")
            .is_file()
    );
    assert!(!data_path().join("sets/legacy/dagger_mastery.lua").exists());
}

#[test]
fn constructed_starting_heroes_are_localized_and_replay_exact() {
    let runtime = LuaCardRuntime::load_dir_with_locale(data_path(), Locale::ZhTw).unwrap();
    let game = Game::new_unrestricted_with_hero_powers_and_classes(
        runtime,
        repeated("CS2_120"),
        repeated("CS2_120"),
        79,
        ["HERO_01bp".to_owned(), "HERO_11bp".to_owned()],
        ["warrior".to_owned(), "death_knight".to_owned()],
    )
    .unwrap();
    let warrior = game.state().hero(PlayerId::ONE);
    let death_knight = game.state().hero(PlayerId::TWO);
    assert_eq!(
        (warrior.card_id.as_str(), warrior.name.as_str()),
        ("HERO_01", "卡爾洛斯‧地獄吼")
    );
    assert_eq!(
        (death_knight.card_id.as_str(), death_knight.name.as_str()),
        ("HERO_11", "巫妖王")
    );

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir_with_locale(data_path(), Locale::ZhTw).unwrap(),
        &game.replay(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
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
fn zombeast_dispatches_choose_one_from_its_attached_beast_script() {
    let mut runtime = Some(LuaCardRuntime::load_dir(data_path()).unwrap());
    let mut discovered = None;
    for seed in 0..128 {
        let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
            runtime.take().unwrap(),
            repeated("EX1_144"),
            repeated("CS2_120"),
            seed,
            ["ICC_828p".to_owned(), "HERO_08bp".to_owned()],
            ["hunter".to_owned(), "mage".to_owned()],
        )
        .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        game.dispatch(PlayerCommand::UseHeroPower { target: None })
            .unwrap();
        let robo_cub = game
            .state()
            .pending_input
            .as_ref()
            .unwrap()
            .options
            .iter()
            .position(|option| option.value == ChoiceValue::Card("GVG_030".to_owned()));
        if let Some(index) = robo_cub {
            discovered = Some((game, index));
            break;
        }
        runtime = Some(game.into_runtime());
    }
    let (mut game, robo_cub) = discovered.expect("Robo Cub was not offered in 128 discoveries");

    game.dispatch(PlayerCommand::Choose { index: robo_cub })
        .unwrap();
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    let zombeast = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entities[entity].card_id == "ICC_828t")
        .unwrap();
    assert!(
        game.state().entities[&zombeast]
            .attached_cards
            .contains(&"GVG_030".to_owned())
    );

    advance_to_mana(&mut game, PlayerId::ONE, 10);
    game.dispatch(PlayerCommand::PlayCard {
        card: zombeast,
        target: None,
    })
    .unwrap();
    assert_eq!(
        game.state().pending_input.as_ref().unwrap().prompt,
        "Choose One"
    );
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert!(game.state().player(PlayerId::ONE).board.contains(&zombeast));

    play(&mut game, PlayerId::ONE, "EX1_144", Some(zombeast));
    assert!(game.state().player(PlayerId::ONE).hand.contains(&zombeast));
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    game.dispatch(PlayerCommand::PlayCard {
        card: zombeast,
        target: None,
    })
    .unwrap();
    assert_eq!(
        game.state().pending_input.as_ref().unwrap().prompt,
        "Choose One"
    );
}

#[test]
fn zombeast_inherits_required_targets_from_attached_battlecries() {
    let mut runtime = Some(LuaCardRuntime::load_dir(data_path()).unwrap());
    let mut discovered = None;
    for seed in 0..512 {
        let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
            runtime.take().unwrap(),
            repeated("CS2_120"),
            repeated("CS2_120"),
            seed,
            ["ICC_828p".to_owned(), "HERO_08bp".to_owned()],
            ["hunter".to_owned(), "mage".to_owned()],
        )
        .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        game.dispatch(PlayerCommand::UseHeroPower { target: None })
            .unwrap();
        let dispatch_kodo = game
            .state()
            .pending_input
            .as_ref()
            .unwrap()
            .options
            .iter()
            .position(|option| option.value == ChoiceValue::Card("CFM_335".to_owned()));
        if let Some(index) = dispatch_kodo {
            discovered = Some((game, index));
            break;
        }
        runtime = Some(game.into_runtime());
    }
    let (mut game, dispatch_kodo) =
        discovered.expect("Dispatch Kodo was not offered in 512 discoveries");
    game.dispatch(PlayerCommand::Choose {
        index: dispatch_kodo,
    })
    .unwrap();
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    let zombeast = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entities[entity].card_id == "ICC_828t")
        .unwrap();

    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let plays = game
        .legal_actions()
        .unwrap()
        .into_iter()
        .filter(|command| {
            matches!(
                command,
                PlayerCommand::PlayCard { card, .. } | PlayerCommand::PlayCardAt { card, .. }
                    if *card == zombeast
            )
        })
        .collect::<Vec<_>>();
    assert!(!plays.is_empty());
    assert!(plays.iter().all(|command| match command {
        PlayerCommand::PlayCard { target, .. } | PlayerCommand::PlayCardAt { target, .. } =>
            target.is_some(),
        _ => unreachable!(),
    }));
    game.dispatch(plays.into_iter().next().unwrap()).unwrap();
    assert!(game.state().player(PlayerId::ONE).board.contains(&zombeast));
}

#[test]
fn zombeast_only_dispatches_battlecries_valid_for_the_selected_target() {
    let mut runtime = Some(LuaCardRuntime::load_dir(data_path()).unwrap());
    let mut discovered = None;
    for seed in 0..512 {
        let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
            runtime.take().unwrap(),
            repeated("CS2_120"),
            repeated("UNG_809"),
            seed,
            ["ICC_828p".to_owned(), "HERO_08bp".to_owned()],
            ["hunter".to_owned(), "mage".to_owned()],
        )
        .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        game.dispatch(PlayerCommand::UseHeroPower { target: None })
            .unwrap();
        let huhuran = game
            .state()
            .pending_input
            .as_ref()
            .unwrap()
            .options
            .iter()
            .position(|option| option.value == ChoiceValue::Card("OG_309".to_owned()));
        if let Some(index) = huhuran {
            discovered = Some((game, index));
            break;
        }
        runtime = Some(game.into_runtime());
    }
    let (mut game, huhuran) = discovered.expect("Princess Huhuran was not offered");
    game.dispatch(PlayerCommand::Choose { index: huhuran })
        .unwrap();
    let phoenix = game
        .state()
        .pending_input
        .as_ref()
        .unwrap()
        .options
        .iter()
        .position(|option| option.label == "Fire Plume Phoenix")
        .expect("Fire Plume Phoenix is a valid second Beast");
    game.dispatch(PlayerCommand::Choose { index: phoenix })
        .unwrap();
    let zombeast = hand_card(&game, PlayerId::ONE, "ICC_828t");
    assert_eq!(
        game.state().entities[&zombeast].attached_cards,
        vec!["OG_309".to_owned(), "UNG_084".to_owned()]
    );

    end_turn(&mut game);
    let fire_fly = play(&mut game, PlayerId::TWO, "UNG_809", None);
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    game.dispatch(PlayerCommand::PlayCard {
        card: zombeast,
        target: Some(fire_fly),
    })
    .unwrap();

    assert!(game.state().player(PlayerId::ONE).board.contains(&zombeast));
    assert!(!game.state().player(PlayerId::TWO).board.contains(&fire_fly));
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
