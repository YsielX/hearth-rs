use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use hearth_core::{
    ChoiceValue, DEFAULT_HERO_POWER, EntityId, Game, GameEvent, PlayerCommand, PlayerId, Zone,
};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn repeated(card: &str) -> Vec<String> {
    std::iter::repeat_n(card.to_owned(), 20).collect()
}

fn game_with_runtime(
    runtime: LuaCardRuntime,
    one: Vec<String>,
    two: Vec<String>,
    seed: u64,
    hero_powers: [&str; 2],
) -> Game<LuaCardRuntime> {
    let mut game = Game::new_unrestricted_with_hero_powers_and_classes(
        runtime,
        one,
        two,
        seed,
        hero_powers.map(str::to_owned),
        ["neutral".to_owned(), "neutral".to_owned()],
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

fn board_card(game: &Game<LuaCardRuntime>, player: PlayerId, card_id: &str) -> EntityId {
    game.state()
        .player(player)
        .board
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == card_id)
        .unwrap_or_else(|| panic!("{player} has no {card_id} on board"))
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

fn choose(game: &mut Game<LuaCardRuntime>, index: usize) {
    game.dispatch(PlayerCommand::Choose { index }).unwrap();
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
    let root =
        std::env::temp_dir().join(format!("hearth-rs-gangs-{}-{suffix}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("sets"), root.join("sets")).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("hero_powers"), root.join("hero_powers")).unwrap();
    std::fs::write(
        root.join("test_gangs_effects.lua"),
        r#"
local function clear_hand(ctx, self)
    local player = ctx:controller(self)
    for _, entity in ipairs(ctx:hand(player)) do ctx:discard(player, entity) end
end

local function give_many(ctx, player, cards)
    for _, card_id in ipairs(cards) do ctx:give_card(player, card_id) end
end

return {
    api_version = 1, id = "TEST_GANGS_EFFECTS", name = "GANGS Test Effects", text = "",
    set = "TEST", type = "spell", cost = 0, collectible = false,
    tokens = {
        { id = "TEST_GANGS_DUMMY", name = "Test Dummy", text = "", set = "TEST",
          type = "minion", cost = 0, attack = 0, health = 20, collectible = true },
        { id = "TEST_GANGS_SHEEP", name = "Test Sheep", text = "", set = "TEST",
          type = "minion", cost = 0, attack = 1, health = 1, collectible = false },
        { id = "TEST_GANGS_ATTACKER", name = "Test Attacker", text = "", set = "TEST",
          type = "minion", cost = 0, attack = 1, health = 20, collectible = true,
          keywords = { "charge" } },
        { id = "TEST_GANGS_MURLOC", name = "Test Murloc", text = "", set = "TEST",
          type = "minion", cost = 3, attack = 3, health = 3, collectible = false,
          tags = { "murloc" } },
        { id = "TEST_GANGS_POWER", name = "Test Power", text = "Deal 1 damage.", set = "TEST",
          type = "hero_power", cost = 0, collectible = false, target_mode = "required",
          targets = function(ctx) return ctx:characters() end,
          on_play = function(ctx, self, target) ctx:damage(target, 1) end },
        { id = "TEST_GANGS_KILL", name = "Kill", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false, target_mode = "required",
          targets = function(ctx) return ctx:minions() end,
          on_play = function(ctx, self, target) ctx:destroy(target) end },
        { id = "TEST_GANGS_SILENCE", name = "Silence", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false, target_mode = "required",
          targets = function(ctx) return ctx:minions() end,
          on_play = function(ctx, self, target) ctx:silence(target) end },
        { id = "TEST_GANGS_TRANSFORM", name = "Transform", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false, target_mode = "required",
          targets = function(ctx) return ctx:minions() end,
          on_play = function(ctx, self, target) ctx:transform(target, "TEST_GANGS_SHEEP") end },
        { id = "TEST_GANGS_BUFF", name = "Buff", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false, target_mode = "required",
          targets = function(ctx) return ctx:minions() end,
          on_play = function(ctx, self, target) ctx:buff(target, 3, 0) end },
        { id = "TEST_GANGS_DAMAGE", name = "Damage", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false, target_mode = "required",
          targets = function(ctx) return ctx:characters() end,
          on_play = function(ctx, self, target) ctx:damage(target, 1) end },
        { id = "TEST_GANGS_FILL_SELF", name = "Fill Self", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              for _ = #ctx:board(player) + 1, 7 do ctx:summon(player, "TEST_GANGS_DUMMY") end
          end },
        { id = "TEST_GANGS_FILL_OPPONENT", name = "Fill Opponent", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false,
          on_play = function(ctx, self)
              local enemy = ctx:opponent(ctx:controller(self))
              for _ = #ctx:board(enemy) + 1, 7 do ctx:summon(enemy, "TEST_GANGS_DUMMY") end
          end },
        { id = "TEST_GANGS_GENERATE_MURLOC", name = "Generate Murloc", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false,
          on_play = function(ctx, self) ctx:give_card(ctx:controller(self), "TEST_GANGS_MURLOC") end },
        { id = "TEST_GANGS_OVERLOAD", name = "Overload", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false,
          on_play = function(ctx, self) ctx:overload(ctx:controller(self), 2) end },
        { id = "TEST_GANGS_TEMP_MANA", name = "Temporary Mana", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = false,
          on_play = function(ctx, self) ctx:gain_temporary_mana(ctx:controller(self), 1) end },

        { id = "TEST_GANGS_JADE_SETUP", name = "Jade Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              give_many(ctx, player, { "CFM_602", "CFM_602", "CFM_602", "CFM_602",
                  "TEST_GANGS_SILENCE", "TEST_GANGS_KILL", "TEST_GANGS_FILL_SELF" })
          end },
        { id = "TEST_GANGS_POTION_SETUP", name = "Potion Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              give_many(ctx, player, { "CFM_603", "TEST_GANGS_SILENCE", "TEST_GANGS_TRANSFORM",
                  "TEST_GANGS_FILL_OPPONENT" })
              ctx:summon(ctx:opponent(player), "TEST_GANGS_DUMMY")
          end },
        { id = "TEST_GANGS_MAYOR_SETUP", name = "Mayor Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              ctx:summon(player, "TEST_GANGS_ATTACKER")
              ctx:summon(ctx:opponent(player), "CFM_670")
              ctx:summon(ctx:opponent(player), "TEST_GANGS_DUMMY")
              ctx:summon(ctx:opponent(player), "TEST_GANGS_DUMMY")
              ctx:give_card(player, "TEST_GANGS_DAMAGE")
          end },
        { id = "TEST_GANGS_SEADEVIL_SETUP", name = "Seadevil Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              give_many(ctx, player, { "CFM_699", "TEST_GANGS_GENERATE_MURLOC" })
          end },
        { id = "TEST_GANGS_KUN_SETUP", name = "Kun Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              give_many(ctx, player, { "CFM_308", "TEST_GANGS_OVERLOAD", "TEST_GANGS_TEMP_MANA" })
              ctx:continue_with("discount_kun")
          end,
          discount_kun = function(ctx, self)
              for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                  if ctx:entity(entity).card_id == "CFM_308" then
                      ctx:modify(entity, { stat = "cost", operation = "set", value = 0 })
                  end
              end
          end },
        { id = "TEST_GANGS_RAT_SETUP", name = "Rat Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              give_many(ctx, player, { "CFM_316", "TEST_GANGS_BUFF", "TEST_GANGS_KILL" })
          end },
        { id = "TEST_GANGS_SALLY_SETUP", name = "Sally Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              give_many(ctx, player, { "CFM_341", "TEST_GANGS_BUFF", "TEST_GANGS_KILL" })
              ctx:summon(ctx:opponent(player), "TEST_GANGS_DUMMY")
              ctx:summon(ctx:opponent(player), "TEST_GANGS_DUMMY")
          end },
        { id = "TEST_GANGS_KAZAKUS_SETUP", name = "Kazakus Setup", text = "", set = "TEST", type = "spell",
          cost = 0, collectible = true,
          on_play = function(ctx, self)
              local player = ctx:controller(self)
              clear_hand(ctx, self)
              for _, entity in ipairs(ctx:deck(player)) do ctx:move(entity, "graveyard") end
              ctx:summon(player, "TEST_GANGS_DUMMY")
              ctx:continue_with("finish_kazakus_setup")
          end,
          finish_kazakus_setup = function(ctx, self)
              local player = ctx:controller(self)
              for _, entity in ipairs(ctx:board(player)) do
                  if ctx:entity(entity).card_id == "TEST_GANGS_DUMMY" then
                      ctx:destroy(entity)
                      break
                  end
              end
              ctx:summon(ctx:opponent(player), "TEST_GANGS_DUMMY")
              ctx:give_card(player, "CFM_621")
          end },
    },
}
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(Path::new(&root)).unwrap();
    (TempRuntimeDir(root), runtime)
}

fn fixture_game(setup: &str, seed: u64) -> (TempRuntimeDir, Game<LuaCardRuntime>) {
    let (dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated(setup),
        repeated("TEST_GANGS_DUMMY"),
        seed,
        [DEFAULT_HERO_POWER, DEFAULT_HERO_POWER],
    );
    play(&mut game, PlayerId::ONE, setup, None);
    (dir, game)
}

#[test]
fn gangs_catalog_is_the_exact_132_card_collectible_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|card| card.set == "GANGS" && card.collectible)
        .map(|card| card.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = [
        "CFM_020", "CFM_021", "CFM_025", "CFM_026", "CFM_039", "CFM_060", "CFM_061", "CFM_062",
        "CFM_063", "CFM_064", "CFM_065", "CFM_066", "CFM_067", "CFM_094", "CFM_095", "CFM_120",
        "CFM_300", "CFM_305", "CFM_308", "CFM_310", "CFM_312", "CFM_313", "CFM_315", "CFM_316",
        "CFM_321", "CFM_324", "CFM_325", "CFM_328", "CFM_333", "CFM_334", "CFM_335", "CFM_336",
        "CFM_337", "CFM_338", "CFM_341", "CFM_342", "CFM_343", "CFM_344", "CFM_602", "CFM_603",
        "CFM_604", "CFM_605", "CFM_606", "CFM_608", "CFM_609", "CFM_610", "CFM_611", "CFM_614",
        "CFM_616", "CFM_617", "CFM_619", "CFM_620", "CFM_621", "CFM_623", "CFM_626", "CFM_630",
        "CFM_631", "CFM_634", "CFM_636", "CFM_637", "CFM_639", "CFM_643", "CFM_646", "CFM_647",
        "CFM_648", "CFM_649", "CFM_650", "CFM_651", "CFM_652", "CFM_653", "CFM_654", "CFM_655",
        "CFM_656", "CFM_657", "CFM_658", "CFM_659", "CFM_660", "CFM_661", "CFM_662", "CFM_663",
        "CFM_665", "CFM_666", "CFM_667", "CFM_668", "CFM_669", "CFM_670", "CFM_671", "CFM_672",
        "CFM_685", "CFM_687", "CFM_688", "CFM_690", "CFM_691", "CFM_693", "CFM_694", "CFM_696",
        "CFM_697", "CFM_699", "CFM_707", "CFM_713", "CFM_715", "CFM_716", "CFM_717", "CFM_750",
        "CFM_751", "CFM_752", "CFM_753", "CFM_754", "CFM_755", "CFM_756", "CFM_759", "CFM_760",
        "CFM_781", "CFM_790", "CFM_800", "CFM_806", "CFM_807", "CFM_808", "CFM_809", "CFM_810",
        "CFM_811", "CFM_815", "CFM_816", "CFM_851", "CFM_852", "CFM_853", "CFM_854", "CFM_855",
        "CFM_900", "CFM_902", "CFM_905", "CFM_940",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual.len(), 132);
    assert_eq!(actual, expected);
}

