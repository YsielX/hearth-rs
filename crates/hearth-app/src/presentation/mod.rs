pub mod command_text;
pub mod event_text;
pub mod i18n;
pub mod terms;

pub use i18n::pick;
pub use terms::{
    bot_difficulty_label, class_label, game_over_label, interaction_error, kind_label,
    opening_mulligan_prompt, opening_order_label, outcome_label, player_label, rarity_label,
};
