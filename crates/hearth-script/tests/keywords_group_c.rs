use std::path::PathBuf;

use hearth_core::{DEFAULT_HERO_POWER, EntityId, Game, GameError, PlayerCommand, PlayerId, Zone};
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

fn game_with_hero_powers(
    deck_one: &str,
    deck_two: &str,
    hero_powers: [&str; 2],
    classes: [&str; 2],
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        repeated(deck_one),
        repeated(deck_two),
        37,
        hero_powers.map(str::to_owned),
        classes.map(str::to_owned),
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
    assert_eq!(game.state().active_player, player);
    let card = hand_card(game, player, card_id);
    game.dispatch(PlayerCommand::PlayCard { card, target })
        .unwrap();
    card
}

#[test]
fn lifesteal_vicious_scalehide_heals_its_hero_for_damage_dealt() {
    let mut game = game("GIL_143", "CS2_120");
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    end_turn(&mut game);
    let scalehide = play(&mut game, PlayerId::ONE, "GIL_143", None);
    game.dispatch(PlayerCommand::Attack {
        attacker: scalehide,
        defender: crocolisk,
    })
    .unwrap();
    end_turn(&mut game);
    let hero = game.state().player(PlayerId::ONE).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: crocolisk,
        defender: hero,
    })
    .unwrap();
    assert_eq!(game.state().entity(hero).unwrap().health(), 28);
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::Attack {
        attacker: scalehide,
        defender: enemy_hero,
    })
    .unwrap();
    assert_eq!(game.state().entity(hero).unwrap().health(), 29);
}

#[test]
fn magnetic_wargear_merges_into_a_mech_instead_of_summoning() {
    let deck = ["BOT_309", "BOT_563"]
        .into_iter()
        .cycle()
        .take(20)
        .map(str::to_owned)
        .collect();
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let framebot = play(&mut game, PlayerId::ONE, "BOT_309", None);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let wargear = hand_card(&game, PlayerId::ONE, "BOT_563");
    game.dispatch(PlayerCommand::PlayCardAt {
        card: wargear,
        target: None,
        position: 0,
    })
    .unwrap();
    let merged = game.state().entity(framebot).unwrap();
    assert_eq!((merged.attack, merged.max_health), (7, 10));
    assert_eq!(merged.attached_cards, vec!["BOT_563"]);
    assert_eq!(game.state().entity(wargear).unwrap().zone, Zone::Removed);
    assert_eq!(game.state().player(PlayerId::ONE).board, vec![framebot]);
}

#[test]
fn manathirst_arcane_bolt_upgrades_only_at_eight_mana_crystals() {
    let mut early = game("RLK_843", "CS2_120");
    let early_target = early.state().player(PlayerId::TWO).hero;
    play(&mut early, PlayerId::ONE, "RLK_843", Some(early_target));
    assert_eq!(early.state().entity(early_target).unwrap().health(), 28);

    let mut late = game("RLK_843", "CS2_120");
    advance_to_mana(&mut late, PlayerId::ONE, 8);
    let late_target = late.state().player(PlayerId::TWO).hero;
    play(&mut late, PlayerId::ONE, "RLK_843", Some(late_target));
    assert_eq!(late.state().entity(late_target).unwrap().health(), 27);
}

