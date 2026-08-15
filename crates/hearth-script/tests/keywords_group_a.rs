use std::path::PathBuf;

use hearth_core::{
    ChoiceOptionValueView, ChoiceValue, Game, GameError, GameEvent, PlayerCommand, PlayerId, Zone,
};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn repeated(card: &str) -> Vec<String> {
    std::iter::repeat_n(card.to_owned(), 20).collect()
}

fn mixed(first: &str, second: &str) -> Vec<String> {
    (0..10)
        .flat_map(|_| [first.to_owned(), second.to_owned()])
        .collect()
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

fn game(deck_one: &str, deck_two: &str) -> Game<LuaCardRuntime> {
    game_with_decks(repeated(deck_one), repeated(deck_two))
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

fn keyword(entity: &hearth_core::Entity, keyword: &str) -> bool {
    entity.keywords.iter().any(|value| value == keyword)
}

#[test]
fn adapt_offers_three_adaptations_and_applies_the_chosen_effect() {
    let mut game = game_with_decks(mixed("EX1_008", "UNG_961"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let squire = play(&mut game, PlayerId::ONE, "EX1_008", None);
    let attack = game.state().entity(squire).unwrap().attack;
    let health = game.state().entity(squire).unwrap().max_health;
    play(&mut game, PlayerId::ONE, "UNG_961", Some(squire));

    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(pending.prompt, "Adapt");
    assert_eq!(pending.options.len(), 3);
    let selected = match &pending.options[0].value {
        ChoiceValue::Card(card_id) => card_id.clone(),
        other => panic!("unexpected Adapt choice: {other:?}"),
    };
    let public_choice = game
        .state()
        .player_view(PlayerId::ONE)
        .pending_input
        .unwrap();
    assert!(public_choice.options.iter().all(|option| {
        matches!(&option.value, ChoiceOptionValueView::Card(card_id) if !card_id.is_empty())
    }));
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();

    let adapted = game.state().entity(squire).unwrap();
    match selected.as_str() {
        "UNG_999t2" => assert_eq!(adapted.script_data.get("living_spores"), Some(&1)),
        "UNG_999t3" => assert_eq!(adapted.attack, attack + 3),
        "UNG_999t4" => assert_eq!(adapted.max_health, health + 3),
        "UNG_999t5" => assert!(keyword(adapted, "elusive")),
        "UNG_999t6" => assert!(keyword(adapted, "taunt")),
        "UNG_999t7" => assert!(keyword(adapted, "windfury")),
        "UNG_999t8" => assert!(keyword(adapted, "divine_shield")),
        "UNG_999t10" => assert!(keyword(adapted, "stealth")),
        "UNG_999t13" => assert!(keyword(adapted, "poisonous")),
        "UNG_999t14" => {
            assert_eq!(adapted.attack, attack + 1);
            assert_eq!(adapted.max_health, health + 1);
        }
        other => panic!("unknown Adapt option {other}"),
    }
}

#[test]
fn battlecry_novice_engineer_draws_a_card() {
    let mut game = game("EX1_015", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    let deck_before = game.state().player(PlayerId::ONE).deck.len();
    play(&mut game, PlayerId::ONE, "EX1_015", None);
    assert_eq!(game.state().player(PlayerId::ONE).hand.len(), hand_before);
    assert_eq!(
        game.state().player(PlayerId::ONE).deck.len(),
        deck_before - 1
    );
}

#[test]
fn casts_when_drawn_casts_bomb_damages_its_owner_and_replaces_its_draw() {
    let mut game = game("DAL_060", "EX1_169");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "DAL_060", None);
    assert_eq!(
        game.state()
            .player(PlayerId::TWO)
            .deck
            .iter()
            .filter(|entity| game.state().entity(**entity).unwrap().card_id == "BOT_511t")
            .count(),
        1
    );

    for _ in 0..40 {
        if game
            .state()
            .entities
            .values()
            .any(|entity| entity.card_id == "BOT_511t" && entity.zone == Zone::Graveyard)
        {
            break;
        }
        if game.state().active_player == PlayerId::TWO {
            let innervates = game
                .state()
                .player(PlayerId::TWO)
                .hand
                .iter()
                .copied()
                .filter(|entity| game.state().entity(*entity).unwrap().card_id == "EX1_169")
                .collect::<Vec<_>>();
            for card in innervates {
                game.dispatch(PlayerCommand::PlayCard { card, target: None })
                    .unwrap();
            }
        }
        end_turn(&mut game);
    }
    let bomb = game
        .state()
        .entities
        .values()
        .find(|entity| entity.card_id == "BOT_511t" && entity.zone == Zone::Graveyard)
        .unwrap();
    assert_eq!(bomb.zone, Zone::Graveyard);
    assert_eq!(game.state().hero(PlayerId::TWO).damage, 5);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::SpellCast { spell, .. } if *spell == bomb.id
    )));
    let bomb_draw = game
        .state()
        .log
        .iter()
        .position(|event| {
            matches!(
                event,
                GameEvent::CardDrawn { card, .. } if *card == bomb.id
            )
        })
        .unwrap();
    assert!(
        game.state().log[bomb_draw + 1..]
            .iter()
            .any(|event| matches!(
                event,
                GameEvent::CardDrawn { player, card, .. }
                    if *player == PlayerId::TWO && *card != bomb.id
            ))
    );
}

#[test]
fn charge_minion_can_attack_the_enemy_hero_immediately() {
    let mut game = game("CS2_171", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let boar = play(&mut game, PlayerId::ONE, "CS2_171", None);
    let hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: boar,
        defender: hero,
    })
    .unwrap();
    assert_eq!(game.state().hero(PlayerId::TWO).health(), 29);
}

