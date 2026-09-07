use hearth_core::{CardKind, EntityId, PlayerCommand, PlayerId};

use super::common::{entity, legal, view};
use crate::{BotDifficulty, choose_action, choose_action_for};

#[test]
fn board_lethal_has_priority_over_spending_mana() {
    let mut view = view(
        10,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 3, 3, 3),
            entity(11, CardKind::Minion, PlayerId::ONE, 3, 3, 3),
            entity(12, CardKind::Spell, PlayerId::ONE, 0, 1, 10),
        ],
    );
    view.entities.get_mut(&EntityId(2)).unwrap().max_health = 6;
    let actions = vec![
        legal(
            PlayerCommand::Attack {
                attacker: EntityId(10),
                defender: EntityId(2),
            },
            0,
        ),
        legal(
            PlayerCommand::Attack {
                attacker: EntityId(11),
                defender: EntityId(2),
            },
            0,
        ),
        legal(
            PlayerCommand::PlayCard {
                card: EntityId(12),
                target: None,
            },
            10,
        ),
    ];
    assert!(matches!(
        choose_action(&view, &actions).unwrap(),
        PlayerCommand::Attack { defender, .. } if defender == EntityId(2)
    ));
}

#[test]
fn clean_kill_is_an_advantageous_trade_before_face() {
    let view = view(
        0,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 3, 4, 3),
            entity(20, CardKind::Minion, PlayerId::TWO, 2, 3, 2),
        ],
    );
    let trade = PlayerCommand::Attack {
        attacker: EntityId(10),
        defender: EntityId(20),
    };
    let actions = vec![
        legal(trade.clone(), 0),
        legal(
            PlayerCommand::Attack {
                attacker: EntityId(10),
                defender: EntityId(2),
            },
            0,
        ),
    ];
    assert_eq!(choose_action(&view, &actions).unwrap(), trade);
}

#[test]
fn face_is_preferred_when_no_advantageous_trade_exists() {
    let view = view(
        0,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 2, 2, 2),
            entity(20, CardKind::Minion, PlayerId::TWO, 3, 3, 3),
        ],
    );
    let face = PlayerCommand::Attack {
        attacker: EntityId(10),
        defender: EntityId(2),
    };
    let actions = vec![
        legal(
            PlayerCommand::Attack {
                attacker: EntityId(10),
                defender: EntityId(20),
            },
            0,
        ),
        legal(face.clone(), 0),
    ];
    assert_eq!(choose_action(&view, &actions).unwrap(), face);
}

#[test]
fn taunt_like_restriction_forces_the_best_available_trade() {
    let view = view(
        0,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 2, 2, 2),
            entity(20, CardKind::Minion, PlayerId::TWO, 3, 3, 3),
        ],
    );
    let forced = PlayerCommand::Attack {
        attacker: EntityId(10),
        defender: EntityId(20),
    };
    let actions = vec![legal(forced.clone(), 0), legal(PlayerCommand::EndTurn, 0)];
    assert_eq!(choose_action(&view, &actions).unwrap(), forced);
}

#[test]
fn spending_plan_prefers_an_exact_mana_combination() {
    let view = view(
        5,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 1, 1, 4),
            entity(11, CardKind::Minion, PlayerId::ONE, 1, 1, 3),
            entity(12, CardKind::Minion, PlayerId::ONE, 1, 1, 2),
        ],
    );
    let play = |card, cost| {
        legal(
            PlayerCommand::PlayCard {
                card: EntityId(card),
                target: None,
            },
            cost,
        )
    };
    let command = choose_action(
        &view,
        &[
            play(10, 4),
            play(11, 3),
            play(12, 2),
            legal(PlayerCommand::EndTurn, 0),
        ],
    )
    .unwrap();
    assert!(matches!(
        command,
        PlayerCommand::PlayCard { card, .. } if card == EntityId(11)
    ));
}

#[test]
fn easy_is_naive_while_normal_still_takes_board_lethal() {
    let mut view = view(
        1,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 5, 5, 5),
            entity(11, CardKind::Minion, PlayerId::ONE, 1, 1, 1),
        ],
    );
    view.entities.get_mut(&EntityId(2)).unwrap().max_health = 5;
    let play = PlayerCommand::PlayCard {
        card: EntityId(11),
        target: None,
    };
    let lethal = PlayerCommand::Attack {
        attacker: EntityId(10),
        defender: EntityId(2),
    };
    let actions = vec![legal(play.clone(), 1), legal(lethal.clone(), 0)];

    assert_eq!(
        choose_action_for(BotDifficulty::Easy, &view, &actions).unwrap(),
        play
    );
    assert_eq!(
        choose_action_for(BotDifficulty::Normal, &view, &actions).unwrap(),
        lethal
    );
}

#[test]
fn hard_prioritizes_a_clean_trade_before_spending_mana() {
    let view = view(
        3,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 3, 4, 3),
            entity(11, CardKind::Minion, PlayerId::ONE, 3, 3, 3),
            entity(20, CardKind::Minion, PlayerId::TWO, 2, 3, 2),
        ],
    );
    let spend = PlayerCommand::PlayCard {
        card: EntityId(11),
        target: None,
    };
    let trade = PlayerCommand::Attack {
        attacker: EntityId(10),
        defender: EntityId(20),
    };
    let actions = vec![legal(spend.clone(), 3), legal(trade.clone(), 0)];

    assert_eq!(
        choose_action_for(BotDifficulty::Normal, &view, &actions).unwrap(),
        spend
    );
    assert_eq!(
        choose_action_for(BotDifficulty::Hard, &view, &actions).unwrap(),
        trade
    );
}

#[test]
fn hard_mulligan_replaces_cards_costing_four_or_more() {
    let mut view = view(
        0,
        vec![
            entity(10, CardKind::Minion, PlayerId::ONE, 1, 1, 2),
            entity(11, CardKind::Minion, PlayerId::ONE, 4, 4, 4),
            entity(12, CardKind::Minion, PlayerId::ONE, 7, 7, 7),
        ],
    );
    view.mulligan_eligible = vec![EntityId(10), EntityId(11), EntityId(12)];
    let keep = PlayerCommand::Mulligan {
        replace: Vec::new(),
    };
    let replace = PlayerCommand::Mulligan {
        replace: vec![EntityId(11), EntityId(12)],
    };
    let actions = vec![legal(keep.clone(), 0), legal(replace.clone(), 0)];

    assert_eq!(
        choose_action_for(BotDifficulty::Normal, &view, &actions).unwrap(),
        keep
    );
    assert_eq!(
        choose_action_for(BotDifficulty::Hard, &view, &actions).unwrap(),
        replace
    );
}