#[test]
fn jade_scales_across_failed_summons_and_silence_keeps_intrinsic_stats() {
    let (_dir, mut game) = fixture_game("TEST_GANGS_JADE_SETUP", 101);
    advance_to_mana(&mut game, PlayerId::ONE, 10);

    play(&mut game, PlayerId::ONE, "CFM_602", None);
    choose(&mut game, 0);
    play(&mut game, PlayerId::ONE, "CFM_602", None);
    choose(&mut game, 0);
    let jades = game.state().player(PlayerId::ONE).board.clone();
    assert_eq!(
        (
            game.state().entity(jades[0]).unwrap().attack,
            game.state().entity(jades[0]).unwrap().max_health
        ),
        (1, 1)
    );
    assert_eq!(
        (
            game.state().entity(jades[1]).unwrap().attack,
            game.state().entity(jades[1]).unwrap().max_health
        ),
        (2, 2)
    );
    play(
        &mut game,
        PlayerId::ONE,
        "TEST_GANGS_SILENCE",
        Some(jades[1]),
    );
    assert_eq!(
        (
            game.state().entity(jades[1]).unwrap().attack,
            game.state().entity(jades[1]).unwrap().max_health
        ),
        (2, 2)
    );

    play(&mut game, PlayerId::ONE, "TEST_GANGS_FILL_SELF", None);
    assert_eq!(game.state().player(PlayerId::ONE).board.len(), 7);
    play(&mut game, PlayerId::ONE, "CFM_602", None);
    choose(&mut game, 0);
    assert_eq!(game.state().player(PlayerId::ONE).board.len(), 7);

    let filler = board_card(&game, PlayerId::ONE, "TEST_GANGS_DUMMY");
    play(&mut game, PlayerId::ONE, "TEST_GANGS_KILL", Some(filler));
    play(&mut game, PlayerId::ONE, "CFM_602", None);
    choose(&mut game, 0);
    let last = *game.state().player(PlayerId::ONE).board.last().unwrap();
    assert_eq!(game.state().entity(last).unwrap().card_id, "CFM_712_t01");
    assert_eq!(
        (
            game.state().entity(last).unwrap().attack,
            game.state().entity(last).unwrap().max_health
        ),
        (4, 4)
    );
}

