//! Local tactical estimates used exclusively by the heuristic bot.
use hearth_core::{
    CardDefinition, CardKind, EntityView, LegalAction, PlayerCommand, PlayerView, Zone,
};

use crate::effects::{Effect, effect};

fn material(entity: &EntityView) -> f64 {
    if entity.health() <= 0 {
        return 0.0;
    }
    if entity.kind == CardKind::Hero {
        return f64::from(entity.health() + entity.armor) * 0.25;
    }
    f64::from(entity.attack)
        + f64::from(entity.health()) * 0.5
        + 0.5
        + if entity.has_keyword("divine_shield") {
            1.5
        } else {
            0.0
        }
}

fn hit(entity: &mut EntityView, amount: i32, poisonous: bool) {
    if amount <= 0 || entity.has_keyword("immune") {
        return;
    }
    if entity.has_keyword("divine_shield") {
        entity.keywords.retain(|k| k != "divine_shield");
        return;
    }
    let armor = entity.armor.min(amount);
    entity.armor -= armor;
    entity.damage += amount - armor;
    if poisonous && amount > armor && entity.kind == CardKind::Minion {
        entity.damage = entity.max_health;
    }
}

/// Public material estimate; this is not a learning reward or a value target.
pub fn position_value(view: &PlayerView) -> f64 {
    let mut score = 0.0;
    for entity in view.entities.values() {
        let sign = if entity.controller == view.viewer {
            1.0
        } else {
            -1.0
        };
        let health = entity.health().max(0);
        let value = match entity.zone {
            Zone::Hero => f64::from(health + entity.armor.max(0)) * 0.35,
            Zone::Board if health > 0 => f64::from(entity.attack.max(0) + health) * 0.65,
            Zone::Weapon => f64::from(entity.attack.max(0) * health.max(1)) * 0.3,
            _ => 0.0,
        };
        score += sign * value;
    }
    score += 0.15
        * (view.player(view.viewer).hand_size as f64
            - view.player(view.viewer.opponent()).hand_size as f64);
    (score / 20.0).tanh()
}

pub(super) fn score<'a>(
    view: &PlayerView,
    action: &LegalAction,
    legal: &[LegalAction],
    definition: &impl Fn(&str) -> Option<&'a CardDefinition>,
) -> Option<f64> {
    let (card, target) = match action.command {
        PlayerCommand::EndTurn => return Some(0.0),
        PlayerCommand::Concede | PlayerCommand::ConcedePlayer { .. } => return Some(-10000.0),
        PlayerCommand::Attack { attacker, defender } => {
            let source = view.entity(attacker)?;
            let target = view.entity(defender)?;
            let mut after_source = source.clone();
            let mut after_target = target.clone();
            hit(
                &mut after_target,
                source.attack,
                source.has_keyword("poisonous"),
            );
            hit(
                &mut after_source,
                target.attack,
                target.has_keyword("poisonous"),
            );
            if after_source.kind == CardKind::Hero && after_source.health() <= 0 {
                return Some(-1000.0);
            }
            if after_target.kind == CardKind::Hero && after_target.health() <= 0 {
                return Some(1000.0);
            }
            let mut after = view.clone();
            after.entities.insert(attacker, after_source.clone());
            after.entities.insert(defender, after_target.clone());
            let gain = material(&after_source) - material(source) + material(target)
                - material(&after_target);
            return Some(gain + position_value(&after) - position_value(view));
        }
        PlayerCommand::PlayCard { card, target }
        | PlayerCommand::PlayCardAt { card, target, .. } => (view.entity(card)?, target),
        PlayerCommand::UseHeroPower { target } => {
            (view.entity(view.player(view.viewer).hero_power)?, target)
        }
        _ => return None,
    };
    let cost = if card.kind == CardKind::HeroPower {
        0.0
    } else {
        0.6
    } + f64::from(action.mana_cost) * 0.12;
    if card.kind == CardKind::Minion {
        return Some(material(card) - cost);
    }
    let kind = effect(&definition(&card.card_id)?.text)?;
    if let Effect::Draw(amount) = kind {
        let player = view.player(view.viewer);
        let consumed = usize::from(card.kind != CardKind::HeroPower);
        let space = 10_usize.saturating_sub(player.hand_size.saturating_sub(consumed));
        return Some(
            amount.max(0).min(space as i32).min(player.deck_size as i32) as f64 * 0.8 - cost,
        );
    }
    let target = view.entity(target?)?;
    let mut after = target.clone();
    let friendly = target.controller == view.viewer;
    match kind {
        Effect::Damage(amount) => {
            let spell_damage = if card.kind == CardKind::Spell {
                view.entities
                    .values()
                    .filter(|e| {
                        e.controller == view.viewer
                            && matches!(e.zone, Zone::Hero | Zone::Board | Zone::Weapon)
                    })
                    .map(|e| e.spell_damage)
                    .sum::<i32>()
            } else {
                0
            };
            hit(&mut after, amount + spell_damage, false);
        }
        Effect::Heal(amount) => after.damage = (after.damage - amount).max(0),
        Effect::Buff(attack, health, permanent) => {
            after.attack += attack;
            after.max_health += health;
            // Prefer immediate use, especially when the buff expires this turn.
            let ready = !target.frozen
                && (!target.exhausted
                    || target.has_keyword("charge")
                    || target.has_keyword("rush"))
                && target.attacks_this_turn < if target.has_keyword("windfury") { 2 } else { 1 };
            let mut gain = material(&after) - material(target);
            if !permanent && !ready {
                gain = 0.0;
            }
            if ready {
                gain += f64::from(attack) * 0.5;
            }
            if friendly
                && ready
                && legal.iter().any(|action| {
                    matches!(action.command,
                PlayerCommand::Attack { attacker, defender } if attacker == target.id
                && view.entity(defender).is_some_and(|hero| hero.kind == CardKind::Hero
                    && !hero.has_keyword("immune") && !hero.has_keyword("divine_shield")
                    && after.attack >= hero.health() + hero.armor))
                })
            {
                gain += 1000.0;
            }
            return Some(if friendly { gain - cost } else { -gain - cost });
        }
        Effect::Draw(_) => unreachable!(),
    }
    if after.kind == CardKind::Hero && after.health() <= 0 {
        return Some(if friendly { -1000.0 } else { 1000.0 });
    }
    Some((if friendly { 1.0 } else { -1.0 }) * (material(&after) - material(target)) - cost)
}
