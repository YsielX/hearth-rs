use hearth_core::{LegalAction, PlayerCommand, PlayerController, PlayerView};
use serde::{Deserialize, Serialize};

use crate::policy::{choose_action, choose_action_for};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BotDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl std::str::FromStr for BotDifficulty {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "easy" => Ok(Self::Easy),
            "normal" => Ok(Self::Normal),
            "hard" => Ok(Self::Hard),
            _ => Err("bot difficulty expects easy, normal, or hard".to_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DifficultyBot {
    difficulty: BotDifficulty,
}

impl DifficultyBot {
    pub fn new(difficulty: BotDifficulty) -> Self {
        Self { difficulty }
    }

    pub fn difficulty(&self) -> BotDifficulty {
        self.difficulty
    }
}

impl Default for DifficultyBot {
    fn default() -> Self {
        Self::new(BotDifficulty::Normal)
    }
}

impl PlayerController for DifficultyBot {
    fn choose_action(
        &mut self,
        view: &PlayerView,
        legal_actions: &[LegalAction],
    ) -> Result<PlayerCommand, String> {
        choose_action_for(self.difficulty, view, legal_actions)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimpleBot;

impl PlayerController for SimpleBot {
    fn choose_action(
        &mut self,
        view: &PlayerView,
        legal_actions: &[LegalAction],
    ) -> Result<PlayerCommand, String> {
        choose_action(view, legal_actions)
    }
}