fn stolen_dummy(game: &Game<LuaCardRuntime>) -> EntityId {
    board_card(game, PlayerId::ONE, "TEST_GANGS_DUMMY")
}

#[test]
fn potion_of_madness_handles_end_turn_silence_transform_and_full_return_board() {
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_POTION_SETUP", 110);
        advance_to_mana(&mut game, PlayerId::ONE, 1);
        let target = board_card(&game, PlayerId::TWO, "TEST_GANGS_DUMMY");
        play(&mut game, PlayerId::ONE, "CFM_603", Some(target));
        assert_eq!(
            game.state().entity(target).unwrap().controller,
            PlayerId::ONE
        );
        end_turn(&mut game);
        assert_eq!(
            game.state().entity(target).unwrap().controller,
            PlayerId::TWO
        );
    }
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_POTION_SETUP", 111);
        advance_to_mana(&mut game, PlayerId::ONE, 1);
        let target = board_card(&game, PlayerId::TWO, "TEST_GANGS_DUMMY");
        play(&mut game, PlayerId::ONE, "CFM_603", Some(target));
        play(&mut game, PlayerId::ONE, "TEST_GANGS_SILENCE", Some(target));
        assert_eq!(
            game.state().entity(target).unwrap().controller,
            PlayerId::TWO
        );
        assert!(
            game.state()
                .entity(target)
                .unwrap()
                .temporary_control
                .is_none()
        );
    }
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_POTION_SETUP", 112);
        advance_to_mana(&mut game, PlayerId::ONE, 1);
        let target = board_card(&game, PlayerId::TWO, "TEST_GANGS_DUMMY");
        play(&mut game, PlayerId::ONE, "CFM_603", Some(target));
        play(
            &mut game,
            PlayerId::ONE,
            "TEST_GANGS_TRANSFORM",
            Some(target),
        );
        assert!(
            game.state()
                .entity(target)
                .unwrap()
                .temporary_control
                .is_none()
        );
        end_turn(&mut game);
        assert_eq!(
            game.state().entity(target).unwrap().controller,
            PlayerId::ONE
        );
        assert_eq!(game.state().entity(target).unwrap().zone, Zone::Board);
    }
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_POTION_SETUP", 113);
        advance_to_mana(&mut game, PlayerId::ONE, 1);
        let target = board_card(&game, PlayerId::TWO, "TEST_GANGS_DUMMY");
        play(&mut game, PlayerId::ONE, "CFM_603", Some(target));
        assert_eq!(stolen_dummy(&game), target);
        play(&mut game, PlayerId::ONE, "TEST_GANGS_FILL_OPPONENT", None);
        assert_eq!(game.state().player(PlayerId::TWO).board.len(), 7);
        end_turn(&mut game);
        assert_eq!(game.state().entity(target).unwrap().zone, Zone::Graveyard);
    }
}

