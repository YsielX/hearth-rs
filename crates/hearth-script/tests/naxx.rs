use std::path::PathBuf;

use hearth_core::{CardRuntime, Game, PlayerCommand, PlayerId, Zone};
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
        7,
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
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
fn naxx_catalog_is_complete_with_only_the_required_tokens() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let collectible = runtime
        .definitions()
        .filter(|card| card.set == "NAXX" && card.collectible)
        .count();
    assert_eq!(collectible, 30);
    for token in ["FP1_002t", "FP1_007t", "FP1_012t", "FP1_014t", "FP1_019t"] {
        assert!(!runtime.definition(token).unwrap().collectible);
    }
}

#[test]
fn echoing_ooze_copies_its_exact_end_of_turn_state() {
    let mut game = game(repeated("FP1_003"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let ooze = play(&mut game, PlayerId::ONE, "FP1_003", None);
    end_turn(&mut game);

    assert_eq!(board_count(&game, PlayerId::ONE, "FP1_003"), 2);
    let copies = &game.state().player(PlayerId::ONE).board;
    assert_ne!(copies[0], copies[1]);
    assert_eq!(
        game.state().entity(copies[0]).unwrap().attack,
        game.state().entity(ooze).unwrap().attack
    );
}

#[test]
fn mad_scientist_moves_the_existing_secret_from_deck() {
    let mut game = game(mixed(&["FP1_004", "EX1_287"]), repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let scientist = play(&mut game, PlayerId::ONE, "FP1_004", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(scientist));

    let secrets = &game.state().player(PlayerId::ONE).secrets;
    assert_eq!(secrets.len(), 1);
    assert_eq!(game.state().entity(secrets[0]).unwrap().zone, Zone::Secret);
    assert_eq!(game.state().entity(secrets[0]).unwrap().card_id, "EX1_287");
}

#[test]
fn nerubar_weblord_applies_and_removes_its_battlecry_cost_aura() {
    let mut game = game(mixed(&["FP1_017", "EX1_015"]), repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let engineer = hand_card(&game, PlayerId::ONE, "EX1_015");
    assert_eq!(game.state().entity(engineer).unwrap().cost, 2);
    let weblord = play(&mut game, PlayerId::ONE, "FP1_017", None);
    assert_eq!(game.state().entity(engineer).unwrap().cost, 4);

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(weblord));
    assert_eq!(game.state().entity(engineer).unwrap().cost, 2);
}

#[test]
fn poison_seeds_destroys_one_batch_then_replaces_each_minion() {
    let mut game = game(mixed(&["FP1_019", "CS2_120"]), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "CS2_120", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "FP1_019", None);

    assert_eq!(board_count(&game, PlayerId::ONE, "FP1_019t"), 1);
    assert_eq!(board_count(&game, PlayerId::TWO, "FP1_019t"), 1);
    assert_eq!(
        game.state()
            .player(PlayerId::ONE)
            .minions_died_history
            .len(),
        1
    );
    assert_eq!(
        game.state()
            .player(PlayerId::TWO)
            .minions_died_history
            .len(),
        1
    );
}

#[test]
fn reincarnate_runs_the_deathrattle_before_returning_a_fresh_minion() {
    let mut game = game(mixed(&["FP1_007", "FP1_025"]), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let egg = play(&mut game, PlayerId::ONE, "FP1_007", None);
    end_turn(&mut game);
    end_turn(&mut game);
    play(&mut game, PlayerId::ONE, "FP1_025", Some(egg));

    assert_eq!(board_count(&game, PlayerId::ONE, "FP1_007t"), 1);
    assert_eq!(board_count(&game, PlayerId::ONE, "FP1_007"), 1);
}

#[test]
fn kelthuzad_uses_death_history_even_after_the_entity_left_the_graveyard_model() {
    let mut game = game(mixed(&["FP1_013", "CS2_120"]), repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let crocolisk = play(&mut game, PlayerId::ONE, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    play(&mut game, PlayerId::ONE, "FP1_013", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(crocolisk));
    assert_eq!(board_count(&game, PlayerId::ONE, "CS2_120"), 0);
    end_turn(&mut game);
    assert_eq!(board_count(&game, PlayerId::ONE, "CS2_120"), 1);
}

#[test]
fn stalagg_and_feugen_share_game_wide_death_history() {
    let mut game = game(mixed(&["FP1_014", "FP1_015"]), repeated("FP1_019"));
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "FP1_015", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "FP1_019", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::ONE, "FP1_014", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "FP1_019", None);

    assert_eq!(board_count(&game, PlayerId::ONE, "FP1_014t"), 1);
}

#[test]
fn baron_rivendare_captures_double_deathrattle_before_simultaneous_deaths() {
    let mut game = game(mixed(&["FP1_031", "FP1_007"]), repeated("FP1_019"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "FP1_007", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "FP1_031", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "FP1_019", None);

    assert_eq!(board_count(&game, PlayerId::ONE, "FP1_007t"), 2);
}

#[test]
fn deaths_bite_weapon_deathrattle_damages_all_minions() {
    let mut game = game(repeated("FP1_021"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "FP1_021", None);
    let hero = game.state().player(PlayerId::ONE).hero;
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender: enemy_hero,
    })
    .unwrap();
    end_turn(&mut game);
    end_turn(&mut game);
    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender: enemy_hero,
    })
    .unwrap();

    assert!(game.state().player(PlayerId::ONE).weapon.is_none());
    assert_eq!(game.state().entity(crocolisk).unwrap().health(), 2);
}

#[test]
fn loatheb_cost_increase_lasts_through_the_enemy_turn_only() {
    let mut game = game(repeated("FP1_030"), repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "FP1_030", None);
    let fireball = hand_card(&game, PlayerId::TWO, "CS2_029");
    assert_eq!(game.state().entity(fireball).unwrap().cost, 9);
    end_turn(&mut game);
    assert_eq!(game.state().entity(fireball).unwrap().cost, 9);
    end_turn(&mut game);
    assert_eq!(game.state().entity(fireball).unwrap().cost, 4);
}
