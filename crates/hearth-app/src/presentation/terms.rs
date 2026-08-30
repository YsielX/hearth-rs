use crate::BotDifficulty;
use hearth_core::{CardKind, GameOutcome, Locale, MAX_GAME_TURNS, PlayerId};

use super::pick;

pub fn class_label(locale: Locale, class: &str) -> &str {
    let (zh_cn, zh_tw) = match class {
        "death_knight" => ("死亡骑士", "死亡騎士"),
        "demon_hunter" => ("恶魔猎手", "惡魔獵人"),
        "druid" => ("德鲁伊", "德魯伊"),
        "hunter" => ("猎人", "獵人"),
        "mage" => ("法师", "法師"),
        "paladin" => ("圣骑士", "聖騎士"),
        "priest" => ("牧师", "牧師"),
        "rogue" => ("潜行者", "盜賊"),
        "shaman" => ("萨满祭司", "薩滿"),
        "warlock" => ("术士", "術士"),
        "warrior" => ("战士", "戰士"),
        "neutral" => ("中立", "中立"),
        _ => return class,
    };
    pick(locale, class, zh_cn, zh_tw)
}

pub fn kind_label(locale: Locale, kind: CardKind) -> &'static str {
    match kind {
        CardKind::Minion => pick(locale, "Minion", "随从", "手下"),
        CardKind::Spell => pick(locale, "Spell", "法术", "法術"),
        CardKind::Weapon => pick(locale, "Weapon", "武器", "武器"),
        CardKind::Location => pick(locale, "Location", "地标", "地標"),
        CardKind::Hero => pick(locale, "Hero", "英雄", "英雄"),
        CardKind::HeroPower => pick(locale, "Hero Power", "英雄技能", "英雄能力"),
    }
}

pub fn bot_difficulty_label(locale: Locale, difficulty: BotDifficulty) -> &'static str {
    match difficulty {
        BotDifficulty::Easy => pick(locale, "Easy", "简单", "簡單"),
        BotDifficulty::Normal => pick(locale, "Normal", "普通", "普通"),
        BotDifficulty::Hard => pick(locale, "Hard", "困难", "困難"),
    }
}

pub fn opening_order_label(
    locale: Locale,
    starting_player: PlayerId,
    viewer: PlayerId,
) -> &'static str {
    if starting_player == viewer {
        pick(locale, "You go first", "你先手", "你先手")
    } else {
        pick(locale, "You go second", "你后手", "你後手")
    }
}

pub fn opening_mulligan_prompt(
    locale: Locale,
    starting_player: PlayerId,
    viewer: PlayerId,
) -> &'static str {
    if starting_player == viewer {
        pick(
            locale,
            "You go first — choose cards to replace",
            "你先手 — 选择要替换的卡牌",
            "你先手 — 選擇要替換的卡牌",
        )
    } else {
        pick(
            locale,
            "You go second · The Coin — choose cards to replace",
            "你后手 · 幸运币 — 选择要替换的卡牌",
            "你後手 · 幸運幣 — 選擇要替換的卡牌",
        )
    }
}

pub fn player_label(locale: Locale, player: PlayerId, viewer: PlayerId) -> &'static str {
    if player == viewer {
        pick(locale, "You", "你", "你")
    } else {
        pick(locale, "Opponent", "对手", "對手")
    }
}

pub fn outcome_label(locale: Locale, outcome: GameOutcome, viewer: PlayerId) -> String {
    match outcome {
        GameOutcome::Winner(player) if player == viewer => {
            pick(locale, "Victory", "胜利", "勝利").to_owned()
        }
        GameOutcome::Winner(_) => pick(locale, "Defeat", "失败", "失敗").to_owned(),
        GameOutcome::Draw => pick(locale, "Draw", "平局", "平手").to_owned(),
    }
}

