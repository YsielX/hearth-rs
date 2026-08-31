use std::path::Path;

use hearth_core::{Game, LegalAction, Locale, PlayerCommand, PlayerId};
use hearth_script::LuaCardRuntime;

use super::*;
use crate::{AppError, BotDifficulty, DeckList};

#[test]
fn constructed_hero_powers_are_discovered_from_starting_hero_metadata() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let runtime = LuaCardRuntime::load_dir(root.join("data")).unwrap();
    for (class, expected) in [
        ("warrior", "HERO_01bp"),
        ("shaman", "HERO_02bp"),
        ("rogue", "HERO_03bp"),
        ("paladin", "HERO_04bp"),
        ("hunter", "HERO_05bp"),
        ("druid", "HERO_06bp"),
        ("warlock", "HERO_07bp"),
        ("mage", "HERO_08bp"),
        ("priest", "HERO_09bp"),
        ("demon_hunter", "HERO_10bp"),
        ("death_knight", "HERO_11bp"),
    ] {
        let deck = DeckList {
            name: class.to_owned(),
            format: None,
            class: class.to_owned(),
            cards: Vec::new(),
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };
        assert_eq!(hero_power_for_deck(&runtime, &deck).unwrap(), expected);
    }
}

#[test]
fn match_session_exposes_script_defined_turn_limits() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let game = Game::new_unrestricted(
        LuaCardRuntime::load_dir(root.join("data")).unwrap(),
        std::iter::repeat_n("EX1_560".to_owned(), 20).collect(),
        std::iter::repeat_n("CS2_120".to_owned(), 20).collect(),
        7,
    )
    .unwrap();
    let mut session = MatchSession {
        game,
        deck_names: ["Nozdormu".to_owned(), "Vanilla".to_owned()],
        locale: Locale::EnUs,
    };
    session
        .dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    session
        .dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    assert_eq!(session.turn_time_limit_seconds().unwrap(), None);
    while session.state().active_player != PlayerId::ONE
        || session.state().player(PlayerId::ONE).max_mana < 9
    {
        session.dispatch(PlayerCommand::EndTurn).unwrap();
    }
    let nozdormu = session.state().player(PlayerId::ONE).hand[0];
    session
        .dispatch(PlayerCommand::PlayCard {
            card: nozdormu,
            target: None,
        })
        .unwrap();
    assert_eq!(session.turn_time_limit_seconds().unwrap(), Some(15));
}

#[test]
fn timeout_policy_prefers_end_turn_and_never_concedes() {
    let legal = |command| LegalAction {
        command,
        mana_cost: 0,
        semantic_card_id: None,
    };
    let choices = vec![
        legal(PlayerCommand::Concede),
        legal(PlayerCommand::Choose { index: 0 }),
        legal(PlayerCommand::EndTurn),
    ];
    assert_eq!(timeout_command(&choices), Some(PlayerCommand::EndTurn));
    assert_eq!(
        timeout_command(&choices[..2]),
        Some(PlayerCommand::Choose { index: 0 })
    );
    assert_eq!(timeout_command(&choices[..1]), None);
}

#[test]
fn demo_session_reaches_human_mulligan() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let session = GameSession::load(&MatchConfig::demo(root)).unwrap();
    let view = session.view();
    assert_eq!(view.hero(PlayerId::ONE).card_id, "HERO_08");
    assert_eq!(view.hero(PlayerId::TWO).card_id, "HERO_08");
    assert_eq!(view.input_player, PlayerId::ONE);
    assert_eq!(view.mulligan_eligible.len(), 3);
    assert!(session.legal_actions().unwrap().iter().any(|action| {
        matches!(
            action.command,
            PlayerCommand::Mulligan { ref replace } if replace.is_empty()
        )
    }));
}

#[test]
fn neutral_and_managed_sessions_share_one_authoritative_setup_path() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let config = MatchConfig::demo(root);
    let neutral = MatchSession::load(&config.match_setup()).unwrap();
    let managed = GameSession::load(&config).unwrap();

    assert_eq!(neutral.snapshot(), managed.snapshot().game);
    assert_eq!(
        neutral.deck_name(PlayerId::ONE),
        managed.deck_name(PlayerId::ONE)
    );
    assert_eq!(neutral.locale(), managed.locale());
}

#[test]
fn keeping_the_hand_advances_the_bot_and_starts_the_match() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut session = GameSession::load(&MatchConfig::demo(root)).unwrap();
    session
        .dispatch_human(PlayerCommand::Mulligan {
            replace: Vec::new(),
        })
        .unwrap();

    let view = session.view();
    assert!(view.mulligan_eligible.is_empty());
    assert_eq!(view.input_player, PlayerId::ONE);
    assert_eq!(view.active_player, PlayerId::ONE);
    assert_eq!(view.turn, 1);
    assert!(
        session
            .legal_actions()
            .unwrap()
            .iter()
            .any(|action| matches!(action.command, PlayerCommand::EndTurn))
    );
}

#[test]
fn deferred_dispatch_exposes_exactly_one_bot_action_at_a_time() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut session = GameSession::load(&MatchConfig::demo(root)).unwrap();
    assert!(!session.is_bot_turn());
    assert!(!session.advance_bot_once().unwrap());

    session
        .dispatch_human_only(PlayerCommand::Mulligan {
            replace: Vec::new(),
        })
        .unwrap();
    assert!(session.is_bot_turn());
    let history_before_bot = session.view().history.len();

    assert!(session.advance_bot_once().unwrap());
    assert!(session.view().history.len() > history_before_bot);
    assert!(!session.is_bot_turn());
    assert!(!session.advance_bot_once().unwrap());
    assert_eq!(session.view().turn, 1);
}

