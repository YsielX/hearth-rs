//! Heuristic controllers using player-visible state and engine legal actions.

mod combat;
mod controller;
mod effects;
mod evaluation;
mod policy;
mod spending;

pub use controller::{BotDifficulty, DifficultyBot, SimpleBot};
pub use evaluation::position_value;
pub use policy::{choose_action, choose_action_for, choose_action_with_cards};

#[cfg(test)]
mod tests;
