use hearth_core::{GameSnapshot, PlayerId};
use serde::{Deserialize, Serialize};

use crate::BotDifficulty;

use super::MatchMode;

pub const GAME_SESSION_SNAPSHOT_VERSION: u32 = 1;

/// An authoritative local checkpoint. Frontends must treat this as private
/// because it contains both players' hidden zones; ordinary player-facing
/// export should continue to use PlayerView.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameSessionSnapshot {
    pub format_version: u32,
    pub game: GameSnapshot,
    pub human_player: PlayerId,
    pub match_mode: MatchMode,
    #[serde(default)]
    pub bot_difficulty: BotDifficulty,
    pub deck_names: [String; 2],
}