#[test]
fn mayor_randomizes_only_legal_attack_spell_and_power_targets_and_replays() {
    let (dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated("TEST_GANGS_MAYOR_SETUP"),
        repeated("TEST_GANGS_DUMMY"),
        127,
        ["TEST_GANGS_POWER", "TEST_GANGS_POWER"],
    );
    play(&mut game, PlayerId::ONE, "TEST_GANGS_MAYOR_SETUP", None);
    let attacker = board_card(&game, PlayerId::ONE, "TEST_GANGS_ATTACKER");
    let declared_defender = game.state().player(PlayerId::TWO).hero;
    let legal_defenders = game
        .legal_actions()
        .unwrap()
        .into_iter()
        .filter_map(|command| match command {
            PlayerCommand::Attack {
                attacker: candidate,
                defender,
            } if candidate == attacker => Some(defender),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let random_before = game.state().random_counter;
    game.dispatch(PlayerCommand::Attack {
        attacker,
        defender: declared_defender,
    })
    .unwrap();
    let actual_defender = game
        .state()
        .log
        .iter()
        .rev()
        .find_map(|event| match event {
            GameEvent::Attack {
                attacker: source,
                defender,
                ..
            } if *source == attacker => Some(*defender),
            _ => None,
        })
        .unwrap();
    assert!(legal_defenders.contains(&actual_defender));
    assert_eq!(game.state().random_counter, random_before + 1);

    let valid_characters = game
        .state()
        .entities
        .values()
        .filter(|entity| matches!(entity.zone, Zone::Hero | Zone::Board))
        .map(|entity| entity.id)
        .collect::<BTreeSet<_>>();
    let declared_spell_target = game.state().player(PlayerId::TWO).hero;
    let spell = play(
        &mut game,
        PlayerId::ONE,
        "TEST_GANGS_DAMAGE",
        Some(declared_spell_target),
    );
    let actual_spell_target = game
        .state()
        .log
        .iter()
        .rev()
        .find_map(|event| match event {
            GameEvent::SpellTargeted {
                spell: source,
                target,
                ..
            } if *source == spell => Some(*target),
            _ => None,
        })
        .unwrap();
    assert!(valid_characters.contains(&actual_spell_target));

    let declared_power_target = game.state().player(PlayerId::TWO).hero;
    let power = game.state().player(PlayerId::ONE).hero_power;
    game.dispatch(PlayerCommand::UseHeroPower {
        target: Some(declared_power_target),
    })
    .unwrap();
    let actual_power_target = game
        .state()
        .log
        .iter()
        .rev()
        .find_map(|event| match event {
            GameEvent::HeroPowerUsed {
                hero_power, target, ..
            } if *hero_power == power => *target,
            _ => None,
        })
        .unwrap();
    assert!(valid_characters.contains(&actual_power_target));

    let replay = game.replay();
    let restored = Game::from_replay(
        LuaCardRuntime::load_dir(Path::new(&dir.0)).unwrap(),
        &replay,
    )
    .unwrap();
    assert_eq!(restored.state(), game.state());
}

#[test]
fn seadevil_applies_to_a_later_generated_murloc_and_expires_with_the_turn() {
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_SEADEVIL_SETUP", 130);
        advance_to_mana(&mut game, PlayerId::ONE, 5);
        play(&mut game, PlayerId::ONE, "CFM_699", None);
        play(&mut game, PlayerId::ONE, "TEST_GANGS_GENERATE_MURLOC", None);
        let health_before = game
            .state()
            .entity(game.state().player(PlayerId::ONE).hero)
            .unwrap()
            .health();
        let mana_before = game.state().player(PlayerId::ONE).mana;
        play(&mut game, PlayerId::ONE, "TEST_GANGS_MURLOC", None);
        assert_eq!(game.state().player(PlayerId::ONE).mana, mana_before);
        assert_eq!(
            game.state()
                .entity(game.state().player(PlayerId::ONE).hero)
                .unwrap()
                .health(),
            health_before - 3
        );
    }
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_SEADEVIL_SETUP", 131);
        advance_to_mana(&mut game, PlayerId::ONE, 5);
        play(&mut game, PlayerId::ONE, "CFM_699", None);
        play(&mut game, PlayerId::ONE, "TEST_GANGS_GENERATE_MURLOC", None);
        end_turn(&mut game);
        end_turn(&mut game);
        let health_before = game
            .state()
            .entity(game.state().player(PlayerId::ONE).hero)
            .unwrap()
            .health();
        let mana_before = game.state().player(PlayerId::ONE).mana;
        play(&mut game, PlayerId::ONE, "TEST_GANGS_MURLOC", None);
        assert_eq!(game.state().player(PlayerId::ONE).mana, mana_before - 3);
        assert_eq!(
            game.state()
                .entity(game.state().player(PlayerId::ONE).hero)
                .unwrap()
                .health(),
            health_before
        );
    }
}

