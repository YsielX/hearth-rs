use hearth_core::{CardKind, EntityId, PlayerCommand, PlayerId};

use super::common::{entity, legal, spell, view};
use crate::BotDifficulty;

#[test]
fn card_aware_bot_buffs_friend_and_holds_without_a_friend() {
    let view = view(
        2,
        vec![
            entity(10, CardKind::Spell, PlayerId::ONE, 0, 1, 1),
            entity(11, CardKind::Minion, PlayerId::ONE, 2, 3, 2),
            entity(20, CardKind::Minion, PlayerId::TWO, 5, 5, 5),
        ],
    );
    let buff = spell("CARD_10", "Give a minion +3/+3.");
    let play = |target| {
        legal(
            PlayerCommand::PlayCard {
                card: EntityId(10),
                target: Some(EntityId(target)),
            },
            1,
        )
    };
    let actions = [play(20), play(11), legal(PlayerCommand::EndTurn, 0)];
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == buff.id)
            .then_some(&buff))
        .unwrap(),
        actions[1].command
    );
    let actions = [play(20), legal(PlayerCommand::EndTurn, 0)];
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == buff.id)
            .then_some(&buff))
        .unwrap(),
        PlayerCommand::EndTurn
    );
}

#[test]
fn card_aware_bot_finds_buff_lethal_and_spell_lethal() {
    let mut view = view(
        2,
        vec![
            entity(10, CardKind::Spell, PlayerId::ONE, 0, 1, 1),
            entity(11, CardKind::Minion, PlayerId::ONE, 2, 3, 2),
            entity(12, CardKind::Minion, PlayerId::ONE, 8, 8, 8),
        ],
    );
    view.entities.get_mut(&EntityId(12)).unwrap().exhausted = true;
    view.entities.get_mut(&EntityId(2)).unwrap().max_health = 5;
    let buff = spell("CARD_10", "Give a minion +3/+3.");
    let play = |target| {
        legal(
            PlayerCommand::PlayCard {
                card: EntityId(10),
                target: Some(EntityId(target)),
            },
            1,
        )
    };
    let actions = [
        play(12),
        play(11),
        legal(
            PlayerCommand::Attack {
                attacker: EntityId(11),
                defender: EntityId(2),
            },
            0,
        ),
    ];
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == buff.id)
            .then_some(&buff))
        .unwrap(),
        actions[1].command
    );
    let damage = spell("CARD_10", "Deal $6 damage.");
    let actions = [play(1), play(2), legal(PlayerCommand::EndTurn, 0)];
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == damage.id)
            .then_some(&damage))
        .unwrap(),
        actions[1].command
    );
    view.entities
        .get_mut(&EntityId(2))
        .unwrap()
        .keywords
        .push("immune".into());
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == damage.id)
            .then_some(&damage))
        .unwrap(),
        PlayerCommand::EndTurn
    );
}

#[test]
fn card_aware_bot_does_not_waste_healing_or_draw_into_a_full_hand() {
    let mut view = view(2, vec![entity(10, CardKind::Spell, PlayerId::ONE, 0, 1, 1)]);
    let heal = spell("CARD_10", "Restore #8 Health.");
    let actions = [
        legal(
            PlayerCommand::PlayCard {
                card: EntityId(10),
                target: Some(EntityId(1)),
            },
            1,
        ),
        legal(PlayerCommand::EndTurn, 0),
    ];
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == heal.id)
            .then_some(&heal))
        .unwrap(),
        PlayerCommand::EndTurn
    );
    view.entities.get_mut(&EntityId(1)).unwrap().damage = 10;
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == heal.id)
            .then_some(&heal))
        .unwrap(),
        actions[0].command
    );
    let draw = spell("CARD_10", "Draw 3 cards.");
    let actions = [
        legal(
            PlayerCommand::PlayCard {
                card: EntityId(10),
                target: None,
            },
            1,
        ),
        legal(PlayerCommand::EndTurn, 0),
    ];
    view.players[0].hand_size = 10;
    // One slot freed by casting is useful, but substantially less than three draws.
    let full = crate::evaluation::score(&view, &actions[0], &actions, &|_| Some(&draw)).unwrap();
    view.players[0].hand_size = 1;
    let empty = crate::evaluation::score(&view, &actions[0], &actions, &|_| Some(&draw)).unwrap();
    assert!(empty > full);
    view.players[0].deck_size = 0;
    assert_eq!(
        crate::choose_action_with_cards(BotDifficulty::Normal, &view, &actions, |id| (id
            == draw.id)
            .then_some(&draw))
        .unwrap(),
        PlayerCommand::EndTurn
    );
}
