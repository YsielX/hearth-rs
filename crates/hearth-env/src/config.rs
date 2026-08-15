use hearth_core::DEFAULT_HERO_POWER;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchConfig {
    pub decks: [Vec<String>; 2],
    #[serde(default = "default_hero_powers")]
    pub hero_powers: [String; 2],
    #[serde(default = "default_classes")]
    pub classes: [String; 2],
    #[serde(default)]
    pub unrestricted: bool,
}

pub(crate) fn default_hero_powers() -> [String; 2] {
    [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()]
}

pub(crate) fn default_classes() -> [String; 2] {
    ["mage".to_owned(), "mage".to_owned()]
}

/// Adapter policy that is deliberately separate from the rules of a match.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvConfig {
    /// Zero disables the adapter time limit.
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    /// `None` exposes the complete public history. `Some(n)` keeps the latest
    /// `n` events in each observation while aggregate counters remain complete.
    #[serde(default)]
    pub history_limit: Option<usize>,
}

impl EnvConfig {
    pub fn with_max_steps(max_steps: usize) -> Self {
        Self {
            max_steps,
            ..Self::default()
        }
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            max_steps: default_max_steps(),
            history_limit: None,
        }
    }
}

fn default_max_steps() -> usize {
    1000
}
