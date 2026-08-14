use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use hearth_core::{
    CardKind, CardRuntime, DEFAULT_HERO_POWER, EntityId, Game, PlayerCommand, PlayerId, Zone,
};
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
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_with_hero_powers_and_classes(
        runtime,
        one,
        two,
        41,
        [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()],
        ["neutral".to_owned(), "neutral".to_owned()],
    )
    .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    game
}

fn game(one: Vec<String>, two: Vec<String>) -> Game<LuaCardRuntime> {
    game_with_runtime(LuaCardRuntime::load_dir(data_path()).unwrap(), one, two)
}

static TEMP_RUNTIME_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRuntimeDir(PathBuf);

impl Drop for TempRuntimeDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn golden_monkey_runtime() -> (TempRuntimeDir, LuaCardRuntime) {
    let suffix = TEMP_RUNTIME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("hearth-rs-loe-{}-{suffix}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("sets"), root.join("sets")).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("hero_powers"), root.join("hero_powers")).unwrap();
    std::fs::write(
        root.join("give_golden_monkey.lua"),
        r#"
return {
    api_version = 1,
    id = "TEST_LOE_GIVE_GOLDEN_MONKEY",
    name = "Give Golden Monkey",
    text = "Add the Golden Monkey to your hand.",
    set = "TEST",
    type = "spell",
    cost = 0,
    on_play = function(ctx, self)
        ctx:give_card(ctx:controller(self), "LOE_019t2")
    end,
}
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(Path::new(&root)).unwrap();
    (TempRuntimeDir(root), runtime)
}

fn end_turn(game: &mut Game<LuaCardRuntime>) {
    game.dispatch(PlayerCommand::EndTurn).unwrap();
}

fn advance_to_mana(game: &mut Game<LuaCardRuntime>, player: PlayerId, mana: u8) {
    while game.state().active_player != player || game.state().player(player).max_mana < mana {
        end_turn(game);
    }
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

fn hand_count(game: &Game<LuaCardRuntime>, player: PlayerId, card_id: &str) -> usize {
    game.state()
        .player(player)
        .hand
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == card_id)
        .count()
}

#[test]
fn loe_catalog_is_the_exact_45_card_collectible_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|card| card.set == "LOE" && card.collectible)
        .map(|card| card.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = [
        "LOEA10_3", "LOE_002", "LOE_003", "LOE_006", "LOE_007", "LOE_009", "LOE_010", "LOE_011",
        "LOE_012", "LOE_016", "LOE_017", "LOE_018", "LOE_019", "LOE_020", "LOE_021", "LOE_022",
        "LOE_023", "LOE_026", "LOE_027", "LOE_029", "LOE_038", "LOE_039", "LOE_046", "LOE_047",
        "LOE_050", "LOE_051", "LOE_053", "LOE_061", "LOE_073", "LOE_076", "LOE_077", "LOE_079",
        "LOE_086", "LOE_089", "LOE_092", "LOE_104", "LOE_105", "LOE_107", "LOE_110", "LOE_111",
        "LOE_113", "LOE_115", "LOE_116", "LOE_118", "LOE_119",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual.len(), 45);
    assert_eq!(actual, expected);
}

#[test]
fn brann_repeats_elises_battlecry_exactly_twice() {
    let mut game = game(mixed(&["LOE_077", "LOE_079"]), repeated("CS2_120"));
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    for card_id in ["LOE_077", "LOE_079"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "LOE_077", None);
    play(&mut game, PlayerId::ONE, "LOE_079", None);

    let maps = game
        .state()
        .player(PlayerId::ONE)
        .deck
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "LOE_019t")
        .count();
    assert_eq!(maps, 2);
}