#[test]
fn human_can_concede_while_the_bot_owns_input() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut session = GameSession::load(&MatchConfig::demo(root)).unwrap();
    session
        .dispatch_human_only(PlayerCommand::Mulligan {
            replace: Vec::new(),
        })
        .unwrap();
    assert!(session.is_bot_turn());

    session.concede_human().unwrap();

    assert_eq!(
        session.view().outcome,
        Some(hearth_core::GameOutcome::Winner(PlayerId::TWO))
    );
    assert!(matches!(
        session.snapshot().game.replay.commands.last(),
        Some(PlayerCommand::ConcedePlayer {
            player: PlayerId::ONE
        })
    ));
}

#[test]
fn hotseat_keeps_both_players_interactive_and_switches_the_viewer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut config = MatchConfig::demo(root);
    config.match_mode = MatchMode::Hotseat;
    let mut session = GameSession::load(&config).unwrap();

    assert!(session.is_hotseat());
    assert_eq!(session.human_player(), PlayerId::ONE);
    session
        .dispatch_human(PlayerCommand::Mulligan {
            replace: Vec::new(),
        })
        .unwrap();

    assert_eq!(session.human_player(), PlayerId::TWO);
    assert_eq!(session.view().input_player, PlayerId::TWO);
    assert_eq!(session.view().mulligan_eligible.len(), 4);
    let before = session.view().turn;
    session.advance_bot(10_000).unwrap();
    assert_eq!(session.human_player(), PlayerId::TWO);
    assert_eq!(session.view().turn, before);
}

#[test]
fn seeded_opening_order_gives_the_second_player_four_cards_and_the_coin() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut config = MatchConfig::demo(&root);
    config.match_mode = MatchMode::Hotseat;

    assert_eq!(starting_player_for_seed(20260829), PlayerId::ONE);
    assert_eq!(starting_player_for_seed(20260830), PlayerId::TWO);
    config.seed = 20260830;
    let mut session = GameSession::load(&config).unwrap();
    assert_eq!(session.starting_player(), PlayerId::TWO);
    assert_eq!(session.human_player(), PlayerId::TWO);
    assert_eq!(session.view().mulligan_eligible.len(), 3);

    session
        .dispatch_human(PlayerCommand::Mulligan {
            replace: Vec::new(),
        })
        .unwrap();
    assert_eq!(session.human_player(), PlayerId::ONE);
    assert_eq!(session.view().mulligan_eligible.len(), 4);
    session
        .dispatch_human(PlayerCommand::Mulligan {
            replace: Vec::new(),
        })
        .unwrap();

    let snapshot = session.snapshot();
    assert_eq!(snapshot.game.replay.starting_player, PlayerId::TWO);
    assert_eq!(snapshot.game.state.starting_player, PlayerId::TWO);
    assert_eq!(snapshot.game.state.active_player, PlayerId::TWO);
    assert_eq!(snapshot.game.state.turn, 1);
    let second_player = snapshot.game.state.player(PlayerId::ONE);
    assert_eq!(second_player.hand.len(), 5);
    assert!(second_player.hand.iter().any(|entity| {
        snapshot.game.state.entity(*entity).unwrap().card_id == hearth_core::DEFAULT_COIN
    }));
}

#[test]
fn session_snapshot_round_trips_and_rejects_unknown_versions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut config = MatchConfig::demo(&root);
    config.match_mode = MatchMode::Hotseat;
    config.bot_difficulty = BotDifficulty::Hard;
    let mut session = GameSession::load(&config).unwrap();
    session
        .dispatch_human(PlayerCommand::Mulligan {
            replace: Vec::new(),
        })
        .unwrap();

    let json = serde_json::to_string(&session.snapshot()).unwrap();
    let snapshot = serde_json::from_str::<GameSessionSnapshot>(&json).unwrap();
    let restored = GameSession::from_snapshot(&config.data_dir, config.locale, &snapshot).unwrap();
    assert_eq!(restored.view(), session.view());
    assert_eq!(
        restored.legal_actions().unwrap(),
        session.legal_actions().unwrap()
    );
    assert_eq!(restored.match_mode(), MatchMode::Hotseat);
    assert_eq!(restored.bot_difficulty(), BotDifficulty::Hard);
    assert_eq!(
        restored.deck_name(PlayerId::ONE),
        session.deck_name(PlayerId::ONE)
    );

    let mut legacy_value = serde_json::to_value(&snapshot).unwrap();
    legacy_value
        .as_object_mut()
        .unwrap()
        .remove("bot_difficulty");
    legacy_value["game"]["replay"]
        .as_object_mut()
        .unwrap()
        .remove("starting_player");
    legacy_value["game"]["state"]
        .as_object_mut()
        .unwrap()
        .remove("starting_player");
    let legacy = serde_json::from_value::<GameSessionSnapshot>(legacy_value).unwrap();
    let legacy_restored =
        GameSession::from_snapshot(&config.data_dir, config.locale, &legacy).unwrap();
    assert_eq!(legacy_restored.bot_difficulty(), BotDifficulty::Normal);

    let mut unsupported = snapshot;
    unsupported.format_version += 1;
    assert!(matches!(
        GameSession::from_snapshot(&config.data_dir, config.locale, &unsupported),
        Err(AppError::UnsupportedSessionSnapshot(_))
    ));
}
