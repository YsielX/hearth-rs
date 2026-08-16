use std::collections::BTreeSet;
use std::path::PathBuf;

use hearth_core::{DEFAULT_HERO_POWER, EntityId, Game, GameEvent, PlayerCommand, PlayerId, Zone};
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

fn game_with(
    one: Vec<String>,
    two: Vec<String>,
    powers: [&str; 2],
    classes: [&str; 2],
    seed: u64,
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        one,
        two,
        seed,
        powers.map(str::to_owned),
        classes.map(str::to_owned),
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn game(one: Vec<String>, two: Vec<String>) -> Game<LuaCardRuntime> {
    game_with(
        one,
        two,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["neutral", "neutral"],
        31,
    )
}

fn end_turn(game: &mut Game<LuaCardRuntime>) {
    game.dispatch(PlayerCommand::EndTurn).unwrap();
}

fn advance_to_mana(game: &mut Game<LuaCardRuntime>, player: PlayerId, mana: u8) {
    while game.state().active_player != player || game.state().player(player).max_mana < mana {
        end_turn(game);
    }
}

fn hand_card(game: &Game<LuaCardRuntime>, player: PlayerId, card_id: &str) -> EntityId {
    game.state()
        .player(player)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == card_id)
        .unwrap_or_else(|| panic!("{player} has no {card_id} in hand"))
}

fn wait_for_hand(game: &mut Game<LuaCardRuntime>, player: PlayerId, card_id: &str) {
    while !game
        .state()
        .player(player)
        .hand
        .iter()
        .any(|entity| game.state().entity(*entity).unwrap().card_id == card_id)
    {
        end_turn(game);
    }
}

fn play(
    game: &mut Game<LuaCardRuntime>,
    player: PlayerId,
    card_id: &str,
    target: Option<EntityId>,
) -> EntityId {
    let card = hand_card(game, player, card_id);
    game.dispatch(PlayerCommand::PlayCard { card, target })
        .unwrap();
    card
}

#[test]
fn tgt_catalog_is_the_exact_132_card_collectible_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|card| card.set == "TGT" && card.collectible)
        .map(|card| card.id.clone())
        .collect::<BTreeSet<_>>();
    let mut expected = (1..=106)
        .chain(108..=125)
        .chain(127..=133)
        .map(|number| format!("AT_{number:03}"))
        .collect::<BTreeSet<_>>();
    expected.insert("AT_063t".to_owned());

    assert_eq!(actual.len(), 132);
    assert_eq!(actual, expected);
}

#[test]
fn dark_bargain_clamps_a_deathrattle_position_after_simultaneous_deaths() {
    let mut game = game(mixed(&["CS2_120", "FP1_002"]), repeated("AT_025"));
    wait_for_hand(&mut game, PlayerId::ONE, "CS2_120");
    wait_for_hand(&mut game, PlayerId::ONE, "FP1_002");
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "CS2_120", None);
    play(&mut game, PlayerId::ONE, "FP1_002", None);

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "AT_025", None);

    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 2);
    assert!(
        board
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().card_id == "FP1_002t")
    );
}

#[test]
fn astral_communion_discards_the_hand_fills_mana_and_gives_excess_mana_at_ten() {
    let mut ramp = game(repeated("AT_043"), repeated("CS2_120"));
    advance_to_mana(&mut ramp, PlayerId::ONE, 5);
    let communion = hand_card(&ramp, PlayerId::ONE, "AT_043");
    let discarded = ramp
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .filter(|entity| *entity != communion)
        .collect::<Vec<_>>();
    ramp.dispatch(PlayerCommand::PlayCard {
        card: communion,
        target: None,
    })
    .unwrap();
    assert_eq!(ramp.state().player(PlayerId::ONE).max_mana, 10);
    assert_eq!(ramp.state().player(PlayerId::ONE).mana, 10);
    assert!(ramp.state().player(PlayerId::ONE).hand.is_empty());
    assert!(
        discarded
            .iter()
            .all(|entity| ramp.state().entity(*entity).unwrap().zone == Zone::Graveyard)
    );

    let mut capped = game(repeated("AT_043"), repeated("CS2_120"));
    advance_to_mana(&mut capped, PlayerId::ONE, 10);
    play(&mut capped, PlayerId::ONE, "AT_043", None);
    let hand = &capped.state().player(PlayerId::ONE).hand;
    assert_eq!(capped.state().player(PlayerId::ONE).max_mana, 10);
    assert_eq!(capped.state().player(PlayerId::ONE).mana, 10);
    assert_eq!(hand.len(), 1);
    assert_eq!(capped.state().entity(hand[0]).unwrap().card_id, "CS2_013t");
}

