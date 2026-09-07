use hearth_core::{LegalAction, PlayerCommand, PlayerView};

use crate::combat::{best_advantageous_trade, best_forced_trade, face_attack, lethal_attack};
use crate::controller::BotDifficulty;
use crate::evaluation;
use crate::spending::{best_location_action, spending_action};

/// Card-aware bot entry point shared by game clients and rollout workers.
pub fn choose_action_with_cards<'a>(
    difficulty: BotDifficulty,
    view: &PlayerView,
    legal_actions: &[LegalAction],
    definition: impl Fn(&str) -> Option<&'a hearth_core::CardDefinition>,
) -> Result<PlayerCommand, String> {
    let fallback = choose_action_for(difficulty, view, legal_actions)?;
    if difficulty == BotDifficulty::Easy
        || !view.mulligan_eligible.is_empty()
        || view.pending_input.is_some()
    {
        return Ok(fallback);
    }
    if let Some(lethal) = lethal_attack(view, legal_actions) {
        return Ok(lethal);
    }
    legal_actions
        .iter()
        .map(|action| {
            let score = evaluation::score(view, action, legal_actions, &definition)
                .unwrap_or_else(|| if action.command == fallback { 1.0 } else { 0.1 });
            (action, score)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(action, _)| action.command.clone())
        .ok_or_else(|| "no legal actions are available".to_owned())
}

pub fn choose_action(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Result<PlayerCommand, String> {
    choose_action_for(BotDifficulty::Normal, view, legal_actions)
}

pub fn choose_action_for(
    difficulty: BotDifficulty,
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Result<PlayerCommand, String> {
    if view.input_player != view.viewer {
        return Err(format!(
            "{} cannot choose an action for {}",
            view.viewer, view.input_player
        ));
    }
    if legal_actions.is_empty() {
        return Err("no legal actions are available".to_owned());
    }
    if !view.mulligan_eligible.is_empty() {
        if difficulty == BotDifficulty::Hard {
            let replace = view
                .mulligan_eligible
                .iter()
                .copied()
                .filter(|entity| view.entity(*entity).is_some_and(|card| card.cost >= 4))
                .collect::<Vec<_>>();
            if !replace.is_empty()
                && let Some(action) = legal_actions.iter().find(|action| {
                    matches!(
                        &action.command,
                        PlayerCommand::Mulligan { replace: candidate } if candidate == &replace
                    )
                })
            {
                return Ok(action.command.clone());
            }
        }
        return legal_actions
            .iter()
            .find(|action| {
                matches!(
                    &action.command,
                    PlayerCommand::Mulligan { replace } if replace.is_empty()
                )
            })
            .or_else(|| legal_actions.first())
            .map(|action| action.command.clone())
            .ok_or_else(|| "no Mulligan action is available".to_owned());
    }
    if view.pending_input.is_some() {
        return legal_actions
            .iter()
            .find(|action| matches!(action.command, PlayerCommand::Choose { index: 0 }))
            .or_else(|| {
                legal_actions
                    .iter()
                    .find(|action| matches!(action.command, PlayerCommand::Choose { .. }))
            })
            .map(|action| action.command.clone())
            .ok_or_else(|| "no choice action is available".to_owned());
    }

    match difficulty {
        BotDifficulty::Easy => choose_easy_action(legal_actions),
        BotDifficulty::Normal => choose_normal_action(view, legal_actions),
        BotDifficulty::Hard => choose_hard_action(view, legal_actions),
    }
}

fn choose_normal_action(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Result<PlayerCommand, String> {
    if let Some(lethal) = lethal_attack(view, legal_actions) {
        return Ok(lethal);
    }
    if let Some(spend) = spending_action(view, legal_actions) {
        return Ok(spend);
    }
    if let Some(trade) = best_advantageous_trade(view, legal_actions) {
        return Ok(trade);
    }
    if let Some(location) = best_location_action(view, legal_actions) {
        return Ok(location);
    }
    if let Some(face) = face_attack(view, legal_actions) {
        return Ok(face);
    }
    if let Some(forced) = best_forced_trade(view, legal_actions) {
        return Ok(forced);
    }
    legal_actions
        .iter()
        .find(|action| matches!(action.command, PlayerCommand::EndTurn))
        .or_else(|| {
            legal_actions.iter().find(|action| {
                !matches!(
                    action.command,
                    PlayerCommand::Concede | PlayerCommand::ConcedePlayer { .. }
                )
            })
        })
        .map(|action| action.command.clone())
        .ok_or_else(|| "only Concede is available".to_owned())
}

fn choose_easy_action(legal_actions: &[LegalAction]) -> Result<PlayerCommand, String> {
    legal_actions
        .iter()
        .find(|action| {
            !matches!(
                action.command,
                PlayerCommand::EndTurn
                    | PlayerCommand::Concede
                    | PlayerCommand::ConcedePlayer { .. }
            )
        })
        .or_else(|| {
            legal_actions
                .iter()
                .find(|action| matches!(action.command, PlayerCommand::EndTurn))
        })
        .map(|action| action.command.clone())
        .ok_or_else(|| "only Concede is available".to_owned())
}

fn choose_hard_action(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Result<PlayerCommand, String> {
    if let Some(lethal) = lethal_attack(view, legal_actions) {
        return Ok(lethal);
    }
    if let Some(trade) = best_advantageous_trade(view, legal_actions) {
        return Ok(trade);
    }
    if let Some(spend) = spending_action(view, legal_actions) {
        return Ok(spend);
    }
    if let Some(location) = best_location_action(view, legal_actions) {
        return Ok(location);
    }
    if let Some(face) = face_attack(view, legal_actions) {
        return Ok(face);
    }
    if let Some(forced) = best_forced_trade(view, legal_actions) {
        return Ok(forced);
    }
    legal_actions
        .iter()
        .find(|action| matches!(action.command, PlayerCommand::EndTurn))
        .or_else(|| {
            legal_actions.iter().find(|action| {
                !matches!(
                    action.command,
                    PlayerCommand::Concede | PlayerCommand::ConcedePlayer { .. }
                )
            })
        })
        .map(|action| action.command.clone())
        .ok_or_else(|| "only Concede is available".to_owned())
}
