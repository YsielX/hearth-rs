use std::collections::BTreeSet;

use hearth_core::{EntityId, LegalAction, PlayerCommand};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionSource {
    Entity(EntityId),
    HeroPower,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardPlacement {
    Before(usize),
    End,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InteractionState {
    pub source: Option<ActionSource>,
    pub target: Option<EntityId>,
    pub placement: Option<BoardPlacement>,
    pub mulligan_replace: BTreeSet<EntityId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClickOutcome {
    Changed,
    Dispatch(PlayerCommand),
    Invalid(String),
}

impl InteractionState {
    pub fn reset_after_dispatch(&mut self) {
        self.source = None;
        self.target = None;
        self.placement = None;
        self.mulligan_replace.clear();
    }

    pub fn clear_selection(&mut self) {
        self.source = None;
        self.target = None;
        self.placement = None;
    }

    pub fn toggle_mulligan(&mut self, entity: EntityId, eligible: &[EntityId]) -> ClickOutcome {
        if !eligible.contains(&entity) {
            return ClickOutcome::Invalid("that card is not in the opening hand".to_owned());
        }
        if !self.mulligan_replace.remove(&entity) {
            self.mulligan_replace.insert(entity);
        }
        ClickOutcome::Changed
    }

    pub fn mulligan_command(&self, eligible: &[EntityId]) -> PlayerCommand {
        PlayerCommand::Mulligan {
            replace: eligible
                .iter()
                .copied()
                .filter(|entity| self.mulligan_replace.contains(entity))
                .collect(),
        }
    }
}

pub fn click_entity(
    state: &mut InteractionState,
    legal: &[LegalAction],
    entity: EntityId,
) -> ClickOutcome {
    let clicked_source = ActionSource::Entity(entity);
    let Some(source) = state.source else {
        return select_source(state, legal, clicked_source);
    };

    if source == clicked_source {
        let untargeted = matching_commands(legal, source, None, state.placement);
        return match untargeted.as_slice() {
            [command] => ClickOutcome::Dispatch((*command).clone()),
            [] => {
                state.clear_selection();
                ClickOutcome::Changed
            }
            _ => {
                state.target = None;
                ClickOutcome::Changed
            }
        };
    }

    let targeted = matching_commands(legal, source, Some(entity), state.placement);
    match targeted.as_slice() {
        [command] => ClickOutcome::Dispatch((*command).clone()),
        [] if is_legal_source(legal, clicked_source) => {
            state.source = Some(clicked_source);
            state.target = None;
            ClickOutcome::Changed
        }
        [] => ClickOutcome::Invalid("that character is not a legal target".to_owned()),
        _ => {
            state.target = Some(entity);
            ClickOutcome::Changed
        }
    }
}

pub fn activate_hero_power(state: &mut InteractionState, legal: &[LegalAction]) -> ClickOutcome {
    let source = ActionSource::HeroPower;
    let commands = legal
        .iter()
        .filter(|action| command_source(&action.command) == Some(source))
        .map(|action| &action.command)
        .collect::<Vec<_>>();
    if commands.is_empty() {
        return ClickOutcome::Invalid("the Hero Power cannot be used now".to_owned());
    }
    if let [command] = commands.as_slice()
        && command_target(command).is_none()
    {
        return ClickOutcome::Dispatch((*command).clone());
    }
    state.source = Some(source);
    state.target = None;
    state.placement = None;
    ClickOutcome::Changed
}

pub fn drag_to_entity(
    state: &mut InteractionState,
    legal: &[LegalAction],
    dragged: EntityId,
    target: EntityId,
) -> ClickOutcome {
    if dragged == target {
        return ClickOutcome::Invalid("drop onto a different target".to_owned());
    }
    state.clear_selection();
    let source = ActionSource::Entity(dragged);
    if !is_legal_source(legal, source) {
        return ClickOutcome::Invalid("that card or character cannot act now".to_owned());
    }
    state.source = Some(source);
    click_entity(state, legal, target)
}

pub fn drag_to_board(
    state: &mut InteractionState,
    legal: &[LegalAction],
    dragged: EntityId,
) -> ClickOutcome {
    state.clear_selection();
    let source = ActionSource::Entity(dragged);
    if !is_legal_source(legal, source) {
        return ClickOutcome::Invalid("that card or character cannot act now".to_owned());
    }
    state.source = Some(source);
    let untargeted = matching_commands(legal, source, None, None);
    match untargeted.as_slice() {
        [command] => ClickOutcome::Dispatch((*command).clone()),
        _ => ClickOutcome::Changed,
    }
}

pub fn choose_board_placement(
    state: &mut InteractionState,
    legal: &[LegalAction],
    placement: BoardPlacement,
) -> ClickOutcome {
    if state.source.is_none() {
        return ClickOutcome::Invalid(
            "choose a card before choosing its board position".to_owned(),
        );
    }
    state.placement = Some(placement);
    let commands = legal
        .iter()
        .filter(|action| selection_matches(&action.command, state))
        .map(|action| &action.command)
        .collect::<Vec<_>>();
    match commands.as_slice() {
        [command] => ClickOutcome::Dispatch((*command).clone()),
        [] => {
            state.placement = None;
            ClickOutcome::Invalid("that board position is not legal for this card".to_owned())
        }
        _ => ClickOutcome::Changed,
    }
}

pub fn drag_to_board_placement(
    state: &mut InteractionState,
    legal: &[LegalAction],
    dragged: EntityId,
    placement: BoardPlacement,
) -> ClickOutcome {
    state.clear_selection();
    let source = ActionSource::Entity(dragged);
    if !is_legal_source(legal, source) {
        return ClickOutcome::Invalid("that card or character cannot act now".to_owned());
    }
    state.source = Some(source);
    choose_board_placement(state, legal, placement)
}

pub fn command_source(command: &PlayerCommand) -> Option<ActionSource> {
    match command {
        PlayerCommand::PlayCard { card, .. }
        | PlayerCommand::PlayCardAt { card, .. }
        | PlayerCommand::TradeCard { card }
        | PlayerCommand::UseCardAction { card, .. } => Some(ActionSource::Entity(*card)),
        PlayerCommand::Attack { attacker, .. } => Some(ActionSource::Entity(*attacker)),
        PlayerCommand::UseHeroPower { .. } => Some(ActionSource::HeroPower),
        PlayerCommand::UseLocation { location, .. } => Some(ActionSource::Entity(*location)),
        PlayerCommand::Mulligan { .. }
        | PlayerCommand::EndTurn
        | PlayerCommand::Concede
        | PlayerCommand::ConcedePlayer { .. }
        | PlayerCommand::Choose { .. } => None,
    }
}

pub fn command_target(command: &PlayerCommand) -> Option<EntityId> {
    match command {
        PlayerCommand::PlayCard { target, .. }
        | PlayerCommand::PlayCardAt { target, .. }
        | PlayerCommand::UseCardAction { target, .. }
        | PlayerCommand::UseHeroPower { target }
        | PlayerCommand::UseLocation { target, .. } => *target,
        PlayerCommand::Attack { defender, .. } => Some(*defender),
        PlayerCommand::Mulligan { .. }
        | PlayerCommand::TradeCard { .. }
        | PlayerCommand::EndTurn
        | PlayerCommand::Concede
        | PlayerCommand::ConcedePlayer { .. }
        | PlayerCommand::Choose { .. } => None,
    }
}

pub fn command_placement(command: &PlayerCommand) -> Option<BoardPlacement> {
    match command {
        PlayerCommand::PlayCardAt { position, .. } => Some(BoardPlacement::Before(*position)),
        PlayerCommand::PlayCard { .. } => Some(BoardPlacement::End),
        _ => None,
    }
}

pub fn selection_matches(command: &PlayerCommand, state: &InteractionState) -> bool {
    state
        .source
        .is_none_or(|source| command_source(command) == Some(source))
        && state
            .target
            .is_none_or(|target| command_target(command) == Some(target))
        && state
            .placement
            .is_none_or(|placement| command_placement(command) == Some(placement))
}

pub fn is_legal_source(legal: &[LegalAction], source: ActionSource) -> bool {
    legal
        .iter()
        .any(|action| command_source(&action.command) == Some(source))
}

pub fn is_candidate_target(
    legal: &[LegalAction],
    source: Option<ActionSource>,
    entity: EntityId,
) -> bool {
    source.is_some_and(|source| {
        legal.iter().any(|action| {
            command_source(&action.command) == Some(source)
                && command_target(&action.command) == Some(entity)
        })
    })
}

fn select_source(
    state: &mut InteractionState,
    legal: &[LegalAction],
    source: ActionSource,
) -> ClickOutcome {
    if !is_legal_source(legal, source) {
        return ClickOutcome::Invalid("that card or character cannot act now".to_owned());
    }
    state.source = Some(source);
    state.target = None;
    state.placement = None;
    ClickOutcome::Changed
}

fn matching_commands(
    legal: &[LegalAction],
    source: ActionSource,
    target: Option<EntityId>,
    placement: Option<BoardPlacement>,
) -> Vec<&PlayerCommand> {
    legal
        .iter()
        .filter(|action| {
            command_source(&action.command) == Some(source)
                && command_target(&action.command) == target
                && placement
                    .is_none_or(|placement| command_placement(&action.command) == Some(placement))
        })
        .map(|action| &action.command)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legal(command: PlayerCommand) -> LegalAction {
        LegalAction {
            command,
            mana_cost: 0,
            semantic_card_id: None,
        }
    }

    #[test]
    fn source_then_unique_target_dispatches() {
        let attacker = EntityId(1);
        let defender = EntityId(2);
        let command = PlayerCommand::Attack { attacker, defender };
        let legal = vec![legal(command.clone())];
        let mut state = InteractionState::default();

        assert_eq!(
            click_entity(&mut state, &legal, attacker),
            ClickOutcome::Changed
        );
        assert_eq!(
            click_entity(&mut state, &legal, defender),
            ClickOutcome::Dispatch(command)
        );
    }

    #[test]
    fn ambiguous_target_keeps_a_filtered_selection() {
        let card = EntityId(1);
        let target = EntityId(2);
        let legal = vec![
            legal(PlayerCommand::PlayCardAt {
                card,
                target: Some(target),
                position: 0,
            }),
            legal(PlayerCommand::PlayCardAt {
                card,
                target: Some(target),
                position: 1,
            }),
        ];
        let mut state = InteractionState::default();

        click_entity(&mut state, &legal, card);
        assert_eq!(
            click_entity(&mut state, &legal, target),
            ClickOutcome::Changed
        );
        assert_eq!(state.target, Some(target));
        assert!(
            legal
                .iter()
                .all(|action| selection_matches(&action.command, &state))
        );
    }

    #[test]
    fn untargeted_hero_power_dispatches_from_its_button() {
        let command = PlayerCommand::UseHeroPower { target: None };
        let legal = vec![legal(command.clone())];
        let mut state = InteractionState::default();
        assert_eq!(
            activate_hero_power(&mut state, &legal),
            ClickOutcome::Dispatch(command)
        );
    }

    #[test]
    fn dragging_an_attacker_to_a_defender_dispatches() {
        let attacker = EntityId(7);
        let defender = EntityId(9);
        let command = PlayerCommand::Attack { attacker, defender };
        let legal = vec![legal(command.clone())];
        let mut state = InteractionState::default();
        assert_eq!(
            drag_to_entity(&mut state, &legal, attacker, defender),
            ClickOutcome::Dispatch(command)
        );
    }

    #[test]
    fn dragging_an_untargeted_spell_to_the_board_dispatches() {
        let card = EntityId(7);
        let command = PlayerCommand::PlayCard { card, target: None };
        let legal = vec![legal(command.clone())];
        let mut state = InteractionState::default();
        assert_eq!(
            drag_to_board(&mut state, &legal, card),
            ClickOutcome::Dispatch(command)
        );
    }

    #[test]
    fn dragging_a_targeted_spell_to_empty_board_selects_it() {
        let card = EntityId(7);
        let target = EntityId(9);
        let legal = vec![legal(PlayerCommand::PlayCard {
            card,
            target: Some(target),
        })];
        let mut state = InteractionState::default();
        assert_eq!(
            drag_to_board(&mut state, &legal, card),
            ClickOutcome::Changed
        );
        assert_eq!(state.source, Some(ActionSource::Entity(card)));
        assert_eq!(state.target, None);
    }

    #[test]
    fn choosing_the_end_slot_dispatches_an_untargeted_minion() {
        let card = EntityId(7);
        let command = PlayerCommand::PlayCard { card, target: None };
        let legal = vec![legal(command.clone())];
        let mut state = InteractionState {
            source: Some(ActionSource::Entity(card)),
            ..Default::default()
        };
        assert_eq!(
            choose_board_placement(&mut state, &legal, BoardPlacement::End),
            ClickOutcome::Dispatch(command)
        );
    }

    #[test]
    fn placement_then_target_dispatches_the_exact_battlecry_command() {
        let card = EntityId(7);
        let first_target = EntityId(9);
        let second_target = EntityId(10);
        let expected = PlayerCommand::PlayCardAt {
            card,
            target: Some(first_target),
            position: 0,
        };
        let legal = vec![
            legal(expected.clone()),
            legal(PlayerCommand::PlayCardAt {
                card,
                target: Some(second_target),
                position: 0,
            }),
            legal(PlayerCommand::PlayCardAt {
                card,
                target: Some(first_target),
                position: 1,
            }),
        ];
        let mut state = InteractionState {
            source: Some(ActionSource::Entity(card)),
            ..Default::default()
        };

        assert_eq!(
            choose_board_placement(&mut state, &legal, BoardPlacement::Before(0)),
            ClickOutcome::Changed
        );
        assert_eq!(state.placement, Some(BoardPlacement::Before(0)));
        assert_eq!(
            click_entity(&mut state, &legal, first_target),
            ClickOutcome::Dispatch(expected)
        );
    }

    #[test]
    fn dragging_to_an_insertion_slot_dispatches_that_position() {
        let card = EntityId(7);
        let command = PlayerCommand::PlayCardAt {
            card,
            target: None,
            position: 1,
        };
        let legal = vec![legal(command.clone())];
        let mut state = InteractionState::default();
        assert_eq!(
            drag_to_board_placement(&mut state, &legal, card, BoardPlacement::Before(1)),
            ClickOutcome::Dispatch(command)
        );
    }

    #[test]
    fn mulligan_selection_is_sorted_and_toggleable() {
        let first = EntityId(3);
        let second = EntityId(1);
        let eligible = [first, second];
        let mut state = InteractionState::default();
        state.toggle_mulligan(first, &eligible);
        state.toggle_mulligan(second, &eligible);
        assert_eq!(
            state.mulligan_command(&eligible),
            PlayerCommand::Mulligan {
                replace: vec![first, second]
            }
        );
        state.toggle_mulligan(first, &eligible);
        assert_eq!(
            state.mulligan_command(&eligible),
            PlayerCommand::Mulligan {
                replace: vec![second]
            }
        );
    }
}