#[test]
fn varian_draws_the_top_three_in_order_and_puts_drawn_minions_on_board() {
    let mut game = game_with(
        mixed(&["AT_072", "AT_072", "AT_072", "CS2_120", "CS2_029", "AT_064"]),
        repeated("CS2_120"),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["warrior", "neutral"],
        1,
    );
    while game.state().active_player != PlayerId::ONE
        || game.state().player(PlayerId::ONE).max_mana < 10
    {
        if game.state().active_player == PlayerId::ONE
            && game.state().player(PlayerId::ONE).mana >= 2
            && game
                .state()
                .player(PlayerId::ONE)
                .hand
                .iter()
                .any(|entity| game.state().entity(*entity).unwrap().card_id == "AT_064")
        {
            let enemy_hero = game.state().player(PlayerId::TWO).hero;
            play(&mut game, PlayerId::ONE, "AT_064", Some(enemy_hero));
        }
        end_turn(&mut game);
    }

    let expected = game
        .state()
        .player(PlayerId::ONE)
        .deck
        .iter()
        .copied()
        .take(3)
        .collect::<Vec<_>>();
    let expected_minions = expected
        .iter()
        .copied()
        .filter(|entity| {
            game.state().entity(*entity).unwrap().kind == hearth_core::CardKind::Minion
        })
        .collect::<Vec<_>>();
    let expected_spells = expected
        .iter()
        .copied()
        .filter(|entity| game.state().entity(*entity).unwrap().kind == hearth_core::CardKind::Spell)
        .collect::<Vec<_>>();
    assert!(!expected_minions.is_empty());
    let log_start = game.state().log.len();
    play(&mut game, PlayerId::ONE, "AT_072", None);

    let drawn = game.state().log[log_start..]
        .iter()
        .filter_map(|event| match event {
            GameEvent::CardDrawn {
                player: PlayerId::ONE,
                card,
                ..
            } => Some(*card),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(drawn, expected);
    assert!(
        expected_minions
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().zone == Zone::Board)
    );
    assert!(
        expected_spells
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().zone == Zone::Hand)
    );

    let summoned = game.state().log[log_start..]
        .iter()
        .filter_map(|event| match event {
            GameEvent::MinionSummoned {
                player: PlayerId::ONE,
                entity,
            } if expected_minions.contains(entity) => Some(*entity),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(summoned, expected_minions);
}

#[test]
fn wilfred_only_sets_cards_drawn_by_the_current_hero_power_to_zero() {
    let mut game = game_with(
        mixed(&["AT_027", "CS2_200"]),
        repeated("CS2_120"),
        ["HERO_07bp", DEFAULT_HERO_POWER],
        ["warlock", "neutral"],
        37,
    );
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    wait_for_hand(&mut game, PlayerId::ONE, "AT_027");
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    play(&mut game, PlayerId::ONE, "AT_027", None);

    let natural_log = game.state().log.len();
    end_turn(&mut game);
    end_turn(&mut game);
    let natural = game.state().log[natural_log..]
        .iter()
        .find_map(|event| match event {
            GameEvent::CardDrawn {
                player: PlayerId::ONE,
                card,
                source: None,
            } => Some(*card),
            _ => None,
        })
        .unwrap();
    assert_eq!(game.state().entity(natural).unwrap().cost, 6);

    let power = game.state().player(PlayerId::ONE).hero_power;
    let power_log = game.state().log.len();
    game.dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    let tapped = game.state().log[power_log..]
        .iter()
        .find_map(|event| match event {
            GameEvent::CardDrawn {
                player: PlayerId::ONE,
                card,
                source: Some(source),
            } if *source == power => Some(*card),
            _ => None,
        })
        .unwrap();
    assert_eq!(game.state().entity(tapped).unwrap().cost, 0);
}

#[test]
fn bolf_redirects_hero_damage_to_himself() {
    let mut game = game(repeated("AT_124"), repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let bolf = play(&mut game, PlayerId::ONE, "AT_124", None);
    advance_to_mana(&mut game, PlayerId::TWO, 6);
    let hero = game.state().player(PlayerId::ONE).hero;
    play(&mut game, PlayerId::TWO, "CS2_029", Some(hero));

    assert_eq!(game.state().entity(hero).unwrap().damage, 0);
    assert_eq!(game.state().entity(bolf).unwrap().health(), 3);
}

#[test]
fn poisoned_blade_intercepts_the_rogue_power_weapon_replacement() {
    let mut game = game_with(
        repeated("AT_034"),
        repeated("CS2_120"),
        ["HERO_03bp", DEFAULT_HERO_POWER],
        ["rogue", "neutral"],
        39,
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let blade = play(&mut game, PlayerId::ONE, "AT_034", None);
    end_turn(&mut game);
    end_turn(&mut game);
    game.dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();

    assert_eq!(game.state().player(PlayerId::ONE).weapon, Some(blade));
    let blade = game.state().entity(blade).unwrap();
    assert_eq!(blade.attack, 2);
    assert_eq!(blade.health(), 3);
    assert!(!game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::WeaponEquipped { weapon, .. } if *weapon != blade.id
    )));
}

#[test]
fn garrison_commander_allows_two_uses_and_coldarra_allows_mana_limited_uses() {
    let mut garrison = game_with(
        repeated("AT_080"),
        repeated("CS2_120"),
        ["HERO_08bp", DEFAULT_HERO_POWER],
        ["mage", "neutral"],
        41,
    );
    advance_to_mana(&mut garrison, PlayerId::ONE, 2);
    play(&mut garrison, PlayerId::ONE, "AT_080", None);
    advance_to_mana(&mut garrison, PlayerId::ONE, 4);
    let enemy = garrison.state().player(PlayerId::TWO).hero;
    for _ in 0..2 {
        garrison
            .dispatch(PlayerCommand::UseHeroPower {
                target: Some(enemy),
            })
            .unwrap();
    }
    assert_eq!(
        garrison
            .state()
            .player(PlayerId::ONE)
            .hero_power_uses_this_turn,
        2
    );
    assert_eq!(garrison.state().entity(enemy).unwrap().damage, 2);

    let mut coldarra = game_with(
        repeated("AT_008"),
        repeated("CS2_120"),
        ["HERO_08bp", DEFAULT_HERO_POWER],
        ["mage", "neutral"],
        43,
    );
    advance_to_mana(&mut coldarra, PlayerId::ONE, 6);
    play(&mut coldarra, PlayerId::ONE, "AT_008", None);
    advance_to_mana(&mut coldarra, PlayerId::ONE, 10);
    let enemy = coldarra.state().player(PlayerId::TWO).hero;
    for _ in 0..5 {
        coldarra
            .dispatch(PlayerCommand::UseHeroPower {
                target: Some(enemy),
            })
            .unwrap();
    }
    assert_eq!(
        coldarra
            .state()
            .player(PlayerId::ONE)
            .hero_power_uses_this_turn,
        5
    );
    assert_eq!(coldarra.state().entity(enemy).unwrap().damage, 5);
}

#[test]
fn multiple_fallen_heroes_add_to_the_pending_hero_power_damage() {
    let mut game = game_with(
        repeated("AT_003"),
        repeated("CS2_120"),
        ["HERO_08bp", DEFAULT_HERO_POWER],
        ["mage", "neutral"],
        47,
    );
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    play(&mut game, PlayerId::ONE, "AT_003", None);
    play(&mut game, PlayerId::ONE, "AT_003", None);
    let enemy = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::UseHeroPower {
        target: Some(enemy),
    })
    .unwrap();

    assert_eq!(game.state().entity(enemy).unwrap().damage, 3);
}

#[test]
fn justicar_replaces_each_original_class_power_with_its_canonical_upgrade() {
    let cases = [
        ("warrior", "HERO_01bp", "HERO_01bp2"),
        ("shaman", "HERO_02bp", "HERO_02bp2"),
        ("rogue", "HERO_03bp", "HERO_03bp2"),
        ("paladin", "HERO_04bp", "HERO_04bp2"),
        ("hunter", "HERO_05bp", "HERO_05bp2"),
        ("druid", "HERO_06bp", "HERO_06bp2"),
        ("warlock", "HERO_07bp", "HERO_07bp2"),
        ("mage", "HERO_08bp", "HERO_08bp2"),
        ("priest", "HERO_09bp", "HERO_09bp2"),
    ];

    for (class, basic, upgraded) in cases {
        let mut game = game_with(
            repeated("AT_132"),
            repeated("CS2_120"),
            [basic, DEFAULT_HERO_POWER],
            [class, "neutral"],
            47,
        );
        advance_to_mana(&mut game, PlayerId::ONE, 5);
        play(&mut game, PlayerId::ONE, "AT_132", None);
        let power = game.state().player(PlayerId::ONE).hero_power;
        assert_eq!(
            game.state().entity(power).unwrap().card_id,
            upgraded,
            "{class}"
        );

        if class == "warrior" {
            advance_to_mana(&mut game, PlayerId::ONE, 6);
            game.dispatch(PlayerCommand::UseHeroPower { target: None })
                .unwrap();
            assert_eq!(
                game.state()
                    .entity(game.state().player(PlayerId::ONE).hero)
                    .unwrap()
                    .armor,
                4
            );
        }
    }
}
