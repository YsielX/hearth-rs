use bevy::{
    input_focus::tab_navigation::TabIndex,
    prelude::*,
    text::{EditableText, TextCursorStyle, TextEditChange},
};

use crate::frontend::{FrontendState, spawn_frontend_button};
use crate::i18n::pick;
use crate::{
    ACTION, ACTION_HOVER, BACKGROUND, CARD_SELECTED, FRIENDLY, MUTED_TEXT, PANEL, TEXT, UiAction,
    text_font,
};

#[derive(Component, Clone, Copy)]
enum LlmField {
    Url,
    Model,
    Key,
}

#[derive(Component)]
struct KeyMask;

pub(crate) fn spawn_llm_settings(root: &mut ChildSpawnerCommands, state: &FrontendState) {
    let locale = state.config.locale;
    root.spawn((Node {
        width: percent(100), height: percent(100), flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center, justify_content: JustifyContent::Center,
        row_gap: px(12), padding: UiRect::all(px(20)), ..default()
    }, BackgroundColor(BACKGROUND))).with_children(|screen| {
        screen.spawn((Text::new(pick(locale, "LLM OPPONENT", "LLM 陪玩设置", "LLM 陪玩設定")), text_font(30.0), TextColor(TEXT)));
        screen.spawn((Text::new(pick(locale,
            "Chat Completions compatible API. Enter a base URL ending in /v1, or the full endpoint.",
            "兼容 Chat Completions 的接口。填写以 /v1 结尾的基础 URL，或完整接口地址。",
            "相容 Chat Completions 的介面。填寫以 /v1 結尾的基礎 URL，或完整介面位址。")), text_font(15.0), TextColor(MUTED_TEXT)));
        for (index, (field, title, value)) in [
            (LlmField::Url, "URL", &state.config.llm.base_url),
            (LlmField::Model, pick(locale, "Model", "模型名称", "模型名稱"), &state.config.llm.model),
            (LlmField::Key, "API Key", &state.config.llm.api_key),
        ].into_iter().enumerate() {
            screen.spawn((Text::new(title), text_font(16.0), TextColor(TEXT)));
            screen.spawn(Node { width: percent(80), max_width: px(950), height: px(46), ..default() }).with_children(|row| {
                let secret = matches!(field, LlmField::Key);
                let mut input = EditableText::new(value.clone());
                input.allow_newlines = false;
                input.max_characters = Some(4096);
                row.spawn((input, field, TabIndex(index as i32), text_font(16.0),
                    TextColor(if secret { Color::NONE } else { TEXT }),
                    TextCursorStyle { color: CARD_SELECTED, selected_text_color: Some(if secret { Color::NONE } else { BACKGROUND }), ..default() },
                    Node { width: percent(100), height: percent(100), padding: UiRect::all(px(10)), overflow: Overflow::clip(), border: UiRect::all(px(1)), ..default() },
                    BackgroundColor(PANEL), BorderColor::all(ACTION_HOVER),
                )).observe(sync_input);
                if secret {
                    row.spawn((KeyMask, Text::new(mask(value)), text_font(16.0), TextColor(TEXT),
                        Node { position_type: PositionType::Absolute, left: px(11), top: px(11), ..default() }, Pickable::IGNORE));
                }
            });
        }
        screen.spawn((Text::new(pick(locale,
            "The key stays in memory for this run. You can also use HEARTH_LLM_API_KEY.",
            "Key 仅在本次运行中保留，也可通过 HEARTH_LLM_API_KEY 环境变量提供。",
            "Key 僅在本次執行中保留，也可透過 HEARTH_LLM_API_KEY 環境變數提供。")), text_font(14.0), TextColor(MUTED_TEXT)));
        spawn_frontend_button(screen, if state.config.llm.json_mode {
            pick(locale, "JSON mode: on", "JSON 模式：开启", "JSON 模式：開啟")
        } else { pick(locale, "JSON mode: off", "JSON 模式：关闭", "JSON 模式：關閉") }, UiAction::ToggleLlmJsonMode, ACTION, 260.0);
        screen.spawn(Node { column_gap: px(12), ..default() }).with_children(|row| {
            spawn_frontend_button(row, pick(locale, "APPLY", "应用设置", "套用設定"), UiAction::SaveLlmSettings, FRIENDLY, 200.0);
            spawn_frontend_button(row, pick(locale, "BACK", "返回", "返回"), UiAction::CloseLlmSettings, ACTION, 200.0);
        });
        if let Some(status) = &state.status {
            screen.spawn((Text::new(status), text_font(15.0), TextColor(CARD_SELECTED)));
        }
    });
}

fn mask(value: &str) -> String {
    "•".repeat(value.chars().count().min(40))
}

fn sync_input(
    event: On<TextEditChange>,
    inputs: Query<(&EditableText, &LlmField)>,
    mut masks: Query<&mut Text, With<KeyMask>>,
    mut state: ResMut<FrontendState>,
) {
    let Ok((input, field)) = inputs.get(event.event_target()) else {
        return;
    };
    let value = input.value().to_string();
    match field {
        LlmField::Url => state.config.llm.base_url = value,
        LlmField::Model => state.config.llm.model = value,
        LlmField::Key => {
            for mut text in &mut masks {
                **text = mask(&value);
            }
            state.config.llm.api_key = value;
        }
    }
}