pub fn game_over_label(
    locale: Locale,
    outcome: GameOutcome,
    viewer: PlayerId,
    turn: u32,
) -> String {
    if outcome == GameOutcome::Draw && turn >= MAX_GAME_TURNS {
        return pick(
            locale,
            "Turn limit reached — Draw",
            "达到回合上限 — 平局",
            "達到回合上限 — 平手",
        )
        .to_owned();
    }
    format!(
        "{}: {}",
        pick(locale, "Game over", "对局结束", "對戰結束"),
        outcome_label(locale, outcome, viewer)
    )
}

pub fn rarity_label(locale: Locale, rarity: &str) -> &str {
    let (zh_cn, zh_tw) = match rarity {
        "free" => ("免费", "免費"),
        "common" => ("普通", "普通"),
        "rare" => ("稀有", "精良"),
        "epic" => ("史诗", "史詩"),
        "legendary" => ("传说", "傳說"),
        _ => return rarity,
    };
    pick(locale, rarity, zh_cn, zh_tw)
}

pub fn interaction_error(locale: Locale, message: &str) -> String {
    let translated = match message {
        "that card is not in the opening hand" => ("该卡牌不在起手牌中", "該卡牌不在起手牌中"),
        "that character is not a legal target" => ("该角色不是合法目标", "該角色不是合法目標"),
        "the Hero Power cannot be used now" => ("当前无法使用英雄技能", "目前無法使用英雄能力"),
        "emotes are cooling down" => ("英雄表情正在冷却", "英雄表情正在冷卻"),
        "drop onto a different target" => ("请拖到另一个目标上", "請拖到另一個目標上"),
        "that card or character cannot act now" => {
            ("该卡牌或角色当前无法行动", "該卡牌或角色目前無法行動")
        }
        "choose a card before choosing its board position" => (
            "请先选择卡牌，再选择落场位置",
            "請先選擇卡牌，再選擇進場位置",
        ),
        "that board position is not legal for this card" => {
            ("该卡牌不能放在这个位置", "該卡牌不能放在這個位置")
        }
        "choose a playable Minion or Location before choosing a board position" => (
            "请先选择可打出的随从或地标，再选择落场位置",
            "請先選擇可打出的手下或地標，再選擇進場位置",
        ),
        _ => return message.to_owned(),
    };
    pick(locale, message, translated.0, translated.1).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_game_terms_are_localized() {
        assert_eq!(class_label(Locale::ZhCn, "shaman"), "萨满祭司");
        assert_eq!(kind_label(Locale::ZhTw, CardKind::Minion), "手下");
        assert_eq!(
            bot_difficulty_label(Locale::ZhCn, BotDifficulty::Hard),
            "困难"
        );
        assert_eq!(
            opening_order_label(Locale::ZhTw, PlayerId::TWO, PlayerId::ONE),
            "你後手"
        );
        assert!(
            opening_mulligan_prompt(Locale::EnUs, PlayerId::TWO, PlayerId::ONE)
                .contains("The Coin")
        );
        assert_eq!(
            outcome_label(
                Locale::ZhCn,
                GameOutcome::Winner(PlayerId::ONE),
                PlayerId::ONE,
            ),
            "胜利"
        );
        assert_eq!(
            game_over_label(
                Locale::ZhTw,
                GameOutcome::Draw,
                PlayerId::ONE,
                MAX_GAME_TURNS,
            ),
            "達到回合上限 — 平手"
        );
    }

    #[test]
    fn unknown_future_class_identifiers_remain_visible() {
        assert_eq!(class_label(Locale::ZhCn, "custom_class"), "custom_class");
    }

    #[test]
    fn common_interaction_errors_have_chinese_copy_and_unknown_errors_survive() {
        assert_eq!(
            interaction_error(Locale::ZhCn, "the Hero Power cannot be used now"),
            "当前无法使用英雄技能"
        );
        assert_eq!(
            interaction_error(
                Locale::ZhTw,
                "that board position is not legal for this card"
            ),
            "該卡牌不能放在這個位置"
        );
        assert_eq!(
            interaction_error(Locale::ZhTw, "future engine detail"),
            "future engine detail"
        );
    }
}
