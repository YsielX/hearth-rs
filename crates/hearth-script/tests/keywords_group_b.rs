use std::path::PathBuf;

use hearth_core::{CardRuntime, DEFAULT_HERO_POWER, EntityId, Game, PlayerCommand, PlayerId};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn repeated(card: &str) -> Vec<String> {
    std::iter::repeat_n(card.to_owned(), 20).collect()
}

fn game_with(
    one: Vec<String>,
    two: Vec<String>,
    powers: [&str; 2],
    classes: [&str; 2],
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_with_hero_powers_and_classes(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        one,
        two,
        7,
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

fn game(one: &str, two: &str) -> Game<LuaCardRuntime> {
    game_with(
        repeated(one),
        repeated(two),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["neutral", "neutral"],
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

fn count_card_in_player_zones(
    game: &Game<LuaCardRuntime>,
    player: PlayerId,
    card_id: &str,
) -> usize {
    let state = game.state();
    let player = state.player(player);
    player
        .deck
        .iter()
        .chain(player.hand.iter())
        .chain(player.board.iter())
        .filter(|entity| state.entity(**entity).unwrap().card_id == card_id)
        .count()
}

#[test]
fn excavate_kobold_miner_adds_a_tier_one_treasure() {
    let mut game = game("WW_001", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "WW_001", None);

    let tier_one = ["WW_001t", "WW_001t2", "WW_001t3", "WW_001t4", "WW_001t18"];
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| {
                tier_one.contains(&game.state().entity(*entity).unwrap().card_id.as_str())
            })
    );
    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["excavate_count"],
        1
    );
}

#[test]
fn fabled_lady_azshara_adds_both_associated_legendaries() {
    let mut deck = repeated("CS2_120");
    deck[0] = "TIME_211".to_owned();
    let game = game_with(
        deck,
        repeated("CS2_120"),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["druid", "neutral"],
    );

    assert_eq!(
        count_card_in_player_zones(&game, PlayerId::ONE, "TIME_211t1"),
        1
    );
    assert_eq!(
        count_card_in_player_zones(&game, PlayerId::ONE, "TIME_211t2"),
        1
    );
}

#[test]
fn finale_ghost_writer_discovers_twice_only_at_exact_mana() {
    let mut exact = game_with(
        repeated("ETC_088"),
        repeated("CS2_120"),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["mage", "neutral"],
    );
    advance_to_mana(&mut exact, PlayerId::ONE, 5);
    play(&mut exact, PlayerId::ONE, "ETC_088", None);
    assert_eq!(
        exact.state().pending_input.as_ref().unwrap().prompt,
        "Discover a spell"
    );
    exact.dispatch(PlayerCommand::Choose { index: 0 }).unwrap();
    assert_eq!(
        exact.state().pending_input.as_ref().unwrap().prompt,
        "Finale: Discover another spell"
    );

    let mut non_exact = game_with(
        repeated("ETC_088"),
        repeated("CS2_120"),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["mage", "neutral"],
    );
    advance_to_mana(&mut non_exact, PlayerId::ONE, 6);
    play(&mut non_exact, PlayerId::ONE, "ETC_088", None);
    non_exact
        .dispatch(PlayerCommand::Choose { index: 0 })
        .unwrap();
    assert!(non_exact.state().pending_input.is_none());
}

#[test]
fn forge_storm_giant_spends_two_mana_and_reduces_its_cost() {
    let mut game = game("TTN_724", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let giant = hand_card(&game, PlayerId::ONE, "TTN_724");
    game.dispatch(PlayerCommand::UseCardAction {
        card: giant,
        action: "forge".to_owned(),
        target: None,
    })
    .unwrap();
    assert_eq!(game.state().entity(giant).unwrap().cost, 6);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 1);
}

#[test]
fn freeze_frostbolt_damages_and_freezes_its_target() {
    let mut game = game("CS2_024", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "CS2_024", Some(hero));
    assert_eq!(game.state().entity(hero).unwrap().health(), 27);
    assert!(game.state().entity(hero).unwrap().frozen);
}

#[test]
fn frenzy_sunwell_initiate_gains_divine_shield_once_after_surviving_damage() {
    let mut game = game("BAR_025", "CS2_024");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    let initiate = play(&mut game, PlayerId::ONE, "BAR_025", None);
    end_turn(&mut game);
    play(&mut game, PlayerId::TWO, "CS2_024", Some(initiate));
    let initiate = game.state().entity(initiate).unwrap();
    assert_eq!(initiate.health(), 1);
    assert!(initiate.has_keyword("divine_shield"));
    assert!(!initiate.has_keyword("frenzy"));
}

#[test]
fn gigantify_snuggle_teddy_adds_the_official_eight_mana_eight_eight() {
    let mut game = game("MIS_300", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "MIS_300", None);
    let gigantic = hand_card(&game, PlayerId::ONE, "MIS_300t");
    let gigantic = game.state().entity(gigantic).unwrap();
    assert_eq!(
        (gigantic.cost, gigantic.attack, gigantic.health()),
        (8, 8, 8)
    );
    assert!(gigantic.has_keyword("taunt"));
}

#[test]
fn herald_skywall_sentinel_summons_and_upgrades_soldiers() {
    let mut game = game("CATA_565", "CS2_120");
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let first_sentinel = play(&mut game, PlayerId::ONE, "CATA_565", None);
    assert_eq!(
        count_card_in_player_zones(&game, PlayerId::ONE, "CATA_565t"),
        1
    );
    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["herald_count"],
        1
    );

    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "CATA_565", None);
    assert_eq!(
        count_card_in_player_zones(&game, PlayerId::ONE, "CATA_565t"),
        2
    );
    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["herald_count"],
        2
    );
    assert_eq!(game.state().entity(first_sentinel).unwrap().aura_attack, 2);
}

