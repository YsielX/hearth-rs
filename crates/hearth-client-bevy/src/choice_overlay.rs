use bevy::prelude::*;
use hearth_app::GameSession;
use hearth_core::{ChoiceOptionValueView, PendingInputView, PlayerCommand};

use crate::card_preview::{InspectableCard, hide_card_preview, show_card_preview};
use crate::i18n::pick;
use crate::{
    ACTION, ACTION_HOVER, BACKGROUND, ButtonColors, CARD, CARD_SELECTED, MUTED_TEXT, TEXT,
    UiAction, handle_ui_click, shorten, text_font,
};

pub fn spawn_choice_overlay(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    pending: &PendingInputView,
) {
    let card_options = pending
        .options
        .iter()
        .filter(|option| choice_card_id(&option.value).is_some())
        .count();
    let heading = if card_options == pending.options.len() && card_options > 0 {
        pick(session.locale(), "DISCOVER", "发现", "發現")
    } else {
        pick(session.locale(), "MAKE A CHOICE", "做出选择", "做出選擇")
    };
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(14),
                padding: UiRect::all(px(22)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.035, 0.055, 0.975)),
            GlobalZIndex(120),
            Pickable::default(),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new(heading),
                text_font(34.0),
                TextColor(CARD_SELECTED),
                Pickable::IGNORE,
            ));
            overlay.spawn((
                Text::new(&pending.prompt),
                text_font(21.0),
                TextColor(TEXT),
                TextLayout::justify(Justify::Center),
                Pickable::IGNORE,
            ));
            overlay
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: px(16),
                    row_gap: px(16),
                    ..default()
                })
                .with_children(|options| {
                    for (index, option) in pending.options.iter().enumerate() {
                        spawn_choice_option(options, session, index, option);
                    }
                });
            overlay.spawn((
                Text::new(pick(
                    session.locale(),
                    "Choose one option to continue · the authoritative action list remains available on the right",
                    "选择一个选项以继续 · 右侧仍保留完整合法操作列表",
                    "選擇一個選項以繼續 · 右側仍保留完整合法操作列表",
                )),
                text_font(12.0),
                TextColor(MUTED_TEXT),
                TextLayout::justify(Justify::Center),
                Pickable::IGNORE,
            ));
        });
}

fn spawn_choice_option(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    index: usize,
    option: &hearth_core::ChoiceOptionView,
) {
    let card_id = choice_card_id(&option.value);
    let mut button = parent.spawn((
        Button,
        UiAction::Dispatch(PlayerCommand::Choose { index }),
        ButtonColors {
            normal: if card_id.is_some() { CARD } else { ACTION },
            hovered: Color::srgb(0.93, 0.82, 0.57),
            pressed: CARD_SELECTED,
        },
        Node {
            width: px(if card_id.is_some() { 190 } else { 230 }),
            min_height: px(if card_id.is_some() { 238 } else { 108 }),
            max_height: px(260),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: px(9),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::all(px(13)),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BorderColor::all(if card_id.is_some() {
            CARD_SELECTED
        } else {
            ACTION_HOVER
        }),
        BackgroundColor(if card_id.is_some() { CARD } else { ACTION }),
        Pickable::default(),
    ));
    if let Some(card_id) = card_id {
        button
            .insert(InspectableCard(card_id.to_owned()))
            .observe(show_card_preview)
            .observe(hide_card_preview);
    }
    button.observe(handle_ui_click).with_children(|choice| {
        choice.spawn((
            Text::new(format!("{}. {}", index + 1, option.label)),
            text_font(if card_id.is_some() { 18.0 } else { 17.0 }),
            TextColor(if card_id.is_some() { BACKGROUND } else { TEXT }),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
        if let Some(card_id) = card_id {
            choice.spawn((
                Text::new(format!(
                    "{}\n\n{}",
                    session.card_name(card_id),
                    shorten(&session.card_text(card_id), 160)
                )),
                text_font(13.0),
                TextColor(Color::srgb(0.13, 0.10, 0.065)),
                TextLayout::justify(Justify::Center),
                Pickable::IGNORE,
            ));
            choice.spawn((
                Text::new(pick(
                    session.locale(),
                    "Hover for full card details",
                    "悬停查看完整卡牌详情",
                    "懸停查看完整卡牌詳情",
                )),
                text_font(11.0),
                TextColor(Color::srgb(0.28, 0.22, 0.12)),
                Pickable::IGNORE,
            ));
        }
    });
}

fn choice_card_id(value: &ChoiceOptionValueView) -> Option<&str> {
    match value {
        ChoiceOptionValueView::Entity(entity) => Some(&entity.card_id),
        ChoiceOptionValueView::Card(card_id) => Some(card_id),
        ChoiceOptionValueView::Opaque => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_and_entity_choices_expose_preview_ids() {
        assert_eq!(
            choice_card_id(&ChoiceOptionValueView::Card("card".to_owned())),
            Some("card")
        );
        assert_eq!(
            choice_card_id(&ChoiceOptionValueView::Entity(hearth_core::PublicEntity {
                id: hearth_core::EntityId(9),
                card_id: "entity-card".to_owned(),
            })),
            Some("entity-card")
        );
        assert_eq!(choice_card_id(&ChoiceOptionValueView::Opaque), None);
    }
}
