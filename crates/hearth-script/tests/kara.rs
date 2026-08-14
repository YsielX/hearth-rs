use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use hearth_core::{DEFAULT_HERO_POWER, EntityId, Game, GameEvent, PlayerCommand, PlayerId, Zone};
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
    seed: u64,
    classes: [&str; 2],
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_with_hero_powers_and_classes(
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

fn deck_card(game: &Game<LuaCardRuntime>, player: PlayerId, card_id: &str) -> EntityId {
    game.state()
        .player(player)
        .deck
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == card_id)
        .unwrap_or_else(|| panic!("{player} has no {card_id} in deck"))
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

static TEMP_RUNTIME_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRuntimeDir(PathBuf);

impl Drop for TempRuntimeDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture_runtime() -> (TempRuntimeDir, LuaCardRuntime) {
    let suffix = TEMP_RUNTIME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("hearth-rs-kara-{}-{suffix}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("sets"), root.join("sets")).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("hero_powers"), root.join("hero_powers")).unwrap();
    std::fs::write(
        root.join("test_kara_effects.lua"),
        r#"
local function clear_hand(ctx, self)
    local player = ctx:controller(self)
    for _, entity in ipairs(ctx:hand(player)) do ctx:discard(player, entity) end
end

return {
    api_version = 1, id = "TEST_KARA_EFFECTS", name = "KARA Test Effects", text = "",
    set = "TEST", type = "spell", cost = 0, collectible = false,
    tokens = {
        { id = "TEST_KARA_DISCARD", name = "Discard", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true, target_mode = "required",
          targets = function(ctx, self) return ctx:hand(ctx:controller(self)) end,
          on_play = function(ctx, self, target) ctx:discard(ctx:controller(self), target) end },
        { id = "TEST_KARA_KILL", name = "Kill", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true, target_mode = "required",
          targets = function(ctx) return ctx:minions() end,
          on_play = function(ctx, self, target) ctx:destroy(target) end },
        { id = "TEST_KARA_NOOP", name = "No-op", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true },
        { id = "TEST_KARA_DISCOUNT", name = "Discount", text = "", set = "TEST",
          type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                  if ctx:entity(entity).card_id == "KAR_013" then
                      ctx:modify(entity, { stat = "cost", operation = "add", value = -1 })
                  end
              end
          end },
        { id = "TEST_KARA_BARNES_SETUP", name = "Barnes Setup", text = "", set = "TEST",
          type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              ctx:give_card(player, "KAR_114")
              ctx:shuffle_card_into_deck(player, "OG_221")
          end },
        { id = "TEST_KARA_CURATOR_SETUP", name = "Curator Setup", text = "", set = "TEST",
          type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              ctx:give_card(player, "KAR_061")
              ctx:shuffle_card_into_deck(player, "KAR_005")
              ctx:shuffle_card_into_deck(player, "OG_271")
              ctx:shuffle_card_into_deck(player, "OG_156")
          end },
        { id = "TEST_KARA_ATIESH_SETUP", name = "Atiesh Setup", text = "", set = "TEST",
          type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              ctx:give_card(player, "KAR_097")
              ctx:give_card(player, "TEST_KARA_DISCOUNT")
              ctx:give_card(player, "KAR_013")
              ctx:give_card(player, "KAR_013")
              ctx:give_card(player, "KAR_013")
              ctx:give_card(player, "OG_086")
          end },
        { id = "TEST_KARA_MOAT_SETUP", name = "Moat Setup", text = "", set = "TEST",
          type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              ctx:give_card(player, "OG_221")
              ctx:give_card(player, "OG_223")
              ctx:give_card(player, "KAR_041")
              ctx:give_card(player, "TEST_KARA_KILL")
          end },
    },
}
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(Path::new(&root)).unwrap();
    (TempRuntimeDir(root), runtime)
}

#[test]
fn kara_catalog_is_the_exact_45_card_collectible_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|card| card.set == "KARA" && card.collectible)
        .map(|card| card.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = [
        "KAR_004", "KAR_005", "KAR_006", "KAR_009", "KAR_010", "KAR_011", "KAR_013", "KAR_021",
        "KAR_025", "KAR_026", "KAR_028", "KAR_029", "KAR_030a", "KAR_033", "KAR_035", "KAR_036",
        "KAR_037", "KAR_041", "KAR_044", "KAR_057", "KAR_061", "KAR_062", "KAR_063", "KAR_065",
        "KAR_069", "KAR_070", "KAR_073", "KAR_075", "KAR_076", "KAR_077", "KAR_089", "KAR_091",
        "KAR_092", "KAR_094", "KAR_095", "KAR_096", "KAR_097", "KAR_114", "KAR_204", "KAR_205",
        "KAR_300", "KAR_702", "KAR_710", "KAR_711", "KAR_712",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual.len(), 45);
    assert_eq!(actual, expected);
}

#[test]
fn silverware_golem_discard_summons_the_same_entity_from_graveyard() {
    let (_dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        mixed(&["TEST_KARA_DISCARD", "KAR_205"]),
        repeated("CS2_120"),
        83,
        ["warlock", "neutral"],
    );
    for card_id in ["TEST_KARA_DISCARD", "KAR_205"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    let golem = hand_card(&game, PlayerId::ONE, "KAR_205");
    let log_start = game.state().log.len();
    play(&mut game, PlayerId::ONE, "TEST_KARA_DISCARD", Some(golem));

    assert_eq!(game.state().entity(golem).unwrap().zone, Zone::Board);
    assert!(game.state().player(PlayerId::ONE).board.contains(&golem));
    assert!(
        !game
            .state()
            .player(PlayerId::ONE)
            .graveyard
            .contains(&golem)
    );
    assert!(game.state().log[log_start..].iter().any(|event| matches!(
        event,
        GameEvent::CardDiscarded { card, .. } if *card == golem
    )));
    assert!(game.state().log[log_start..].iter().any(|event| matches!(
        event,
        GameEvent::MinionSummoned { entity, .. } if *entity == golem
    )));
}

#[test]
fn barnes_uses_a_deck_template_but_summons_a_fresh_one_one_copy() {
    let (_dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_KARA_BARNES_SETUP"),
        repeated("CS2_120"),
        89,
        ["neutral", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "TEST_KARA_BARNES_SETUP", None);
    let template = deck_card(&game, PlayerId::ONE, "OG_221");
    play(&mut game, PlayerId::ONE, "KAR_114", None);

    let copy = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "OG_221")
        .unwrap();
    assert_ne!(copy, template);
    assert_eq!(game.state().entity(template).unwrap().zone, Zone::Deck);
    assert_eq!(game.state().entity(copy).unwrap().attack, 1);
    assert_eq!(game.state().entity(copy).unwrap().max_health, 1);
    assert!(
        game.state()
            .entity(copy)
            .unwrap()
            .has_keyword("deathrattle")
    );
}

#[test]
fn curator_draws_the_original_beast_dragon_and_murloc_entities_in_order() {
    let (_dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_KARA_CURATOR_SETUP"),
        repeated("CS2_120"),
        97,
        ["neutral", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    play(&mut game, PlayerId::ONE, "TEST_KARA_CURATOR_SETUP", None);
    let expected = [
        deck_card(&game, PlayerId::ONE, "KAR_005"),
        deck_card(&game, PlayerId::ONE, "OG_271"),
        deck_card(&game, PlayerId::ONE, "OG_156"),
    ];
    let log_start = game.state().log.len();
    play(&mut game, PlayerId::ONE, "KAR_061", None);

    assert!(
        expected
            .iter()
            .all(|entity| game.state().player(PlayerId::ONE).hand.contains(entity))
    );
    let drawn = game.state().log[log_start..]
        .iter()
        .filter_map(|event| match event {
            GameEvent::CardDrawn {
                player: PlayerId::ONE,
                card,
                ..
            } => Some(*card),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(drawn, expected);
}

#[test]
fn atiesh_summons_a_same_cost_minion_per_spell_and_publishes_destruction_at_zero() {
    let (_dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_KARA_ATIESH_SETUP"),
        repeated("CS2_120"),
        101,
        ["priest", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    play(&mut game, PlayerId::ONE, "TEST_KARA_ATIESH_SETUP", None);
    play(&mut game, PlayerId::ONE, "TEST_KARA_DISCOUNT", None);
    let medivh = play(&mut game, PlayerId::ONE, "KAR_097", None);
    let atiesh = game.state().player(PlayerId::ONE).weapon.unwrap();
    assert_eq!(game.state().entity(atiesh).unwrap().card_id, "KAR_097t");

    advance_to_mana(&mut game, PlayerId::ONE, 9);
    let log_start = game.state().log.len();
    for _ in 0..3 {
        play(&mut game, PlayerId::ONE, "KAR_013", Some(medivh));
    }

    let summoned = game.state().log[log_start..]
        .iter()
        .filter_map(|event| match event {
            GameEvent::MinionSummoned {
                player: PlayerId::ONE,
                entity,
            } if *entity != medivh => Some(*entity),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(summoned.len(), 3);
    let summoned_cards = summoned
        .iter()
        .map(|entity| {
            let entity = game.state().entity(*entity).unwrap();
            (entity.card_id.clone(), entity.base_cost, entity.cost)
        })
        .collect::<Vec<_>>();
    assert!(
        summoned
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().base_cost == 1),
        "Atiesh summons were {summoned_cards:?}"
    );
    assert!(game.state().player(PlayerId::ONE).weapon.is_none());
    assert_eq!(game.state().entity(atiesh).unwrap().zone, Zone::Graveyard);
    assert!(game.state().log[log_start..].iter().any(|event| matches!(
        event,
        GameEvent::WeaponDestroyed { weapon, .. } if *weapon == atiesh
    )));
}

#[test]
fn atiesh_uses_the_played_cost_not_mana_spent_by_the_spell_effect() {
    let (_dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_KARA_ATIESH_SETUP"),
        repeated("CS2_120"),
        102,
        ["mage", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    play(&mut game, PlayerId::ONE, "TEST_KARA_ATIESH_SETUP", None);
    let medivh = play(&mut game, PlayerId::ONE, "KAR_097", None);
    let atiesh = game.state().player(PlayerId::ONE).weapon.unwrap();

    advance_to_mana(&mut game, PlayerId::ONE, 9);
    let log_start = game.state().log.len();
    play(&mut game, PlayerId::ONE, "OG_086", Some(medivh));

    let summoned = game.state().log[log_start..]
        .iter()
        .find_map(|event| match event {
            GameEvent::MinionSummoned {
                player: PlayerId::ONE,
                entity,
            } => Some(*entity),
            _ => None,
        })
        .expect("Atiesh should summon for Forbidden Flame");
    assert_eq!(game.state().entity(summoned).unwrap().base_cost, 0);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 0);
    assert_eq!(game.state().player(PlayerId::ONE).weapon, Some(atiesh));
}

#[test]
fn prince_malchezaar_adds_five_distinct_eligible_legendaries_and_excludes_starting_ones() {
    let mut one = repeated("CS2_120");
    one[0] = "KAR_096".to_owned();
    one[1] = "OG_133".to_owned();
    let game = game_with_runtime(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        one,
        repeated("CS2_120"),
        103,
        ["mage", "neutral"],
    );
    // Construction and mulligan fully resolve all Start-of-Game continuations.
    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.deck.len() + player.hand.len(), 25);

    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let mut legendary_ids = player
        .deck
        .iter()
        .chain(player.hand.iter())
        .filter_map(|entity| {
            let card_id = &game.state().entity(*entity).unwrap().card_id;
            runtime
                .definitions()
                .find(|definition| definition.id == *card_id)
                .filter(|definition| definition.rarity.as_deref() == Some("legendary"))
                .map(|_| card_id.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        legendary_ids.iter().filter(|id| *id == "KAR_096").count(),
        1
    );
    assert_eq!(legendary_ids.iter().filter(|id| *id == "OG_133").count(), 1);
    legendary_ids.retain(|id| id != "KAR_096" && id != "OG_133");
    assert_eq!(legendary_ids.len(), 5);
    assert_eq!(legendary_ids.iter().collect::<BTreeSet<_>>().len(), 5);
    for card_id in legendary_ids {
        let definition = runtime
            .definitions()
            .find(|definition| definition.id == card_id)
            .unwrap();
        assert!(definition.class == "neutral" || definition.class == "mage");
    }
}

#[test]
fn moat_lurker_resummons_a_fresh_unbuffed_minion_for_its_recorded_controller() {
    let (_dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_KARA_MOAT_SETUP"),
        repeated("CS2_120"),
        107,
        ["neutral", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    play(&mut game, PlayerId::ONE, "TEST_KARA_MOAT_SETUP", None);
    let original = play(&mut game, PlayerId::ONE, "OG_221", None);
    play(&mut game, PlayerId::ONE, "OG_223", Some(original));
    assert_eq!(game.state().entity(original).unwrap().attack, 3);
    assert_eq!(game.state().entity(original).unwrap().max_health, 3);
    let lurker = play(&mut game, PlayerId::ONE, "KAR_041", Some(original));
    assert_eq!(game.state().entity(original).unwrap().zone, Zone::Graveyard);
    play(&mut game, PlayerId::ONE, "TEST_KARA_KILL", Some(lurker));

    let resummoned = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "OG_221")
        .unwrap();
    assert_ne!(resummoned, original);
    assert_eq!(game.state().entity(original).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.state().entity(resummoned).unwrap().attack, 2);
    assert_eq!(game.state().entity(resummoned).unwrap().max_health, 1);
    assert!(
        game.state()
            .entity(resummoned)
            .unwrap()
            .enchantments
            .is_empty()
    );
}

#[test]
fn cat_trick_reveals_after_the_opponents_spell_and_summons_its_stealthed_token() {
    let (_dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        mixed(&["KAR_004", "CS2_120"]),
        repeated("TEST_KARA_NOOP"),
        109,
        ["hunter", "neutral"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    wait_for_hand(&mut game, PlayerId::ONE, "KAR_004");
    let secret = play(&mut game, PlayerId::ONE, "KAR_004", None);
    assert!(game.state().player(PlayerId::ONE).secrets.contains(&secret));
    end_turn(&mut game);
    let log_start = game.state().log.len();
    let spell = play(&mut game, PlayerId::TWO, "TEST_KARA_NOOP", None);

    let panther = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "KAR_004a")
        .unwrap();
    assert_eq!(game.state().entity(secret).unwrap().zone, Zone::Graveyard);
    assert!(game.state().entity(panther).unwrap().has_keyword("stealth"));
    let spell_cast = game.state().log[log_start..]
        .iter()
        .position(
            |event| matches!(event, GameEvent::SpellCast { spell: cast, .. } if *cast == spell),
        )
        .unwrap();
    let summoned = game.state().log[log_start..]
        .iter()
        .position(
            |event| matches!(event, GameEvent::MinionSummoned { entity, .. } if *entity == panther),
        )
        .unwrap();
    assert!(spell_cast < summoned);
}