#[test]
fn mega_windfury_walking_mountain_can_attack_exactly_four_times() {
    let mut game = game("WW_382", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    let mountain = play(&mut game, PlayerId::ONE, "WW_382", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    for _ in 0..4 {
        game.dispatch(PlayerCommand::Attack {
            attacker: mountain,
            defender: enemy_hero,
        })
        .unwrap();
    }
    assert!(
        !game
            .legal_actions()
            .unwrap()
            .contains(&PlayerCommand::Attack {
                attacker: mountain,
                defender: enemy_hero,
            })
    );
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 14);
}

#[test]
fn miniaturize_tigress_plushy_adds_the_official_one_cost_mini() {
    let mut game = game("TOY_811", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "TOY_811", None);
    let mini = hand_card(&game, PlayerId::ONE, "TOY_811t");
    let mini = game.state().entity(mini).unwrap();
    assert_eq!((mini.cost, mini.attack, mini.max_health), (1, 1, 1));
    assert!(mini.has_keyword("rush"));
    assert!(mini.has_keyword("lifesteal"));
    assert!(mini.has_keyword("divine_shield"));
}

#[test]
fn outcast_spectral_sight_draws_an_extra_card_only_from_a_hand_edge() {
    let mut edge = game("BT_491", "CS2_120");
    advance_to_mana(&mut edge, PlayerId::ONE, 2);
    let edge_card = edge.state().player(PlayerId::ONE).hand[0];
    let edge_deck_before = edge.state().player(PlayerId::ONE).deck.len();
    edge.dispatch(PlayerCommand::PlayCard {
        card: edge_card,
        target: None,
    })
    .unwrap();
    assert_eq!(
        edge.state().player(PlayerId::ONE).deck.len(),
        edge_deck_before - 2
    );

    let mut middle = game("BT_491", "CS2_120");
    advance_to_mana(&mut middle, PlayerId::ONE, 2);
    let hand = &middle.state().player(PlayerId::ONE).hand;
    assert!(hand.len() >= 3);
    let middle_card = hand[1];
    let middle_deck_before = middle.state().player(PlayerId::ONE).deck.len();
    middle
        .dispatch(PlayerCommand::PlayCard {
            card: middle_card,
            target: None,
        })
        .unwrap();
    assert_eq!(
        middle.state().player(PlayerId::ONE).deck.len(),
        middle_deck_before - 1
    );
}

#[test]
fn overheal_holy_champion_gains_attack_when_healed_above_full_health() {
    let mut game = game_with_hero_powers(
        "AT_011",
        "CS2_120",
        ["HERO_09bp", DEFAULT_HERO_POWER],
        ["priest", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let champion = play(&mut game, PlayerId::ONE, "AT_011", None);
    end_turn(&mut game);
    end_turn(&mut game);
    game.dispatch(PlayerCommand::UseHeroPower {
        target: Some(champion),
    })
    .unwrap();
    assert_eq!(game.state().entity(champion).unwrap().attack, 3);
}

#[test]
fn overkill_ticket_scalper_draws_two_after_excess_combat_damage() {
    let mut game = game("TRL_015", "CS2_120");
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let crocolisk = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let scalper = play(&mut game, PlayerId::ONE, "TRL_015", None);
    end_turn(&mut game);
    end_turn(&mut game);
    let deck_before = game.state().player(PlayerId::ONE).deck.len();
    game.dispatch(PlayerCommand::Attack {
        attacker: scalper,
        defender: crocolisk,
    })
    .unwrap();
    assert_eq!(
        game.state().player(PlayerId::ONE).deck.len(),
        deck_before - 2
    );
}

#[test]
fn overload_lightning_bolt_queues_one_locked_crystal() {
    let mut game = game("EX1_238", "CS2_120");
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "EX1_238", Some(enemy_hero));
    assert_eq!(game.state().entity(enemy_hero).unwrap().health(), 27);
    assert_eq!(game.state().player(PlayerId::ONE).overload_pending, 1);
    end_turn(&mut game);
    end_turn(&mut game);
    assert_eq!(game.state().player(PlayerId::ONE).overloaded_mana, 1);
}

#[test]
fn passive_deaths_shadow_cannot_be_manually_activated() {
    let mut game = game_with_hero_powers(
        "CS2_120",
        "CS2_120",
        ["ICC_827p", DEFAULT_HERO_POWER],
        ["rogue", "mage"],
    );
    assert!(
        !game
            .legal_actions()
            .unwrap()
            .iter()
            .any(|action| matches!(action, PlayerCommand::UseHeroPower { .. }))
    );
    assert!(matches!(
        game.dispatch(PlayerCommand::UseHeroPower { target: None }),
        Err(GameError::PassiveHeroPower)
    ));
}

#[test]
fn poisonous_cactus_rager_destroys_any_minion_it_damages() {
    let mut game = game("WW_376", "BOT_309");
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    let framebot = play(&mut game, PlayerId::TWO, "BOT_309", None);
    end_turn(&mut game);
    let cactus = play(&mut game, PlayerId::ONE, "WW_376", None);
    end_turn(&mut game);
    end_turn(&mut game);
    game.dispatch(PlayerCommand::Attack {
        attacker: cactus,
        defender: framebot,
    })
    .unwrap();
    assert_eq!(game.state().entity(framebot).unwrap().zone, Zone::Graveyard);
}

#[test]
fn prepare_tunneling_geomancer_spends_remaining_mana_for_a_later_discount() {
    let mut game = game("CATA_EVENT_401", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let geomancer = hand_card(&game, PlayerId::ONE, "CATA_EVENT_401");
    game.dispatch(PlayerCommand::UseCardAction {
        card: geomancer,
        action: "prepare".to_owned(),
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().player(PlayerId::ONE).mana, 0);
    assert_eq!(game.state().entity(geomancer).unwrap().cost, 0);
    assert!(!game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard { card, .. } if *card == geomancer
    )));
    end_turn(&mut game);
    end_turn(&mut game);
    assert!(game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard { card, .. } if *card == geomancer
    )));
}

