use hearth_core::{Locale, PlayerId, PlayerView, PublicEntity, PublicEvent};

use super::{outcome_label, pick, player_label};
use crate::{GameSession, MatchSession};

pub trait EventTextSource {
    fn locale(&self) -> Locale;
    fn card_name(&self, card_id: &str) -> String;
}

impl EventTextSource for MatchSession {
    fn locale(&self) -> Locale {
        MatchSession::locale(self)
    }

    fn card_name(&self, card_id: &str) -> String {
        MatchSession::card_name(self, card_id)
    }
}

impl EventTextSource for GameSession {
    fn locale(&self) -> Locale {
        GameSession::locale(self)
    }

    fn card_name(&self, card_id: &str) -> String {
        GameSession::card_name(self, card_id)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerTextStyle {
    #[default]
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntityTextStyle {
    #[default]
    Name,
    NameAndId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventVerbosity {
    #[default]
    Compact,
    Detailed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventTextOptions {
    pub players: PlayerTextStyle,
    pub entities: EntityTextStyle,
    pub verbosity: EventVerbosity,
}

pub fn recent_event_lines(
    session: &impl EventTextSource,
    view: &PlayerView,
    limit: usize,
) -> Vec<String> {
    let mut lines = view
        .history
        .iter()
        .rev()
        .filter_map(|record| {
            event_summary(session, view.viewer, &record.event)
                .map(|summary| format!("T{}  {summary}", record.turn))
        })
        .take(limit)
        .collect::<Vec<_>>();
    lines.reverse();
    lines
}

pub fn event_summary(
    session: &impl EventTextSource,
    viewer: PlayerId,
    event: &PublicEvent,
) -> Option<String> {
    event_summary_with_options(session, viewer, event, EventTextOptions::default())
}

pub fn event_summary_with_options(
    session: &impl EventTextSource,
    viewer: PlayerId,
    event: &PublicEvent,
    options: EventTextOptions,
) -> Option<String> {
    let locale = session.locale();
    let player = |id| match options.players {
        PlayerTextStyle::Relative => player_label(locale, id, viewer).to_owned(),
        PlayerTextStyle::Absolute => id.to_string(),
    };
    let card = |entity: &PublicEntity| {
        let name = session.card_name(&entity.card_id);
        match options.entities {
            EntityTextStyle::Name => name,
            EntityTextStyle::NameAndId => format!("{name}[{}]", entity.id),
        }
    };
    let hidden_card = |entity: &Option<PublicEntity>| {
        entity
            .as_ref()
            .map(&card)
            .unwrap_or_else(|| pick(locale, "a card", "一张卡牌", "一張卡牌").to_owned())
    };
    match event {
        PublicEvent::GameStarted => {
            Some(pick(locale, "Game started", "对局开始", "對戰開始").to_owned())
        }
        PublicEvent::TurnStarted { player: id, turn } => Some(localized3(
            locale,
            format!("{} started turn {turn}", player(*id)),
            format!("{}开始了回合 {turn}", player(*id)),
            format!("{}開始了回合 {turn}", player(*id)),
        )),
        PublicEvent::TurnEnded { player: id, .. } => Some(localized3(
            locale,
            format!("{} ended the turn", player(*id)),
            format!("{}结束了回合", player(*id)),
            format!("{}結束了回合", player(*id)),
        )),
        PublicEvent::CardDrawn {
            player: id,
            card: drawn,
            ..
        } => Some(localized3(
            locale,
            format!("{} drew {}", player(*id), hidden_card(drawn)),
            format!("{}抽到了{}", player(*id), hidden_card(drawn)),
            format!("{}抽到了{}", player(*id), hidden_card(drawn)),
        )),
        PublicEvent::CardBurned {
            player: id,
            card: burned,
            ..
        } => Some(localized3(
            locale,
            format!("{} burned {}", player(*id), card(burned)),
            format!("{}爆掉了{}", player(*id), card(burned)),
            format!("{}爆掉了{}", player(*id), card(burned)),
        )),
        PublicEvent::CardCreated {
            player: id,
            card: created,
            ..
        } => Some(localized3(
            locale,
            format!("{} created {}", player(*id), hidden_card(created)),
            format!("{}生成了{}", player(*id), hidden_card(created)),
            format!("{}生成了{}", player(*id), hidden_card(created)),
        )),
        PublicEvent::Fatigue { player: id, amount } => Some(localized3(
            locale,
            format!("{} took {amount} fatigue damage", player(*id)),
            format!("{}受到 {amount} 点疲劳伤害", player(*id)),
            format!("{}受到 {amount} 點疲勞傷害", player(*id)),
        )),
        PublicEvent::CardPlayed {
            player: id,
            card: played,
            cost,
        } => Some(localized3(
            locale,
            format!("{} played {} for {cost}", player(*id), card(played)),
            format!("{}打出{}，消耗 {cost} 点法力", player(*id), card(played)),
            format!("{}打出{}，消耗 {cost} 點法力", player(*id), card(played)),
        )),
        PublicEvent::SpellCast {
            player: id,
            spell,
            target,
            ..
        } => Some(with_target(
            localized3(
                locale,
                format!("{} cast {}", player(*id), card(spell)),
                format!("{}施放了{}", player(*id), card(spell)),
                format!("{}施放了{}", player(*id), card(spell)),
            ),
            target.as_ref().map(&card),
            locale,
        )),
        PublicEvent::SpellTargeted { spell, target, .. } => Some(localized3(
            locale,
            format!("{} targeted {}", card(spell), card(target)),
            format!("{}指定{}为目标", card(spell), card(target)),
            format!("{}指定{}為目標", card(spell), card(target)),
        )),
        PublicEvent::MinionPlayed { player: id, minion } => Some(localized3(
            locale,
            format!("{} summoned {}", player(*id), card(minion)),
            format!("{}召唤了{}", player(*id), card(minion)),
            format!("{}召喚了{}", player(*id), card(minion)),
        )),
        PublicEvent::WeaponPlayed { player: id, weapon } => Some(localized3(
            locale,
            format!("{} played {}", player(*id), card(weapon)),
            format!("{}打出了{}", player(*id), card(weapon)),
            format!("{}打出了{}", player(*id), card(weapon)),
        )),
        PublicEvent::LocationPlayed {
            player: id,
            location,
        } => Some(localized3(
            locale,
            format!("{} played {}", player(*id), card(location)),
            format!("{}打出了{}", player(*id), card(location)),
            format!("{}打出了{}", player(*id), card(location)),
        )),
        PublicEvent::CardCountered {
            player: id,
            card: countered,
        } => Some(localized3(
            locale,
            format!("{} had {} countered", player(*id), card(countered)),
            format!("{}的{}被反制", player(*id), card(countered)),
            format!("{}的{}被反制", player(*id), card(countered)),
        )),
        PublicEvent::CardDiscarded {
            player: id,
            card: discarded,
            ..
        } => Some(localized3(
            locale,
            format!("{} discarded {}", player(*id), card(discarded)),
            format!("{}弃掉了{}", player(*id), card(discarded)),
            format!("{}棄掉了{}", player(*id), card(discarded)),
        )),
        PublicEvent::CardTraded {
            player: id,
            card: traded,
        } => Some(localized3(
            locale,
            format!("{} traded {}", player(*id), hidden_card(traded)),
            format!("{}交易了{}", player(*id), hidden_card(traded)),
            format!("{}交換了{}", player(*id), hidden_card(traded)),
        )),
        PublicEvent::TradeDraw { player: id } => Some(localized3(
            locale,
            format!("{} drew after trading", player(*id)),
            format!("{}在交易后抽了一张牌", player(*id)),
            format!("{}在交換後抽了一張牌", player(*id)),
        )),
        PublicEvent::MinionSummoned { player: id, entity } => Some(localized3(
            locale,
            format!("{} summoned {}", player(*id), card(entity)),
            format!("{}召唤了{}", player(*id), card(entity)),
            format!("{}召喚了{}", player(*id), card(entity)),
        )),
        PublicEvent::Magnetized {
            attachment, target, ..
        } => Some(localized3(
            locale,
            format!("{} magnetized onto {}", card(attachment), card(target)),
            format!("{}磁力吸附到{}", card(attachment), card(target)),
            format!("{}合體到{}", card(attachment), card(target)),
        )),
        PublicEvent::WeaponEquipped { player: id, weapon } => Some(localized3(
            locale,
            format!("{} equipped {}", player(*id), card(weapon)),
            format!("{}装备了{}", player(*id), card(weapon)),
            format!("{}裝備了{}", player(*id), card(weapon)),
        )),
        PublicEvent::WeaponDestroyed { player: id, weapon } => Some(localized3(
            locale,
            format!("{} lost {}", player(*id), card(weapon)),
            format!("{}失去了{}", player(*id), card(weapon)),
            format!("{}失去了{}", player(*id), card(weapon)),
        )),
        PublicEvent::LocationUsed {
            player: id,
            location,
            target,
        } => Some(with_target(
            localized3(
                locale,
                format!("{} used {}", player(*id), card(location)),
                format!("{}使用了{}", player(*id), card(location)),
                format!("{}使用了{}", player(*id), card(location)),
            ),
            target.as_ref().map(&card),
            locale,
        )),
        PublicEvent::LocationDestroyed {
            player: id,
            location,
        } => Some(localized3(
            locale,
            format!("{} lost {}", player(*id), card(location)),
            format!("{}失去了{}", player(*id), card(location)),
            format!("{}失去了{}", player(*id), card(location)),
        )),
        PublicEvent::HeroPowerUsed {
            player: id,
            hero_power,
            target,
        } => Some(with_target(
            localized3(
                locale,
                format!("{} used {}", player(*id), card(hero_power)),
                format!("{}使用了{}", player(*id), card(hero_power)),
                format!("{}使用了{}", player(*id), card(hero_power)),
            ),
            target.as_ref().map(&card),
            locale,
        )),
        PublicEvent::HeroPowerReplaced {
            player: id, new, ..
        } => Some(localized3(
            locale,
            format!("{} gained Hero Power {}", player(*id), card(new)),
            format!("{}获得英雄技能{}", player(*id), card(new)),
            format!("{}獲得英雄能力{}", player(*id), card(new)),
        )),
        PublicEvent::HeroReplaced {
            player: id, new, ..
        } => Some(localized3(
            locale,
            format!("{} became {}", player(*id), card(new)),
            format!("{}变成了{}", player(*id), card(new)),
            format!("{}變成了{}", player(*id), card(new)),
        )),
        PublicEvent::SecretPlayed { player: id, .. } => Some(localized3(
            locale,
            format!("{} played a Secret", player(*id)),
            format!("{}打出了一个奥秘", player(*id)),
            format!("{}打出了一個秘密", player(*id)),
        )),
        PublicEvent::SecretRevealed { player: id, secret } => Some(localized3(
            locale,
            format!("{} revealed {}", player(*id), card(secret)),
            format!("{}揭示了{}", player(*id), card(secret)),
            format!("{}揭示了{}", player(*id), card(secret)),
        )),
        PublicEvent::ControllerChanged {
            entity, from, to, ..
        } => Some(localized3(
            locale,
            format!(
                "{} changed control from {} to {}",
                card(entity),
                player(*from),
                player(*to)
            ),
            format!(
                "{}的控制权从{}转移给{}",
                card(entity),
                player(*from),
                player(*to)
            ),
            format!(
                "{}的控制權從{}轉移給{}",
                card(entity),
                player(*from),
                player(*to)
            ),
        )),
        PublicEvent::Transformed {
            entity, to_card, ..
        } => Some(localized3(
            locale,
            format!(
                "{} transformed into {}",
                card(entity),
                session.card_name(to_card)
            ),
            format!("{}变形为{}", card(entity), session.card_name(to_card)),
            format!("{}變形為{}", card(entity), session.card_name(to_card)),
        )),
        PublicEvent::Attack {
            attacker, defender, ..
        } => Some(localized3(
            locale,
            format!("{} attacked {}", card(attacker), card(defender)),
            format!("{}攻击了{}", card(attacker), card(defender)),
            format!("{}攻擊了{}", card(attacker), card(defender)),
        )),
        PublicEvent::Damaged { target, amount, .. } => Some(localized3(
            locale,
            format!("{} took {amount} damage", card(target)),
            format!("{}受到 {amount} 点伤害", card(target)),
            format!("{}受到 {amount} 點傷害", card(target)),
        )),
        PublicEvent::DamagePrevented { target, reason, .. } => Some(localized3(
            locale,
            format!("{} prevented damage: {reason}", card(target)),
            format!("{}防止了伤害：{reason}", card(target)),
            format!("{}防止了傷害：{reason}", card(target)),
        )),
        PublicEvent::Healed { target, amount, .. } => Some(localized3(
            locale,
            format!("{} restored {amount} Health", card(target)),
            format!("{}恢复了 {amount} 点生命值", card(target)),
            format!("{}恢復了 {amount} 點生命值", card(target)),
        )),
        PublicEvent::ArmorGained { target, amount, .. } => Some(localized3(
            locale,
            format!("{} gained {amount} Armor", card(target)),
            format!("{}获得了 {amount} 点护甲", card(target)),
            format!("{}獲得了 {amount} 點護甲", card(target)),
        )),
        PublicEvent::PlayerResourceGained {
            player: id,
            resource,
            amount,
            ..
        } if resource == "corpses" => {
            Some(corpse_event_summary(locale, &player(*id), *amount, true))
        }
        PublicEvent::PlayerResourceSpent {
            player: id,
            resource,
            amount,
            ..
        } if resource == "corpses" => {
            Some(corpse_event_summary(locale, &player(*id), *amount, false))
        }
        PublicEvent::PlayerResourceGained {
            player: id,
            resource,
            amount,
            ..
        } => Some(format!("{} gained {amount} {resource}", player(*id))),
        PublicEvent::PlayerResourceSpent {
            player: id,
            resource,
            amount,
            ..
        } => Some(format!("{} spent {amount} {resource}", player(*id))),
        PublicEvent::KeywordDisabled {
            target, keyword, ..
        } => Some(localized3(
            locale,
            format!("{} lost {keyword}", card(target)),
            format!("{}失去了关键词 {keyword}", card(target)),
            format!("{}失去了關鍵字 {keyword}", card(target)),
        )),
        PublicEvent::Frozen { target, .. } => Some(localized3(
            locale,
            format!("{} was Frozen", card(target)),
            format!("{}被冻结", card(target)),
            format!("{}被凍結", card(target)),
        )),
        PublicEvent::EntityDied { entity, .. } => Some(localized3(
            locale,
            format!("{} died", card(entity)),
            format!("{}死亡", card(entity)),
            format!("{}死亡", card(entity)),
        )),
        PublicEvent::Conceded { player: id } => Some(localized3(
            locale,
            format!("{} conceded", player(*id)),
            format!("{}投降了", player(*id)),
            format!("{}投降了", player(*id)),
        )),
        PublicEvent::GameEnded { outcome } => Some(format!(
            "{}: {}",
            pick(locale, "Game ended", "对局结束", "對戰結束"),
            outcome_label(locale, *outcome, viewer)
        )),
        PublicEvent::ChoiceRequested {
            player: id,
            options,
            ..
        } => Some(localized3(
            locale,
            format!("{} must choose from {options} options", player(*id)),
            format!("{}必须从 {options} 个选项中选择", player(*id)),
            format!("{}必須從 {options} 個選項中選擇", player(*id)),
        )),
        PublicEvent::ChoiceMade {
            player: id, index, ..
        } => Some(match index {
            Some(index) => localized3(
                locale,
                format!("{} chose option {}", player(*id), index + 1),
                format!("{}选择了选项 {}", player(*id), index + 1),
                format!("{}選擇了選項 {}", player(*id), index + 1),
            ),
            None => localized3(
                locale,
                format!("{} made a choice", player(*id)),
                format!("{}完成了选择", player(*id)),
                format!("{}完成了選擇", player(*id)),
            ),
        }),
        PublicEvent::OverloadQueued {
            player: id, amount, ..
        } => Some(localized3(
            locale,
            format!("{} overloaded {amount}", player(*id)),
            format!("{}待过载 {amount} 个法力水晶", player(*id)),
            format!("{}待超載 {amount} 個法力水晶", player(*id)),
        )),
        PublicEvent::ManaLocked { player: id, amount } => Some(localized3(
            locale,
            format!("{} locked {amount} Mana", player(*id)),
            format!("{}锁定了 {amount} 个法力水晶", player(*id)),
            format!("{}鎖定了 {amount} 個法力水晶", player(*id)),
        )),
        PublicEvent::ManaUnlocked {
            player: id, amount, ..
        } => Some(localized3(
            locale,
            format!("{} unlocked {amount} Mana", player(*id)),
            format!("{}解锁了 {amount} 个法力水晶", player(*id)),
            format!("{}解鎖了 {amount} 個法力水晶", player(*id)),
        )),
        PublicEvent::TemporaryManaGained {
            player: id, amount, ..
        } => Some(localized3(
            locale,
            format!("{} gained {amount} temporary Mana", player(*id)),
            format!("{}获得了 {amount} 点临时法力", player(*id)),
            format!("{}獲得了 {amount} 點暫時法力", player(*id)),
        )),
        PublicEvent::ManaCrystalsGained {
            player: id, amount, ..
        } => Some(localized3(
            locale,
            format!("{} gained {amount} Mana Crystal", player(*id)),
            format!("{}获得了 {amount} 个法力水晶", player(*id)),
            format!("{}獲得了 {amount} 個法力水晶", player(*id)),
        )),
        PublicEvent::ManaCrystalsDestroyed {
            player: id, amount, ..
        } => Some(localized3(
            locale,
            format!("{} lost {amount} Mana Crystal", player(*id)),
            format!("{}失去了 {amount} 个法力水晶", player(*id)),
            format!("{}失去了 {amount} 個法力水晶", player(*id)),
        )),
        PublicEvent::OverloadCleared {
            player: id,
            pending,
            locked,
            ..
        } => (options.verbosity == EventVerbosity::Detailed).then(|| {
            localized3(
                locale,
                format!(
                    "{} cleared Overload ({locked} locked, {pending} pending)",
                    player(*id)
                ),
                format!(
                    "{}清除了过载（锁定 {locked}，待生效 {pending}）",
                    player(*id)
                ),
                format!(
                    "{}清除了超載（鎖定 {locked}，待生效 {pending}）",
                    player(*id)
                ),
            )
        }),
        PublicEvent::TemporaryManaExpired { player: id, amount } => {
            (options.verbosity == EventVerbosity::Detailed).then(|| {
                localized3(
                    locale,
                    format!("{} lost {amount} temporary Mana", player(*id)),
                    format!("{}失去了 {amount} 点临时法力", player(*id)),
                    format!("{}失去了 {amount} 點暫時法力", player(*id)),
                )
            })
        }
        PublicEvent::ManaSpent {
            player: id,
            amount,
            temporary,
            ..
        } => (options.verbosity == EventVerbosity::Detailed).then(|| {
            localized3(
                locale,
                format!(
                    "{} spent {amount} Mana ({temporary} temporary)",
                    player(*id)
                ),
                format!("{}花费了 {amount} 点法力（临时 {temporary}）", player(*id)),
                format!("{}花費了 {amount} 點法力（暫時 {temporary}）", player(*id)),
            )
        }),
        PublicEvent::ZoneChanged { entity, from, to } => {
            (options.verbosity == EventVerbosity::Detailed).then(|| {
                localized3(
                    locale,
                    format!("{} moved from {from:?} to {to:?}", card(entity)),
                    format!("{}从 {from:?} 移动到 {to:?}", card(entity)),
                    format!("{}從 {from:?} 移動到 {to:?}", card(entity)),
                )
            })
        }
    }
}

fn localized3(locale: Locale, en_us: String, zh_cn: String, zh_tw: String) -> String {
    match locale {
        Locale::EnUs => en_us,
        Locale::ZhCn => zh_cn,
        Locale::ZhTw => zh_tw,
    }
}

fn corpse_event_summary(locale: Locale, player: &str, amount: u32, gained: bool) -> String {
    if gained {
        localized3(
            locale,
            format!("{player} gained {amount} Corpses"),
            format!("{player}获得了 {amount} 份残骸"),
            format!("{player}獲得了 {amount} 具屍體"),
        )
    } else {
        localized3(
            locale,
            format!("{player} spent {amount} Corpses"),
            format!("{player}消耗了 {amount} 份残骸"),
            format!("{player}消耗了 {amount} 具屍體"),
        )
    }
}

fn with_target(mut message: String, target: Option<String>, locale: Locale) -> String {
    if let Some(target) = target {
        message.push_str(pick(locale, " targeting ", "，目标为", "，目標為"));
        message.push_str(&target);
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnglishNames;

    impl EventTextSource for EnglishNames {
        fn locale(&self) -> Locale {
            Locale::EnUs
        }

        fn card_name(&self, card_id: &str) -> String {
            format!("Card {card_id}")
        }
    }

    #[test]
    fn player_labels_are_view_relative() {
        assert_eq!(
            player_label(Locale::EnUs, PlayerId::ONE, PlayerId::ONE),
            "You"
        );
        assert_eq!(
            player_label(Locale::ZhCn, PlayerId::TWO, PlayerId::ONE),
            "对手"
        );
        assert_eq!(
            player_label(Locale::ZhTw, PlayerId::TWO, PlayerId::TWO),
            "你"
        );
    }

    #[test]
    fn targets_are_appended_only_when_present() {
        assert_eq!(with_target("Cast".to_owned(), None, Locale::EnUs), "Cast");
        assert_eq!(
            with_target("Cast".to_owned(), Some("Target".to_owned()), Locale::EnUs,),
            "Cast targeting Target"
        );
        assert_eq!(
            with_target("施放".to_owned(), Some("目标".to_owned()), Locale::ZhCn,),
            "施放，目标为目标"
        );
    }

    #[test]
    fn corpse_events_use_official_localized_resource_terms() {
        assert_eq!(
            corpse_event_summary(Locale::ZhCn, "你", 3, true),
            "你获得了 3 份残骸"
        );
        assert_eq!(
            corpse_event_summary(Locale::ZhCn, "你", 2, false),
            "你消耗了 2 份残骸"
        );
        assert_eq!(
            corpse_event_summary(Locale::ZhTw, "你", 2, false),
            "你消耗了 2 具屍體"
        );
    }

    #[test]
    fn event_text_styles_share_copy_without_leaking_hidden_cards() {
        let source = EnglishNames;
        let hidden = event_summary(
            &source,
            PlayerId::ONE,
            &PublicEvent::CardDrawn {
                player: PlayerId::TWO,
                card: None,
                source: None,
            },
        )
        .unwrap();
        assert_eq!(hidden, "Opponent drew a card");
        assert!(!hidden.contains("SECRET"));

        let entity = PublicEntity {
            id: hearth_core::EntityId(17),
            card_id: "KNOWN".to_owned(),
        };
        let detailed = event_summary_with_options(
            &source,
            PlayerId::ONE,
            &PublicEvent::ZoneChanged {
                entity,
                from: hearth_core::Zone::Hand,
                to: hearth_core::Zone::Graveyard,
            },
            EventTextOptions {
                players: PlayerTextStyle::Absolute,
                entities: EntityTextStyle::NameAndId,
                verbosity: EventVerbosity::Detailed,
            },
        )
        .unwrap();
        assert!(detailed.contains("Card KNOWN[17]"));
        assert!(
            event_summary(
                &source,
                PlayerId::ONE,
                &PublicEvent::ZoneChanged {
                    entity: PublicEntity {
                        id: hearth_core::EntityId(17),
                        card_id: "KNOWN".to_owned(),
                    },
                    from: hearth_core::Zone::Hand,
                    to: hearth_core::Zone::Graveyard,
                },
            )
            .is_none()
        );
    }
}