#[test]
fn honorable_kill_gnome_private_requires_exact_lethal_damage() {
    let mut exact = game("AV_121", "EDR_871");
    advance_to_mana(&mut exact, PlayerId::ONE, 1);
    let gnome = play(&mut exact, PlayerId::ONE, "AV_121", None);
    advance_to_mana(&mut exact, PlayerId::TWO, 2);
    let victim = play(&mut exact, PlayerId::TWO, "EDR_871", None);
    end_turn(&mut exact);
    exact
        .dispatch(PlayerCommand::Attack {
            attacker: gnome,
            defender: victim,
        })
        .unwrap();
    assert_eq!(exact.state().entity(gnome).unwrap().attack, 3);
    end_turn(&mut exact);
    let wisp = play(&mut exact, PlayerId::TWO, "CS2_231", None);
    end_turn(&mut exact);
    exact
        .dispatch(PlayerCommand::Attack {
            attacker: gnome,
            defender: wisp,
        })
        .unwrap();
    assert_eq!(exact.state().entity(gnome).unwrap().attack, 3);
}

#[test]
fn imbue_spirit_gatherer_replaces_the_mage_power_and_gets_a_wisp() {
    let mut game = game_with(
        repeated("EDR_871"),
        repeated("CS2_120"),
        ["HERO_08bp", DEFAULT_HERO_POWER],
        ["mage", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "EDR_871", None);
    assert_eq!(
        game.state()
            .entity(game.state().player(PlayerId::ONE).hero_power)
            .unwrap()
            .card_id,
        "EDR_851p"
    );
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "CS2_231" })
    );
    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["imbue_count"],
        1
    );
}

#[test]
fn immune_malganis_protects_the_hero_and_buffs_other_demons() {
    let mut game = game_with(
        repeated("GVG_021"),
        repeated("CS2_024"),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["warlock", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    play(&mut game, PlayerId::ONE, "GVG_021", None);
    let hero = game.state().player(PlayerId::ONE).hero;
    assert!(game.state().entity(hero).unwrap().has_keyword("immune"));
    end_turn(&mut game);
    let frostbolt = hand_card(&game, PlayerId::TWO, "CS2_024");
    assert!(!game.valid_targets(frostbolt).unwrap().contains(&hero));
}

#[test]
fn infuse_priest_of_the_deceased_absorbs_three_friendly_minion_deaths() {
    let mut game = game("REV_956", "CS2_024");
    for mana in 2..=4 {
        advance_to_mana(&mut game, PlayerId::ONE, mana);
        let victim = play(&mut game, PlayerId::ONE, "REV_956", None);
        end_turn(&mut game);
        play(&mut game, PlayerId::TWO, "CS2_024", Some(victim));
    }

    let infused = hand_card(&game, PlayerId::ONE, "REV_956");
    let infused = game.state().entity(infused).unwrap();
    assert_eq!((infused.attack, infused.health()), (4, 5));
    assert!(!infused.has_keyword("infuse"));
}

#[test]
fn inspire_lowly_squire_gains_attack_after_hero_power_use() {
    let mut game = game_with(
        repeated("AT_082"),
        repeated("CS2_120"),
        ["HERO_04bp", DEFAULT_HERO_POWER],
        ["paladin", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let squire = play(&mut game, PlayerId::ONE, "AT_082", None);
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    game.dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    assert_eq!(game.state().entity(squire).unwrap().attack, 2);
}

#[test]
fn invoke_devoted_maniac_uses_warlock_galakronds_power() {
    let mut game = game_with(
        repeated("DRG_050"),
        repeated("CS2_120"),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["warlock", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "DRG_050", None);
    assert_eq!(
        count_card_in_player_zones(&game, PlayerId::ONE, "DRG_238t12t2"),
        2
    );
    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["invoke_count"],
        1
    );
}

#[test]
fn kindred_ambush_predators_repeats_after_a_shadow_spell_last_turn() {
    let mut game = game_with(
        repeated("TLC_519"),
        repeated("CS2_120"),
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
        ["rogue", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "TLC_519", None);
    assert_eq!(
        count_card_in_player_zones(&game, PlayerId::ONE, "TLC_519t"),
        1
    );

    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "TLC_519", None);
    assert_eq!(
        count_card_in_player_zones(&game, PlayerId::ONE, "TLC_519t"),
        3
    );
}

#[test]
fn group_b_examples_are_official_card_library_links() {
    let examples: Vec<serde_json::Value> = serde_json::from_str(
        &std::fs::read_to_string(data_path().join("keyword_examples/group_b.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(examples.len(), 15);
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    for example in examples {
        let card_id = example["card_id"].as_str().unwrap();
        assert!(runtime.definition(card_id).is_some(), "missing {card_id}");
        assert!(
            example["official_url"]
                .as_str()
                .unwrap()
                .starts_with("https://hearthstone.blizzard.com/")
        );
    }
}
