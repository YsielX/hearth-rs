use hearth_core::{LegalAction, PlayerCommand};

use crate::AppError;

use super::GameSession;

/// Chooses the safest deterministic command after a frontend turn clock
/// expires: end the turn when possible, otherwise take the first
/// non-concession action.
pub fn timeout_command(legal: &[LegalAction]) -> Option<PlayerCommand> {
    legal
        .iter()
        .map(|action| &action.command)
        .find(|command| matches!(command, PlayerCommand::EndTurn))
        .or_else(|| {
            legal
                .iter()
                .map(|action| &action.command)
                .find(|command| !matches!(command, PlayerCommand::Concede))
        })
        .cloned()
}

impl GameSession {
    /// Applies the deterministic timeout policy until the current turn ends or
    /// the game reaches another terminal/input boundary.
    pub fn dispatch_timeout_actions(&mut self, action_limit: usize) -> Result<(), AppError> {
        let starting_turn = self.view().turn;
        for _ in 0..action_limit {
            let legal = self.legal_actions()?;
            let command = timeout_command(&legal).ok_or(AppError::NoTimeoutAction)?;
            let ends_turn = matches!(command, PlayerCommand::EndTurn);
            self.dispatch_human_only(command)?;
            let view = self.view();
            if ends_turn || view.outcome.is_some() || view.turn != starting_turn {
                return Ok(());
            }
        }
        Err(AppError::TimeoutActionLimit(action_limit))
    }
}