#[test]
fn explorers_hat_stacks_deathrattles_and_silence_allows_only_a_new_attachment() {
    let mut game = game(
        mixed(&["CS2_120", "LOE_105", "EX1_332"]),
        repeated("CS2_029"),
    );
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    for card_id in ["CS2_120", "LOE_105", "EX1_332"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 10);

    let target = play(&mut game, PlayerId::ONE, "CS2_120", None);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(target));
    wait_for_hand(&mut game, PlayerId::ONE, "LOE_105");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(target));
    assert_eq!(
        game.state()
            .entity(target)
            .unwrap()
            .scripts_for_hook("on_deathrattle"),
        ["LOE_105", "LOE_105"]
    );

    play(&mut game, PlayerId::ONE, "EX1_332", Some(target));
    assert!(
        game.state()
            .entity(target)
            .unwrap()
            .scripts_for_hook("on_deathrattle")
            .is_empty()
    );
    wait_for_hand(&mut game, PlayerId::ONE, "LOE_105");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(target));
    assert_eq!(
        game.state()
            .entity(target)
            .unwrap()
            .scripts_for_hook("on_deathrattle"),
        ["LOE_105"]
    );

    let hats_before_death = hand_count(&game, PlayerId::ONE, "LOE_105");
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(target));
    assert_eq!(game.state().entity(target).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        hand_count(&game, PlayerId::ONE, "LOE_105"),
        hats_before_death + 1
    );
}

#[test]
fn unearthed_raptor_copies_native_and_hook_attached_deathrattles() {
    let mut game = game(
        mixed(&["EX1_096", "LOE_105", "LOE_019", "CS2_120"]),
        repeated("CS2_029"),
    );
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    for card_id in ["EX1_096", "LOE_105", "LOE_019"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 10);

    let hoarder = play(&mut game, PlayerId::ONE, "EX1_096", None);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(hoarder));
    let raptor = play(&mut game, PlayerId::ONE, "LOE_019", Some(hoarder));
    assert_eq!(
        game.state()
            .entity(raptor)
            .unwrap()
            .scripts_for_hook("on_deathrattle"),
        ["EX1_096", "LOE_105"]
    );

    let deck_before = game.state().player(PlayerId::ONE).deck.len();
    let hats_before = hand_count(&game, PlayerId::ONE, "LOE_105");
    advance_to_mana(&mut game, PlayerId::TWO, 4);
    play(&mut game, PlayerId::TWO, "CS2_029", Some(raptor));
    assert_eq!(game.state().entity(raptor).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.state().player(PlayerId::ONE).deck.len(),
        deck_before - 1
    );
    assert_eq!(hand_count(&game, PlayerId::ONE, "LOE_105"), hats_before + 1);
}

#[test]
fn djinni_copies_only_spells_targeting_another_friendly_minion() {
    let mut game = game(
        mixed(&["LOE_053", "CS2_120", "LOE_105"]),
        repeated("CS2_120"),
    );
    advance_to_mana(&mut game, PlayerId::TWO, 2);
    let enemy = play(&mut game, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    for card_id in ["LOE_053", "CS2_120", "LOE_105"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 10);

    let djinni = play(&mut game, PlayerId::ONE, "LOE_053", None);
    let friend = play(&mut game, PlayerId::ONE, "CS2_120", None);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(friend));
    assert_eq!(
        (
            game.state().entity(friend).unwrap().attack,
            game.state().entity(djinni).unwrap().attack
        ),
        (3, 5)
    );

    wait_for_hand(&mut game, PlayerId::ONE, "LOE_105");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(djinni));
    assert_eq!(game.state().entity(djinni).unwrap().attack, 6);

    wait_for_hand(&mut game, PlayerId::ONE, "LOE_105");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(enemy));
    assert_eq!(game.state().entity(enemy).unwrap().attack, 3);
    assert_eq!(game.state().entity(djinni).unwrap().attack, 6);
}