#[test]
fn kun_refresh_preserves_locked_and_temporary_mana() {
    let (_dir, mut game) = fixture_game("TEST_GANGS_KUN_SETUP", 140);
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "TEST_GANGS_OVERLOAD", None);
    end_turn(&mut game);
    end_turn(&mut game);
    assert_eq!(game.state().player(PlayerId::ONE).overloaded_mana, 2);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 8);
    play(&mut game, PlayerId::ONE, "TEST_GANGS_TEMP_MANA", None);
    assert_eq!(game.state().player(PlayerId::ONE).temporary_mana, 1);
    assert_eq!(game.state().player(PlayerId::ONE).mana, 9);
    play(&mut game, PlayerId::ONE, "CFM_308", None);
    choose(&mut game, 1);
    let player = game.state().player(PlayerId::ONE);
    assert_eq!(player.overloaded_mana, 2);
    assert_eq!(player.overload_pending, 0);
    assert_eq!(player.temporary_mana, 1);
    assert_eq!(player.mana, 9);
}

#[test]
fn rat_pack_and_sergeant_sally_use_attack_frozen_at_death() {
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_RAT_SETUP", 150);
        advance_to_mana(&mut game, PlayerId::ONE, 3);
        let rat_pack = play(&mut game, PlayerId::ONE, "CFM_316", None);
        play(&mut game, PlayerId::ONE, "TEST_GANGS_BUFF", Some(rat_pack));
        assert_eq!(game.state().entity(rat_pack).unwrap().attack, 5);
        play(&mut game, PlayerId::ONE, "TEST_GANGS_KILL", Some(rat_pack));
        let rats = game
            .state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .filter(|entity| game.state().entity(**entity).unwrap().card_id == "CFM_316t")
            .count();
        assert_eq!(rats, 5);
        assert_eq!(
            game.state().entity(rat_pack).unwrap().attack_at_death,
            Some(5)
        );
    }
    {
        let (_dir, mut game) = fixture_game("TEST_GANGS_SALLY_SETUP", 151);
        advance_to_mana(&mut game, PlayerId::ONE, 3);
        let enemies = game.state().player(PlayerId::TWO).board.clone();
        let sally = play(&mut game, PlayerId::ONE, "CFM_341", None);
        play(&mut game, PlayerId::ONE, "TEST_GANGS_BUFF", Some(sally));
        play(&mut game, PlayerId::ONE, "TEST_GANGS_KILL", Some(sally));
        assert_eq!(game.state().entity(sally).unwrap().attack_at_death, Some(4));
        for enemy in enemies {
            assert_eq!(game.state().entity(enemy).unwrap().damage, 4);
        }
    }
}

