use hearth_core::{EntityId, LegalAction, Locale, PlayerCommand, PlayerView};

use super::event_text::EventTextSource;
use super::pick;

/// Produces a localized, viewer-safe action label suitable for graphical and
/// menu-driven clients. Terminal command serialization intentionally remains
/// a frontend concern because it must stay copy/pasteable.
pub fn command_label(
    source: &impl EventTextSource,
    view: &PlayerView,
    action: &LegalAction,
) -> String {
    let locale = source.locale();
    let mana_cost = action.mana_cost;
    let entity_name = |entity: EntityId| {
        view.entity(entity)
            .map(|view| source.card_name(&view.card_id))
            .unwrap_or_else(|| format!("#{entity}"))
    };
    let target = |target: Option<EntityId>| {
        target
            .map(|entity| format!(" -> {}", entity_name(entity)))
            .unwrap_or_default()
    };
    match &action.command {
        PlayerCommand::Mulligan { replace } if replace.is_empty() => {
            pick(locale, "Keep opening hand", "保留起手牌", "保留起手牌").to_owned()
        }
        PlayerCommand::Mulligan { replace } => {
            let cards = replace
                .iter()
                .map(|entity| entity_name(*entity))
                .collect::<Vec<_>>()
                .join(", ");
            match locale {
                Locale::EnUs => format!("Replace {cards}"),
                Locale::ZhCn => format!("替换 {cards}"),
                Locale::ZhTw => format!("替換 {cards}"),
            }
        }
        PlayerCommand::PlayCard { card, target: to } => match locale {
            Locale::EnUs => format!("Play {}{}  ({mana_cost})", entity_name(*card), target(*to)),
            Locale::ZhCn | Locale::ZhTw => {
                format!(
                    "打出 {}{}  （{mana_cost}）",
                    entity_name(*card),
                    target(*to)
                )
            }
        },
        PlayerCommand::PlayCardAt {
            card,
            target: to,
            position,
        } => match locale {
            Locale::EnUs => format!(
                "Play {}{} at slot {}  ({mana_cost})",
                entity_name(*card),
                target(*to),
                position + 1
            ),
            Locale::ZhCn => format!(
                "在位置 {} 打出 {}{}  （{mana_cost}）",
                position + 1,
                entity_name(*card),
                target(*to)
            ),
            Locale::ZhTw => format!(
                "在位置 {} 打出 {}{}  （{mana_cost}）",
                position + 1,
                entity_name(*card),
                target(*to)
            ),
        },
        PlayerCommand::TradeCard { card } => match locale {
            Locale::EnUs => format!("Trade {}  (1)", entity_name(*card)),
            Locale::ZhCn => format!("交易 {}  （1）", entity_name(*card)),
            Locale::ZhTw => format!("交換 {}  （1）", entity_name(*card)),
        },
        PlayerCommand::UseCardAction {
            card,
            action,
            target: to,
        } => format!(
            "{}: {}{}  ({mana_cost})",
            entity_name(*card),
            card_action_label(locale, action),
            target(*to)
        ),
        PlayerCommand::Attack { attacker, defender } => format!(
            "{}: {} -> {}",
            pick(locale, "Attack", "攻击", "攻擊"),
            entity_name(*attacker),
            entity_name(*defender)
        ),
        PlayerCommand::UseHeroPower { target: to } => format!(
            "{}{}  ({mana_cost})",
            pick(locale, "Hero Power", "英雄技能", "英雄能力"),
            target(*to)
        ),
        PlayerCommand::UseLocation {
            location,
            target: to,
        } => format!(
            "{} {}{}",
            pick(locale, "Use", "使用", "使用"),
            entity_name(*location),
            target(*to)
        ),
        PlayerCommand::EndTurn => pick(locale, "End turn", "结束回合", "結束回合").to_owned(),
        PlayerCommand::Concede | PlayerCommand::ConcedePlayer { .. } => {
            pick(locale, "Concede", "投降", "投降").to_owned()
        }
        PlayerCommand::Choose { index } => view
            .pending_input
            .as_ref()
            .and_then(|pending| pending.options.get(*index))
            .map(|option| {
                format!(
                    "{} {}: {}",
                    pick(locale, "Choose", "选择", "選擇"),
                    index + 1,
                    option.label
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{} {}",
                    pick(locale, "Choose option", "选择选项", "選擇選項"),
                    index + 1
                )
            }),
    }
}

pub fn card_action_label(locale: Locale, action: &str) -> String {
    match action {
        "forge" => pick(locale, "Forge", "锻造", "鑄造"),
        "prepare" => pick(locale, "Prepare", "预备", "預備"),
        "launch" => pick(locale, "Launch Starship", "发射星舰", "發射星艦"),
        "titan_1" => pick(locale, "Titan Ability I", "泰坦技能一", "泰坦能力一"),
        "titan_2" => pick(locale, "Titan Ability II", "泰坦技能二", "泰坦能力二"),
        "titan_3" => pick(locale, "Titan Ability III", "泰坦技能三", "泰坦能力三"),
        other => other,
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_card_actions_have_readable_localized_labels() {
        assert_eq!(
            card_action_label(Locale::EnUs, "titan_1"),
            "Titan Ability I"
        );
        assert_eq!(card_action_label(Locale::ZhCn, "titan_2"), "泰坦技能二");
        assert_eq!(card_action_label(Locale::ZhTw, "forge"), "鑄造");
        assert_eq!(
            card_action_label(Locale::EnUs, "future_action"),
            "future_action"
        );
    }
}
