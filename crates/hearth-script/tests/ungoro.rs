use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use hearth_core::{
    ChoiceOptionValueView, ChoiceValue, DEFAULT_HERO_POWER, EntityId, Game, GameEvent,
    PlayerCommand, PlayerId, Zone,
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
    classes: [&str; 2],
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

fn choose_card(game: &mut Game<LuaCardRuntime>, card_id: &str) -> bool {
    let Some(pending) = game.state().pending_input.as_ref() else {
        return false;
    };
    let Some(index) = pending.options.iter().position(
        |option| matches!(&option.value, ChoiceValue::Card(candidate) if candidate == card_id),
    ) else {
        return false;
    };
    game.dispatch(PlayerCommand::Choose { index }).unwrap();
    true
}

#[test]
fn umbra_ignores_a_stale_summon_after_sacred_trial_destroyed_the_minion() {
    let mut game = game_with_runtime(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        mixed(&["UNG_900", "CS2_120", "EX1_534"]),
        repeated("LOE_027"),
        20264534,
        ["hunter", "paladin"],
    );

    advance_to_mana(&mut game, PlayerId::TWO, 1);
    play(&mut game, PlayerId::TWO, "LOE_027", None);
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "UNG_900", None);
    play(&mut game, PlayerId::ONE, "CS2_120", None);
    play(&mut game, PlayerId::ONE, "CS2_120", None);
    end_turn(&mut game);
    end_turn(&mut game);

    let highmane = play(&mut game, PlayerId::ONE, "EX1_534", None);
    assert_eq!(game.state().entity(highmane).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.state()
            .player(PlayerId::ONE)
            .board
            .iter()
            .filter(|entity| game.state().entity(**entity).unwrap().card_id == "EX1_534t")
            .count(),
        2
    );
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
        std::env::temp_dir().join(format!("hearth-rs-ungoro-{}-{suffix}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("sets"), root.join("sets")).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("hero_powers"), root.join("hero_powers")).unwrap();
    std::os::unix::fs::symlink(data_path().join("libraries"), root.join("libraries")).unwrap();
    std::fs::write(
        root.join("test_ungoro_effects.lua"),
        r#"
local function clear_hand(ctx, player)
    for _, entity in ipairs(ctx:hand(player)) do ctx:discard(player, entity) end
end

local function give_many(ctx, player, cards)
    for _, card_id in ipairs(cards) do ctx:create_card(player, card_id) end
end

local function make_free(ctx, player, card_id)
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).card_id == card_id then
            cardlib.effects.modify(ctx, entity, { stat = "cost", operation = "set", value = 0, silenciable = false })
        end
    end
end

local cards = {
    { id = "TEST_UNGORO_VICTIM", name = "Dred Victim", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 3, health = 12, collectible = true },
    { id = "TEST_UNGORO_WALL", name = "Test Wall", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 0, health = 30, collectible = false },
    { id = "TEST_UNGORO_SPELLPOWER", name = "Spell Power Test", text = "Spell Damage +2",
      set = "TEST", type = "minion", cost = 0, attack = 0, health = 10,
      collectible = false, keywords = { "spell_damage" }, keyword_params = { spell_damage = 2 } },
    { id = "TEST_UNGORO_BIG", name = "Damaged Giant", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 8, health = 8, collectible = false },
    { id = "TEST_UNGORO_TARGET", name = "Adapt Target", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 2, health = 8, collectible = false },
    { id = "TEST_UNGORO_LEFT", name = "Left Marker", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 1, health = 10, collectible = false },
    { id = "TEST_UNGORO_RIGHT", name = "Right Marker", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 1, health = 10, collectible = false },
    { id = "TEST_UNGORO_TAUNT", name = "Free Taunt", text = "Taunt", set = "TEST",
      type = "minion", cost = 0, attack = 1, health = 1, collectible = false,
      keywords = { "taunt" } },
    { id = "TEST_UNGORO_KILL", name = "Kill", text = "", set = "TEST", type = "spell",
      cost = 0, collectible = false, target_mode = "required",
      targets = function(ctx) return ctx:minions() end,
      on_play = function(ctx, self, target) cardlib.effects.destroy(ctx, target) end },
    { id = "TEST_UNGORO_FILLER", name = "Filler", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = false },

    { id = "TEST_UNGORO_TIME_WARP_SETUP", name = "Time Warp Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local one, two = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, one)
          give_many(ctx, one, { "UNG_028t", "UNG_028t" })
          give_many(ctx, two, { "UNG_028t", "UNG_028t" })
      end },
    { id = "TEST_UNGORO_DRED_SETUP", name = "Dred Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          clear_hand(ctx, ctx:controller(self)); ctx:summon(ctx:controller(self), "UNG_919")
      end },
    { id = "TEST_UNGORO_BATCH_SETUP", name = "Batch Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, enemy = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player)
          ctx:summon(player, "TEST_UNGORO_SPELLPOWER")
          ctx:summon(enemy, "TEST_UNGORO_WALL")
          ctx:summon(enemy, "TEST_UNGORO_WALL")
          ctx:summon(enemy, "TEST_UNGORO_WALL")
          give_many(ctx, player, { "UNG_910", "UNG_955" })
          ctx:continue_with("finish_batch_setup")
      end,
      finish_batch_setup = function(ctx, self)
          make_free(ctx, ctx:controller(self), "UNG_910"); make_free(ctx, ctx:controller(self), "UNG_955")
      end },
    { id = "TEST_UNGORO_TARIM_SETUP", name = "Tarim Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, enemy = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player)
          ctx:summon(player, "TEST_UNGORO_BIG"); ctx:summon(enemy, "TEST_UNGORO_BIG")
          ctx:continue_with("damage_tarim_targets")
          ctx:create_card(player, "UNG_015"); ctx:continue_with("finish_tarim_setup")
      end,
      damage_tarim_targets = function(ctx, self)
          for _, entity in ipairs(ctx:minions()) do
              if ctx:entity(entity).card_id == "TEST_UNGORO_BIG" then cardlib.effects.damage_ignoring_spell_damage(ctx, entity, 5) end
          end
      end,
      finish_tarim_setup = function(ctx, self) make_free(ctx, ctx:controller(self), "UNG_015") end },
    { id = "TEST_UNGORO_ADAPT_SETUP", name = "Adapt Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self); clear_hand(ctx, player)
          ctx:summon(player, "TEST_UNGORO_TARGET")
          give_many(ctx, player, { "UNG_961", "TEST_UNGORO_KILL" })
      end },
    { id = "TEST_UNGORO_GLIMMER_SETUP", name = "Glimmerroot Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, enemy = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player); clear_hand(ctx, enemy)
          for _, entity in ipairs(ctx:deck(enemy)) do ctx:move(entity, "graveyard") end
          ctx:create_card(player, "UNG_035"); ctx:continue_with("finish_glimmer_setup")
      end,
      finish_glimmer_setup = function(ctx, self) make_free(ctx, ctx:controller(self), "UNG_035") end },
    { id = "TEST_UNGORO_OBSIDIAN_SETUP", name = "Obsidian Setup", text = "", set = "TEST",
      type = "spell", class = "rogue", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self); clear_hand(ctx, player)
          give_many(ctx, player, { "UNG_061", "CS2_029", "CS2_029" })
      end },
    { id = "TEST_UNGORO_SHERAZIN_SETUP", name = "Sherazin Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self); clear_hand(ctx, player)
          ctx:summon(player, "TEST_UNGORO_LEFT"); ctx:summon(player, "UNG_065"); ctx:summon(player, "TEST_UNGORO_RIGHT")
          ctx:continue_with("kill_sherazin")
          give_many(ctx, player, { "TEST_UNGORO_FILLER", "TEST_UNGORO_FILLER", "TEST_UNGORO_FILLER" })
      end,
      kill_sherazin = function(ctx, self)
          for _, entity in ipairs(ctx:board(ctx:controller(self))) do
              if ctx:entity(entity).card_id == "UNG_065" then cardlib.effects.destroy(ctx, entity); return end
          end
      end },
    { id = "TEST_UNGORO_FIRE_PLUME_SETUP", name = "Fire Plume Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self); clear_hand(ctx, player)
          ctx:create_card(player, "UNG_934")
          for _ = 1, 7 do ctx:create_card(player, "TEST_UNGORO_TAUNT") end
          ctx:continue_with("finish_fire_plume_setup")
      end,
      finish_fire_plume_setup = function(ctx, self)
          make_free(ctx, ctx:controller(self), "UNG_934"); make_free(ctx, ctx:controller(self), "TEST_UNGORO_TAUNT")
      end },
}

return { api_version = 1, id = "TEST_UNGORO_ROOT", name = "Un'Goro Test Root", text = "",
    set = "TEST", type = "spell", cost = 0, collectible = false, tokens = cards }
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(&root).unwrap();
    (TempRuntimeDir(root), runtime)
}

fn fixture_game(
    setup: &str,
    opponent_card: &str,
    seed: u64,
    classes: [&str; 2],
) -> (TempRuntimeDir, Game<LuaCardRuntime>) {
    let (dir, runtime) = fixture_runtime();
    let mut game = game_with_runtime(
        runtime,
        repeated(setup),
        repeated(opponent_card),
        seed,
        classes,
    );
    play(&mut game, PlayerId::ONE, setup, None);
    (dir, game)
}

#[test]
fn ungoro_catalog_is_the_exact_135_card_collectible_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|card| card.set == "UNGORO" && card.collectible)
        .map(|card| card.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = [
        "UNG_001", "UNG_002", "UNG_004", "UNG_009", "UNG_010", "UNG_011", "UNG_015", "UNG_018",
        "UNG_019", "UNG_020", "UNG_021", "UNG_022", "UNG_024", "UNG_025", "UNG_027", "UNG_028",
        "UNG_029", "UNG_030", "UNG_032", "UNG_034", "UNG_035", "UNG_037", "UNG_047", "UNG_049",
        "UNG_057", "UNG_058", "UNG_060", "UNG_061", "UNG_063", "UNG_064", "UNG_065", "UNG_067",
        "UNG_070", "UNG_071", "UNG_072", "UNG_073", "UNG_075", "UNG_076", "UNG_078", "UNG_079",
        "UNG_082", "UNG_083", "UNG_084", "UNG_085", "UNG_086", "UNG_087", "UNG_088", "UNG_089",
        "UNG_099", "UNG_100", "UNG_101", "UNG_103", "UNG_108", "UNG_109", "UNG_111", "UNG_113",
        "UNG_116", "UNG_201", "UNG_202", "UNG_205", "UNG_208", "UNG_211", "UNG_800", "UNG_801",
        "UNG_803", "UNG_806", "UNG_807", "UNG_808", "UNG_809", "UNG_810", "UNG_812", "UNG_813",
        "UNG_814", "UNG_816", "UNG_817", "UNG_818", "UNG_823", "UNG_829", "UNG_830", "UNG_831",
        "UNG_832", "UNG_833", "UNG_834", "UNG_835", "UNG_836", "UNG_838", "UNG_840", "UNG_843",
        "UNG_844", "UNG_845", "UNG_846", "UNG_847", "UNG_848", "UNG_851", "UNG_852", "UNG_854",
        "UNG_856", "UNG_900", "UNG_907", "UNG_910", "UNG_912", "UNG_913", "UNG_914", "UNG_915",
        "UNG_916", "UNG_917", "UNG_919", "UNG_920", "UNG_922", "UNG_923", "UNG_925", "UNG_926",
        "UNG_927", "UNG_928", "UNG_929", "UNG_933", "UNG_934", "UNG_937", "UNG_938", "UNG_940",
        "UNG_941", "UNG_942", "UNG_946", "UNG_948", "UNG_950", "UNG_952", "UNG_953", "UNG_954",
        "UNG_955", "UNG_956", "UNG_957", "UNG_960", "UNG_961", "UNG_962", "UNG_963",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(actual.len(), 135);
    assert_eq!(actual, expected);
}

#[test]
fn time_warp_grants_only_one_extra_turn_to_each_player_per_game() {
    let (_dir, mut game) = fixture_game(
        "TEST_UNGORO_TIME_WARP_SETUP",
        "TEST_UNGORO_VICTIM",
        201,
        ["mage", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 10);
    play(&mut game, PlayerId::ONE, "UNG_028t", None);
    play(&mut game, PlayerId::ONE, "UNG_028t", None);
    assert_eq!(game.state().player(PlayerId::ONE).extra_turns, 1);
    end_turn(&mut game);
    assert_eq!(game.state().active_player, PlayerId::ONE);
    assert_eq!(game.state().player(PlayerId::ONE).extra_turns, 0);
    end_turn(&mut game);
    assert_eq!(game.state().active_player, PlayerId::TWO);

    advance_to_mana(&mut game, PlayerId::TWO, 10);
    play(&mut game, PlayerId::TWO, "UNG_028t", None);
    play(&mut game, PlayerId::TWO, "UNG_028t", None);
    assert_eq!(game.state().player(PlayerId::TWO).extra_turns, 1);
    end_turn(&mut game);
    assert_eq!(game.state().active_player, PlayerId::TWO);
    end_turn(&mut game);
    assert_eq!(game.state().active_player, PlayerId::ONE);
}

#[test]
fn swamp_king_dred_force_attacks_a_minion_after_the_opponent_plays_it() {
    let (_dir, mut game) = fixture_game(
        "TEST_UNGORO_DRED_SETUP",
        "TEST_UNGORO_VICTIM",
        202,
        ["hunter", "neutral"],
    );
    let dred = board_card(&game, PlayerId::ONE, "UNG_919");
    end_turn(&mut game);
    let victim = play(&mut game, PlayerId::TWO, "TEST_UNGORO_VICTIM", None);
    assert_eq!(game.state().entity(dred).unwrap().damage, 3);
    assert_eq!(game.state().entity(victim).unwrap().damage, 9);
    assert!(game.state().log.iter().any(|event| matches!(
        event,
        GameEvent::Attack { attacker, defender, .. } if *attacker == dred && *defender == victim
    )));
}

#[test]
fn grievous_bite_and_meteor_batch_distinct_damage_with_spell_damage() {
    let (_dir, mut game) = fixture_game(
        "TEST_UNGORO_BATCH_SETUP",
        "TEST_UNGORO_VICTIM",
        203,
        ["hunter", "neutral"],
    );
    let walls = game.state().player(PlayerId::TWO).board.clone();
    assert_eq!(walls.len(), 3);
    play(&mut game, PlayerId::ONE, "UNG_910", Some(walls[1]));
    assert_eq!(game.state().entity(walls[0]).unwrap().damage, 3);
    assert_eq!(game.state().entity(walls[1]).unwrap().damage, 5);
    assert_eq!(game.state().entity(walls[2]).unwrap().damage, 3);

    play(&mut game, PlayerId::ONE, "UNG_955", Some(walls[1]));
    assert_eq!(game.state().entity(walls[0]).unwrap().damage, 9);
    assert_eq!(game.state().entity(walls[1]).unwrap().damage, 22);
    assert_eq!(game.state().entity(walls[2]).unwrap().damage, 9);
}

#[test]
fn sunkeeper_tarim_atomically_sets_other_minions_to_undamaged_three_threes() {
    let (_dir, mut game) = fixture_game(
        "TEST_UNGORO_TARIM_SETUP",
        "TEST_UNGORO_VICTIM",
        204,
        ["paladin", "neutral"],
    );
    let own = board_card(&game, PlayerId::ONE, "TEST_UNGORO_BIG");
    let enemy = board_card(&game, PlayerId::TWO, "TEST_UNGORO_BIG");
    assert_eq!(game.state().entity(own).unwrap().damage, 5);
    assert_eq!(game.state().entity(enemy).unwrap().damage, 5);
    let tarim = play(&mut game, PlayerId::ONE, "UNG_015", None);
    for entity in [own, enemy] {
        let entity = game.state().entity(entity).unwrap();
        assert_eq!((entity.attack, entity.max_health, entity.damage), (3, 3, 0));
    }
    let tarim = game.state().entity(tarim).unwrap();
    assert_eq!((tarim.attack, tarim.max_health), (3, 7));
}

fn adapt_game_with_choice(wanted: &str) -> (TempRuntimeDir, Game<LuaCardRuntime>, EntityId) {
    for seed in 220..420 {
        let (dir, mut game) = fixture_game(
            "TEST_UNGORO_ADAPT_SETUP",
            "TEST_UNGORO_VICTIM",
            seed,
            ["paladin", "neutral"],
        );
        let target = board_card(&game, PlayerId::ONE, "TEST_UNGORO_TARGET");
        play(&mut game, PlayerId::ONE, "UNG_961", Some(target));
        if choose_card(&mut game, wanted) {
            return (dir, game, target);
        }
    }
    panic!("Adapt never offered {wanted}");
}

#[test]
fn adapt_living_spores_uses_the_death_position_and_stealth_expires_next_turn() {
    let (_dir, mut game, target) = adapt_game_with_choice("UNG_999t2");
    play(&mut game, PlayerId::ONE, "TEST_UNGORO_KILL", Some(target));
    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board.len(), 2);
    assert!(
        board
            .iter()
            .all(|entity| game.state().entity(*entity).unwrap().card_id == "UNG_999t2t1")
    );

    let (_dir, mut game, target) = adapt_game_with_choice("UNG_999t10");
    assert!(game.state().entity(target).unwrap().has_keyword("stealth"));
    end_turn(&mut game);
    assert!(game.state().entity(target).unwrap().has_keyword("stealth"));
    end_turn(&mut game);
    assert!(!game.state().entity(target).unwrap().has_keyword("stealth"));
}

#[test]
fn glimmerroot_uses_the_opponents_starting_deck_after_current_cards_are_removed() {
    let (_dir, mut game) = fixture_game(
        "TEST_UNGORO_GLIMMER_SETUP",
        "CS2_029",
        205,
        ["priest", "mage"],
    );
    assert!(game.state().player(PlayerId::TWO).deck.is_empty());
    assert!(game.state().player(PlayerId::TWO).hand.is_empty());
    assert!(
        game.state()
            .player(PlayerId::TWO)
            .starting_deck
            .iter()
            .all(|card| card.as_str() == "CS2_029")
    );
    play(&mut game, PlayerId::ONE, "UNG_035", None);
    let pending = game.state().pending_input.as_ref().unwrap();
    let correct = pending
        .options
        .iter()
        .position(|option| matches!(option.value, ChoiceValue::Number(1)))
        .expect("Glimmerroot should offer one correct answer");
    assert_eq!(
        pending
            .options
            .iter()
            .filter(|option| matches!(option.value, ChoiceValue::Number(1)))
            .count(),
        1
    );
    let public_choice = game
        .state()
        .player_view(PlayerId::ONE)
        .pending_input
        .unwrap();
    assert!(
        public_choice
            .options
            .iter()
            .all(|option| matches!(option.value, ChoiceOptionValueView::Card(_)))
    );
    assert!(
        public_choice
            .options
            .iter()
            .all(|option| option.semantic_card_ids.len() == 1)
    );
    game.dispatch(PlayerCommand::Choose { index: correct })
        .unwrap();
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "CS2_029" })
    );
}

