use std::path::PathBuf;

use hearth_core::{Game, GameEvent, PlayerCommand, PlayerId, Zone};
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

fn game(deck_one: Vec<String>, deck_two: Vec<String>) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        deck_one,
        deck_two,
        0x4558_5031,
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn advance_to_mana(game: &mut Game<LuaCardRuntime>, player: PlayerId, mana: u8) {
    while game.state().active_player != player || game.state().player(player).max_mana < mana {
        game.dispatch(PlayerCommand::EndTurn).unwrap();
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
    let card = hand_card(game, player, card_id);
    game.dispatch(PlayerCommand::PlayCard { card, target })
        .unwrap();
    card
}

#[test]
fn legacy_and_expert1_contain_the_complete_collectible_basic_and_classic_pools() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let legacy = runtime
        .definitions()
        .filter(|card| card.collectible && card.set == "LEGACY")
        .count();
    let expert1 = runtime
        .definitions()
        .filter(|card| card.collectible && card.set == "EXPERT1")
        .count();
    assert_eq!(legacy, 133, "original Basic collectible count");
    assert_eq!(expert1, 239, "original Classic collectible count");
}

#[test]
fn spellbender_delays_the_spell_body_and_replaces_its_actual_target() {
    let mut game = game(repeated("tt_010"), mixed(&["CS2_120", "CS2_092"]));
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "tt_010", None);

    advance_to_mana(&mut game, PlayerId::TWO, 6);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    play(&mut game, PlayerId::TWO, "CS2_092", Some(crocolisk));

    let crocolisk = game.state().entity(crocolisk).unwrap();
    assert_eq!((crocolisk.attack, crocolisk.health()), (2, 3));
    let bender = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "tt_010a")
        .expect("Spellbender should summon its official token");
    let bender = game.state().entity(bender).unwrap();
    assert_eq!((bender.attack, bender.health()), (5, 7));
    assert!(game.state().player(PlayerId::ONE).secrets.is_empty());
}

#[test]
fn betrayal_uses_the_chosen_minion_as_one_simultaneous_damage_source() {
    let mut game = game(
        repeated("EX1_126"),
        mixed(&["CS2_120", "LOOT_315", "CS2_120"]),
    );
    advance_to_mana(&mut game, PlayerId::TWO, 9);
    let left = play(&mut game, PlayerId::TWO, "CS2_120", None);
    let source = play(&mut game, PlayerId::TWO, "LOOT_315", None);
    let right = play(&mut game, PlayerId::TWO, "CS2_120", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    play(&mut game, PlayerId::ONE, "EX1_126", Some(source));

    assert_eq!(game.state().entity(left).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().entity(right).unwrap().zone, Zone::Graveyard);
    let hits = game
        .state()
        .log
        .iter()
        .filter(|event| {
            matches!(event, GameEvent::Damaged { source: actual, target, .. }
                if *actual == source && (*target == left || *target == right))
        })
        .count();
    assert_eq!(hits, 2);
}

#[test]
fn multiple_prophet_velens_multiply_the_pending_amount_in_sequence() {
    let mut game = game(
        mixed(&["EX1_350", "EX1_572", "CS1_130"]),
        repeated("CS2_120"),
    );
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "EX1_350", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    play(&mut game, PlayerId::ONE, "EX1_350", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    let ysera = play(&mut game, PlayerId::ONE, "EX1_572", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "CS1_130", Some(ysera));
    assert_eq!(game.state().entity(ysera).unwrap().zone, Zone::Graveyard);
}

#[test]
fn gorehowl_uses_attack_against_minions_and_durability_against_heroes() {
    let mut game = game(repeated("EX1_411"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    let gorehowl = play(&mut game, PlayerId::ONE, "EX1_411", None);
    let hero = game.state().player(PlayerId::ONE).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender: crocolisk,
    })
    .unwrap();
    let weapon = game.state().entity(gorehowl).unwrap();
    assert_eq!((weapon.attack, weapon.health()), (6, 1));

    game.dispatch(PlayerCommand::EndTurn).unwrap();
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender: enemy_hero,
    })
    .unwrap();
    assert_eq!(game.state().entity(gorehowl).unwrap().zone, Zone::Graveyard);
}

#[test]
fn gladiators_longbow_prevents_combat_damage_to_its_attacking_hero() {
    let mut game = game(repeated("DS1_188"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "DS1_188", None);
    let hero = game.state().player(PlayerId::ONE).hero;
    let before = game.state().entity(hero).unwrap().health();
    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender: crocolisk,
    })
    .unwrap();
    assert_eq!(game.state().entity(hero).unwrap().health(), before);
}

#[test]
fn newly_completed_classic_cards_survive_legal_action_walks_and_replay() {
    let ids = [
        "DS1_188", "EX1_076", "EX1_083", "EX1_130", "EX1_132", "EX1_136", "EX1_289", "EX1_294",
        "EX1_295", "EX1_303", "EX1_304", "EX1_310", "EX1_317", "EX1_320", "EX1_323", "EX1_334",
        "EX1_339", "EX1_341", "EX1_345", "EX1_350", "EX1_363", "EX1_365", "EX1_366", "EX1_379",
        "EX1_384", "EX1_398", "EX1_411", "EX1_531", "EX1_533", "EX1_536", "EX1_537", "EX1_538",
        "EX1_544", "EX1_549", "EX1_554", "EX1_557", "EX1_560", "EX1_572", "EX1_583", "EX1_584",
        "EX1_590", "EX1_594", "EX1_596", "EX1_609", "EX1_611", "EX1_612", "EX1_625", "NEW1_005",
        "NEW1_014", "NEW1_029", "NEW1_041", "tt_010",
    ];
    let deck_one = ids
        .iter()
        .step_by(2)
        .cycle()
        .take(30)
        .map(|id| (*id).to_owned())
        .collect::<Vec<_>>();
    let deck_two = ids
        .iter()
        .skip(1)
        .step_by(2)
        .cycle()
        .take(30)
        .map(|id| (*id).to_owned())
        .collect::<Vec<_>>();
    for seed in 0_u64..6 {
        let mut game = Game::new_unrestricted(
            LuaCardRuntime::load_dir(data_path()).unwrap(),
            deck_one.clone(),
            deck_two.clone(),
            seed,
        )
        .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
            .unwrap();
        for step in 0_usize..160 {
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
            let index = (seed as usize * 37 + step * 41) % actions.len();
            let command = actions[index].clone();
            game.dispatch(command.clone()).unwrap_or_else(|error| {
                panic!("legal command {command:?} failed for seed {seed}, step {step}: {error}")
            });
            game.state()
                .validate()
                .unwrap_or_else(|error| panic!("seed {seed}, step {step}: {error}"));
        }
        if seed == 0 {
            let replayed = Game::from_replay(
                LuaCardRuntime::load_dir(data_path()).unwrap(),
                &game.replay(),
            )
            .unwrap();
            assert_eq!(replayed.state(), game.state());
        }
    }
}
