use std::collections::BTreeMap;
use std::path::PathBuf;

use hearth_core::CardRuntime;
use hearth_script::LuaCardRuntime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OfficialExample {
    keyword: String,
    card_id: String,
    official_url: String,
}

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn every_public_keyword_has_one_unique_implemented_official_example() {
    let data = data_path();
    let runtime = LuaCardRuntime::load_dir(&data).unwrap();
    let mut examples = BTreeMap::new();

    for file in [
        "group_a.json",
        "group_b.json",
        "group_c.json",
        "group_d_basic.json",
        "group_d_existing.json",
        "group_d_hard.json",
    ] {
        let source = std::fs::read_to_string(data.join("keyword_examples").join(file)).unwrap();
        let group: Vec<OfficialExample> = serde_json::from_str(&source).unwrap();
        for example in group {
            assert!(
                example
                    .official_url
                    .starts_with("https://hearthstone.blizzard.com/"),
                "{} must link to Blizzard's official card library",
                example.keyword
            );
            assert!(
                runtime.definition(&example.card_id).is_some(),
                "{} maps to missing card {}",
                example.keyword,
                example.card_id
            );
            assert!(
                examples.insert(example.keyword.clone(), example).is_none(),
                "duplicate official example for a keyword"
            );
        }
    }

    let mut public_keywords = runtime.keyword_ids().collect::<Vec<_>>();
    public_keywords.retain(|keyword| *keyword != "conditional_charge");
    public_keywords.retain(|keyword| *keyword != "deathrattle_repeater");
    public_keywords.retain(|keyword| *keyword != "hero_power_can_target_minions");
    public_keywords.retain(|keyword| *keyword != "dragon_consort_discount");
    public_keywords.retain(|keyword| *keyword != "hero_power_twice_per_turn");
    public_keywords.retain(|keyword| *keyword != "hero_power_unlimited");
    public_keywords.retain(|keyword| *keyword != "cannot_be_attacked_by_icehowl");
    public_keywords.retain(|keyword| *keyword != "hero_power_next_turn_surcharge");
    public_keywords.retain(|keyword| *keyword != "next_hero_power_discount");
    public_keywords.retain(|keyword| *keyword != "power_word_glory");
    public_keywords.retain(|keyword| *keyword != "battlecry_repeater");
    public_keywords.retain(|keyword| *keyword != "costs_health_instead_of_mana");
    public_keywords.retain(|keyword| *keyword != "cthun_buffs");
    public_keywords.retain(|keyword| *keyword != "cthun_taunt");
    public_keywords.retain(|keyword| *keyword != "healing_becomes_damage");
    public_keywords.retain(|keyword| *keyword != "fools_bane_unlimited_attacks");
    public_keywords.retain(|keyword| *keyword != "randomize_targets");
    public_keywords.retain(|keyword| *keyword != "cannot_be_attacked_by_fools_bane");
    public_keywords.retain(|keyword| *keyword != "raza_hero_power_zero");
    public_keywords.retain(|keyword| *keyword != "next_secret_cost_one_this_turn");
    public_keywords.retain(|keyword| *keyword != "next_spell_cost_zero_this_turn");
    public_keywords.retain(|keyword| *keyword != "next_murloc_costs_health");
    public_keywords.retain(|keyword| *keyword != "radiant_elemental_minimum_cost");
    public_keywords.retain(|keyword| *keyword != "cannot_be_attacked_by_charged_devilsaur");
    public_keywords.retain(|keyword| *keyword != "corrupting_mist_curse");
    public_keywords.retain(|keyword| *keyword != "next_spell_costs_health");
    public_keywords.retain(|keyword| *keyword != "weapon_durability_immune");
    public_keywords.retain(|keyword| *keyword != "hero_power_disabled");
    public_keywords.retain(|keyword| *keyword != "end_of_turn_repeater");
    public_keywords.retain(|keyword| *keyword != "no_corpse");
    public_keywords.retain(|keyword| *keyword != "death_knight_corpses");
    public_keywords.retain(|keyword| *keyword != "unending_plagues");
    public_keywords.retain(|keyword| *keyword != "frost_plague_surcharge");
    public_keywords.retain(|keyword| *keyword != "frozen_solid");
    public_keywords.retain(|keyword| *keyword != "primus_frost_runes");
    public_keywords.retain(|keyword| *keyword != "mograine");
    public_keywords.sort_unstable();
    assert_eq!(public_keywords.len(), 68);
    assert_eq!(
        public_keywords,
        examples.keys().map(String::as_str).collect::<Vec<_>>()
    );
}
