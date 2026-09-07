use std::collections::BTreeMap;

use hearth_core::{CardKind, EntityId, EntityView, LegalAction, PlayerCommand, PlayerView};

pub(super) fn lethal_attack(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Option<PlayerCommand> {
    let opponent_hero = view.player(view.viewer.opponent()).hero;
    let effective_health = view
        .entity(opponent_hero)
        .map(|hero| hero.health().saturating_add(hero.armor))?;
    let mut attackers = BTreeMap::new();
    for action in legal_actions {
        let PlayerCommand::Attack { attacker, defender } = action.command else {
            continue;
        };
        if defender != opponent_hero {
            continue;
        }
        let entity = view.entity(attacker)?;
        if entity.kind == CardKind::Minion && entity.controller == view.viewer {
            attackers.insert(attacker, entity.attack.max(0));
        }
    }
    if attackers.values().copied().sum::<i32>() < effective_health {
        return None;
    }
    attackers
        .into_iter()
        .max_by_key(|(_, attack)| *attack)
        .map(|(attacker, _)| PlayerCommand::Attack {
            attacker,
            defender: opponent_hero,
        })
}

pub(super) fn best_advantageous_trade(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Option<PlayerCommand> {
    attack_trades(view, legal_actions)
        .filter(|(_, _, advantageous)| *advantageous)
        .max_by_key(|(_, score, _)| *score)
        .map(|(command, _, _)| command)
}

pub(super) fn best_forced_trade(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Option<PlayerCommand> {
    attack_trades(view, legal_actions)
        .max_by_key(|(_, score, _)| *score)
        .map(|(command, _, _)| command)
}

fn attack_trades<'a>(
    view: &'a PlayerView,
    legal_actions: &'a [LegalAction],
) -> impl Iterator<Item = (PlayerCommand, i32, bool)> + 'a {
    legal_actions.iter().filter_map(|action| {
        let PlayerCommand::Attack { attacker, defender } = action.command else {
            return None;
        };
        let attacker_view = view.entity(attacker)?;
        let defender_view = view.entity(defender)?;
        if defender_view.kind != CardKind::Minion
            || defender_view.controller != view.viewer.opponent()
        {
            return None;
        }
        let shield_blocks_kill = defender_view.has_keyword("divine_shield");
        let poisonous_kill = attacker_view.has_keyword("poisonous") && attacker_view.attack > 0;
        let kills = !shield_blocks_kill
            && (attacker_view.attack >= defender_view.health() || poisonous_kill);
        let attacker_shield = attacker_view.has_keyword("divine_shield");
        let poisonous_return = defender_view.has_keyword("poisonous") && defender_view.attack > 0;
        let survives = attacker_shield
            || defender_view.attack <= 0
            || (!poisonous_return && attacker_view.health() > defender_view.attack);
        let loss = if survives {
            defender_view.attack.max(0)
        } else {
            combat_value(attacker_view)
        };
        let score = combat_value(defender_view) - loss;
        let advantageous = kills && (survives || score > 0);
        Some((
            PlayerCommand::Attack { attacker, defender },
            score,
            advantageous,
        ))
    })
}

pub(super) fn combat_value(entity: &EntityView) -> i32 {
    let mut value = entity.attack.max(0).saturating_mul(2) + entity.health().max(0);
    for (keyword, premium) in [
        ("taunt", 2),
        ("divine_shield", 4),
        ("poisonous", 4),
        ("lifesteal", 2),
        ("windfury", 3),
        ("mega_windfury", 5),
        ("deathrattle", 1),
    ] {
        if entity.has_keyword(keyword) {
            value += premium;
        }
    }
    value
}

pub(super) fn face_attack(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Option<PlayerCommand> {
    let opponent_hero = view.player(view.viewer.opponent()).hero;
    legal_actions
        .iter()
        .filter_map(|action| {
            let PlayerCommand::Attack { attacker, defender } = action.command else {
                return None;
            };
            (defender == opponent_hero).then_some((attacker, view.entity(attacker)?.attack))
        })
        .max_by_key(|(_, attack)| *attack)
        .map(|(attacker, _)| PlayerCommand::Attack {
            attacker,
            defender: opponent_hero,
        })
}
