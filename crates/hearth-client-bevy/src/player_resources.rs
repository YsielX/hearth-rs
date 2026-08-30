use bevy::prelude::*;
use hearth_core::{Locale, PlayerStateView};

use crate::i18n::pick;
use crate::{CARD_SELECTED, TEXT, text_font};

const MANA_COLOR: Color = Color::srgb(0.08, 0.32, 0.58);
const DECK_COLOR: Color = Color::srgb(0.17, 0.20, 0.27);
const CORPSE_COLOR: Color = Color::srgb(0.18, 0.34, 0.26);

pub fn spawn_player_resources(
    parent: &mut ChildSpawnerCommands,
    state: &PlayerStateView,
    locale: Locale,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        })
        .with_children(|resources| {
            spawn_resource_badge(resources, &deck_label(state, locale), DECK_COLOR, 82.0);
            if state.class.eq_ignore_ascii_case("death_knight") || state.corpses > 0 {
                spawn_resource_badge(resources, &corpse_label(state, locale), CORPSE_COLOR, 72.0);
            }
            spawn_resource_badge(resources, &mana_label(state, locale), MANA_COLOR, 118.0);
        });
}

fn corpse_label(state: &PlayerStateView, locale: Locale) -> String {
    format!(
        "{}\n{}",
        pick(locale, "CORPSES", "残骸", "屍體"),
        state.corpses
    )
}

fn spawn_resource_badge(parent: &mut ChildSpawnerCommands, label: &str, color: Color, width: f32) {
    parent
        .spawn((
            Node {
                width: px(width),
                min_height: px(44),
                padding: UiRect::all(px(4)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(9)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(CARD_SELECTED),
            BackgroundColor(color),
            Pickable::IGNORE,
        ))
        .with_child((
            Text::new(label),
            text_font(10.0),
            TextColor(TEXT),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
}

fn mana_label(state: &PlayerStateView, locale: Locale) -> String {
    let mut details = Vec::new();
    if state.temporary_mana > 0 {
        details.push(format!(
            "{} {}",
            pick(locale, "TEMP", "临时", "暫時"),
            state.temporary_mana
        ));
    }
    if state.overloaded_mana > 0 {
        details.push(format!(
            "{} {}",
            pick(locale, "LOCKED", "锁定", "鎖定"),
            state.overloaded_mana
        ));
    }
    if state.overload_pending > 0 {
        details.push(format!(
            "{} {}",
            pick(locale, "NEXT", "待过载", "待超載"),
            state.overload_pending
        ));
    }
    let details = (!details.is_empty()).then(|| format!("\n{}", details.join(" · ")));
    format!(
        "{}  {}/{}{}",
        pick(locale, "MANA", "法力", "法力"),
        state.mana,
        state.max_mana,
        details.unwrap_or_default()
    )
}

fn deck_label(state: &PlayerStateView, locale: Locale) -> String {
    if state.deck_size == 0 {
        format!(
            "{}\n{} {}",
            pick(locale, "DECK EMPTY", "牌库已空", "牌庫已空"),
            pick(locale, "FATIGUE", "疲劳", "疲勞"),
            state.fatigue.saturating_add(1)
        )
    } else if state.fatigue > 0 {
        format!(
            "{}  {}\n{} {}",
            pick(locale, "DECK", "牌库", "牌庫"),
            state.deck_size,
            pick(locale, "FATIGUE", "疲劳", "疲勞"),
            state.fatigue
        )
    } else {
        format!(
            "{}\n{}",
            pick(locale, "DECK", "牌库", "牌庫"),
            state.deck_size
        )
    }
}

#[cfg(test)]
mod tests {
    use hearth_core::{EntityId, PlayerId};

    use super::*;

    fn player() -> PlayerStateView {
        PlayerStateView {
            id: PlayerId::ONE,
            class: "shaman".to_owned(),
            hero: EntityId(1),
            deck_size: 20,
            hand_size: 5,
            hand: Vec::new(),
            sideboards: Default::default(),
            board: Vec::new(),
            weapon: None,
            hero_power: EntityId(2),
            hero_power_used: false,
            hero_power_uses_this_turn: 0,
            secrets_count: 0,
            secrets: Vec::new(),
            public_objectives: Vec::new(),
            mana: 3,
            max_mana: 5,
            temporary_mana: 0,
            corpses: 0,
            corpses_spent: 0,
            public_keywords: Vec::new(),
            overload_pending: 0,
            overloaded_mana: 0,
            fatigue: 0,
            cards_played_this_turn: 0,
        }
    }

    #[test]
    fn mana_label_distinguishes_temporary_locked_and_pending_resources() {
        let mut state = player();
        state.temporary_mana = 1;
        state.overloaded_mana = 2;
        state.overload_pending = 3;
        assert_eq!(
            mana_label(&state, Locale::EnUs),
            "MANA  3/5\nTEMP 1 · LOCKED 2 · NEXT 3"
        );
        assert_eq!(
            mana_label(&state, Locale::ZhCn),
            "法力  3/5\n临时 1 · 锁定 2 · 待过载 3"
        );
    }

    #[test]
    fn empty_deck_reports_the_next_fatigue_damage() {
        let mut state = player();
        state.deck_size = 0;
        state.fatigue = 4;
        assert_eq!(deck_label(&state, Locale::EnUs), "DECK EMPTY\nFATIGUE 5");
    }

    #[test]
    fn corpse_label_is_localized() {
        let mut state = player();
        state.class = "death_knight".to_owned();
        state.corpses = 12;
        assert_eq!(corpse_label(&state, Locale::EnUs), "CORPSES\n12");
        assert_eq!(corpse_label(&state, Locale::ZhCn), "残骸\n12");
        assert_eq!(corpse_label(&state, Locale::ZhTw), "屍體\n12");
    }

    #[test]
    fn a_refilled_deck_preserves_already_taken_fatigue_information() {
        let mut state = player();
        state.deck_size = 3;
        state.fatigue = 2;
        assert_eq!(deck_label(&state, Locale::EnUs), "DECK  3\nFATIGUE 2");
    }
}
