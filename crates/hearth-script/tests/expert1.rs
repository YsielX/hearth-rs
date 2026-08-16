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
    assert_eq!(expert1, 245, "original Classic collectible count");
}

#[test]
fn azure_drake_draws_and_provides_spell_damage() {
    let mut game = game(repeated("EX1_284"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    let drake = play(&mut game, PlayerId::ONE, "EX1_284", None);

    let drake = game.state().entity(drake).unwrap();
    assert_eq!((drake.attack, drake.health()), (4, 5));
    assert_eq!(drake.spell_damage, 1);
    assert_eq!(game.state().player(PlayerId::ONE).hand.len(), hand_before);
}

#[test]
fn sylvanas_takes_a_random_enemy_minion_on_death() {
    let mut game = game(repeated("EX1_016"), mixed(&["CS2_120", "CS2_076"]));
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();

    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let sylvanas = play(&mut game, PlayerId::ONE, "EX1_016", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    play(&mut game, PlayerId::TWO, "CS2_076", Some(sylvanas));

    assert_eq!(game.state().entity(sylvanas).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.state().entity(crocolisk).unwrap().controller,
        PlayerId::ONE
    );
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .board
            .contains(&crocolisk)
    );
}

#[test]
fn ragnaros_cannot_attack_and_hits_one_random_enemy_at_end_of_turn() {
    let mut game = game(repeated("EX1_298"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    play(&mut game, PlayerId::TWO, "CS2_120", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();

    advance_to_mana(&mut game, PlayerId::ONE, 8);
    let ragnaros = play(&mut game, PlayerId::ONE, "EX1_298", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::Damaged { source, target, amount }
            if *source == ragnaros
                && *amount == 8
                && game.state().entity(*target).unwrap().controller == PlayerId::TWO
    )));

    game.dispatch(PlayerCommand::EndTurn).unwrap();
    assert!(!game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::Attack { attacker, .. } if *attacker == ragnaros
    )));
}

#[test]
fn lifesteal_rejects_recursive_triggers_when_healing_becomes_damage() {
    let mut game = game(mixed(&["EX1_591", "ICC_802"]), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();

    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let auchenai = play(&mut game, PlayerId::ONE, "EX1_591", None);
    game.dispatch(PlayerCommand::EndTurn).unwrap();

    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let hero = game.state().player(PlayerId::ONE).hero;
    let lash = play(&mut game, PlayerId::ONE, "ICC_802", None);

    assert_eq!(game.state().entity(hero).unwrap().health(), 28);
    assert_eq!(game.state().entity(auchenai).unwrap().health(), 4);
    assert_eq!(game.state().entity(crocolisk).unwrap().health(), 2);
    assert_eq!(
        game.state()
            .log
            .iter()
            .filter(|event| matches!(
                event,
                GameEvent::Damaged {
                    source,
                    target,
                    amount: 1,
                } if *source == lash && *target == hero
            ))
            .count(),
        2
    );
}

#[test]
fn power_overwhelming_destroys_at_end_of_turn_unless_silenced() {
    let deck = mixed(&["CS2_120", "EX1_316", "EX1_332"]);
    let mut doomed = game(deck.clone(), repeated("CS2_120"));
    advance_to_mana(&mut doomed, PlayerId::ONE, 3);
    let target = play(&mut doomed, PlayerId::ONE, "CS2_120", None);
    play(&mut doomed, PlayerId::ONE, "EX1_316", Some(target));
    assert_eq!(
        (
            doomed.state().entity(target).unwrap().attack,
            doomed.state().entity(target).unwrap().health()
        ),
        (6, 7)
    );
    doomed.dispatch(PlayerCommand::EndTurn).unwrap();
    assert_eq!(doomed.state().entity(target).unwrap().zone, Zone::Graveyard);

    let mut silenced = game(deck, repeated("CS2_120"));
    advance_to_mana(&mut silenced, PlayerId::ONE, 3);
    let target = play(&mut silenced, PlayerId::ONE, "CS2_120", None);
    play(&mut silenced, PlayerId::ONE, "EX1_316", Some(target));
    play(&mut silenced, PlayerId::ONE, "EX1_332", Some(target));
    silenced.dispatch(PlayerCommand::EndTurn).unwrap();
    let target = silenced.state().entity(target).unwrap();
    assert_eq!(target.zone, Zone::Board);
    assert_eq!((target.attack, target.health()), (2, 3));
}

#[test]
fn ice_lance_freezes_first_and_damages_an_already_frozen_character() {
    let mut game = game(repeated("CS2_031"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;

    play(&mut game, PlayerId::ONE, "CS2_031", Some(enemy_hero));
    assert!(game.state().entity(enemy_hero).unwrap().frozen);
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 30);

    play(&mut game, PlayerId::ONE, "CS2_031", Some(enemy_hero));
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 26);
}

#[test]
fn conceal_grants_stealth_until_the_casters_next_turn() {
    let mut game = game(mixed(&["CS2_120", "EX1_128"]), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let target = play(&mut game, PlayerId::ONE, "CS2_120", None);
    play(&mut game, PlayerId::ONE, "EX1_128", None);
    assert!(game.state().entity(target).unwrap().has_keyword("stealth"));

    game.dispatch(PlayerCommand::EndTurn).unwrap();
    assert!(game.state().entity(target).unwrap().has_keyword("stealth"));
    game.dispatch(PlayerCommand::EndTurn).unwrap();
    assert!(!game.state().entity(target).unwrap().has_keyword("stealth"));
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
        "CS2_031", "DS1_188", "EX1_016", "EX1_076", "EX1_083", "EX1_128", "EX1_130", "EX1_132",
        "EX1_136", "EX1_284", "EX1_289", "EX1_294", "EX1_295", "EX1_298", "EX1_303", "EX1_304",
        "EX1_310", "EX1_316", "EX1_317", "EX1_320", "EX1_323", "EX1_334", "EX1_339", "EX1_341",
        "EX1_345", "EX1_350", "EX1_363", "EX1_365", "EX1_366", "EX1_379", "EX1_384", "EX1_398",
        "EX1_411", "EX1_531", "EX1_533", "EX1_536", "EX1_537", "EX1_538", "EX1_544", "EX1_549",
        "EX1_554", "EX1_557", "EX1_560", "EX1_572", "EX1_583", "EX1_584", "EX1_590", "EX1_594",
        "EX1_596", "EX1_609", "EX1_611", "EX1_612", "EX1_625", "NEW1_005", "NEW1_014", "NEW1_029",
        "NEW1_041", "tt_010",
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
