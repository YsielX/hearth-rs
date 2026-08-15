use std::path::PathBuf;

use hearth_core::{
    DEFAULT_HERO_POWER, EntityId, Game, GameError, GameEvent, PlayerCommand, PlayerId, Zone,
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
        7,
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

fn board_count(game: &Game<LuaCardRuntime>, player: PlayerId, card_id: &str) -> usize {
    game.state()
        .player(player)
        .board
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == card_id)
        .count()
}

#[test]
fn gvg_catalog_contains_all_123_collectible_cards() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    assert_eq!(
        runtime
            .definitions()
            .filter(|card| card.set == "GVG" && card.collectible)
            .count(),
        123
    );
}

#[test]
fn card_level_can_play_rule_blocks_empty_random_target_spells() {
    let mut game = game(repeated("GVG_001"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let flamecannon = hand_card(&game, PlayerId::ONE, "GVG_001");
    assert_eq!(
        game.dispatch(PlayerCommand::PlayCard {
            card: flamecannon,
            target: None,
        }),
        Err(GameError::CardCannotBePlayed(flamecannon))
    );

    end_turn(&mut game);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::ONE, "GVG_001", None);
    assert_eq!(
        game.state().entity(crocolisk).unwrap().zone,
        Zone::Graveyard
    );
}

#[test]
fn feign_death_triggers_deathrattles_and_baron_doubles_them() {
    let mut game = game(
        mixed(&["FP1_002", "FP1_031", "GVG_026"]),
        repeated("CS2_120"),
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let creeper = play(&mut game, PlayerId::ONE, "FP1_002", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "FP1_031", None);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "GVG_026", None);

    assert_eq!(game.state().entity(creeper).unwrap().zone, Zone::Board);
    assert_eq!(board_count(&game, PlayerId::ONE, "FP1_002t"), 4);
}

#[test]
fn steamwheedle_sniper_expands_steady_shot_targets_only_while_present() {
    let mut game = game_with(
        repeated("GVG_087"),
        mixed(&["CS2_120", "CS2_029"]),
        ["HERO_05bp", DEFAULT_HERO_POWER],
        ["hunter", "mage"],
        7,
    );
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let sniper = play(&mut game, PlayerId::ONE, "GVG_087", None);
    let steady_shot = game.state().player(PlayerId::ONE).hero_power;

    assert!(
        game.valid_targets(steady_shot)
            .unwrap()
            .contains(&crocolisk)
    );

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(sniper));
    assert_eq!(game.state().entity(sniper).unwrap().zone, Zone::Graveyard);
    assert!(
        !game
            .valid_targets(steady_shot)
            .unwrap()
            .contains(&crocolisk)
    );
    assert_eq!(
        game.valid_targets(steady_shot).unwrap(),
        vec![game.state().player(PlayerId::TWO).hero]
    );
}

#[test]
fn malorne_keeps_its_entity_dormant_and_revives_after_two_friendly_beasts_die() {
    let mut game = game(
        mixed(&["GVG_035", "CS2_171"]),
        mixed(&["CS2_200", "EX1_005"]),
    );
    advance_to_mana(&mut game, PlayerId::TWO, 6);
    let ogre = play(&mut game, PlayerId::TWO, "CS2_200", None);
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    let malorne = play(&mut game, PlayerId::ONE, "GVG_035", None);
    advance_to_mana(&mut game, PlayerId::TWO, 7);
    play(&mut game, PlayerId::TWO, "EX1_005", Some(malorne));

    assert_eq!(game.state().entity(malorne).unwrap().zone, Zone::Board);
    assert!(game.state().entity(malorne).unwrap().has_keyword("dormant"));
    assert!(game.state().player(PlayerId::ONE).board.contains(&malorne));

    advance_to_mana(&mut game, PlayerId::ONE, 8);
    let first = play(&mut game, PlayerId::ONE, "CS2_171", None);
    game.dispatch(PlayerCommand::Attack {
        attacker: first,
        defender: ogre,
    })
    .unwrap();
    assert!(game.state().entity(malorne).unwrap().has_keyword("dormant"));

    let second = play(&mut game, PlayerId::ONE, "CS2_171", None);
    game.dispatch(PlayerCommand::Attack {
        attacker: second,
        defender: ogre,
    })
    .unwrap();

    assert_eq!(game.state().entity(malorne).unwrap().zone, Zone::Board);
    assert!(!game.state().entity(malorne).unwrap().has_keyword("dormant"));
    assert_eq!(game.state().entity(malorne).unwrap().health(), 7);
}

#[test]
fn voljin_swaps_exact_current_health_without_publishing_healed() {
    let mut game = game(mixed(&["CS2_029", "GVG_014"]), repeated("CS2_200"));
    advance_to_mana(&mut game, PlayerId::TWO, 6);
    let ogre = play(&mut game, PlayerId::TWO, "CS2_200", None);
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    play(&mut game, PlayerId::ONE, "CS2_029", Some(ogre));
    assert_eq!(game.state().entity(ogre).unwrap().health(), 1);

    let log_start = game.state().log.len();
    let voljin = play(&mut game, PlayerId::ONE, "GVG_014", Some(ogre));

    assert_eq!(game.state().entity(voljin).unwrap().health(), 1);
    assert_eq!(game.state().entity(voljin).unwrap().max_health, 1);
    assert_eq!(game.state().entity(ogre).unwrap().health(), 2);
    assert_eq!(game.state().entity(ogre).unwrap().max_health, 2);
    assert!(
        !game.state().log[log_start..]
            .iter()
            .any(|event| matches!(event, GameEvent::Healed { .. }))
    );
}

#[test]
fn sabotage_combo_publishes_weapon_destroyed() {
    let mut game = game(mixed(&["CS2_171", "GVG_047"]), repeated("CS2_106"));
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let weapon = play(&mut game, PlayerId::TWO, "CS2_106", None);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "CS2_171", None);
    let log_start = game.state().log.len();
    play(&mut game, PlayerId::ONE, "GVG_047", None);

    assert!(game.state().player(PlayerId::TWO).weapon.is_none());
    assert_eq!(game.state().entity(weapon).unwrap().zone, Zone::Graveyard);
    assert!(game.state().log[log_start..].iter().any(|event| matches!(
        event,
        GameEvent::WeaponDestroyed {
            player: PlayerId::TWO,
            weapon: destroyed,
        } if *destroyed == weapon
    )));
}

#[test]
fn wrong_attack_uses_set_attack_defender_to_redirect_combat() {
    let mut game = game(repeated("GVG_065"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let brute = play(&mut game, PlayerId::ONE, "GVG_065", None);
    advance_to_mana(&mut game, PlayerId::TWO, 3);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let declared = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: brute,
        defender: declared,
    })
    .unwrap();

    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::Attack {
            attacker,
            defender,
            ..
        } if *attacker == brute && *defender == crocolisk
    )));
    assert_eq!(
        game.state().entity(crocolisk).unwrap().zone,
        Zone::Graveyard
    );
    assert_eq!(game.state().entity(declared).unwrap().health(), 30);
}