#[test]
fn quest_caverns_below_starts_in_hand_and_grants_its_reward_on_completion() {
    let mut deck = vec!["UNG_067".to_owned()];
    deck.extend(std::iter::repeat_n("KAR_069".to_owned(), 19));
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    let quest = hand_card(&game, PlayerId::ONE, "UNG_067");
    play(&mut game, PlayerId::ONE, "UNG_067", None);
    assert_eq!(game.state().entity(quest).unwrap().zone, Zone::Secret);
    for index in 0..4 {
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        play(&mut game, PlayerId::ONE, "KAR_069", None);
        if index < 3 {
            end_turn(&mut game);
        }
    }
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "UNG_067t1" })
    );
    assert_eq!(game.state().entity(quest).unwrap().zone, Zone::Graveyard);
}

#[test]
fn questline_raid_the_docks_tracks_pirates_and_resolves_its_first_reward() {
    let mut deck = vec!["SW_028".to_owned()];
    deck.extend(std::iter::repeat_n("CS2_146".to_owned(), 9));
    deck.extend(std::iter::repeat_n("CS2_106".to_owned(), 10));
    let mut game = game_with_decks(deck, repeated("CS2_120"));
    let questline = hand_card(&game, PlayerId::ONE, "SW_028");
    play(&mut game, PlayerId::ONE, "SW_028", None);
    assert_eq!(game.state().entity(questline).unwrap().zone, Zone::Secret);
    end_turn(&mut game);

    for index in 0..3 {
        while !game
            .state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| game.state().entity(*entity).unwrap().card_id == "CS2_146")
        {
            end_turn(&mut game);
        }
        advance_to_mana(&mut game, PlayerId::ONE, 1);
        play(&mut game, PlayerId::ONE, "CS2_146", None);
        if index < 2 {
            end_turn(&mut game);
        }
    }

    assert_eq!(
        game.state().entity(questline).unwrap().script_data["progress"],
        3
    );
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "CS2_106" })
    );
}

#[test]
fn quickdraw_bounty_wrangler_gives_a_coin_when_played_on_its_draw_turn() {
    let mut game = game("WW_363", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let turn = game.state().turn;
    let wrangler = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().entered_hand_turn == Some(turn))
        .expect("a Bounty Wrangler should have been drawn this turn");
    let coins_before = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "GAME_005")
        .count();
    game.dispatch(PlayerCommand::PlayCard {
        card: wrangler,
        target: None,
    })
    .unwrap();
    let coins_after = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "GAME_005")
        .count();
    assert_eq!(coins_after, coins_before + 1);
}
