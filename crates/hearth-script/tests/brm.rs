use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use hearth_core::{DEFAULT_HERO_POWER, EntityId, Game, PlayerCommand, PlayerId, Zone};
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

fn game_with_runtime(
    runtime: LuaCardRuntime,
    one: Vec<String>,
    two: Vec<String>,
    classes: [&str; 2],
    seed: u64,
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
        runtime,
        one,
        two,
        seed,
        [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()],
        classes.map(str::to_owned),
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn game(one: Vec<String>, two: Vec<String>) -> Game<LuaCardRuntime> {
    game_with_runtime(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        one,
        two,
        ["neutral", "neutral"],
        17,
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

fn wait_for_hand(game: &mut Game<LuaCardRuntime>, player: PlayerId, card_id: &str) {
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
fn brm_catalog_is_the_exact_31_card_collectible_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|card| card.set == "BRM" && card.collectible)
        .map(|card| card.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = [
        "BRM_001", "BRM_002", "BRM_003", "BRM_004", "BRM_005", "BRM_006", "BRM_007", "BRM_008",
        "BRM_009", "BRM_010", "BRM_011", "BRM_012", "BRM_013", "BRM_014", "BRM_015", "BRM_016",
        "BRM_017", "BRM_018", "BRM_019", "BRM_020", "BRM_022", "BRM_024", "BRM_025", "BRM_026",
        "BRM_027", "BRM_028", "BRM_029", "BRM_030", "BRM_031", "BRM_033", "BRM_034",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual.len(), 31);
    assert_eq!(actual, expected);
}

#[test]
fn majordomo_replaces_the_hero_at_eight_health_and_installs_ragnaros_power() {
    let mut game = game(repeated("BRM_027"), repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    let old_hero = game.state().player(PlayerId::ONE).hero;
    play(&mut game, PlayerId::ONE, "BRM_027", None);

    advance_to_mana(&mut game, PlayerId::TWO, 9);
    let majordomo = game.state().player(PlayerId::ONE).board[0];
    play(&mut game, PlayerId::TWO, "CS2_029", Some(majordomo));
    play(&mut game, PlayerId::TWO, "CS2_029", Some(majordomo));

    let player = game.state().player(PlayerId::ONE);
    let ragnaros = player.hero;
    assert_ne!(ragnaros, old_hero);
    assert_eq!(game.state().entity(old_hero).unwrap().zone, Zone::Removed);
    assert_eq!(game.state().entity(ragnaros).unwrap().card_id, "BRM_027h");
    assert_eq!(game.state().entity(ragnaros).unwrap().health(), 8);
    assert_eq!(
        game.state().entity(player.hero_power).unwrap().card_id,
        "BRM_027p"
    );

    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    game.dispatch(PlayerCommand::UseHeroPower { target: None })
        .unwrap();
    assert_eq!(game.state().entity(enemy_hero).unwrap().damage, 8);
}

#[test]
fn dragonkin_sorcerer_only_buffs_when_it_is_the_declared_spell_target() {
    let mut game = game(
        mixed(&["BRM_020", "CS2_120", "BRM_005", "BRM_011"]),
        repeated("CS2_171"),
    );
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    for card_id in ["BRM_020", "CS2_120", "BRM_005", "BRM_011"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let sorcerer = play(&mut game, PlayerId::ONE, "BRM_020", None);
    let crocolisk = play(&mut game, PlayerId::ONE, "CS2_120", None);
    play(&mut game, PlayerId::ONE, "BRM_005", None);
    assert_eq!(game.state().entity(sorcerer).unwrap().attack, 3);
    assert_eq!(game.state().entity(sorcerer).unwrap().max_health, 5);

    end_turn(&mut game);
    let boar = play(&mut game, PlayerId::TWO, "CS2_171", None);
    game.dispatch(PlayerCommand::Attack {
        attacker: boar,
        defender: sorcerer,
    })
    .unwrap();
    assert_eq!(game.state().entity(sorcerer).unwrap().health(), 2);

    end_turn(&mut game);

    play(&mut game, PlayerId::ONE, "BRM_011", Some(crocolisk));
    assert_eq!(game.state().entity(sorcerer).unwrap().attack, 3);
    assert_eq!(game.state().entity(sorcerer).unwrap().max_health, 5);

    play(&mut game, PlayerId::ONE, "BRM_011", Some(sorcerer));
    let buffed = game.state().entity(sorcerer).unwrap();
    assert_eq!((buffed.attack, buffed.max_health), (4, 6));
    assert_eq!(buffed.health(), 1);
    assert_eq!(buffed.zone, Zone::Board);
}

static TEMP_RUNTIME_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRuntimeDir(PathBuf);

impl Drop for TempRuntimeDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn consort_test_runtime() -> (TempRuntimeDir, LuaCardRuntime) {
    let suffix = TEMP_RUNTIME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("hearth-rs-brm-{}-{suffix}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("sets"), root.join("sets")).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("hero_powers"), root.join("hero_powers")).unwrap();
    std::fs::write(
        root.join("repeat_battlecry.lua"),
        r#"
return {
    api_version = 1,
    id = "TEST_BRM_REPEAT_BATTLECRY",
    name = "Repeat Battlecry",
    text = "Trigger a friendly minion's Battlecry twice.",
    set = "TEST",
    type = "spell",
    cost = 0,
    target_mode = "required",
    targets = function(ctx, self) return ctx:friendly_minions(self) end,
    on_play = function(ctx, self, target)
        ctx:trigger_hook(target, "on_battlecry")
        ctx:trigger_hook(target, "on_battlecry")
        ctx:move(target, "hand")
    end,
}
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(&root).unwrap();
    (TempRuntimeDir(root), runtime)
}

#[test]
fn dragon_consort_discount_is_player_owned_stacks_and_only_the_next_dragon_consumes_it() {
    let (runtime_dir, runtime) = consort_test_runtime();
    let one = mixed(&[
        "BRM_018",
        "TEST_BRM_REPEAT_BATTLECRY",
        "EX1_332",
        "BRM_027",
        "BRM_030",
        "BRM_026",
    ]);
    let two = mixed(&["CS2_022", "CS2_029"]);
    let mut game = game_with_runtime(runtime, one, two, ["paladin", "mage"], 29);

    advance_to_mana(&mut game, PlayerId::ONE, 6);
    for card_id in ["BRM_018", "TEST_BRM_REPEAT_BATTLECRY", "EX1_332"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    let consort = play(&mut game, PlayerId::ONE, "BRM_018", None);
    play(
        &mut game,
        PlayerId::ONE,
        "TEST_BRM_REPEAT_BATTLECRY",
        Some(consort),
    );
    assert_eq!(game.state().entity(consort).unwrap().zone, Zone::Hand);
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    game.dispatch(PlayerCommand::PlayCard {
        card: consort,
        target: None,
    })
    .unwrap();
    play(&mut game, PlayerId::ONE, "EX1_332", Some(consort));
    assert!(
        !game
            .state()
            .entity(consort)
            .unwrap()
            .has_keyword("battlecry")
    );

    wait_for_hand(&mut game, PlayerId::ONE, "BRM_030");
    let nefarian = hand_card(&game, PlayerId::ONE, "BRM_030");
    assert_eq!(game.state().entity(nefarian).unwrap().cost, 3);

    wait_for_hand(&mut game, PlayerId::TWO, "CS2_022");
    wait_for_hand(&mut game, PlayerId::TWO, "CS2_029");
    advance_to_mana(&mut game, PlayerId::TWO, 8);
    play(&mut game, PlayerId::TWO, "CS2_022", Some(consort));
    assert_eq!(game.state().entity(consort).unwrap().card_id, "CS2_tk1");
    play(&mut game, PlayerId::TWO, "CS2_029", Some(consort));
    assert_eq!(game.state().entity(consort).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().entity(nefarian).unwrap().cost, 3);

    wait_for_hand(&mut game, PlayerId::ONE, "BRM_027");
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    let majordomo = play(&mut game, PlayerId::ONE, "BRM_027", None);
    wait_for_hand(&mut game, PlayerId::TWO, "CS2_029");
    advance_to_mana(&mut game, PlayerId::TWO, 9);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(majordomo));
    play(&mut game, PlayerId::TWO, "CS2_029", Some(majordomo));
    assert_eq!(
        game.state()
            .entity(game.state().player(PlayerId::ONE).hero)
            .unwrap()
            .card_id,
        "BRM_027h"
    );

    wait_for_hand(&mut game, PlayerId::ONE, "BRM_026");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    assert_eq!(game.state().entity(nefarian).unwrap().cost, 3);
    play(&mut game, PlayerId::ONE, "BRM_030", None);
    let hungry_dragon = hand_card(&game, PlayerId::ONE, "BRM_026");
    assert_eq!(game.state().entity(hungry_dragon).unwrap().cost, 4);

    let replay = game.replay();
    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(Path::new(&runtime_dir.0)).unwrap(),
        &replay,
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}

#[test]
fn fireguard_destroyer_random_continuation_is_replay_exact() {
    let mut game = game(repeated("BRM_012"), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    let fireguard = play(&mut game, PlayerId::ONE, "BRM_012", None);
    assert!((4..=7).contains(&game.state().entity(fireguard).unwrap().attack));

    let replay = game.replay();
    let replayed =
        Game::from_replay(LuaCardRuntime::load_dir(data_path()).unwrap(), &replay).unwrap();
    assert_eq!(replayed.state(), game.state());
}
