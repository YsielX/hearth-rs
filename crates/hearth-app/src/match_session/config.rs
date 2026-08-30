use std::path::{Path, PathBuf};

use hearth_core::{Locale, PlayerId};
use serde::{Deserialize, Serialize};

use crate::BotDifficulty;

#[derive(Clone, Debug)]
pub struct MatchConfig {
    pub data_dir: PathBuf,
    pub deck_one: PathBuf,
    pub deck_two: PathBuf,
    pub seed: u64,
    pub locale: Locale,
    pub human_player: PlayerId,
    pub match_mode: MatchMode,
    pub bot_difficulty: BotDifficulty,
}

/// Controller-neutral inputs required to construct an authoritative match.
/// Frontends layer human, bot, hotseat, and diagnostic controller policies on
/// top of the resulting MatchSession.
#[derive(Clone, Debug)]
pub struct MatchSetup {
    pub data_dir: PathBuf,
    pub deck_one: PathBuf,
    pub deck_two: PathBuf,
    pub seed: u64,
    pub locale: Locale,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum MatchMode {
    #[default]
    VsBot,
    Hotseat,
}

impl MatchConfig {
    pub fn demo(workspace_root: impl AsRef<Path>) -> Self {
        let root = workspace_root.as_ref();
        Self {
            data_dir: root.join("data"),
            deck_one: root.join("decks/demo.json"),
            deck_two: root.join("decks/demo.json"),
            seed: 20260829,
            locale: Locale::EnUs,
            human_player: PlayerId::ONE,
            match_mode: MatchMode::VsBot,
            bot_difficulty: BotDifficulty::Normal,
        }
    }

    pub fn match_setup(&self) -> MatchSetup {
        MatchSetup {
            data_dir: self.data_dir.clone(),
            deck_one: self.deck_one.clone(),
            deck_two: self.deck_two.clone(),
            seed: self.seed,
            locale: self.locale,
        }
    }
}

/// Selects a fair, deterministic first player from the match seed.
///
/// The SplitMix64 finalizer avoids coupling opening order to the engine's
/// replay RNG stream, so adding this rule does not perturb card shuffles.
pub fn starting_player_for_seed(seed: u64) -> PlayerId {
    let mut mixed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^= mixed >> 31;
    if mixed & 1 == 0 {
        PlayerId::TWO
    } else {
        PlayerId::ONE
    }
}
