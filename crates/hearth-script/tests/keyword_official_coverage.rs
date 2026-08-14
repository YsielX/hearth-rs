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
    public_keywords.sort_unstable();
    assert_eq!(public_keywords.len(), 68);
    assert_eq!(
        public_keywords,
        examples.keys().map(String::as_str).collect::<Vec<_>>()
    );
}
