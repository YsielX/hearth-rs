use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use hearth_core::{DEFAULT_HERO_POWER, EntityId, Game, GameOutcome, PlayerCommand, PlayerId, Zone};
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

#[test]
fn multiple_keening_banshees_remove_distinct_top_cards() {
    let mut game = game_with_runtime(
        LuaCardRuntime::load_dir(data_path()).unwrap(),
        repeated("CS2_120"),
        repeated("ICC_911"),
        7,
        ["mage", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::TWO, 8);
    play(&mut game, PlayerId::TWO, "ICC_911", None);
    play(&mut game, PlayerId::TWO, "ICC_911", None);
    let before = game.state().player(PlayerId::TWO).deck.len();
    play(&mut game, PlayerId::TWO, "GAME_005", None);
    assert_eq!(
        game.state().player(PlayerId::TWO).deck.len(),
        before.saturating_sub(6)
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
    let root = std::env::temp_dir().join(format!(
        "hearth-rs-icecrown-{}-{suffix}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(data_path().join("sets"), root.join("sets")).unwrap();
    std::os::unix::fs::symlink(data_path().join("keywords"), root.join("keywords")).unwrap();
    std::os::unix::fs::symlink(data_path().join("hero_powers"), root.join("hero_powers")).unwrap();
    std::os::unix::fs::symlink(data_path().join("libraries"), root.join("libraries")).unwrap();
    std::fs::write(
        root.join("test_icecrown_effects.lua"),
        r#"
local function clear_hand(ctx, player)
    for _, entity in ipairs(ctx:hand(player)) do ctx:discard(player, entity) end
end

local function clear_deck(ctx, player)
    for _, entity in ipairs(ctx:deck(player)) do ctx:move(entity, "removed") end
end

local function make_free(ctx, player, card_id)
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).card_id == card_id then
            cardlib.effects.modify(ctx, entity, { stat = "cost", operation = "set", value = 0, silenciable = false })
        end
    end
end

local function give_many(ctx, player, card_ids)
    for _, card_id in ipairs(card_ids) do ctx:create_card(player, card_id) end
end

local cards = {
    { id = "TEST_ICC_COPY_TARGET", name = "Copy Target", text = "Taunt. Deathrattle.",
      set = "TEST", type = "minion", cost = 4, attack = 4, health = 6,
      collectible = false, keywords = { "taunt", "deathrattle" },
      on_deathrattle = function() end },
    { id = "TEST_ICC_ATTACHED_DEATHRATTLE", name = "Attached Deathrattle", text = "",
      set = "TEST", type = "spell", cost = 0, collectible = false,
      on_deathrattle = function() end },
    { id = "TEST_ICC_ATTACHED_SCRIPT", name = "Attached Script", text = "",
      set = "TEST", type = "spell", cost = 0, collectible = false },
    { id = "TEST_ICC_SILENCE", name = "Test Silence", text = "Silence a minion.",
      set = "TEST", type = "spell", cost = 0, collectible = false,
      target_mode = "required", targets = function(ctx) return ctx:minions() end,
      on_play = function(ctx, self, target) ctx:silence(target) end },

    { id = "TEST_ICC_END_TURN", name = "End Turn Counter", text = "",
      set = "TEST", type = "minion", cost = 0, attack = 0, health = 10,
      collectible = false, triggers = {{
          event = "turn_ended", timing = "after", active_zones = { "board" },
          condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
          effect = function(ctx, self)
              local opponent = ctx:opponent(ctx:controller(self))
              cardlib.effects.damage_ignoring_spell_damage(ctx, ctx:player(opponent).hero, 1)
          end,
      }} },

    { id = "TEST_ICC_EVOLVE_A", name = "Evolve A", text = "", set = "TEST",
      type = "minion", cost = 97, attack = 1, health = 4, collectible = false },
    { id = "TEST_ICC_EVOLVE_B", name = "Evolve B", text = "", set = "TEST",
      type = "minion", cost = 97, attack = 2, health = 5, collectible = false },
    { id = "TEST_ICC_EVOLVED_A", name = "Evolved A", text = "", set = "TEST",
      type = "minion", cost = 99, attack = 7, health = 7, collectible = true },
    { id = "TEST_ICC_EVOLVED_B", name = "Evolved B", text = "", set = "TEST",
      type = "minion", cost = 99, attack = 8, health = 8, collectible = true },
    { id = "TEST_ICC_TRANSFORM_WATCHER", name = "Transform Watcher", text = "",
      set = "TEST", type = "minion", cost = 0, attack = 0, health = 10,
      collectible = false, triggers = {{
          event = "transformed", timing = "after", active_zones = { "board" },
          condition = function(ctx, self, event)
              return event.from_card == "TEST_ICC_EVOLVE_A"
                  or event.from_card == "TEST_ICC_EVOLVE_B"
          end,
          effect = function(ctx, self)
              for _, minion in ipairs(ctx:minions()) do
                  local id = ctx:entity(minion).card_id
                  if id == "TEST_ICC_EVOLVE_A" or id == "TEST_ICC_EVOLVE_B" then
                      ctx:set_data(self, "saw_partial_transform", 1)
                      return
                  end
              end
          end,
      }} },

    { id = "TEST_ICC_SHADOW_VICTIM", name = "Shadowmourne Victim", text = "Deathrattle.",
      set = "TEST", type = "minion", cost = 0, attack = 0, health = 4,
      collectible = false, keywords = { "deathrattle" },
      on_deathrattle = function(ctx, self)
          local player = ctx:controller(self)
          if ctx:get_player_data(player, "shadowmourne_first_deathrattle_count") ~= 0 then return end
          local count = 0
          for _, entity in ipairs(ctx:graveyard(player)) do
              if ctx:entity(entity).card_id == "TEST_ICC_SHADOW_VICTIM" then count = count + 1 end
          end
          ctx:set_player_data(player, "shadowmourne_first_deathrattle_count", count)
      end },

    { id = "TEST_ICC_ARMY_MINION_A", name = "Army A", text = "", set = "TEST",
      type = "minion", cost = 1, attack = 1, health = 1, collectible = false },
    { id = "TEST_ICC_ARMY_MINION_B", name = "Army B", text = "", set = "TEST",
      type = "minion", cost = 2, attack = 2, health = 2, collectible = false },
    { id = "TEST_ICC_ARMY_MINION_C", name = "Army C", text = "", set = "TEST",
      type = "minion", cost = 3, attack = 3, health = 3, collectible = false },
    { id = "TEST_ICC_ARMY_MINION_D", name = "Army D", text = "", set = "TEST",
      type = "minion", cost = 4, attack = 4, health = 4, collectible = false },
    { id = "TEST_ICC_ARMY_MINION_E", name = "Army E", text = "", set = "TEST",
      type = "minion", cost = 5, attack = 5, health = 5, collectible = false },
    { id = "TEST_ICC_GRIP_MINION", name = "Grip Victim", text = "", set = "TEST",
      type = "minion", cost = 1, attack = 1, health = 1, collectible = false },
    { id = "TEST_ICC_DEFILE_VICTIM", name = "Defile Victim", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 1, health = 1, collectible = false },

    { id = "TEST_ICC_REFLECTION_A", name = "Reflection A", text = "", set = "TEST",
      type = "minion", cost = 0, attack = 2, health = 2, collectible = false },
    { id = "TEST_ICC_REFLECTION_B", name = "Reflection B", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = false },

    { id = "TEST_ICC_SHADOW_ESSENCE_SETUP", name = "Shadow Essence Setup", text = "",
      set = "TEST", type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self)
          clear_hand(ctx, player); clear_deck(ctx, player)
          give_many(ctx, player, { "ICC_235", "TEST_ICC_COPY_TARGET" })
          ctx:continue_with("pack_shadow_essence_deck")
      end,
      pack_shadow_essence_deck = function(ctx, self)
          local player = ctx:controller(self)
          for _, entity in ipairs(ctx:hand(player)) do
              if ctx:entity(entity).card_id == "TEST_ICC_COPY_TARGET" then
                  ctx:move(entity, "deck_random", { player = player })
                  break
              end
          end
          ctx:continue_with("mark_shadow_essence_template")
      end,
      mark_shadow_essence_template = function(ctx, self)
          local player = ctx:controller(self)
          for _, entity in ipairs(ctx:deck(player)) do
              if ctx:entity(entity).card_id == "TEST_ICC_COPY_TARGET" then
                  ctx:buff(entity, { attack = 2, health = 3 })
                  ctx:set_data(entity, "shadow_essence_marker", 42)
                  break
              end
          end
          make_free(ctx, player, "ICC_235")
      end },

    { id = "TEST_ICC_TALDARAM_SETUP", name = "Taldaram Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self)
          clear_hand(ctx, player); clear_deck(ctx, player)
          ctx:summon(player, "TEST_ICC_COPY_TARGET")
          give_many(ctx, player, { "ICC_852", "TEST_ICC_SILENCE" })
          ctx:continue_with("finish_taldaram_setup")
      end,
      finish_taldaram_setup = function(ctx, self)
          local player = ctx:controller(self)
          for _, minion in ipairs(ctx:board(player)) do
              if ctx:entity(minion).card_id == "TEST_ICC_COPY_TARGET" then
                  ctx:buff(minion, { attack = 2, health = 3 })
                  cardlib.effects.damage_ignoring_spell_damage(ctx, minion, 2)
                  ctx:freeze(minion)
                  ctx:set_data(minion, "copied_marker", 42)
                  ctx:attach_hook(minion, "on_deathrattle", "TEST_ICC_ATTACHED_DEATHRATTLE")
                  ctx:attach_script(minion, "TEST_ICC_ATTACHED_SCRIPT")
                  break
              end
          end
          make_free(ctx, player, "ICC_852")
      end },

    { id = "TEST_ICC_DRAKKARI_SETUP", name = "Drakkari Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, opponent = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player)
          ctx:summon(player, "ICC_901"); ctx:summon(player, "ICC_901")
          ctx:summon(player, "TEST_ICC_END_TURN")
          ctx:summon(opponent, "TEST_ICC_END_TURN")
      end },

    { id = "TEST_ICC_DEFILE_SETUP", name = "Defile Setup", text = "",
      set = "TEST", type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, opponent = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player)
          ctx:summon(opponent, "BRM_019"); ctx:summon(opponent, "BRM_019")
          ctx:summon(opponent, "TEST_ICC_DEFILE_VICTIM")
          ctx:create_card(player, "ICC_041")
          ctx:continue_with("finish_defile_setup")
      end,
      finish_defile_setup = function(ctx, self)
          local player, opponent = ctx:controller(self), ctx:opponent(ctx:controller(self))
          for _, minion in ipairs(ctx:board(opponent)) do
              if ctx:entity(minion).card_id == "BRM_019" then
                  cardlib.effects.damage_ignoring_spell_damage(ctx, minion, 1)
                  break
              end
          end
          make_free(ctx, player, "ICC_041")
      end },

    { id = "TEST_ICC_THRALL_SETUP", name = "Thrall Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, opponent = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player)
          ctx:summon(player, "TEST_ICC_EVOLVE_A"); ctx:summon(player, "TEST_ICC_EVOLVE_B")
          ctx:summon(opponent, "TEST_ICC_TRANSFORM_WATCHER")
          ctx:create_card(player, "ICC_481"); ctx:continue_with("finish_thrall_setup")
      end,
      finish_thrall_setup = function(ctx, self) make_free(ctx, ctx:controller(self), "ICC_481") end },

    { id = "TEST_ICC_SHADOWMOURNE_SETUP", name = "Shadowmourne Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, opponent = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player)
          ctx:equip_weapon(player, "ICC_834w")
          ctx:summon(opponent, "TEST_ICC_SHADOW_VICTIM")
          ctx:summon(opponent, "TEST_ICC_SHADOW_VICTIM")
          ctx:summon(opponent, "TEST_ICC_SHADOW_VICTIM")
          ctx:continue_with("weaken_shadowmourne")
      end,
      weaken_shadowmourne = function(ctx, self)
          local weapon = ctx:player(ctx:controller(self)).weapon
          if weapon then ctx:lose_weapon_durability(weapon, 2) end
      end },

    { id = "TEST_ICC_ARMY_SETUP", name = "Army Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self)
          clear_hand(ctx, player); clear_deck(ctx, player)
          give_many(ctx, player, { "TEST_ICC_ARMY_MINION_A", "TEST_ICC_ARMY_MINION_B",
              "TEST_ICC_ARMY_MINION_C", "TEST_ICC_ARMY_MINION_D", "TEST_ICC_ARMY_MINION_E" })
          ctx:continue_with("pack_army_deck")
      end,
      pack_army_deck = function(ctx, self)
          local player = ctx:controller(self)
          for _, entity in ipairs(ctx:hand(player)) do
              if string.sub(ctx:entity(entity).card_id, 1, 20) == "TEST_ICC_ARMY_MINION" then
                  ctx:move(entity, "deck_random", { player = player })
              end
          end
          ctx:create_card(player, "ICC_314t2"); ctx:continue_with("free_army")
      end,
      free_army = function(ctx, self) make_free(ctx, ctx:controller(self), "ICC_314t2") end },

    { id = "TEST_ICC_GRIP_SETUP", name = "Death Grip Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player, opponent = ctx:controller(self), ctx:opponent(ctx:controller(self))
          clear_hand(ctx, player); clear_hand(ctx, opponent); clear_deck(ctx, opponent)
          ctx:create_card(opponent, "TEST_ICC_GRIP_MINION")
          ctx:continue_with("pack_grip_deck")
      end,
      pack_grip_deck = function(ctx, self)
          local player, opponent = ctx:controller(self), ctx:opponent(ctx:controller(self))
          for _, entity in ipairs(ctx:hand(opponent)) do
              if ctx:entity(entity).card_id == "TEST_ICC_GRIP_MINION" then
                  ctx:move(entity, "deck_random", { player = opponent })
              end
          end
          ctx:create_card(player, "ICC_314t4"); ctx:continue_with("free_grip")
      end,
      free_grip = function(ctx, self) make_free(ctx, ctx:controller(self), "ICC_314t4") end },

    { id = "TEST_ICC_VALEERA_SETUP", name = "Valeera Setup", text = "", set = "TEST",
      type = "spell", cost = 0, collectible = true,
      on_play = function(ctx, self)
          local player = ctx:controller(self)
          clear_hand(ctx, player)
          give_many(ctx, player, { "ICC_827", "TEST_ICC_REFLECTION_A", "TEST_ICC_REFLECTION_B" })
          ctx:continue_with("free_valeera_cards")
      end,
      free_valeera_cards = function(ctx, self)
          local player = ctx:controller(self)
          make_free(ctx, player, "ICC_827")
          make_free(ctx, player, "TEST_ICC_REFLECTION_A")
          make_free(ctx, player, "TEST_ICC_REFLECTION_B")
      end },
}

return { api_version = 1, id = "TEST_ICC_ROOT", name = "Icecrown Test Root", text = "",
    set = "TEST", type = "spell", cost = 0, collectible = false, tokens = cards }
"#,
    )
    .unwrap();
    let runtime = LuaCardRuntime::load_dir(&root).unwrap();
    (TempRuntimeDir(root), runtime)
}

fn fixture_game(setup: &str, classes: [&str; 2]) -> (TempRuntimeDir, Game<LuaCardRuntime>) {
    let (guard, runtime) = fixture_runtime();
    let mut game = game_with_runtime(runtime, repeated(setup), repeated("CS2_120"), 7, classes);
    play(&mut game, PlayerId::ONE, setup, None);
    (guard, game)
}

#[test]
fn icecrown_collectible_ids_match_the_official_135_card_set() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let actual = runtime
        .definitions()
        .filter(|definition| definition.set == "ICECROWN" && definition.collectible)
        .map(|definition| definition.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = [
        "ICC_018", "ICC_019", "ICC_021", "ICC_023", "ICC_025", "ICC_026", "ICC_027", "ICC_028",
        "ICC_029", "ICC_031", "ICC_032", "ICC_034", "ICC_038", "ICC_039", "ICC_041", "ICC_047",
        "ICC_049", "ICC_050", "ICC_051", "ICC_052", "ICC_054", "ICC_055", "ICC_056", "ICC_058",
        "ICC_062", "ICC_064", "ICC_065", "ICC_067", "ICC_068", "ICC_069", "ICC_071", "ICC_075",
        "ICC_078", "ICC_079", "ICC_081", "ICC_082", "ICC_083", "ICC_085", "ICC_086", "ICC_088",
        "ICC_089", "ICC_090", "ICC_091", "ICC_092", "ICC_093", "ICC_094", "ICC_096", "ICC_097",
        "ICC_098", "ICC_099", "ICC_200", "ICC_201", "ICC_204", "ICC_206", "ICC_207", "ICC_210",
        "ICC_212", "ICC_213", "ICC_214", "ICC_215", "ICC_218", "ICC_220", "ICC_221", "ICC_233",
        "ICC_235", "ICC_236", "ICC_238", "ICC_240", "ICC_243", "ICC_244", "ICC_245", "ICC_252",
        "ICC_257", "ICC_281", "ICC_289", "ICC_314", "ICC_405", "ICC_407", "ICC_408", "ICC_415",
        "ICC_419", "ICC_450", "ICC_466", "ICC_467", "ICC_468", "ICC_469", "ICC_481", "ICC_700",
        "ICC_701", "ICC_702", "ICC_705", "ICC_706", "ICC_801", "ICC_802", "ICC_807", "ICC_808",
        "ICC_809", "ICC_810", "ICC_811", "ICC_812", "ICC_820", "ICC_823", "ICC_825", "ICC_827",
        "ICC_828", "ICC_829", "ICC_830", "ICC_831", "ICC_832", "ICC_833", "ICC_834", "ICC_835",
        "ICC_836", "ICC_837", "ICC_838", "ICC_841", "ICC_849", "ICC_850", "ICC_851", "ICC_852",
        "ICC_853", "ICC_854", "ICC_855", "ICC_856", "ICC_858", "ICC_900", "ICC_901", "ICC_902",
        "ICC_903", "ICC_904", "ICC_905", "ICC_910", "ICC_911", "ICC_912", "ICC_913",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(actual.len(), 135);
    assert_eq!(actual, expected);
}

#[test]
fn defile_stops_after_fourteen_waves_when_grim_patrons_keep_the_loop_alive() {
    let (_guard, mut game) = fixture_game("TEST_ICC_DEFILE_SETUP", ["warlock", "warrior"]);
    let defile = play(&mut game, PlayerId::ONE, "ICC_041", None);

    assert_eq!(
        game.state().entity(defile).unwrap().script_data["wave_count"],
        14
    );
    assert!(game.state().outcome.is_none());
}

#[test]
fn shadow_essence_copies_a_deck_minions_state_and_sets_the_copy_to_five_five() {
    let (_guard, mut game) = fixture_game("TEST_ICC_SHADOW_ESSENCE_SETUP", ["priest", "mage"]);
    let template = game
        .state()
        .player(PlayerId::ONE)
        .deck
        .iter()
        .copied()
        .find(|entity| game.state().entity(*entity).unwrap().card_id == "TEST_ICC_COPY_TARGET")
        .unwrap();
    assert_eq!(game.state().entity(template).unwrap().enchantments.len(), 1);
    assert_eq!(
        game.state().entity(template).unwrap().script_data["shadow_essence_marker"],
        42
    );

    play(&mut game, PlayerId::ONE, "ICC_235", None);

    let copy = board_card(&game, PlayerId::ONE, "TEST_ICC_COPY_TARGET");
    assert_ne!(copy, template);
    assert_eq!(game.state().entity(template).unwrap().zone, Zone::Deck);
    let copy = game.state().entity(copy).unwrap();
    assert_eq!((copy.attack, copy.max_health, copy.damage), (5, 5, 0));
    assert_eq!(copy.script_data["shadow_essence_marker"], 42);
    assert_eq!(copy.enchantments.len(), 2);
}

#[test]
fn prince_taldaram_copies_full_state_then_silence_reveals_the_copied_base_body() {
    let (_guard, mut game) = fixture_game("TEST_ICC_TALDARAM_SETUP", ["neutral", "mage"]);
    let template = board_card(&game, PlayerId::ONE, "TEST_ICC_COPY_TARGET");
    let taldaram = play(&mut game, PlayerId::ONE, "ICC_852", Some(template));

    let copied = game.state().entity(taldaram).unwrap();
    assert_eq!(copied.card_id, "TEST_ICC_COPY_TARGET");
    assert_eq!((copied.attack, copied.max_health, copied.damage), (3, 3, 0));
    assert!(copied.frozen);
    assert_eq!(copied.script_data["copied_marker"], 42);
    assert_eq!(
        copied.scripts_for_hook("on_deathrattle"),
        ["TEST_ICC_ATTACHED_DEATHRATTLE"]
    );
    assert_eq!(copied.attached_cards, ["TEST_ICC_ATTACHED_SCRIPT"]);

    play(&mut game, PlayerId::ONE, "TEST_ICC_SILENCE", Some(taldaram));
    let silenced = game.state().entity(taldaram).unwrap();
    assert_eq!(silenced.card_id, "TEST_ICC_COPY_TARGET");
    assert_eq!((silenced.attack, silenced.max_health), (4, 6));
    assert!(silenced.silenced);
    assert!(silenced.scripts_for_hook("on_deathrattle").is_empty());
    assert!(silenced.attached_cards.is_empty());
}

#[test]
fn drakkari_only_repeats_friendly_end_of_turn_effects_and_does_not_stack() {
    let (_guard, mut game) = fixture_game("TEST_ICC_DRAKKARI_SETUP", ["neutral", "mage"]);
    end_turn(&mut game);
    assert_eq!(game.state().hero(PlayerId::ONE).damage, 0);
    assert_eq!(game.state().hero(PlayerId::TWO).damage, 2);

    end_turn(&mut game);
    assert_eq!(game.state().hero(PlayerId::ONE).damage, 1);
    assert_eq!(game.state().hero(PlayerId::TWO).damage, 2);
}

#[test]
fn thrall_deathseer_commits_all_random_transformations_atomically() {
    let (_guard, mut game) = fixture_game("TEST_ICC_THRALL_SETUP", ["shaman", "mage"]);
    let first = board_card(&game, PlayerId::ONE, "TEST_ICC_EVOLVE_A");
    let second = board_card(&game, PlayerId::ONE, "TEST_ICC_EVOLVE_B");
    let watcher = board_card(&game, PlayerId::TWO, "TEST_ICC_TRANSFORM_WATCHER");
    play(&mut game, PlayerId::ONE, "ICC_481", None);

    for entity in [first, second] {
        let id = game.state().entity(entity).unwrap().card_id.as_str();
        assert!(matches!(id, "TEST_ICC_EVOLVED_A" | "TEST_ICC_EVOLVED_B"));
    }
    assert_eq!(
        game.state()
            .entity(watcher)
            .unwrap()
            .script_data
            .get("saw_partial_transform"),
        None
    );
}

#[test]
fn shadowmourne_collateral_is_in_the_combat_batch_and_fires_on_final_durability() {
    let (_guard, mut game) = fixture_game("TEST_ICC_SHADOWMOURNE_SETUP", ["warrior", "mage"]);
    let weapon = game.state().player(PlayerId::ONE).weapon.unwrap();
    assert_eq!(game.state().entity(weapon).unwrap().health(), 1);
    let victims = game.state().player(PlayerId::TWO).board.clone();
    let defender = victims[1];
    let hero = game.state().player(PlayerId::ONE).hero;

    game.dispatch(PlayerCommand::Attack {
        attacker: hero,
        defender,
    })
    .unwrap();

    for victim in victims {
        assert_eq!(game.state().entity(victim).unwrap().zone, Zone::Graveyard);
    }
    assert_eq!(game.state().player(PlayerId::ONE).weapon, None);
    assert_eq!(game.state().entity(weapon).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.state().player(PlayerId::TWO).script_data["shadowmourne_first_deathrattle_count"],
        3
    );
}

#[test]
fn lich_army_and_death_grip_move_the_original_entities() {
    let (_army_guard, mut army) = fixture_game("TEST_ICC_ARMY_SETUP", ["neutral", "mage"]);
    let removed_minions = army
        .state()
        .player(PlayerId::ONE)
        .deck
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(removed_minions.len(), 5);
    play(&mut army, PlayerId::ONE, "ICC_314t2", None);
    for entity in removed_minions {
        let snapshot = army.state().entity(entity).unwrap();
        assert_eq!(snapshot.zone, Zone::Board);
        assert_eq!(snapshot.controller, PlayerId::ONE);
        assert!(army.state().player(PlayerId::ONE).board.contains(&entity));
    }

    let (_grip_guard, mut grip) = fixture_game("TEST_ICC_GRIP_SETUP", ["neutral", "mage"]);
    let stolen = grip.state().player(PlayerId::TWO).deck[0];
    play(&mut grip, PlayerId::ONE, "ICC_314t4", None);
    let snapshot = grip.state().entity(stolen).unwrap();
    assert_eq!(snapshot.zone, Zone::Hand);
    assert_eq!(snapshot.controller, PlayerId::ONE);
    assert!(grip.state().player(PlayerId::ONE).hand.contains(&stolen));
    assert!(!grip.state().player(PlayerId::TWO).deck.contains(&stolen));
}

#[test]
fn uther_four_horsemen_uses_win_game_after_all_four_are_present() {
    let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
    let mut game = game_with_runtime(
        runtime,
        repeated("ICC_829"),
        repeated("CS2_120"),
        17,
        ["paladin", "mage"],
    );
    advance_to_mana(&mut game, PlayerId::ONE, 9);
    play(&mut game, PlayerId::ONE, "ICC_829", None);
    end_turn(&mut game);

    for count in 1..=4 {
        advance_to_mana(&mut game, PlayerId::ONE, 2);
        game.dispatch(PlayerCommand::UseHeroPower { target: None })
            .unwrap();
        assert_eq!(game.state().player(PlayerId::ONE).board.len(), count);
        if count < 4 {
            assert!(game.state().outcome.is_none());
            end_turn(&mut game);
        }
    }
    assert_eq!(
        game.state().outcome,
        Some(GameOutcome::Winner(PlayerId::ONE))
    );
}

#[test]
fn valeera_shadow_reflection_keeps_transforming_the_same_temporary_entity() {
    let (_guard, mut game) = fixture_game("TEST_ICC_VALEERA_SETUP", ["rogue", "mage"]);
    play(&mut game, PlayerId::ONE, "ICC_827", None);
    let reflection = hand_card(&game, PlayerId::ONE, "ICC_827t");

    play(&mut game, PlayerId::ONE, "TEST_ICC_REFLECTION_A", None);
    let first = game.state().entity(reflection).unwrap();
    assert_eq!(first.card_id, "TEST_ICC_REFLECTION_A");
    assert!(first.has_keyword("temporary"));

    play(&mut game, PlayerId::ONE, "TEST_ICC_REFLECTION_B", None);
    let second = game.state().entity(reflection).unwrap();
    assert_eq!(second.card_id, "TEST_ICC_REFLECTION_B");
    assert!(second.has_keyword("temporary"));

    end_turn(&mut game);
    assert_eq!(game.state().entity(reflection).unwrap().zone, Zone::Removed);
}