#[test]
fn choose_one_malfurion_resolves_only_the_selected_option() {
    let mut game = game("ICC_832", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "ICC_832", None);
    assert_eq!(
        game.state().pending_input.as_ref().unwrap().options.len(),
        2
    );
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 2);
    assert!(
        board
            .iter()
            .all(|entity| { game.state().entity(*entity).unwrap().card_id == "ICC_832t4" })
    );
}

#[test]
fn choose_multiple_fandral_combines_both_malfurion_options() {
    let mut game = game_with_decks(mixed("OG_044", "ICC_832"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "OG_044", None);
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "ICC_832", None);
    assert!(game.state().pending_input.is_none());
    let ids = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .map(|entity| game.state().entity(*entity).unwrap().card_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.iter().filter(|id| **id == "ICC_832t3").count(), 2);
    assert_eq!(ids.iter().filter(|id| **id == "ICC_832t4").count(), 2);
}

#[test]
fn choose_multiple_fandral_combines_both_plague_lord_power_options() {
    let mut game = game_with_decks(mixed("OG_044", "ICC_832"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "OG_044", None);
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "ICC_832", None);
    advance_to_mana(&mut game, PlayerId::ONE, 8);

    let hero = game.state().player(PlayerId::ONE).hero;
    let armor_before = game.state().hero(PlayerId::ONE).armor;
    game.dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();

    assert!(game.state().pending_input.is_none());
    assert_eq!(game.state().entity(hero).unwrap().attack, 3);
    assert_eq!(game.state().hero(PlayerId::ONE).armor, armor_before + 3);
}

#[test]
fn colossal_summons_colaques_appendage_and_activates_its_aura() {
    let mut game = game_with_decks(repeated("TSC_026"), mixed("CS2_072", "CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    let colaque = play(&mut game, PlayerId::ONE, "TSC_026", None);
    let ids = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .map(|entity| game.state().entity(*entity).unwrap().card_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["TSC_026", "TSC_026t"]);
    assert!(keyword(game.state().entity(colaque).unwrap(), "immune"));
    let shell = game.state().player(PlayerId::ONE).board[1];
    assert!(keyword(game.state().entity(shell).unwrap(), "taunt"));

    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "CS2_072", Some(shell));
    play(&mut game, PlayerId::TWO, "CS2_029", Some(shell));
    assert_eq!(game.state().entity(shell).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().hero(PlayerId::ONE).armor, 8);
    assert!(!keyword(game.state().entity(colaque).unwrap(), "immune"));
}

#[test]
fn combo_si7_agent_only_deals_damage_after_another_card() {
    let mut game = game_with_decks(mixed("EX1_145", "EX1_134"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "EX1_145", None);
    play(&mut game, PlayerId::ONE, "EX1_134", Some(hero));
    assert_eq!(game.state().hero(PlayerId::TWO).health(), 27);
}

#[test]
fn corrupt_transforms_dunk_tank_after_a_higher_cost_card_is_played() {
    let mut game = game_with_decks(mixed("DMF_701", "CS2_200"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    end_turn(&mut game);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    play(&mut game, PlayerId::ONE, "CS2_200", None);
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "DMF_701t" })
    );
    end_turn(&mut game);
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "DMF_701t", Some(enemy_hero));
    assert_eq!(game.state().hero(PlayerId::TWO).health(), 26);
    assert_eq!(game.state().entity(crocolisk).unwrap().health(), 1);
}

#[test]
fn counterspell_cancels_the_opponents_spell_effect() {
    let mut game = game("EX1_287", "EX1_169");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "EX1_287", None);
    end_turn(&mut game);
    let mana_before = game.state().player(PlayerId::TWO).mana;
    play(&mut game, PlayerId::TWO, "EX1_169", None);
    assert_eq!(game.state().player(PlayerId::TWO).temporary_mana, 0);
    assert_eq!(game.state().player(PlayerId::TWO).mana, mana_before);
    assert!(game.state().player(PlayerId::ONE).secrets.is_empty());
}

