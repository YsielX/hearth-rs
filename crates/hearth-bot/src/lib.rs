use std::collections::BTreeMap;

use hearth_core::{
    CardKind, EntityId, EntityView, LegalAction, PlayerCommand, PlayerController, PlayerView,
};

#[derive(Clone, Debug, Default)]
pub struct SimpleBot;

impl PlayerController for SimpleBot {
    fn choose_action(
        &mut self,
        view: &PlayerView,
        legal_actions: &[LegalAction],
    ) -> Result<PlayerCommand, String> {
        choose_action(view, legal_actions)
    }
}

pub fn choose_action(
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
            legal_actions
                .iter()
                .find(|action| !matches!(action.command, PlayerCommand::Concede))
        })
        .map(|action| action.command.clone())
        .ok_or_else(|| "only Concede is available".to_owned())
}

fn lethal_attack(view: &PlayerView, legal_actions: &[LegalAction]) -> Option<PlayerCommand> {
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

fn spending_action(view: &PlayerView, legal_actions: &[LegalAction]) -> Option<PlayerCommand> {
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

fn best_advantageous_trade(
    view: &PlayerView,
    legal_actions: &[LegalAction],
) -> Option<PlayerCommand> {
    attack_trades(view, legal_actions)
        .filter(|(_, _, advantageous)| *advantageous)
        .max_by_key(|(_, score, _)| *score)
        .map(|(command, _, _)| command)
}

fn best_forced_trade(view: &PlayerView, legal_actions: &[LegalAction]) -> Option<PlayerCommand> {
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

fn combat_value(entity: &EntityView) -> i32 {
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

fn face_attack(view: &PlayerView, legal_actions: &[LegalAction]) -> Option<PlayerCommand> {
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

fn best_location_action(view: &PlayerView, legal_actions: &[LegalAction]) -> Option<PlayerCommand> {
    legal_actions
        .iter()
        .filter(|action| matches!(action.command, PlayerCommand::UseLocation { .. }))
        .max_by_key(|action| action_preference(view, &action.command))
        .map(|action| action.command.clone())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use hearth_core::{
        CardKind, EntityId, EntityView, LegalAction, PlayerCommand, PlayerId, PlayerStateView,
        PlayerView, Zone,
    };

    use super::choose_action;

    fn entity(
        id: u64,
        kind: CardKind,
        controller: PlayerId,
        attack: i32,
        health: i32,
        cost: u8,
    ) -> EntityView {
        EntityView {
            id: EntityId(id),
            card_id: format!("CARD_{id}"),
            kind,
            owner: controller,
            controller,
            zone: match kind {
                CardKind::Hero => Zone::Hero,
                CardKind::HeroPower => Zone::HeroPower,
                _ => Zone::Board,
            },
            attack,
            max_health: health,
            damage: 0,
            armor: 0,
            cost,
            spell_damage: 0,
            exhausted: false,
            frozen: false,
            attacks_this_turn: 0,
            location_cooldown: 0,
            keywords: Vec::new(),
            silenced: false,
        }
    }

    fn view(mana: u8, extra: Vec<EntityView>) -> PlayerView {
        let mut entities = BTreeMap::new();
        for entity in [
            entity(1, CardKind::Hero, PlayerId::ONE, 0, 30, 0),
            entity(2, CardKind::Hero, PlayerId::TWO, 0, 30, 0),
            entity(3, CardKind::HeroPower, PlayerId::ONE, 0, 1, 2),
            entity(4, CardKind::HeroPower, PlayerId::TWO, 0, 1, 2),
        ]
        .into_iter()
        .chain(extra)
        {
            entities.insert(entity.id, entity);
        }
        let player = |id, hero, power| PlayerStateView {
            id,
            class: "neutral".to_owned(),
            hero,
            deck_size: 20,
            hand_size: 0,
            hand: Vec::new(),
            board: entities
                .values()
                .filter(|entity| entity.controller == id && entity.kind == CardKind::Minion)
                .map(|entity| entity.id)
                .collect(),
            weapon: None,
            hero_power: power,
            hero_power_used: false,
            secrets_count: 0,
            secrets: Vec::new(),
            public_objectives: Vec::new(),
            mana: if id == PlayerId::ONE { mana } else { 0 },
            max_mana: mana,
            temporary_mana: 0,
            overload_pending: 0,
            overloaded_mana: 0,
            fatigue: 0,
        };
        PlayerView {
            viewer: PlayerId::ONE,
            turn: 1,
            active_player: PlayerId::ONE,
            input_player: PlayerId::ONE,
            players: [
                player(PlayerId::ONE, EntityId(1), EntityId(3)),
                player(PlayerId::TWO, EntityId(2), EntityId(4)),
            ],
            entities,
            outcome: None,
            mulligan_eligible: Vec::new(),
            pending_input: None,
        }
    }

    fn legal(command: PlayerCommand, mana_cost: u8) -> LegalAction {
        LegalAction { command, mana_cost }
    }

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
}
