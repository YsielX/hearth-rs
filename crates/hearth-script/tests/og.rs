use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use hearth_core::{
    CardRuntime, DEFAULT_HERO_POWER, EntityId, Game, GameEvent, PlayerCommand, PlayerId,
    PublicEvent, Zone,
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
    seed: u64,
    powers: [&str; 2],
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
        runtime,
        one,
        two,
        seed,
        powers.map(str::to_owned),
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
    game_with_runtime(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        one,
        two,
        43,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
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

static TEMP_RUNTIME_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRuntimeDir(PathBuf);

impl Drop for TempRuntimeDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture_runtime(files: &[(&str, &str)]) -> (TempRuntimeDir, LuaCardRuntime) {
    let suffix = TEMP_RUNTIME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("hearth-rs-og-{}-{suffix}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("sets"), root.join("sets")).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("hero_powers"), root.join("hero_powers")).unwrap();
    for (name, source) in files {
        std::fs::write(root.join(name), source).unwrap();
    }
    let runtime = LuaCardRuntime::load_dir(Path::new(&root)).unwrap();
    (TempRuntimeDir(root), runtime)
}

const TEST_EFFECTS: &str = r#"
return {
    api_version = 1, id = "TEST_OG_EFFECTS", name = "OG Test Effects", text = "", set = "TEST",
    type = "spell", cost = 0, collectible = false,
    tokens = {
        { id = "TEST_OG_CTHUN_SETUP", name = "C'Thun Setup", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              for _, card_id in ipairs({ "OG_280", "OG_334", "OG_281", "OG_096",
                  "TEST_OG_HURT_FRIENDS", "TEST_OG_HEAL_FRIENDS", "TEST_OG_HURT_HERO" }) do
                  ctx:give_card(player, card_id)
              end
          end },
        { id = "TEST_OG_DISCARD", name = "Discard", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          target_mode = "required", targets = function(ctx, self) return ctx:hand(ctx:controller(self)) end,
          on_play = function(ctx, self, target) ctx:discard(ctx:controller(self), target) end },
        { id = "TEST_OG_KILL", name = "Kill", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          target_mode = "required", targets = function(ctx) return ctx:minions() end,
          on_play = function(ctx, self, target) ctx:destroy(target) end },
        { id = "TEST_OG_SILENCE", name = "Silence", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          target_mode = "required", targets = function(ctx) return ctx:minions() end,
          on_play = function(ctx, self, target) ctx:silence(target) end },
        { id = "TEST_OG_CHARGER", name = "Charger", text = "", set = "TEST", type = "minion", cost = 0,
          attack = 1, health = 1, collectible = true, keywords = { "charge" } },
        { id = "TEST_OG_LOCATION", name = "Location", text = "", set = "TEST", type = "location", cost = 0,
          health = 3, collectible = true, on_location_use = function() end },
        { id = "TEST_OG_HURT_FRIENDS", name = "Hurt Friends", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local targets = { ctx:player(ctx:controller(self)).hero }
              for _, minion in ipairs(ctx:friendly_minions(self)) do targets[#targets + 1] = minion end
              ctx:damage_all(targets, 1)
          end },
        { id = "TEST_OG_HEAL_FRIENDS", name = "Heal Friends", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local targets = { ctx:player(ctx:controller(self)).hero }
              for _, minion in ipairs(ctx:friendly_minions(self)) do targets[#targets + 1] = minion end
              ctx:heal_all(targets, 10)
          end },
        { id = "TEST_OG_HURT_HERO", name = "Hurt Hero", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              ctx:damage(ctx:player(player).hero, 10)
              ctx:give_card(player, "TEST_OG_SILENCE")
          end },
        { id = "TEST_OG_HURT_HEROES", name = "Hurt Heroes", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self) ctx:damage_all({ ctx:player(0).hero, ctx:player(1).hero }, 3) end },
        { id = "TEST_OG_HEAL_HEROES", name = "Heal Heroes", text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
          on_play = function(ctx, self) ctx:heal_all({ ctx:player(0).hero, ctx:player(1).hero }, 4) end },
    },
}
"#;

#[test]
fn og_catalog_is_the_exact_134_card_collectible_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|card| card.set == "OG" && card.collectible)
        .map(|card| card.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = [
        "OG_006", "OG_023", "OG_024", "OG_026", "OG_027", "OG_028", "OG_031", "OG_033", "OG_034",
        "OG_042", "OG_044", "OG_045", "OG_047", "OG_048", "OG_051", "OG_061", "OG_070", "OG_072",
        "OG_073", "OG_080", "OG_081", "OG_082", "OG_083", "OG_085", "OG_086", "OG_087", "OG_090",
        "OG_094", "OG_096", "OG_100", "OG_101", "OG_102", "OG_104", "OG_109", "OG_113", "OG_114",
        "OG_116", "OG_118", "OG_120", "OG_121", "OG_122", "OG_123", "OG_131", "OG_133", "OG_134",
        "OG_138", "OG_141", "OG_142", "OG_145", "OG_147", "OG_149", "OG_150", "OG_151", "OG_152",
        "OG_153", "OG_156", "OG_158", "OG_161", "OG_162", "OG_173", "OG_174", "OG_176", "OG_179",
        "OG_188", "OG_195", "OG_198", "OG_200", "OG_202", "OG_206", "OG_207", "OG_209", "OG_211",
        "OG_216", "OG_218", "OG_220", "OG_221", "OG_222", "OG_223", "OG_229", "OG_234", "OG_239",
        "OG_241", "OG_247", "OG_248", "OG_249", "OG_254", "OG_255", "OG_256", "OG_267", "OG_271",
        "OG_272", "OG_273", "OG_276", "OG_280", "OG_281", "OG_282", "OG_283", "OG_284", "OG_286",
        "OG_290", "OG_291", "OG_292", "OG_293", "OG_295", "OG_300", "OG_301", "OG_302", "OG_303",
        "OG_308", "OG_309", "OG_310", "OG_311", "OG_312", "OG_313", "OG_314", "OG_315", "OG_316",
        "OG_317", "OG_318", "OG_320", "OG_321", "OG_322", "OG_323", "OG_325", "OG_326", "OG_327",
        "OG_328", "OG_330", "OG_334", "OG_335", "OG_337", "OG_338", "OG_339", "OG_340",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual.len(), 134);
    assert_eq!(actual, expected);
}

#[test]
fn cthun_buffs_are_atomic_for_one_heal_group_apply_across_zones_and_unlock_ten_attack() {
    let (_dir, runtime) = fixture_runtime(&[("test_og_effects.lua", TEST_EFFECTS)]);
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_OG_CTHUN_SETUP"),
        repeated("CS2_120"),
        47,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    play(&mut game, PlayerId::ONE, "TEST_OG_CTHUN_SETUP", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "OG_334", None);
    play(&mut game, PlayerId::ONE, "TEST_OG_HURT_FRIENDS", None);
    play(&mut game, PlayerId::ONE, "TEST_OG_HEAL_FRIENDS", None);

    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["cthun_attack_buff"],
        2
    );
    assert_eq!(
        game.state().player(PlayerId::ONE).script_data["cthun_health_buff"],
        2
    );

    advance_to_mana(&mut game, PlayerId::ONE, 7);
    play(&mut game, PlayerId::ONE, "TEST_OG_HURT_HERO", None);
    play(&mut game, PlayerId::ONE, "OG_281", None);
    let hero = game.state().player(PlayerId::ONE).hero;
    assert_eq!(game.state().entity(hero).unwrap().health(), 20);
    let cthun = hand_card(&game, PlayerId::ONE, "OG_280");
    assert_eq!(game.state().entity(cthun).unwrap().attack, 10);
    assert_eq!(game.state().entity(cthun).unwrap().max_health, 10);
    end_turn(&mut game);
    end_turn(&mut game);
    play(&mut game, PlayerId::ONE, "OG_096", None);
    assert_eq!(game.state().entity(hero).unwrap().health(), 30);

    // Darkmender's heal is itself another healed event for Hooded Acolyte.
    assert_eq!(game.state().entity(cthun).unwrap().attack, 11);
    assert_eq!(game.state().entity(cthun).unwrap().max_health, 11);
    end_turn(&mut game);
    end_turn(&mut game);
    play(&mut game, PlayerId::ONE, "OG_280", None);
    assert_eq!(game.state().entity(cthun).unwrap().zone, Zone::Board);
    assert_eq!(game.state().entity(cthun).unwrap().attack, 11);
    assert_eq!(game.state().entity(cthun).unwrap().max_health, 11);
    play(&mut game, PlayerId::ONE, "TEST_OG_SILENCE", Some(cthun));
    assert_eq!(game.state().entity(cthun).unwrap().attack, 11);
    assert_eq!(game.state().entity(cthun).unwrap().max_health, 11);
}

#[test]
fn cthun_fixed_copy_ignores_old_rituals_but_receives_future_ones() {
    let (_dir, runtime) = fixture_runtime(&[(
        "test_og_cthun_copy_setup.lua",
        r#"
return {
    api_version = 1, id = "TEST_OG_CTHUN_COPY_SETUP", name = "C'Thun Copy Setup",
    text = "", set = "TEST", type = "spell", cost = 0, collectible = true,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        for _, card_id in ipairs({ "OG_280", "OG_281", "OG_281", "OG_281", "OG_291" }) do
            ctx:give_card(player, card_id)
        end
    end,
}
"#,
    )]);
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_OG_CTHUN_COPY_SETUP"),
        repeated("CS2_120"),
        79,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    play(&mut game, PlayerId::ONE, "TEST_OG_CTHUN_COPY_SETUP", None);
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "OG_281", None);
    play(&mut game, PlayerId::ONE, "OG_281", None);
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    let cthun = play(&mut game, PlayerId::ONE, "OG_280", None);
    assert_eq!(game.state().entity(cthun).unwrap().attack, 10);
    end_turn(&mut game);
    end_turn(&mut game);
    let shadowcaster = play(&mut game, PlayerId::ONE, "OG_291", Some(cthun));
    let copy = game
        .state()
        .log
        .iter()
        .rev()
        .find_map(|event| match event {
            GameEvent::CardCreated { source, card, .. } if *source == shadowcaster => Some(*card),
            _ => None,
        })
        .unwrap();
    assert_eq!(game.state().entity(copy).unwrap().attack, 1);
    assert_eq!(game.state().entity(copy).unwrap().max_health, 1);
    play(&mut game, PlayerId::ONE, "OG_281", None);
    assert_eq!(game.state().entity(copy).unwrap().attack, 3);
    assert_eq!(game.state().entity(copy).unwrap().max_health, 3);
    assert_eq!(game.state().entity(cthun).unwrap().attack, 12);
}

#[test]
fn nzoth_uses_native_deathrattle_even_when_the_minion_died_silenced() {
    let (_dir, runtime) = fixture_runtime(&[("test_og_effects.lua", TEST_EFFECTS)]);
    let mut game = game_with_runtime(
        runtime,
        mixed(&["OG_221", "TEST_OG_KILL", "TEST_OG_SILENCE", "OG_133"]),
        repeated("CS2_120"),
        53,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    for card_id in ["OG_221", "TEST_OG_KILL"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let first = play(&mut game, PlayerId::ONE, "OG_221", None);
    play(&mut game, PlayerId::ONE, "TEST_OG_KILL", Some(first));

    for card_id in ["OG_221", "TEST_OG_SILENCE", "TEST_OG_KILL"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    let second = play(&mut game, PlayerId::ONE, "OG_221", None);
    play(&mut game, PlayerId::ONE, "TEST_OG_SILENCE", Some(second));
    play(&mut game, PlayerId::ONE, "TEST_OG_KILL", Some(second));
    wait_for_hand(&mut game, PlayerId::ONE, "OG_133");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "OG_133", None);

    let resurrected = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .filter(|entity| game.state().entity(**entity).unwrap().card_id == "OG_221")
        .count();
    assert_eq!(resurrected, 2);
}

#[test]
fn shifter_zerus_keeps_its_script_and_transforms_on_consecutive_turns() {
    let mut game = game(mixed(&["OG_123", "CS2_120"]), repeated("CS2_120"));
    wait_for_hand(&mut game, PlayerId::ONE, "OG_123");
    let zerus = hand_card(&game, PlayerId::ONE, "OG_123");
    let log_start = game.state().log.len();
    end_turn(&mut game);
    end_turn(&mut game);
    end_turn(&mut game);
    end_turn(&mut game);

    let transforms = game.state().log[log_start..]
        .iter()
        .filter(|event| matches!(event, GameEvent::Transformed { entity, .. } if *entity == zerus))
        .count();
    assert_eq!(transforms, 2);
    assert_eq!(
        game.state()
            .public_history(PlayerId::ONE)
            .iter()
            .filter(|record| {
                matches!(&record.event, PublicEvent::Transformed { entity, .. } if entity.id == zerus)
            })
            .count(),
        2
    );
    assert!(
        game.state()
            .public_history(PlayerId::TWO)
            .iter()
            .all(|record| {
                !matches!(&record.event, PublicEvent::Transformed { entity, .. } if entity.id == zerus)
            })
    );
    assert_eq!(game.state().entity(zerus).unwrap().zone, Zone::Hand);
    assert!(
        game.state()
            .entity(zerus)
            .unwrap()
            .attached_cards
            .iter()
            .any(|card_id| card_id == "OG_123")
    );
}

#[test]
fn chogall_returns_the_original_discarded_entity_and_it_costs_health() {
    let (_dir, runtime) = fixture_runtime(&[("test_og_effects.lua", TEST_EFFECTS)]);
    let mut game = game_with_runtime(
        runtime,
        mixed(&["TEST_OG_DISCARD", "CS2_029", "OG_121"]),
        repeated("CS2_120"),
        59,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    for card_id in ["TEST_OG_DISCARD", "CS2_029", "OG_121"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 1);
    let discarded = hand_card(&game, PlayerId::ONE, "CS2_029");
    play(&mut game, PlayerId::ONE, "TEST_OG_DISCARD", Some(discarded));
    assert_eq!(
        game.state().entity(discarded).unwrap().zone,
        Zone::Graveyard
    );

    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "OG_121", None);
    assert!(game.state().player(PlayerId::ONE).hand.contains(&discarded));
    assert!(
        game.state()
            .entity(discarded)
            .unwrap()
            .has_keyword("costs_health_instead_of_mana")
    );
    let mana = game.state().player(PlayerId::ONE).mana;
    let hero = game.state().player(PlayerId::ONE).hero;
    let health = game.state().entity(hero).unwrap().health();
    assert!(
        mana < game.state().entity(discarded).unwrap().cost,
        "the regression requires Health, not Mana, to make the card affordable"
    );
    assert!(game.legal_actions().unwrap().iter().any(|action| matches!(
        action,
        PlayerCommand::PlayCard { card, target: Some(_) } if *card == discarded
    )));
    game.dispatch(PlayerCommand::PlayCard {
        card: discarded,
        target: Some(game.state().player(PlayerId::TWO).hero),
    })
    .unwrap();
    assert_eq!(game.state().player(PlayerId::ONE).mana, mana);
    assert_eq!(game.state().entity(hero).unwrap().health(), health - 4);
}

#[test]
fn infest_attached_deathrattle_can_resume_its_random_beast_choice() {
    let (_dir, runtime) = fixture_runtime(&[("test_og_effects.lua", TEST_EFFECTS)]);
    let mut game = game_with_runtime(
        runtime,
        mixed(&["OG_045", "CS2_120", "TEST_OG_KILL"]),
        repeated("CS2_120"),
        60,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    for card_id in ["OG_045", "CS2_120", "TEST_OG_KILL"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let minion = play(&mut game, PlayerId::ONE, "CS2_120", None);
    play(&mut game, PlayerId::ONE, "OG_045", None);
    assert_eq!(
        game.state()
            .entity(minion)
            .unwrap()
            .scripts_for_hook("on_deathrattle"),
        &["OG_045".to_owned()]
    );

    let highest_entity_before_deathrattle = game.state().entities.keys().next_back().unwrap().0;
    play(&mut game, PlayerId::ONE, "TEST_OG_KILL", Some(minion));
    let generated = game
        .state()
        .player(PlayerId::ONE)
        .hand
        .iter()
        .copied()
        .find(|entity| entity.0 > highest_entity_before_deathrattle)
        .expect("Infest's attached Deathrattle should add a Beast");
    let generated = game.state().entity(generated).unwrap();
    let definition = game.runtime().definition(&generated.card_id).unwrap();
    assert!(
        definition
            .tags
            .iter()
            .any(|tag| tag == "beast" || tag == "all")
    );
}

#[test]
fn locations_are_never_enumerated_as_attack_defenders() {
    let (_dir, runtime) = fixture_runtime(&[("test_og_effects.lua", TEST_EFFECTS)]);
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_OG_CHARGER"),
        repeated("TEST_OG_LOCATION"),
        61,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    end_turn(&mut game);
    let location = play(&mut game, PlayerId::TWO, "TEST_OG_LOCATION", None);
    end_turn(&mut game);
    let attacker = play(&mut game, PlayerId::ONE, "TEST_OG_CHARGER", None);
    let attacks = game
        .legal_actions()
        .unwrap()
        .into_iter()
        .filter_map(|action| match action {
            PlayerCommand::Attack {
                attacker: candidate,
                defender,
            } if candidate == attacker => Some(defender),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(attacks, vec![game.state().player(PlayerId::TWO).hero]);
    assert!(!attacks.contains(&location));
}

#[test]
fn embrace_the_shadow_converts_every_member_of_a_heal_group_to_damage() {
    let (_dir, runtime) = fixture_runtime(&[("test_og_effects.lua", TEST_EFFECTS)]);
    let mut game = game_with_runtime(
        runtime,
        mixed(&["TEST_OG_HURT_HEROES", "OG_104", "TEST_OG_HEAL_HEROES"]),
        repeated("CS2_120"),
        61,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    for card_id in ["TEST_OG_HURT_HEROES", "OG_104", "TEST_OG_HEAL_HEROES"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 2);
    play(&mut game, PlayerId::ONE, "TEST_OG_HURT_HEROES", None);
    play(&mut game, PlayerId::ONE, "OG_104", None);
    play(&mut game, PlayerId::ONE, "TEST_OG_HEAL_HEROES", None);

    for player in [PlayerId::ONE, PlayerId::TWO] {
        let hero = game.state().player(player).hero;
        assert_eq!(game.state().entity(hero).unwrap().health(), 23);
    }
}

#[test]
fn herald_volazj_copies_full_minion_state_then_sets_the_copy_to_one_one() {
    let mut game = game(mixed(&["OG_221", "OG_223", "OG_316"]), repeated("CS2_120"));
    for card_id in ["OG_221", "OG_223", "OG_316"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    let original = play(&mut game, PlayerId::ONE, "OG_221", None);
    play(&mut game, PlayerId::ONE, "OG_223", Some(original));
    assert_eq!(game.state().entity(original).unwrap().attack, 3);
    assert_eq!(game.state().entity(original).unwrap().max_health, 3);
    play(&mut game, PlayerId::ONE, "OG_316", None);

    let copy = game
        .state()
        .player(PlayerId::ONE)
        .board
        .iter()
        .copied()
        .find(|entity| {
            *entity != original && game.state().entity(*entity).unwrap().card_id == "OG_221"
        })
        .unwrap();
    let original_state = game.state().entity(original).unwrap();
    let copy_state = game.state().entity(copy).unwrap();
    assert_eq!(copy_state.attack, 1);
    assert_eq!(copy_state.max_health, 1);
    assert_eq!(
        copy_state.enchantments.len(),
        original_state.enchantments.len() + 1
    );
    assert_eq!(
        copy_state.enchantments[0].attack,
        original_state.enchantments[0].attack
    );
    assert_eq!(
        copy_state.enchantments[0].health,
        original_state.enchantments[0].health
    );
}

#[test]
fn shadowcaster_creates_an_unenchanted_one_one_one_hand_copy() {
    let mut game = game(mixed(&["OG_221", "LOE_105", "OG_291"]), repeated("CS2_120"));
    for card_id in ["OG_221", "LOE_105", "OG_291"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut game, PlayerId::ONE, 8);
    let original = play(&mut game, PlayerId::ONE, "OG_221", None);
    play(&mut game, PlayerId::ONE, "LOE_105", Some(original));
    assert_eq!(
        game.state()
            .entity(original)
            .unwrap()
            .scripts_for_hook("on_deathrattle"),
        ["LOE_105"]
    );
    let log_start = game.state().log.len();
    let shadowcaster = play(&mut game, PlayerId::ONE, "OG_291", Some(original));
    let copy = game.state().log[log_start..]
        .iter()
        .find_map(|event| match event {
            GameEvent::CardCreated { source, card, .. } if *source == shadowcaster => Some(*card),
            _ => None,
        })
        .unwrap();
    let copy_state = game.state().entity(copy).unwrap();
    assert_eq!(copy_state.card_id, "OG_221");
    assert_eq!(copy_state.zone, Zone::Hand);
    assert_eq!(copy_state.attack, 1);
    assert_eq!(copy_state.max_health, 1);
    assert_eq!(copy_state.cost, 1);
    assert!(copy_state.scripts_for_hook("on_deathrattle").is_empty());
    assert_eq!(copy_state.enchantments.len(), 1);
}

#[test]
fn deathwing_dragonlord_summons_the_original_dragon_entities_from_hand() {
    let (_dir, runtime) = fixture_runtime(&[("test_og_effects.lua", TEST_EFFECTS)]);
    let mut game = game_with_runtime(
        runtime,
        mixed(&["OG_317", "OG_271", "OG_320", "TEST_OG_KILL"]),
        repeated("CS2_120"),
        67,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    for card_id in ["OG_317", "OG_271", "OG_320", "TEST_OG_KILL"] {
        wait_for_hand(&mut game, PlayerId::ONE, card_id);
    }
    let dragons = [
        hand_card(&game, PlayerId::ONE, "OG_271"),
        hand_card(&game, PlayerId::ONE, "OG_320"),
    ];
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let deathwing = play(&mut game, PlayerId::ONE, "OG_317", None);
    play(&mut game, PlayerId::ONE, "TEST_OG_KILL", Some(deathwing));

    assert_eq!(
        game.state().entity(deathwing).unwrap().zone,
        Zone::Graveyard
    );
    for dragon in dragons {
        assert_eq!(game.state().entity(dragon).unwrap().zone, Zone::Board);
        assert!(game.state().player(PlayerId::ONE).board.contains(&dragon));
    }
}

#[test]
fn forbidden_cards_spend_exactly_the_remaining_mana() {
    let mut flame = game(mixed(&["OG_086", "CS2_120"]), repeated("CS2_120"));
    advance_to_mana(&mut flame, PlayerId::ONE, 5);
    for card_id in ["OG_086", "CS2_120"] {
        wait_for_hand(&mut flame, PlayerId::ONE, card_id);
    }
    advance_to_mana(&mut flame, PlayerId::ONE, 5);
    let victim = play(&mut flame, PlayerId::ONE, "CS2_120", None);
    play(&mut flame, PlayerId::ONE, "OG_086", Some(victim));
    assert_eq!(flame.state().player(PlayerId::ONE).mana, 0);
    assert_eq!(flame.state().entity(victim).unwrap().zone, Zone::Graveyard);

    let mut shaping = game(repeated("OG_101"), repeated("CS2_120"));
    advance_to_mana(&mut shaping, PlayerId::ONE, 4);
    play(&mut shaping, PlayerId::ONE, "OG_101", None);
    assert_eq!(shaping.state().player(PlayerId::ONE).mana, 0);
    let summoned = shaping.state().player(PlayerId::ONE).board[0];
    assert_eq!(shaping.state().entity(summoned).unwrap().base_cost, 4);
}

fn isolated_yogg_runtime() -> (TempRuntimeDir, LuaCardRuntime) {
    let suffix = TEMP_RUNTIME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("hearth-rs-og-yogg-{}-{suffix}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("libraries"), root.join("libraries")).unwrap();
    std::os::unix::fs::symlink(
        data_path().join("sets/core/the_coin.lua"),
        root.join("the_coin.lua"),
    )
    .unwrap();
    for card in ["yogg_saron_hopes_end.lua", "servant_of_yogg_saron.lua"] {
        std::os::unix::fs::symlink(data_path().join("sets/og").join(card), root.join(card))
            .unwrap();
    }
    std::fs::write(
        root.join("fixture.lua"),
        r#"
return {
    api_version = 1, id = "TEST_OG_WIPE", name = "Wipe", text = "Destroy all minions.",
    set = "TEST", type = "spell", cost = 5,
    on_play = function(ctx, self)
        local targets = ctx:minions()
        if #targets > 0 then ctx:destroy_all(targets) end
    end,
    tokens = {{ api_version = 1, id = "TEST_OG_HP", name = "Test Hero Power", text = "",
        set = "TEST", type = "hero_power", cost = 2, collectible = false }},
}
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(Path::new(&root)).unwrap();
    (TempRuntimeDir(root), runtime)
}

#[test]
fn yogg_finishes_its_frozen_batch_after_the_first_spell_removes_it() {
    let (_dir, runtime) = isolated_yogg_runtime();
    let mut game = game_with_runtime(
        runtime,
        mixed(&["TEST_OG_WIPE", "OG_134"]),
        repeated("TEST_OG_WIPE"),
        71,
        ["TEST_OG_HP", "TEST_OG_HP"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    wait_for_hand(&mut game, PlayerId::ONE, "TEST_OG_WIPE");
    play(&mut game, PlayerId::ONE, "TEST_OG_WIPE", None);
    advance_to_mana(&mut game, PlayerId::ONE, 6);
    wait_for_hand(&mut game, PlayerId::ONE, "TEST_OG_WIPE");
    play(&mut game, PlayerId::ONE, "TEST_OG_WIPE", None);
    wait_for_hand(&mut game, PlayerId::ONE, "OG_134");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let log_start = game.state().log.len();
    let yogg = play(&mut game, PlayerId::ONE, "OG_134", None);

    let generated = game.state().log[log_start..]
        .iter()
        .filter(|event| matches!(event, GameEvent::SpellCast { generated_by: Some(source), .. } if *source == yogg))
        .count();
    assert_eq!(game.state().entity(yogg).unwrap().zone, Zone::Graveyard);
    assert_eq!(generated, 2);
}

#[test]
fn yogg_continuation_keeps_its_lua_owner_after_yogg_is_transformed() {
    let suffix = TEMP_RUNTIME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "hearth-rs-og-yogg-transform-{}-{suffix}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("libraries"), root.join("libraries")).unwrap();
    std::os::unix::fs::symlink(
        data_path().join("sets/core/the_coin.lua"),
        root.join("the_coin.lua"),
    )
    .unwrap();
    for source in [
        data_path().join("sets/og/yogg_saron_hopes_end.lua"),
        data_path().join("sets/gangs/kun_the_forgotten_king.lua"),
    ] {
        std::os::unix::fs::symlink(&source, root.join(source.file_name().unwrap())).unwrap();
    }
    std::fs::write(
        root.join("fixture.lua"),
        r#"
local card = {
    api_version = 1, id = "TEST_OG_TRANSFORM", name = "Transform", text = "",
    set = "TEST", type = "spell", cost = 0,
    on_play = function(ctx, self)
        local minions = ctx:minions()
        if #minions > 0 then ctx:random_entity(minions, "transform_chosen") end
    end,
}
function card.transform_chosen(ctx, self, target) ctx:transform(target, "CFM_308") end
card.tokens = {{ api_version = 1, id = "TEST_OG_HP", name = "Test Hero Power", text = "",
    set = "TEST", type = "hero_power", cost = 2, collectible = false }}
return card
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(Path::new(&root)).unwrap();
    let _dir = TempRuntimeDir(root);
    let mut game = game_with_runtime(
        runtime,
        mixed(&["TEST_OG_TRANSFORM", "OG_134"]),
        repeated("TEST_OG_TRANSFORM"),
        72,
        ["TEST_OG_HP", "TEST_OG_HP"],
    );
    for _ in 0..2 {
        wait_for_hand(&mut game, PlayerId::ONE, "TEST_OG_TRANSFORM");
        play(&mut game, PlayerId::ONE, "TEST_OG_TRANSFORM", None);
        end_turn(&mut game);
        end_turn(&mut game);
    }
    wait_for_hand(&mut game, PlayerId::ONE, "OG_134");
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    let log_start = game.state().log.len();
    let yogg = play(&mut game, PlayerId::ONE, "OG_134", None);

    assert_eq!(game.state().entity(yogg).unwrap().card_id, "CFM_308");
    let generated = game.state().log[log_start..]
        .iter()
        .filter(|event| matches!(event, GameEvent::SpellCast { generated_by: Some(source), .. } if *source == yogg))
        .count();
    assert_eq!(generated, 2);
}

#[test]
fn servant_of_yogg_resolves_a_spell_that_removes_the_servant() {
    let (_dir, runtime) = isolated_yogg_runtime();
    let mut game = game_with_runtime(
        runtime,
        mixed(&["OG_087", "TEST_OG_WIPE"]),
        repeated("TEST_OG_WIPE"),
        73,
        ["TEST_OG_HP", "TEST_OG_HP"],
    );
    wait_for_hand(&mut game, PlayerId::ONE, "OG_087");
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    let log_start = game.state().log.len();
    let servant = play(&mut game, PlayerId::ONE, "OG_087", None);
    let generated = game.state().log[log_start..]
        .iter()
        .filter(|event| matches!(event, GameEvent::SpellCast { generated_by: Some(source), .. } if *source == servant))
        .count();
    assert_eq!(game.state().entity(servant).unwrap().zone, Zone::Graveyard);
    assert_eq!(generated, 1);
}

#[test]
fn random_spell_casts_resolve_choices_with_authoritative_rng_without_pausing() {
    let suffix = TEMP_RUNTIME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "hearth-rs-og-random-choice-{}-{suffix}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(
        data_path().join("sets/core/the_coin.lua"),
        root.join("the_coin.lua"),
    )
    .unwrap();
    std::fs::write(
        root.join("fixture.lua"),
        r#"
local card = {
    api_version = 1, id = "TEST_OG_RANDOM_CASTER", name = "Random Caster", text = "",
    set = "TEST", type = "spell", cost = 0, collectible = true,
    on_play = function(ctx, self)
        ctx:cast_spell(ctx:controller(self), "TEST_OG_RANDOM_CHOICE", {
            choice_policy = "random",
        })
    end,
}
card.tokens = {
    { api_version = 1, id = "TEST_OG_RANDOM_CHOICE", name = "Random Choice", text = "",
      set = "TEST", type = "spell", cost = 1,
      on_play = function(ctx, self)
          ctx:choose_options(ctx:controller(self), "Choose", {
              { label = "One", value = 1 }, { label = "Two", value = 2 },
          }, "chosen")
      end,
      chosen = function(ctx, self, amount) ctx:gain_armor(ctx:controller(self), amount) end },
    { api_version = 1, id = "TEST_OG_RANDOM_HP", name = "Test Hero Power", text = "",
      set = "TEST", type = "hero_power", cost = 2 },
}
return card
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(Path::new(&root)).unwrap();
    let _dir = TempRuntimeDir(root);
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_OG_RANDOM_CASTER"),
        repeated("TEST_OG_RANDOM_CASTER"),
        83,
        ["TEST_OG_RANDOM_HP", "TEST_OG_RANDOM_HP"],
    );
    play(&mut game, PlayerId::ONE, "TEST_OG_RANDOM_CASTER", None);
    assert!(game.state().pending_input.is_none());
    assert!(matches!(
        game.state()
            .entity(game.state().player(PlayerId::ONE).hero)
            .unwrap()
            .armor,
        1 | 2
    ));
    assert!(
        game.state()
            .log
            .iter()
            .any(|event| matches!(event, GameEvent::RandomChoiceMade { options: 2, .. }))
    );
}

#[test]
fn spreading_madness_ignores_spell_damage() {
    let mut game = game(mixed(&["OG_082", "OG_116"]), repeated("CS2_120"));
    wait_for_hand(&mut game, PlayerId::ONE, "OG_082");
    advance_to_mana(&mut game, PlayerId::ONE, 4);
    play(&mut game, PlayerId::ONE, "OG_082", None);
    wait_for_hand(&mut game, PlayerId::ONE, "OG_116");
    while game.state().active_player != PlayerId::ONE || game.state().player(PlayerId::ONE).mana < 3
    {
        end_turn(&mut game);
    }
    let log_start = game.state().log.len();
    let spell = play(&mut game, PlayerId::ONE, "OG_116", None);
    let total = game.state().log[log_start..]
        .iter()
        .filter_map(|event| match event {
            GameEvent::Damaged { source, amount, .. } if *source == spell => Some(*amount),
            _ => None,
        })
        .sum::<i32>();
    assert_eq!(total, 13);
}
