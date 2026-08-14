use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use hearth_core::{CardRuntime, EntityId, Game, PlayerCommand, PlayerId, Zone};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn repeated(card: &str) -> Vec<String> {
    std::iter::repeat_n(card.to_owned(), 20).collect()
}

fn game(deck_one: &str, deck_two: &str) -> Game<LuaCardRuntime> {
    game_with_decks(repeated(deck_one), repeated(deck_two))
}

fn game_with_decks(deck_one: Vec<String>, deck_two: Vec<String>) -> Game<LuaCardRuntime> {
    let mut game = Game::new(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        deck_one,
        deck_two,
        37,
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn game_with_hero_power(deck: &str, hero_power: &str) -> Game<LuaCardRuntime> {
    let mut game = Game::new_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        repeated(deck),
        repeated("CS2_120"),
        37,
        [hero_power.to_owned(), "HERO_08bp".to_owned()],
        ["rogue".to_owned(), "mage".to_owned()],
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

#[test]
fn rewind_semi_stable_portal_can_keep_or_reroll_a_reduced_random_minion() {
    for choice in [0, 1] {
        let mut game = game("TIME_000", "CS2_120");
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        let before = game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        play(&mut game, PlayerId::ONE, "TIME_000", None);
        assert_eq!(
            game.state().pending_input.as_ref().unwrap().options.len(),
            2
        );
        game.dispatch(PlayerCommand::Choose { index: choice })
            .unwrap();
        let generated = game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .copied()
            .find(|entity| !before.contains(entity))
            .unwrap();
        let entity = game.state().entity(generated).unwrap();
        let printed = game.runtime().definition(&entity.card_id).unwrap();
        assert_eq!(entity.cost, printed.cost.saturating_sub(3));
        assert_eq!(printed.kind, hearth_core::CardKind::Minion);

        let replay = game.replay();
        let restored =
            Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
        assert_eq!(restored.state(), game.state());
    }
}

#[test]
fn shatter_arcane_flow_splits_at_opposite_hand_ends() {
    let game = game("CATA_489", "CS2_120");
    let hand = &game.state().player(PlayerId::ONE).hand;
    assert!(hand.len() >= 2);
    assert_eq!(game.state().entity(hand[0]).unwrap().card_id, "CATA_489t");
    assert_eq!(
        game.state().entity(*hand.last().unwrap()).unwrap().card_id,
        "CATA_489t2"
    );
    assert!(
        game.state()
            .entities
            .values()
            .any(|entity| entity.card_id == "CATA_489" && entity.zone == Zone::Removed)
    );
}

#[test]
fn starship_piece_dies_assembles_and_launches_the_combined_ship() {
    let mut game = game("GDB_100", "CS2_029");
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let piece = play(&mut game, PlayerId::ONE, "GDB_100", None);
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(piece));
    assert_eq!(game.state().hero(PlayerId::ONE).armor, 4);
    let ship = hand_card(&game, PlayerId::ONE, "GDB_100t2");
    assert_eq!(game.state().entity(ship).unwrap().attack, 3);
    assert_eq!(game.state().entity(ship).unwrap().max_health, 4);

    advance_to_mana(&mut game, PlayerId::ONE, 5);
    game.dispatch(PlayerCommand::UseCardAction {
        card: ship,
        action: "launch".to_owned(),
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().entity(ship).unwrap().zone, Zone::Board);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 0);
}

#[test]
fn temporary_shadow_reflection_expires_even_after_transforming() {
    let mut untouched = game_with_hero_power("CS2_120", "ICC_827p");
    let reflection = hand_card(&untouched, PlayerId::ONE, "ICC_827t");
    assert!(
        untouched
            .state()
            .entity(reflection)
            .unwrap()
            .has_keyword("temporary")
    );
    end_turn(&mut untouched);
    assert_eq!(
        untouched.state().entity(reflection).unwrap().zone,
        Zone::Removed
    );

    let mut transformed = game_with_hero_power("CS2_120", "ICC_827p");
    advance_to_mana(&mut transformed, PlayerId::ONE, 2);
    let reflection = hand_card(&transformed, PlayerId::ONE, "ICC_827t");
    play(&mut transformed, PlayerId::ONE, "CS2_120", None);
    assert_eq!(
        transformed.state().entity(reflection).unwrap().card_id,
        "CS2_120"
    );
    end_turn(&mut transformed);
    assert_eq!(
        transformed.state().entity(reflection).unwrap().zone,
        Zone::Removed
    );
}

#[test]
fn amitus_caps_damage_and_uses_a_titan_ability_once_per_turn() {
    let mut game = game("TTN_858", "CS2_029");
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    let amitus = play(&mut game, PlayerId::ONE, "TTN_858", None);
    game.dispatch(PlayerCommand::UseCardAction {
        card: amitus,
        action: "titan_1".to_owned(),
        target: None,
    })
    .unwrap();
    assert_eq!(
        game.state().entity(amitus).unwrap().script_data["titan_uses_this_turn"],
        1
    );
    assert!(!game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::UseCardAction { card, .. } if *card == amitus
    )));

    end_turn(&mut game);
    let health_before = game.state().entity(amitus).unwrap().health();
    play(&mut game, PlayerId::TWO, "CS2_029", Some(amitus));
    assert_eq!(
        game.state().entity(amitus).unwrap().health(),
        health_before - 2
    );
}

#[test]
fn buttons_draws_one_spell_from_each_school_and_grants_shaman_tourism() {
    let mut deck = vec!["VAC_437".to_owned(); 15];
    deck.extend(
        ["CS2_029", "EX1_238", "CS1_113", "RLK_038", "CFM_305"]
            .into_iter()
            .map(str::to_owned),
    );
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let schools = game
        .state()
        .player(PlayerId::ONE)
        .deck
        .iter()
        .filter_map(|entity| {
            let card_id = &game.state().entity(*entity).unwrap().card_id;
            game.runtime()
                .definition(card_id)
                .and_then(|definition| definition.spell_school.clone())
        })
        .collect::<BTreeSet<_>>();
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    play(&mut game, PlayerId::ONE, "VAC_437", None);
    assert_eq!(
        game.state().player(PlayerId::ONE).hand.len(),
        hand_before - 1 + schools.len()
    );

    let allowances = game.runtime().deck_allowances("VAC_437").unwrap();
    assert_eq!(allowances.len(), 1);
    assert_eq!(allowances[0].class, "shaman");
    assert_eq!(allowances[0].set, "ISLAND_VACATION");
    assert_eq!(allowances[0].excluded_keywords, vec!["tourist"]);
}