fn ingredient_category(card_id: &str) -> i64 {
    match card_id {
        "CFM_621t4" => 1,
        "CFM_621t6" => 2,
        "CFM_621t2" => 3,
        "CFM_621t5" => 4,
        "CFM_621t37" => 5,
        "CFM_621t8" => 6,
        "CFM_621t9" => 7,
        "CFM_621t3" => 8,
        "CFM_621t10" => 9,
        other => panic!("unexpected 1-Cost Kazakus ingredient {other}"),
    }
}

fn pending_card(game: &Game<LuaCardRuntime>, index: usize) -> String {
    match &game.state().pending_input.as_ref().unwrap().options[index].value {
        ChoiceValue::Card(card_id) => card_id.clone(),
        other => panic!("expected card choice, got {other:?}"),
    }
}

#[test]
fn kazakus_discovers_two_distinct_ingredients_and_executes_the_custom_potion() {
    let (dir, mut game) = fixture_game("TEST_GANGS_KAZAKUS_SETUP", 160);
    advance_to_mana(&mut game, PlayerId::ONE, 5);
    assert!(game.state().player(PlayerId::ONE).deck.is_empty());
    play(&mut game, PlayerId::ONE, "CFM_621", None);
    assert_eq!(
        game.state().pending_input.as_ref().unwrap().prompt,
        "Choose a potion Cost"
    );
    choose(&mut game, 0);
    let first = pending_card(&game, 0);
    choose(&mut game, 0);
    let second = pending_card(&game, 0);
    choose(&mut game, 0);
    let first_category = ingredient_category(&first);
    let second_category = ingredient_category(&second);
    assert_ne!(first_category, second_category);

    let potion = hand_card(&game, PlayerId::ONE, "CFM_621t");
    assert_eq!(game.state().entity(potion).unwrap().cost, 1);
    assert_eq!(
        game.state()
            .entity(potion)
            .unwrap()
            .script_data
            .get("kazakus_first"),
        Some(&first_category)
    );
    assert_eq!(
        game.state()
            .entity(potion)
            .unwrap()
            .script_data
            .get("kazakus_second"),
        Some(&second_category)
    );

    let own_hero = game.state().player(PlayerId::ONE).hero;
    let enemy_hero = game.state().player(PlayerId::TWO).hero;
    let kazakus = board_card(&game, PlayerId::ONE, "CFM_621");
    let enemy_dummy = board_card(&game, PlayerId::TWO, "TEST_GANGS_DUMMY");
    let own_health_before = game.state().entity(own_hero).unwrap().health();
    let enemy_health_before = game.state().entity(enemy_hero).unwrap().health();
    let armor_before = game.state().entity(own_hero).unwrap().armor;
    let kazakus_health_before = game.state().entity(kazakus).unwrap().max_health;
    let enemy_damage_before = game.state().entity(enemy_dummy).unwrap().damage;
    let fatigue_before = game.state().player(PlayerId::ONE).fatigue;
    let hand_before = game.state().player(PlayerId::ONE).hand.len();
    let target = (first_category == 3 || second_category == 3).then_some(enemy_hero);
    game.dispatch(PlayerCommand::PlayCard {
        card: potion,
        target,
    })
    .unwrap();
    assert_eq!(game.state().entity(potion).unwrap().zone, Zone::Graveyard);

    let categories = [first_category, second_category];
    if categories.contains(&1) {
        assert_eq!(
            game.state().entity(enemy_dummy).unwrap().damage,
            enemy_damage_before + 2
        );
    }
    if categories.contains(&2) {
        assert_eq!(
            game.state().entity(kazakus).unwrap().max_health,
            kazakus_health_before + 2
        );
    }
    if categories.contains(&3) {
        assert_eq!(
            game.state().entity(enemy_hero).unwrap().health(),
            enemy_health_before - 3
        );
    }
    if categories.contains(&4) {
        assert!(game.state().entity(enemy_dummy).unwrap().frozen);
    }
    if categories.contains(&5) {
        assert!(
            game.state()
                .player(PlayerId::ONE)
                .board
                .iter()
                .any(|entity| game.state().entity(*entity).unwrap().card_id == "TEST_GANGS_DUMMY")
        );
    }
    if categories.contains(&6) {
        assert_eq!(
            game.state().player(PlayerId::ONE).fatigue,
            fatigue_before + 1
        );
        assert_eq!(
            game.state().entity(own_hero).unwrap().health(),
            own_health_before - 1
        );
    }
    if categories.contains(&7) {
        assert_eq!(game.state().player(PlayerId::ONE).hand.len(), hand_before);
    }
    if categories.contains(&8) {
        assert_eq!(
            game.state().entity(own_hero).unwrap().armor,
            armor_before + 4
        );
    }
    if categories.contains(&9) {
        assert!(
            game.state()
                .player(PlayerId::ONE)
                .board
                .iter()
                .any(|entity| game.state().entity(*entity).unwrap().card_id == "CFM_621_m4")
        );
    }

    let replay = game.replay();
    let restored = Game::from_replay(
        LuaCardRuntime::load_dir(Path::new(&dir.0)).unwrap(),
        &replay,
    )
    .unwrap();
    assert_eq!(restored.state(), game.state());
}
