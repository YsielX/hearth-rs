use std::path::PathBuf;

use hearth_core::{
    CardRuntime, ChoiceOptionValueView, ChoiceValue, Game, GameEvent, PlayerCommand, PlayerId,
};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn repeated(card: &str) -> Vec<String> {
    std::iter::repeat_n(card.to_owned(), 20).collect()
}

fn game(card_one: &str, card_two: &str) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        repeated(card_one),
        repeated(card_two),
        0xC001_CE01,
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

fn play(
    game: &mut Game<LuaCardRuntime>,
    player: PlayerId,
    card_id: &str,
    target: Option<hearth_core::EntityId>,
) -> hearth_core::EntityId {
    let card = game
        .state()
        .player(player)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == card_id)
        .unwrap();
    game.dispatch(PlayerCommand::PlayCard { card, target })
        .unwrap();
    card
}

#[test]
fn every_choose_one_branch_has_an_official_noncollectible_card_definition() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let cards_and_options: &[(&str, &[&str])] = &[
        ("ICC_832p", &["ICC_832pb", "ICC_832pa"]),
        ("BRM_010", &["BRM_010a", "BRM_010b"]),
        ("NEW1_008", &["NEW1_008a", "NEW1_008b"]),
        ("EX1_178", &["EX1_178b", "EX1_178a"]),
        ("EX1_573", &["EX1_573a", "EX1_573b"]),
        ("EX1_165", &["EX1_165a", "EX1_165b"]),
        ("EX1_166", &["EX1_166a", "EX1_166b"]),
        ("EX1_155", &["EX1_155a", "EX1_155b"]),
        ("EX1_164", &["EX1_164a", "EX1_164b"]),
        ("EX1_160", &["EX1_160b", "EX1_160a"]),
        ("NEW1_007", &["NEW1_007b", "NEW1_007a"]),
        ("EX1_154", &["EX1_154a", "EX1_154b"]),
        ("CFM_602", &["CFM_602a", "CFM_602b"]),
        ("CFM_308", &["CFM_308a", "CFM_308b"]),
        ("GVG_030", &["GVG_030a", "GVG_030b"]),
        ("GVG_041", &["GVG_041b", "GVG_041a"]),
        ("GVG_032", &["GVG_032a", "GVG_032b"]),
        ("ICC_051", &["ICC_051a", "ICC_051b"]),
        ("ICC_047", &["ICC_047b", "ICC_047a"]),
        ("ICC_832", &["ICC_832a", "ICC_832b"]),
        ("LOE_115", &["LOE_115a", "LOE_115b"]),
        ("OG_047", &["OG_047a", "OG_047b"]),
        ("OG_202", &["OG_202a", "OG_202b"]),
        ("OG_195", &["OG_195a", "OG_195b"]),
        ("AT_042", &["AT_042a", "AT_042b"]),
        ("AT_037", &["AT_037a", "AT_037b"]),
        ("TIME_211", &["TIME_211a", "TIME_211b"]),
        ("UNG_101", &["UNG_101a", "UNG_101b"]),
        ("LOOT_054", &["LOOT_054d", "LOOT_054b", "LOOT_054c"]),
    ];

    for (card_id, option_ids) in cards_and_options {
        assert!(runtime.definition(card_id).is_some(), "missing {card_id}");
        for option_id in *option_ids {
            let option = runtime
                .definition(option_id)
                .unwrap_or_else(|| panic!("{card_id} is missing option card {option_id}"));
            assert!(
                !option.collectible,
                "{card_id} option {option_id} must be noncollectible"
            );
        }
    }
}

#[test]
fn choose_one_is_exposed_as_cards_without_playing_the_selected_option() {
    let mut game = game("EX1_164", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let nourish = play(&mut game, PlayerId::ONE, "EX1_164", None);

    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(
        pending
            .options
            .iter()
            .map(|option| option.value.clone())
            .collect::<Vec<_>>(),
        [
            ChoiceValue::Card("EX1_164a".to_owned()),
            ChoiceValue::Card("EX1_164b".to_owned()),
        ]
    );
    let public = game
        .state()
        .player_view(PlayerId::ONE)
        .pending_input
        .unwrap();
    assert!(public.options.iter().all(|option| matches!(
        option.value,
        ChoiceOptionValueView::Card(ref card_id) if card_id.starts_with("EX1_164")
    )));

    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert_eq!(game.state().player(PlayerId::ONE).max_mana, 7);
    assert!(game.state().pending_input.is_none());
    assert_eq!(
        game.state()
            .log
            .iter()
            .filter(|event| matches!(event, GameEvent::CardPlayed { .. }))
            .count(),
        1
    );
    assert!(
        game.state()
            .entities
            .values()
            .all(|entity| entity.card_id != "EX1_164a" && entity.card_id != "EX1_164b")
    );
    assert_eq!(
        game.state().entity(nourish).unwrap().zone,
        hearth_core::Zone::Graveyard
    );
}

#[test]
fn branching_paths_offers_card_choices_twice_without_casting_them() {
    let mut game = game("LOOT_054", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "LOOT_054", None);

    let first = game.state().pending_input.as_ref().unwrap();
    assert_eq!(
        first
            .options
            .iter()
            .map(|option| option.value.clone())
            .collect::<Vec<_>>(),
        [
            ChoiceValue::Card("LOOT_054d".to_owned()),
            ChoiceValue::Card("LOOT_054b".to_owned()),
            ChoiceValue::Card("LOOT_054c".to_owned()),
        ]
    );
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert!(game.state().pending_input.is_some());
    game.dispatch(PlayerCommand::Choose { index: 2 }).unwrap();

    assert_eq!(game.state().hero(PlayerId::ONE).armor, 6);
    assert!(game.state().pending_input.is_none());
    assert_eq!(
        game.state()
            .log
            .iter()
            .filter(|event| matches!(event, GameEvent::CardPlayed { .. }))
            .count(),
        1
    );
}