#[test]
fn obsidian_shard_counts_each_non_rogue_card_added_to_hand_once() {
    let (_dir, game) = fixture_game(
        "TEST_UNGORO_OBSIDIAN_SETUP",
        "TEST_UNGORO_VICTIM",
        206,
        ["rogue", "neutral"],
    );
    let shard = hand_card(&game, PlayerId::ONE, "UNG_061");
    let added = &game
        .state()
        .player(PlayerId::ONE)
        .cards_added_to_hand_history;
    assert_eq!(
        added
            .iter()
            .filter(|card| card.as_str() == "CS2_029")
            .count(),
        2
    );
    assert_eq!(game.state().entity(shard).unwrap().cost, 2);
}

#[test]
fn sherazin_preserves_its_original_board_position_and_revives_after_four_cards() {
    let (_dir, mut game) = fixture_game(
        "TEST_UNGORO_SHERAZIN_SETUP",
        "TEST_UNGORO_VICTIM",
        207,
        ["rogue", "neutral"],
    );
    let board = game.state().player(PlayerId::ONE).board.clone();
    assert_eq!(board.len(), 3);
    assert_eq!(
        game.state().entity(board[0]).unwrap().card_id,
        "TEST_UNGORO_LEFT"
    );
    assert_eq!(game.state().entity(board[1]).unwrap().card_id, "UNG_065t");
    assert_eq!(
        game.state().entity(board[2]).unwrap().card_id,
        "TEST_UNGORO_RIGHT"
    );
    let sherazin = board[1];
    for _ in 0..3 {
        play(&mut game, PlayerId::ONE, "TEST_UNGORO_FILLER", None);
    }
    let board = &game.state().player(PlayerId::ONE).board;
    assert_eq!(board[1], sherazin);
    assert_eq!(game.state().entity(sherazin).unwrap().card_id, "UNG_065");
}

#[test]
fn fire_plumes_heart_rewards_sulfuras_after_seven_played_taunts() {
    let (_dir, mut game) = fixture_game(
        "TEST_UNGORO_FIRE_PLUME_SETUP",
        "TEST_UNGORO_VICTIM",
        208,
        ["warrior", "neutral"],
    );
    let quest = play(&mut game, PlayerId::ONE, "UNG_934", None);
    assert_eq!(game.state().entity(quest).unwrap().zone, Zone::Secret);
    for _ in 0..7 {
        play(&mut game, PlayerId::ONE, "TEST_UNGORO_TAUNT", None);
    }
    assert_eq!(game.state().entity(quest).unwrap().zone, Zone::Graveyard);
    assert!(
        game.state()
            .player(PlayerId::ONE)
            .hand
            .iter()
            .any(|entity| { game.state().entity(*entity).unwrap().card_id == "UNG_934t1" })
    );
    advance_to_mana(&mut game, PlayerId::ONE, 3);
    play(&mut game, PlayerId::ONE, "UNG_934t1", None);
    let power = game.state().player(PlayerId::ONE).hero_power;
    assert_eq!(game.state().entity(power).unwrap().card_id, "UNG_934t2");
}
