use std::collections::BTreeMap;

use hearth_core::{CardKind, EntityId, LegalAction, PlayerCommand, PlayerView};

use crate::combat::combat_value;

pub(super) fn spending_action(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Option<PlayerCommand> {
    let mut groups: BTreeMap<String, Vec<&LegalAction>> = BTreeMap::new();
    for action in legal_actions {
        let Some(key) = spending_group(&action.command) else {
            continue;
        };
        groups.entry(key).or_default().push(action);
    }
    let representatives = groups
        .values()
        .filter_map(|variants| {
            variants
                .iter()
                .copied()
                .max_by_key(|action| action_preference(view, &action.command))
        })
        .collect::<Vec<_>>();
    let mana = usize::from(view.player(view.viewer).mana);
    let mut plans: Vec<Option<Vec<usize>>> = vec![None; mana + 1];
    plans[0] = Some(Vec::new());
    for (index, action) in representatives.iter().enumerate() {
        let cost = usize::from(action.mana_cost);
        if cost == 0 || cost > mana {
            continue;
        }
        for spent in (cost..=mana).rev() {
            if plans[spent].is_some() {
                continue;
            }
            let Some(previous) = plans[spent - cost].clone() else {
                continue;
            };
            let mut plan = previous;
            plan.push(index);
            plans[spent] = Some(plan);
        }
    }
    if let Some(plan) = (1..plans.len())
        .rev()
        .find_map(|spent| plans[spent].as_ref())
    {
        return plan
            .iter()
            .copied()
            .max_by_key(|index| representatives[*index].mana_cost)
            .map(|index| representatives[index].command.clone());
    }
    representatives
        .into_iter()
        .filter(|action| action.mana_cost == 0 && safe_zero_cost_action(view, &action.command))
        .max_by_key(|action| action_preference(view, &action.command))
        .map(|action| action.command.clone())
}

fn safe_zero_cost_action(view: &PlayerView, command: &PlayerCommand) -> bool {
    let target = match command {
        PlayerCommand::PlayCard { target, .. }
        | PlayerCommand::PlayCardAt { target, .. }
        | PlayerCommand::UseCardAction { target, .. }
        | PlayerCommand::UseHeroPower { target } => *target,
        _ => None,
    };
    target.is_none_or(|target| {
        view.entity(target)
            .is_some_and(|entity| entity.controller == view.viewer.opponent())
    })
}

fn spending_group(command: &PlayerCommand) -> Option<String> {
    match command {
        PlayerCommand::PlayCard { card, .. } | PlayerCommand::PlayCardAt { card, .. } => {
            Some(format!("play:{card}"))
        }
        PlayerCommand::TradeCard { card } => Some(format!("trade:{card}")),
        PlayerCommand::UseCardAction { card, action, .. } => {
            Some(format!("action:{card}:{action}"))
        }
        PlayerCommand::UseHeroPower { .. } => Some("hero_power".to_owned()),
        _ => None,
    }
}

fn action_preference(view: &PlayerView, command: &PlayerCommand) -> i32 {
    let target = match command {
        PlayerCommand::PlayCard { target, .. }
        | PlayerCommand::PlayCardAt { target, .. }
        | PlayerCommand::UseCardAction { target, .. }
        | PlayerCommand::UseHeroPower { target }
        | PlayerCommand::UseLocation { target, .. } => *target,
        _ => None,
    };
    let target_score = target.map_or(20, |target| target_preference(view, target));
    let command_score = match command {
        PlayerCommand::PlayCard { .. } => 8,
        PlayerCommand::PlayCardAt { .. } => 6,
        PlayerCommand::UseCardAction { .. } => 5,
        PlayerCommand::UseHeroPower { .. } => 4,
        PlayerCommand::TradeCard { .. } => 1,
        _ => 0,
    };
    target_score + command_score
}

fn target_preference(view: &PlayerView, target: EntityId) -> i32 {
    let Some(entity) = view.entity(target) else {
        return 0;
    };
    let opponent = view.viewer.opponent();
    if target == view.player(opponent).hero {
        40
    } else if entity.controller == opponent && entity.kind == CardKind::Minion {
        35 + combat_value(entity)
    } else if entity.controller == view.viewer && entity.damage > 0 {
        25 + entity.damage
    } else if entity.controller == view.viewer {
        10
    } else {
        5
    }
}

pub(super) fn best_location_action(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Option<PlayerCommand> {
    legal_actions
        .iter()
        .filter(|action| matches!(action.command, PlayerCommand::UseLocation { .. }))
        .max_by_key(|action| action_preference(view, &action.command))
        .map(|action| action.command.clone())
}
