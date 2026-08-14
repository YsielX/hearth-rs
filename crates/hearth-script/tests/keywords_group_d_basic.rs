use std::{collections::BTreeSet, path::PathBuf};

use hearth_core::{CardRuntime, EntityId, Game, PlayerCommand, PlayerId, Zone};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn repeated(card: &str) -> Vec<String> {
    std::iter::repeat_n(card.to_owned(), 20).collect()
}

fn game_with_decks(deck_one: Vec<String>, deck_two: Vec<String>) -> Game<LuaCardRuntime> {
    let mut game = Game::new(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        deck_one,
        deck_two,
        91,
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

fn advance_until_hand(game: &mut Game<LuaCardRuntime>, player: PlayerId, card_id: &str) {
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
    assert_eq!(game.state().active_player, player);
    let card = hand_card(game, player, card_id);
    game.dispatch(PlayerCommand::PlayCard { card, target })
        .unwrap();
    card
}

#[test]
fn recruit_call_to_arms_summons_three_eligible_minions_from_the_deck() {
    let mut deck = std::iter::repeat_n("LOOT_093".to_owned(), 5).collect::<Vec<_>>();
    deck.extend(std::iter::repeat_n("CS2_120".to_owned(), 15));
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_until_hand(&mut game, PlayerId::ONE, "LOOT_093");
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let deck_before = game.state().player(PlayerId::ONE).deck.len();
    play(&mut game, PlayerId::ONE, "LOOT_093", None);

    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.board.len(), 3);
    assert_eq!(player.deck.len(), deck_before - 3);
    assert!(
        player
            .board
            .iter()
            .all(|entity| { game.state().entity(*entity).unwrap().card_id == "CS2_120" })
    );
}

#[test]
fn sidequest_strength_in_numbers_counts_mana_and_recruits_at_ten() {
    let deck = ["DRG_051", "CS2_200", "CS2_120"]
        .into_iter()
        .cycle()
        .take(24)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_until_hand(&mut game, PlayerId::ONE, "DRG_051");
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let sidequest = play(&mut game, PlayerId::ONE, "DRG_051", None);
    assert_eq!(game.state().entity(sidequest).unwrap().zone, Zone::Secret);

    end_turn(&mut game);
    advance_until_hand(&mut game, PlayerId::ONE, "CS2_200");
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    play(&mut game, PlayerId::ONE, "CS2_200", None);
    assert_eq!(
        game.state().entity(sidequest).unwrap().script_data["mana_spent"],
        6
    );

    for expected in [8, 10] {
        end_turn(&mut game);
        advance_until_hand(&mut game, PlayerId::ONE, "CS2_120");
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        play(&mut game, PlayerId::ONE, "CS2_120", None);
        assert_eq!(
            game.state().entity(sidequest).unwrap().script_data["mana_spent"],
            expected
        );
    }

    assert_eq!(
        game.state().entity(sidequest).unwrap().zone,
        Zone::Graveyard
    );
    assert_eq!(game.state().player(PlayerId::ONE).board.len(), 4);
}

#[test]
fn start_of_game_prince_malchezaar_adds_five_unique_legendary_minions() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let mut deck = vec!["KAR_096".to_owned()];
    deck.extend(std::iter::repeat_n("CS2_120".to_owned(), 19));
    let game = Game::new(runtime, deck, repeated("CS2_120"), 91).unwrap();
    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.deck.len() + player.hand.len(), 25);

    let generated = player
        .deck
        .iter()
        .chain(&player.hand)
        .filter_map(|entity| {
            let card_id = &game.state().entity(*entity).unwrap().card_id;
            (card_id != "KAR_096" && card_id != "CS2_120").then_some(card_id.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(generated.len(), 5, "generated cards: {generated:?}");
    assert_eq!(generated.iter().collect::<BTreeSet<_>>().len(), 5);
    assert!(generated.iter().all(|card_id| {
        let definition = game.runtime().definition(card_id).unwrap();
        definition.kind == hearth_core::CardKind::Minion
            && definition.rarity.as_deref() == Some("legendary")
    }));
}

#[test]
fn summoned_when_drawn_frost_tyrant_summons_itself_and_draws_a_replacement() {
    let deck = ["TTN_083", "EX1_169"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    while game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .any(|entity| game.state().entity(*entity).unwrap().card_id == "EX1_169")
    {
        play(&mut game, PlayerId::ONE, "EX1_169", None);
    }
    let deck_before_shuffle = game.state().player(PlayerId::ONE).deck.len();
    play(&mut game, PlayerId::ONE, "TTN_083", None);
    assert_eq!(
        game.state().player(PlayerId::ONE).deck.len(),
        deck_before_shuffle + 4
    );

    end_turn(&mut game);
    let mut observed = false;
    for _ in 0..30 {
        let deck_before_draw = game.state().player(PlayerId::ONE).deck.len();
        end_turn(&mut game);
        let frost_tyrants = game
            .state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .filter(|entity| game.state().entity(**entity).unwrap().card_id == "TTN_083t")
            .count();
        if frost_tyrants > 0 {
            assert_eq!(
                game.state().player(PlayerId::ONE).deck.len(),
                deck_before_draw - 2
            );
            observed = true;
            break;
        }
        while game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| game.state().entity(*entity).unwrap().card_id == "EX1_169")
        {
            play(&mut game, PlayerId::ONE, "EX1_169", None);
        }
        if game.state().player(PlayerId::ONE).board.len() < 7
            && game.state().player(PlayerId::ONE).mana >= 8
            && game
                .state()
                .player(PlayerId::ONE)
                .hand
                .iter()
                .any(|entity| game.state().entity(*entity).unwrap().card_id == "TTN_083")
        {
            play(&mut game, PlayerId::ONE, "TTN_083", None);
        }
        end_turn(&mut game);
    }
    assert!(
        observed,
        "a shuffled Frost Tyrant should eventually be drawn"
    );
}

#[test]
fn twinspell_unleash_the_beast_adds_one_non_twinspell_copy() {
    let mut game = game_with_decks(repeated("DAL_378"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    play(&mut game, PlayerId::ONE, "DAL_378", None);
    assert_eq!(game.state().player(PlayerId::ONE).board.len(), 1);
    let copy = hand_card(&game, PlayerId::ONE, "DAL_378ts");
    assert!(!game.state().entity(copy).unwrap().has_keyword("twinspell"));

    end_turn(&mut game);
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    game.dispatch(PlayerCommand::PlayCard {
        card: copy,
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().player(PlayerId::ONE).board.len(), 2);
    assert!(
        !game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "DAL_378ts" })
    );
}