#[test]
fn entomb_and_excavated_evil_shuffle_the_original_entity_across_players() {
    let mut entomb = game(repeated("LOE_104"), repeated("CS2_120"));
    advance_to_mana(&mut entomb, PlayerId::TWO, 2);
    let minion = play(&mut entomb, PlayerId::TWO, "CS2_120", None);
    advance_to_mana(&mut entomb, PlayerId::ONE, 6);
    play(&mut entomb, PlayerId::ONE, "LOE_104", Some(minion));
    assert!(entomb.state().player(PlayerId::ONE).deck.contains(&minion));
    let moved = entomb.state().entity(minion).unwrap();
    assert_eq!(moved.zone, Zone::Deck);
    assert_eq!(
        (moved.owner, moved.controller),
        (PlayerId::ONE, PlayerId::ONE)
    );

    let mut evil = game(repeated("LOE_111"), repeated("CS2_120"));
    advance_to_mana(&mut evil, PlayerId::ONE, 5);
    let spell = hand_card(&evil, PlayerId::ONE, "LOE_111");
    evil.dispatch(PlayerCommand::PlayCard {
        card: spell,
        target: None,
    })
    .unwrap();
    assert!(evil.state().player(PlayerId::TWO).deck.contains(&spell));
    let moved = evil.state().entity(spell).unwrap();
    assert_eq!(moved.zone, Zone::Deck);
    assert_eq!(
        (moved.owner, moved.controller),
        (PlayerId::TWO, PlayerId::TWO)
    );
}

#[test]
fn golden_monkey_replaces_hand_and_deck_in_place_with_legendary_minions() {
    let (runtime_dir, runtime) = golden_monkey_runtime();
    let mut game = game_with_runtime(
        runtime,
        mixed(&[
            "TEST_LOE_GIVE_GOLDEN_MONKEY",
            "CS2_029",
            "CS2_106",
            "CS2_120",
        ]),
        repeated("CS2_120"),
    );
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    wait_for_hand(&mut game, PlayerId::ONE, "TEST_LOE_GIVE_GOLDEN_MONKEY");
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(
        &mut game,
        PlayerId::ONE,
        "TEST_LOE_GIVE_GOLDEN_MONKEY",
        None,
    );
    wait_for_hand(&mut game, PlayerId::ONE, "LOE_019t2");
    advance_to_mana(&mut game, PlayerId::ONE, 4);

    let monkey = hand_card(&game, PlayerId::ONE, "LOE_019t2");
    let hand_before = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .filter(|entity| *entity != monkey)
        .collect::<Vec<_>>();
    let deck_before = game
        .state()
        .player(PlayerId::ONE)
        .deck
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let original_kinds = hand_before
        .iter()
        .chain(&deck_before)
        .map(|entity| game.state().entity(*entity).unwrap().kind)
        .collect::<Vec<_>>();
    assert!(original_kinds.contains(&CardKind::Spell));
    assert!(original_kinds.contains(&CardKind::Weapon));

    game.dispatch(PlayerCommand::PlayCard {
        card: monkey,
        target: None,
    })
    .unwrap();

    assert_eq!(game.state().player(PlayerId::ONE).hand, hand_before);
    assert_eq!(
        game.state()
            .player(PlayerId::ONE)
            .deck
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        deck_before
    );
    for entity in hand_before.iter().chain(&deck_before) {
        let transformed = game.state().entity(*entity).unwrap();
        let definition = game.runtime().definition(&transformed.card_id).unwrap();
        assert_eq!(transformed.kind, CardKind::Minion);
        assert_eq!(definition.rarity.as_deref(), Some("legendary"));
    }
    drop(runtime_dir);
}

#[test]
fn jungle_moonkin_grants_spell_damage_two_to_both_players() {
    let mut game = game(mixed(&["LOE_051", "CS2_029"]), repeated("CS2_029"));
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    for card_id in ["LOE_051", "CS2_029"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    play(&mut game, PlayerId::ONE, "LOE_051", None);
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    play(&mut game, PlayerId::ONE, "CS2_029", Some(enemy_hero));
    assert_eq!(game.state().entity(enemy_hero).unwrap().damage, 8);

    advance_to_mana(&mut game, PlayerId::TWO, 4);
    let friendly_hero = game.state().player(PlayerId::ONE).hero;
    play(&mut game, PlayerId::TWO, "CS2_029", Some(friendly_hero));
    assert_eq!(game.state().entity(friendly_hero).unwrap().damage, 8);
}