#[test]
fn deathrattle_loot_hoarder_draws_when_it_dies() {
    let mut game = game("EX1_096", "CS2_072");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let hoarder = play(&mut game, PlayerId::ONE, "EX1_096", None);
    end_turn(&mut game);
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    play(&mut game, PlayerId::TWO, "CS2_072", Some(hoarder));
    assert_eq!(game.state().entity(hoarder).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.state().player(PlayerId::ONE).hand.len(),
        hand_before + 1
    );
}

#[test]
fn discover_venomous_scorpid_offers_three_spells_and_keeps_the_choice() {
    let mut game = game("BAR_065", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "BAR_065", None);
    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(pending.options.len(), 3);
    let chosen = match &pending.options[0].value {
        ChoiceValue::Card(card_id) => card_id.clone(),
        other => panic!("unexpected Discover choice: {other:?}"),
    };
    game.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == chosen })
    );
}

#[test]
fn divine_shield_prevents_damage_once_and_is_consumed() {
    let mut game = game("EX1_008", "CS2_072");
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let squire = play(&mut game, PlayerId::ONE, "EX1_008", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "CS2_072", Some(squire));
    let squire = game.state().entity(squire).unwrap();
    assert_eq!(squire.health(), 1);
    assert!(!keyword(squire, "divine_shield"));
}

#[test]
fn dormant_satyr_awakens_after_two_friendly_turns_and_reduces_a_minion() {
    let mut game = game_with_decks(mixed("BT_127", "CS2_200"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let satyr = play(&mut game, PlayerId::ONE, "BT_127", None);
    let initial_costs = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .map(|entity| (*entity, game.state().entity(*entity).unwrap().cost))
        .collect::<std::collections::BTreeMap<_, _>>();
    for _ in 0..2 {
        end_turn(&mut game);
        end_turn(&mut game);
    }
    let satyr = game.state().entity(satyr).unwrap();
    assert!(!keyword(satyr, "dormant"));
    assert!(initial_costs.iter().any(|(entity, old_cost)| {
        game.state()
            .entity(*entity)
            .is_some_and(|card| card.zone == Zone::Hand && card.cost < *old_cost)
    }));
}

#[test]
fn dredge_moves_the_selected_bottom_card_to_the_top() {
    let mut game = game("TSC_909", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "TSC_909", None);
    let pending = game.state().pending_input.as_ref().unwrap();
    assert_eq!(pending.prompt, "Dredge");
    assert_eq!(pending.options.len(), 3);
    let selected = match pending.options[1].value {
        ChoiceValue::Entity(entity) => entity,
        ref other => panic!("unexpected Dredge choice: {other:?}"),
    };
    game.dispatch(PlayerCommand::Choose { index: 1 }).unwrap();
    assert_eq!(
        game.state().player(PlayerId::ONE).deck.front(),
        Some(&selected)
    );
}

#[test]
fn echo_copies_inherit_cost_with_a_floor_of_one_and_expire() {
    let mut game = game_with_decks(mixed("BT_127", "GIL_207"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "BT_127", None);
    for _ in 0..2 {
        end_turn(&mut game);
        end_turn(&mut game);
    }
    let original = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .find(|entity| {
            let entity = game.state().entity(*entity).unwrap();
            entity.card_id == "GIL_207" && entity.cost == 0
        })
        .expect("Satyr should reduce an original Phantom Militia to 0");
    assert_eq!(game.state().entity(original).unwrap().cost, 0);
    game.dispatch(PlayerCommand::PlayCard {
        card: original,
        target: None,
    })
    .unwrap();

    let echo_copies = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .filter(|entity| {
            let entity = game.state().entity(*entity).unwrap();
            entity.card_id == "GIL_207" && entity.script_data.get("echo_copy") == Some(&1)
        })
        .collect::<Vec<_>>();
    assert!(!echo_copies.is_empty());
    assert!(
        echo_copies
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().cost >= 1)
    );
    end_turn(&mut game);
    assert!(
        echo_copies
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().zone == Zone::Removed)
    );
}

#[test]
fn elusive_rejects_enemy_spell_targets() {
    let mut game = game("DRG_079", "CS2_029");
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let wyrm = play(&mut game, PlayerId::ONE, "DRG_079", None);
    end_turn(&mut game);
    let fireball = hand_card(&game, PlayerId::TWO, "CS2_029");
    assert_eq!(
        game.dispatch(PlayerCommand::PlayCard {
            card: fireball,
            target: Some(wyrm),
        }),
        Err(GameError::InvalidTarget(wyrm))
    );
}
