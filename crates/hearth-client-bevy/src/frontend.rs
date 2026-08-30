use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle, TextEditChange};
use hearth_app::{BotDifficulty, CardCatalogEntry, DeckLibrary, DeckList, MatchConfig, MatchMode};

use crate::card_preview::{InspectableCard, hide_card_preview, show_card_preview};
use crate::i18n::{bot_difficulty_label, class_label, kind_label, pick};
use crate::turn_timer::TurnTimerConfig;

use super::{
    ACTION, ACTION_HOVER, BACKGROUND, ButtonColors, CARD_SELECTED, DisplaySettings, ENEMY,
    FRIENDLY, MUTED_TEXT, PANEL, TEXT, UiAction, handle_ui_click, text_font,
};

const DECKS_PER_PAGE: usize = 6;
const CARDS_PER_PAGE: usize = 6;
const DRAFT_ROWS: usize = 9;
const CONSTRUCTED_CLASSES: [&str; 11] = [
    "death_knight",
    "demon_hunter",
    "druid",
    "hunter",
    "mage",
    "paladin",
    "priest",
    "rogue",
    "shaman",
    "warlock",
    "warrior",
];

#[derive(Component)]
struct CatalogSearchInput;

#[derive(Component)]
struct DeckNameInput;

#[derive(Component)]
struct DeckCodeInput;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClientScene {
    #[default]
    MainMenu,
    Settings,
    DeckSelect,
    DeckBuilder,
    DeckCode,
    Match,
}

#[derive(Resource)]
pub struct ClientCatalog(pub DeckLibrary);

#[derive(Resource)]
pub struct FrontendState {
    pub scene: ClientScene,
    pub config: MatchConfig,
    pub player_deck: usize,
    pub opponent_deck: usize,
    pub deck_page: usize,
    pub catalog_page: usize,
    pub draft_page: usize,
    pub catalog_cost: Option<u8>,
    pub catalog_kind: Option<hearth_core::CardKind>,
    pub catalog_query: String,
    pub draft: Option<DeckList>,
    pub draft_sideboard_owner: Option<String>,
    pub draft_baseline: Option<DeckList>,
    pub draft_source: Option<PathBuf>,
    pub pending_delete_deck: Option<usize>,
    pub pending_abandon_match: bool,
    pub match_menu_open: bool,
    pub pending_concede: bool,
    pub settings_return: ClientScene,
    pub deck_code: String,
    pub deck_code_return: ClientScene,
    pub handoff_player: Option<hearth_core::PlayerId>,
    pub resume_available: bool,
    pub status: Option<String>,
    pub match_number: u64,
}

impl FrontendState {
    pub fn new(config: MatchConfig, library: &DeckLibrary, quick_start: bool) -> Self {
        let player_path = if config.match_mode == MatchMode::Hotseat
            || config.human_player == hearth_core::PlayerId::ONE
        {
            &config.deck_one
        } else {
            &config.deck_two
        };
        let opponent_path = if config.match_mode == MatchMode::Hotseat
            || config.human_player == hearth_core::PlayerId::ONE
        {
            &config.deck_two
        } else {
            &config.deck_one
        };
        Self {
            scene: if quick_start {
                ClientScene::Match
            } else {
                ClientScene::MainMenu
            },
            player_deck: library.index_of_path(player_path).unwrap_or(0),
            opponent_deck: library.index_of_path(opponent_path).unwrap_or(0),
            config,
            deck_page: 0,
            catalog_page: 0,
            draft_page: 0,
            catalog_cost: None,
            catalog_kind: None,
            catalog_query: String::new(),
            draft: None,
            draft_sideboard_owner: None,
            draft_baseline: None,
            draft_source: None,
            pending_delete_deck: None,
            pending_abandon_match: false,
            match_menu_open: false,
            pending_concede: false,
            settings_return: ClientScene::MainMenu,
            deck_code: String::new(),
            deck_code_return: ClientScene::DeckSelect,
            handoff_player: None,
            resume_available: false,
            status: None,
            match_number: 0,
        }
    }

    pub fn pauses_match_progress(&self) -> bool {
        self.match_menu_open
            || (self.scene == ClientScene::Settings && self.settings_return == ClientScene::Match)
    }

    pub fn open_builder(&mut self, library: &DeckLibrary) {
        let Some(stored) = library.deck(self.player_deck) else {
            self.status = Some(
                pick(
                    self.config.locale,
                    "No deck is selected.",
                    "尚未选择套牌。",
                    "尚未選擇牌組。",
                )
                .to_owned(),
            );
            return;
        };
        let mut draft = stored.deck.clone();
        let is_custom = library.is_custom(self.player_deck);
        if !is_custom {
            let suffix = pick(self.config.locale, " (Custom)", "（自定义）", "（自訂）");
            if !draft.name.ends_with(suffix) {
                draft.name.push_str(suffix);
            }
        }
        self.draft_baseline = Some(draft.clone());
        self.draft = Some(draft);
        self.draft_sideboard_owner = None;
        self.draft_source = is_custom.then(|| stored.path.clone());
        self.catalog_page = 0;
        self.draft_page = 0;
        self.catalog_cost = None;
        self.catalog_kind = None;
        self.catalog_query.clear();
        self.pending_delete_deck = None;
        self.handoff_player = None;
        self.status = None;
        self.scene = ClientScene::DeckBuilder;
    }

    pub fn open_new_deck(&mut self) {
        let draft = DeckList {
            name: pick(
                self.config.locale,
                "New Custom Deck",
                "新建自定义套牌",
                "新建自訂牌組",
            )
            .to_owned(),
            format: Some("wild".to_owned()),
            class: "mage".to_owned(),
            cards: Vec::new(),
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };
        self.draft_baseline = Some(draft.clone());
        self.draft = Some(draft);
        self.draft_sideboard_owner = None;
        self.draft_source = None;
        self.catalog_page = 0;
        self.draft_page = 0;
        self.catalog_cost = None;
        self.catalog_kind = None;
        self.catalog_query.clear();
        self.pending_delete_deck = None;
        self.handoff_player = None;
        self.status = None;
        self.scene = ClientScene::DeckBuilder;
    }

    pub fn reset_draft(&mut self) {
        if let Some(baseline) = self.draft_baseline.clone() {
            self.draft = Some(baseline);
            self.draft_sideboard_owner = None;
            self.catalog_page = 0;
            self.draft_page = 0;
            self.status = None;
        }
    }

    pub fn open_deck_code(&mut self, code: String) {
        self.deck_code_return = self.scene;
        self.deck_code = code;
        self.status = None;
        self.handoff_player = None;
        self.scene = ClientScene::DeckCode;
    }

    pub fn open_imported_deck(&mut self, deck: DeckList) {
        self.draft_baseline = Some(deck.clone());
        self.draft = Some(deck);
        self.draft_sideboard_owner = None;
        self.draft_source = None;
        self.catalog_page = 0;
        self.draft_page = 0;
        self.catalog_cost = None;
        self.catalog_kind = None;
        self.catalog_query.clear();
        self.pending_delete_deck = None;
        self.handoff_player = None;
        self.scene = ClientScene::DeckBuilder;
    }

    pub fn set_draft_class(&mut self, class: &str) -> bool {
        let Some(draft) = self.draft.as_mut() else {
            return false;
        };
        if draft.class == class {
            return true;
        }
        if !draft.cards.is_empty() {
            return false;
        }
        draft.class = class.to_owned();
        draft.hero_power = None;
        self.catalog_page = 0;
        true
    }

    pub fn repair_deck_indices_after_removal(&mut self, removed: usize, remaining: usize) {
        fn repair(selected: usize, removed: usize, remaining: usize) -> usize {
            if remaining == 0 {
                0
            } else if selected > removed {
                selected - 1
            } else if selected == removed {
                removed.min(remaining - 1)
            } else {
                selected
            }
        }

        self.player_deck = repair(self.player_deck, removed, remaining);
        self.opponent_deck = repair(self.opponent_deck, removed, remaining);
        let pages = remaining.max(1).div_ceil(DECKS_PER_PAGE);
        self.deck_page = self.deck_page.min(pages - 1);
        self.pending_delete_deck = None;
    }

    pub fn restore_deck_selections_after_save(
        &mut self,
        library: &DeckLibrary,
        saved_path: &Path,
        source: Option<&Path>,
        opponent_before: Option<&Path>,
    ) {
        let opponent_target = if source.is_some() && source == opponent_before {
            Some(saved_path)
        } else {
            opponent_before
        };
        self.restore_deck_selections_by_path(library, Some(saved_path), opponent_target);
    }

    pub fn restore_deck_selections_by_path(
        &mut self,
        library: &DeckLibrary,
        player_before: Option<&Path>,
        opponent_before: Option<&Path>,
    ) {
        if let Some(index) = player_before.and_then(|path| library.index_of_path(path)) {
            self.player_deck = index;
        }
        if let Some(index) = opponent_before.and_then(|path| library.index_of_path(path)) {
            self.opponent_deck = index;
        }
    }

    pub fn apply_selected_decks(&mut self, library: &DeckLibrary) -> Result<(), String> {
        let player = library.deck(self.player_deck).ok_or_else(|| {
            pick(
                self.config.locale,
                "selected player deck no longer exists",
                "已选择的玩家套牌已不存在",
                "已選擇的玩家牌組已不存在",
            )
            .to_owned()
        })?;
        let opponent = library.deck(self.opponent_deck).ok_or_else(|| {
            pick(
                self.config.locale,
                "selected opponent deck no longer exists",
                "已选择的对手套牌已不存在",
                "已選擇的對手牌組已不存在",
            )
            .to_owned()
        })?;
        if self.config.match_mode == MatchMode::Hotseat
            || self.config.human_player == hearth_core::PlayerId::ONE
        {
            self.config.deck_one = player.path.clone();
            self.config.deck_two = opponent.path.clone();
        } else {
            self.config.deck_one = opponent.path.clone();
            self.config.deck_two = player.path.clone();
        }
        self.match_number = self.match_number.saturating_add(1);
        self.config.seed = self.config.seed.wrapping_add(self.match_number);
        Ok(())
    }
}

pub fn spawn_frontend(
    root: &mut ChildSpawnerCommands,
    state: &FrontendState,
    catalog: &ClientCatalog,
    timer: &TurnTimerConfig,
    display: &DisplaySettings,
) {
    match state.scene {
        ClientScene::MainMenu => spawn_main_menu(root, state, &catalog.0),
        ClientScene::Settings => spawn_settings(root, state, timer, display),
        ClientScene::DeckSelect => spawn_deck_select(root, state, &catalog.0),
        ClientScene::DeckBuilder => spawn_deck_builder(root, state, &catalog.0),
        ClientScene::DeckCode => spawn_deck_code(root, state),
        ClientScene::Match => {}
    }
}

fn spawn_main_menu(root: &mut ChildSpawnerCommands, state: &FrontendState, library: &DeckLibrary) {
    let locale = state.config.locale;
    root.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(18),
            ..default()
        },
        BackgroundColor(BACKGROUND),
    ))
    .with_children(|menu| {
        menu.spawn((
            Text::new("HEARTH-RS"),
            text_font(58.0),
            TextColor(CARD_SELECTED),
            Pickable::IGNORE,
        ));
        menu.spawn((
            Text::new(pick(
                locale,
                "A deterministic Hearthstone-style client powered by Bevy 0.19",
                "由 Bevy 0.19 驱动的确定性炉石风格客户端",
                "由 Bevy 0.19 驅動的確定性爐石風格客戶端",
            )),
            text_font(19.0),
            TextColor(TEXT),
            Pickable::IGNORE,
        ));
        let selected = library
            .deck(state.player_deck)
            .map(|deck| deck.deck.name.as_str())
            .unwrap_or(pick(locale, "No deck", "无套牌", "無牌組"));
        menu.spawn((
            Text::new(match locale {
                hearth_core::Locale::EnUs => format!(
                    "{} decks  ·  {} collectible cards  ·  selected: {selected}",
                    library.decks().len(),
                    library.cards().len()
                ),
                hearth_core::Locale::ZhCn => format!(
                    "{} 副套牌  ·  {} 张可收藏卡牌  ·  当前：{selected}",
                    library.decks().len(),
                    library.cards().len()
                ),
                hearth_core::Locale::ZhTw => format!(
                    "{} 副牌組  ·  {} 張可收藏卡牌  ·  目前：{selected}",
                    library.decks().len(),
                    library.cards().len()
                ),
            }),
            text_font(15.0),
            TextColor(MUTED_TEXT),
            Pickable::IGNORE,
        ));
        if state.resume_available {
            spawn_frontend_button(
                menu,
                pick(locale, "CONTINUE", "继续对局", "繼續對戰"),
                UiAction::ContinueMatch,
                CARD_SELECTED,
                330.0,
            );
            if state.pending_abandon_match {
                spawn_frontend_button(
                    menu,
                    pick(locale, "CONFIRM ABANDON", "确认放弃对局", "確認放棄對戰"),
                    UiAction::AbandonMatch,
                    ENEMY,
                    330.0,
                );
                spawn_frontend_button(
                    menu,
                    pick(locale, "KEEP MATCH", "保留对局", "保留對戰"),
                    UiAction::CancelAbandonMatch,
                    ACTION,
                    330.0,
                );
            } else {
                spawn_frontend_button(
                    menu,
                    pick(
                        locale,
                        "ABANDON SAVED MATCH",
                        "放弃已保存对局",
                        "放棄已儲存對戰",
                    ),
                    UiAction::AbandonMatch,
                    ENEMY,
                    330.0,
                );
            }
        }
        spawn_frontend_button(
            menu,
            pick(locale, "PLAY", "开始游戏", "開始遊戲"),
            UiAction::OpenDeckSelect,
            CARD_SELECTED,
            330.0,
        );
        spawn_frontend_button(
            menu,
            pick(locale, "MY COLLECTION", "我的收藏", "我的收藏"),
            UiAction::OpenDeckBuilder,
            FRIENDLY,
            330.0,
        );
        spawn_frontend_button(
            menu,
            pick(locale, "QUICK MATCH", "快速对局", "快速對戰"),
            UiAction::StartMatch,
            ACTION,
            330.0,
        );
        spawn_frontend_button(
            menu,
            pick(locale, "SETTINGS", "设置", "設定"),
            UiAction::OpenSettings,
            ACTION,
            330.0,
        );
        spawn_frontend_button(
            menu,
            pick(locale, "QUIT", "退出游戏", "離開遊戲"),
            UiAction::QuitApplication,
            ENEMY,
            330.0,
        );
        if let Some(status) = &state.status {
            spawn_status(menu, status, locale);
        }
    });
}

fn spawn_settings(
    root: &mut ChildSpawnerCommands,
    state: &FrontendState,
    timer: &TurnTimerConfig,
    display: &DisplaySettings,
) {
    let locale = state.config.locale;
    root.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12),
            padding: UiRect::all(px(18)),
            ..default()
        },
        BackgroundColor(BACKGROUND),
    ))
    .with_children(|settings| {
        settings.spawn((
            Text::new(pick(locale, "SETTINGS", "设置", "設定")),
            text_font(40.0),
            TextColor(CARD_SELECTED),
            Pickable::IGNORE,
        ));
        settings
            .spawn(Node {
                width: percent(100),
                max_width: px(1100),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                column_gap: px(14),
                ..default()
            })
            .with_children(|grid| {
                grid.spawn(Node {
                    width: percent(50),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    ..default()
                })
                .with_children(|column| {
                    spawn_setting_group(
                        column,
                        pick(locale, "LANGUAGE", "语言", "語言"),
                        pick(
                            locale,
                            "Reloads card names and text immediately.",
                            "立即重新加载本地化卡牌名称与正文。",
                            "立即重新載入本地化卡牌名稱與敘述。",
                        ),
                        |row| {
                            for (choice, label) in [
                                (hearth_core::Locale::EnUs, "English"),
                                (hearth_core::Locale::ZhCn, "简体中文"),
                                (hearth_core::Locale::ZhTw, "繁體中文"),
                            ] {
                                spawn_frontend_button(
                                    row,
                                    label,
                                    UiAction::SetLocale(choice),
                                    if choice == locale {
                                        CARD_SELECTED
                                    } else {
                                        ACTION
                                    },
                                    145.0,
                                );
                            }
                        },
                    );
                    spawn_setting_group(
                        column,
                        pick(locale, "TURN TIMER", "回合计时", "回合計時"),
                        pick(
                            locale,
                            "Card rules such as Nozdormu override this client default.",
                            "诺兹多姆等卡牌规则优先于此客户端默认值。",
                            "諾茲多姆等卡牌規則優先於此用戶端預設值。",
                        ),
                        |row| {
                            for seconds in [0, 30, 45, 60, 75, 90] {
                                let label = if seconds == 0 {
                                    pick(locale, "Off", "关闭", "關閉").to_owned()
                                } else {
                                    format!("{seconds}s")
                                };
                                spawn_small_button(
                                    row,
                                    &label,
                                    UiAction::SetTurnSeconds(seconds),
                                    if timer.default_seconds == seconds {
                                        CARD_SELECTED
                                    } else {
                                        ACTION
                                    },
                                );
                            }
                        },
                    );
                });
                grid.spawn(Node {
                    width: percent(50),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    ..default()
                })
                .with_children(|column| {
                    spawn_setting_group(
                        column,
                        pick(locale, "DISPLAY MODE", "显示模式", "顯示模式"),
                        pick(
                            locale,
                            "Borderless fullscreen can also be toggled with F11.",
                            "也可以按 F11 切换无边框全屏。",
                            "也可以按 F11 切換無邊框全螢幕。",
                        ),
                        |row| {
                            for (fullscreen, label) in [
                                (false, pick(locale, "Windowed", "窗口", "視窗")),
                                (
                                    true,
                                    pick(
                                        locale,
                                        "Borderless Fullscreen",
                                        "无边框全屏",
                                        "無邊框全螢幕",
                                    ),
                                ),
                            ] {
                                spawn_frontend_button(
                                    row,
                                    label,
                                    UiAction::SetFullscreen(fullscreen),
                                    if display.fullscreen == fullscreen {
                                        CARD_SELECTED
                                    } else {
                                        ACTION
                                    },
                                    if fullscreen { 245.0 } else { 180.0 },
                                );
                            }
                        },
                    );
                    spawn_setting_group(
                        column,
                        pick(locale, "UI SCALE", "界面缩放", "介面縮放"),
                        pick(
                            locale,
                            "Adjust fixed-size controls and text for your display.",
                            "调整固定尺寸控件与文字以适应你的显示器。",
                            "調整固定尺寸控制項與文字以配合你的顯示器。",
                        ),
                        |row| {
                            for percent in [80, 100, 120] {
                                spawn_small_button(
                                    row,
                                    &format!("{percent}%"),
                                    UiAction::SetUiScale(percent),
                                    if display.ui_scale_percent == percent {
                                        CARD_SELECTED
                                    } else {
                                        ACTION
                                    },
                                );
                            }
                        },
                    );
                });
            });
        spawn_frontend_button(
            settings,
            pick(locale, "BACK", "返回", "返回"),
            UiAction::CloseSettings,
            FRIENDLY,
            260.0,
        );
        if let Some(status) = &state.status {
            spawn_status(settings, status, locale);
        }
    });
}

fn spawn_setting_group(
    parent: &mut ChildSpawnerCommands,
    heading: &str,
    description: &str,
    spawn_controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                min_height: px(145),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(8),
                padding: UiRect::all(px(14)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor::all(Color::srgb(0.28, 0.39, 0.49)),
        ))
        .with_children(|group| {
            group.spawn((
                Text::new(heading),
                text_font(22.0),
                TextColor(TEXT),
                Pickable::IGNORE,
            ));
            group.spawn((
                Text::new(description),
                text_font(13.0),
                TextColor(MUTED_TEXT),
                TextLayout::justify(Justify::Center),
                Node {
                    width: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            group
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(10),
                    row_gap: px(8),
                    ..default()
                })
                .with_children(spawn_controls);
        });
}

fn spawn_deck_select(
    root: &mut ChildSpawnerCommands,
    state: &FrontendState,
    library: &DeckLibrary,
) {
    let locale = state.config.locale;
    root.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(24)),
            row_gap: px(12),
            ..default()
        },
        BackgroundColor(BACKGROUND),
    ))
    .with_children(|screen| {
        spawn_scene_header(
            screen,
            pick(locale, "CHOOSE YOUR DECK", "选择你的套牌", "選擇你的牌組"),
            if state.config.match_mode == MatchMode::Hotseat {
                pick(
                    locale,
                    "Select separate decks for Player 1 and Player 2",
                    "为玩家 1 和玩家 2 分别选择套牌",
                    "為玩家 1 和玩家 2 分別選擇牌組",
                )
            } else {
                pick(
                    locale,
                    "Select decks for you and the built-in AI",
                    "为你和内置 AI 分别选择套牌",
                    "為你和內建 AI 分別選擇牌組",
                )
            },
        );
        screen
            .spawn(Node {
                width: percent(100),
                min_height: px(38),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(10),
                ..default()
            })
            .with_children(|modes| {
                modes.spawn((
                    Text::new(pick(locale, "MODE", "模式", "模式")),
                    text_font(13.0),
                    TextColor(MUTED_TEXT),
                    Pickable::IGNORE,
                ));
                spawn_small_button(
                    modes,
                    pick(locale, "VS BUILT-IN AI", "对战内置 AI", "對戰內建 AI"),
                    UiAction::SetMatchMode(MatchMode::VsBot),
                    if state.config.match_mode == MatchMode::VsBot {
                        CARD_SELECTED
                    } else {
                        ACTION
                    },
                );
                spawn_small_button(
                    modes,
                    pick(locale, "LOCAL TWO PLAYER", "本地双人", "本機雙人"),
                    UiAction::SetMatchMode(MatchMode::Hotseat),
                    if state.config.match_mode == MatchMode::Hotseat {
                        CARD_SELECTED
                    } else {
                        ACTION
                    },
                );
            });
        if state.config.match_mode == MatchMode::VsBot {
            screen
                .spawn(Node {
                    width: percent(100),
                    min_height: px(38),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: px(10),
                    ..default()
                })
                .with_children(|difficulties| {
                    difficulties.spawn((
                        Text::new(pick(locale, "AI DIFFICULTY", "AI 难度", "AI 難度")),
                        text_font(13.0),
                        TextColor(MUTED_TEXT),
                        Pickable::IGNORE,
                    ));
                    for difficulty in [
                        BotDifficulty::Easy,
                        BotDifficulty::Normal,
                        BotDifficulty::Hard,
                    ] {
                        spawn_small_button(
                            difficulties,
                            bot_difficulty_label(locale, difficulty),
                            UiAction::SetBotDifficulty(difficulty),
                            if state.config.bot_difficulty == difficulty {
                                CARD_SELECTED
                            } else {
                                ACTION
                            },
                        );
                    }
                });
        }
        screen
            .spawn(Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                column_gap: px(12),
                ..default()
            })
            .with_children(|selected| {
                spawn_selected_deck(
                    selected,
                    if state.config.match_mode == MatchMode::Hotseat {
                        pick(locale, "PLAYER 1", "玩家 1", "玩家 1")
                    } else {
                        pick(locale, "YOUR DECK", "你的套牌", "你的牌組")
                    },
                    library.deck(state.player_deck),
                    FRIENDLY,
                    locale,
                );
                spawn_selected_deck(
                    selected,
                    if state.config.match_mode == MatchMode::Hotseat {
                        pick(locale, "PLAYER 2", "玩家 2", "玩家 2")
                    } else {
                        pick(locale, "OPPONENT", "对手", "對手")
                    },
                    library.deck(state.opponent_deck),
                    ENEMY,
                    locale,
                );
            });

        let pages = library.decks().len().max(1).div_ceil(DECKS_PER_PAGE);
        let page = state.deck_page.min(pages - 1);
        for (index, stored) in library
            .decks()
            .iter()
            .enumerate()
            .skip(page * DECKS_PER_PAGE)
            .take(DECKS_PER_PAGE)
        {
            screen
                .spawn((
                    Node {
                        width: percent(100),
                        min_height: px(61),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        padding: UiRect::all(px(8)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderColor::all(if index == state.player_deck {
                        CARD_SELECTED
                    } else {
                        ACTION
                    }),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!(
                            "{}\n{} · {} {}{}",
                            stored.deck.name,
                            class_label(locale, &stored.deck.class),
                            stored.deck.cards.len(),
                            pick(locale, "cards", "张卡牌", "張卡牌"),
                            if stored.deck.unrestricted {
                                pick(locale, " · unrestricted", " · 无限制", " · 無限制")
                            } else {
                                ""
                            }
                        )),
                        text_font(14.0),
                        TextColor(TEXT),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    spawn_small_button(
                        row,
                        pick(locale, "Use", "使用", "使用"),
                        UiAction::SelectPlayerDeck(index),
                        FRIENDLY,
                    );
                    spawn_small_button(
                        row,
                        if state.config.match_mode == MatchMode::Hotseat {
                            pick(locale, "P2", "玩家2", "玩家2")
                        } else {
                            pick(locale, "AI", "AI", "AI")
                        },
                        UiAction::SelectOpponentDeck(index),
                        ENEMY,
                    );
                    spawn_small_button(
                        row,
                        pick(locale, "Edit", "编辑", "編輯"),
                        UiAction::EditDeck(index),
                        ACTION,
                    );
                    if library.is_custom(index) {
                        if state.pending_delete_deck == Some(index) {
                            spawn_small_button(
                                row,
                                pick(locale, "Confirm", "确认删除", "確認刪除"),
                                UiAction::DeleteDeck(index),
                                ENEMY,
                            );
                            spawn_small_button(
                                row,
                                pick(locale, "Cancel", "取消", "取消"),
                                UiAction::CancelDeckDelete,
                                ACTION,
                            );
                        } else {
                            spawn_small_button(
                                row,
                                pick(locale, "Delete", "删除", "刪除"),
                                UiAction::DeleteDeck(index),
                                ENEMY,
                            );
                        }
                    }
                });
        }
        spawn_page_controls(
            screen,
            page,
            pages,
            UiAction::PreviousDeckPage,
            UiAction::NextDeckPage,
            locale,
        );
        screen
            .spawn(Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: px(12),
                ..default()
            })
            .with_children(|controls| {
                spawn_frontend_button(
                    controls,
                    pick(locale, "BACK", "返回", "返回"),
                    UiAction::OpenMainMenu,
                    ACTION,
                    150.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "NEW DECK", "新建套牌", "新建牌組"),
                    UiAction::NewDeck,
                    ACTION,
                    180.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "DECK CODE", "套牌代码", "牌組代碼"),
                    UiAction::OpenDeckCode,
                    ACTION,
                    180.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "EDIT YOUR DECK", "编辑你的套牌", "編輯你的牌組"),
                    UiAction::OpenDeckBuilder,
                    FRIENDLY,
                    240.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "START MATCH", "开始对局", "開始對戰"),
                    UiAction::StartMatch,
                    CARD_SELECTED,
                    240.0,
                );
            });
        if let Some(status) = &state.status {
            spawn_status(screen, status, locale);
        }
    });
}

fn spawn_deck_builder(
    root: &mut ChildSpawnerCommands,
    state: &FrontendState,
    library: &DeckLibrary,
) {
    let locale = state.config.locale;
    let Some(draft) = &state.draft else {
        return;
    };
    let required_size = library.required_deck_size(draft);
    let rune_suffix = if draft.class == "death_knight" && !draft.unrestricted {
        format!(
            " · {} {}",
            pick(locale, "RUNES", "符文", "符文"),
            deck_rune_label(library.deck_rune_cost(draft))
        )
    } else {
        String::new()
    };
    root.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(18)),
            row_gap: px(10),
            ..default()
        },
        BackgroundColor(BACKGROUND),
    ))
    .with_children(|screen| {
        spawn_scene_header(
            screen,
            if state.draft_sideboard_owner.is_some() {
                pick(locale, "BUILD THE BAND", "组建乐队", "組建樂團")
            } else {
                pick(locale, "MY COLLECTION", "我的收藏", "我的收藏")
            },
            &format!(
                "{} · {}/{} {}{}",
                class_label(locale, &draft.class),
                draft.cards.len(),
                required_size,
                pick(locale, "cards", "张卡牌", "張卡牌"),
                rune_suffix,
            ),
        );
        spawn_deck_identity_editor(screen, state, draft);
        screen
            .spawn(Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(12),
                ..default()
            })
            .with_children(|columns| {
                spawn_catalog_column(columns, state, library, draft);
                spawn_draft_column(columns, state, library, draft);
            });
        screen
            .spawn(Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: px(12),
                ..default()
            })
            .with_children(|controls| {
                if state.draft_sideboard_owner.is_some() {
                    spawn_frontend_button(
                        controls,
                        pick(locale, "BACK TO DECK", "返回套牌", "返回牌組"),
                        UiAction::CloseDraftSideboard,
                        FRIENDLY,
                        180.0,
                    );
                }
                spawn_frontend_button(
                    controls,
                    pick(locale, "BACK", "返回", "返回"),
                    UiAction::OpenDeckSelect,
                    ACTION,
                    180.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "RESET", "重置", "重設"),
                    UiAction::ResetDraft,
                    ENEMY,
                    180.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "DECK CODE", "套牌代码", "牌組代碼"),
                    UiAction::OpenDeckCode,
                    ACTION,
                    180.0,
                );
                spawn_frontend_button(
                    controls,
                    if state.draft_source.is_some() {
                        pick(locale, "SAVE DECK", "保存套牌", "儲存牌組")
                    } else {
                        pick(locale, "SAVE CUSTOM DECK", "保存自定义套牌", "儲存自訂牌組")
                    },
                    UiAction::SaveDraft,
                    if draft.cards.len() == required_size {
                        CARD_SELECTED
                    } else {
                        ACTION
                    },
                    260.0,
                );
            });
        if let Some(status) = &state.status {
            spawn_status(screen, status, locale);
        }
    });
}

fn spawn_deck_code(root: &mut ChildSpawnerCommands, state: &FrontendState) {
    let locale = state.config.locale;
    root.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(18),
            padding: UiRect::all(px(30)),
            ..default()
        },
        BackgroundColor(BACKGROUND),
    ))
    .with_children(|screen| {
        spawn_scene_header(
            screen,
            pick(locale, "DECK CODE", "套牌代码", "牌組代碼"),
            pick(
                locale,
                "Paste a Hearthstone deck code or the complete text copied by the official client.",
                "粘贴炉石套牌代码，或官方客户端复制出的完整文本。",
                "貼上爐石牌組代碼，或官方用戶端複製出的完整文字。",
            ),
        );
        screen.spawn((
            Text::new(pick(
                locale,
                "Click the field to edit. Ctrl+A, Ctrl+C and Ctrl+V are supported.",
                "点击输入框即可编辑，支持 Ctrl+A、Ctrl+C 和 Ctrl+V。",
                "點擊輸入框即可編輯，支援 Ctrl+A、Ctrl+C 和 Ctrl+V。",
            )),
            text_font(14.0),
            TextColor(MUTED_TEXT),
            Pickable::IGNORE,
        ));
        let mut input = EditableText::new(state.deck_code.clone());
        input.max_characters = Some(8192);
        input.allow_newlines = true;
        input.visible_lines = Some(8.0);
        screen
            .spawn((
                input,
                DeckCodeInput,
                TabIndex(0),
                text_font(16.0),
                TextColor(TEXT),
                TextCursorStyle {
                    color: CARD_SELECTED,
                    selected_text_color: Some(BACKGROUND),
                    ..default()
                },
                Node {
                    width: px(1100),
                    min_height: px(190),
                    padding: UiRect::all(px(16)),
                    border: UiRect::all(px(2)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(ACTION_HOVER),
            ))
            .observe(sync_deck_code_input);
        screen
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: px(12),
                ..default()
            })
            .with_children(|controls| {
                spawn_frontend_button(
                    controls,
                    pick(locale, "BACK", "返回", "返回"),
                    UiAction::CloseDeckCode,
                    ACTION,
                    180.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "REFRESH EXPORT", "刷新导出", "重新匯出"),
                    UiAction::ExportDeckCode,
                    FRIENDLY,
                    240.0,
                );
                spawn_frontend_button(
                    controls,
                    pick(locale, "IMPORT AS NEW DECK", "导入为新套牌", "匯入為新牌組"),
                    UiAction::ImportDeckCode,
                    CARD_SELECTED,
                    280.0,
                );
            });
        if let Some(status) = &state.status {
            spawn_status(screen, status, locale);
        }
    });
}

fn spawn_deck_identity_editor(
    parent: &mut ChildSpawnerCommands,
    state: &FrontendState,
    draft: &DeckList,
) {
    let locale = state.config.locale;
    parent
        .spawn((
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .with_children(|editor| {
            editor
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(pick(locale, "Deck name", "套牌名称", "牌組名稱")),
                        text_font(13.0),
                        TextColor(MUTED_TEXT),
                        Pickable::IGNORE,
                    ));
                    let mut input = EditableText::new(draft.name.clone());
                    input.max_characters = Some(128);
                    row.spawn((
                        input,
                        DeckNameInput,
                        TabIndex(0),
                        text_font(14.0),
                        TextColor(TEXT),
                        TextCursorStyle {
                            color: CARD_SELECTED,
                            selected_text_color: Some(BACKGROUND),
                            ..default()
                        },
                        Node {
                            flex_grow: 1.0,
                            min_height: px(34),
                            padding: UiRect::axes(px(8), px(6)),
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(BACKGROUND),
                        BorderColor::all(ACTION_HOVER),
                    ))
                    .observe(sync_deck_name_input);
                });
            editor
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(5),
                    row_gap: px(5),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(pick(locale, "Class", "职业", "職業")),
                        text_font(12.0),
                        TextColor(MUTED_TEXT),
                        Pickable::IGNORE,
                    ));
                    for class in CONSTRUCTED_CLASSES {
                        spawn_filter_button(
                            row,
                            class_label(locale, class),
                            UiAction::SetDraftClass(class.to_owned()),
                            draft.class == class,
                        );
                    }
                    if !draft.cards.is_empty() {
                        row.spawn((
                            Text::new(pick(
                                locale,
                                "Remove all cards before changing class",
                                "移除全部卡牌后才能更改职业",
                                "移除全部卡牌後才能變更職業",
                            )),
                            text_font(11.0),
                            TextColor(MUTED_TEXT),
                            Pickable::IGNORE,
                        ));
                    }
                });
        });
}

fn spawn_catalog_column(
    parent: &mut ChildSpawnerCommands,
    state: &FrontendState,
    library: &DeckLibrary,
    draft: &DeckList,
) {
    let locale = state.config.locale;
    let cards = library
        .cards()
        .iter()
        .filter(|card| card_matches_filters(card, draft, state, library))
        .collect::<Vec<_>>();
    let pages = cards.len().max(1).div_ceil(CARDS_PER_PAGE);
    let page = state.catalog_page.min(pages - 1);
    parent
        .spawn((
            Node {
                width: percent(64),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .with_children(|column| {
            column.spawn((
                Text::new(match locale {
                    hearth_core::Locale::EnUs => {
                        format!("AVAILABLE CARDS · {} results", cards.len())
                    }
                    hearth_core::Locale::ZhCn => {
                        format!("可用卡牌 · {} 项结果", cards.len())
                    }
                    hearth_core::Locale::ZhTw => {
                        format!("可用卡牌 · {} 項結果", cards.len())
                    }
                }),
                text_font(18.0),
                TextColor(TEXT),
                Pickable::IGNORE,
            ));
            spawn_catalog_filters(column, state);
            let counts = draft_area_counts(state, draft);
            for card in cards
                .iter()
                .skip(page * CARDS_PER_PAGE)
                .take(CARDS_PER_PAGE)
            {
                let copies = counts.get(card.id.as_str()).copied().unwrap_or(0);
                column
                    .spawn((
                        InspectableCard(card.id.clone()),
                        Node {
                            width: percent(100),
                            min_height: px(58),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(8),
                            padding: UiRect::all(px(7)),
                            border: UiRect::all(px(1)),
                            border_radius: BorderRadius::all(px(5)),
                            ..default()
                        },
                        BackgroundColor(BACKGROUND),
                        BorderColor::all(ACTION),
                        Pickable::default(),
                    ))
                    .observe(show_card_preview)
                    .observe(hide_card_preview)
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!(
                                "[{}] {}{}  ·  {}  ·  {}\n{}",
                                card.cost,
                                card.name,
                                card_rune_suffix(card.rune_cost),
                                kind_label(locale, card.kind),
                                card.set,
                                shorten_text(&card.text, 88)
                            )),
                            text_font(13.0),
                            TextColor(TEXT),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        row.spawn((
                            Text::new(format!("×{copies}")),
                            text_font(15.0),
                            TextColor(MUTED_TEXT),
                            Pickable::IGNORE,
                        ));
                        spawn_small_button(
                            row,
                            pick(locale, "ADD", "添加", "加入"),
                            UiAction::AddDraftCard(card.id.clone()),
                            FRIENDLY,
                        );
                    });
            }
            spawn_page_controls(
                column,
                page,
                pages,
                UiAction::PreviousCatalogPage,
                UiAction::NextCatalogPage,
                locale,
            );
        });
}

fn spawn_draft_column(
    parent: &mut ChildSpawnerCommands,
    state: &FrontendState,
    library: &DeckLibrary,
    draft: &DeckList,
) {
    let locale = state.config.locale;
    let required_size = library.required_deck_size(draft);
    let counts = draft_area_counts(state, draft);
    let mut rows = counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|(left, _), (right, _)| {
        card_sort_key(library, left).cmp(&card_sort_key(library, right))
    });
    let pages = rows.len().max(1).div_ceil(DRAFT_ROWS);
    let page = state.draft_page.min(pages - 1);
    parent
        .spawn((
            Node {
                width: percent(36),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.15, 0.20)),
        ))
        .with_children(|column| {
            column.spawn((
                Text::new(
                    if let Some(owner) = state.draft_sideboard_owner.as_deref() {
                        let size = draft
                            .sideboards
                            .iter()
                            .find(|sideboard| sideboard.owner == owner)
                            .map(|sideboard| sideboard.cards.len())
                            .unwrap_or_default();
                        let capacity = library
                            .cards()
                            .iter()
                            .find(|card| card.id == owner)
                            .map(|card| card.sideboard_size)
                            .unwrap_or_default();
                        match locale {
                            hearth_core::Locale::EnUs => format!("BAND · {size}/{capacity}"),
                            hearth_core::Locale::ZhCn => format!("乐队 · {size}/{capacity}"),
                            hearth_core::Locale::ZhTw => format!("樂團 · {size}/{capacity}"),
                        }
                    } else {
                        match locale {
                            hearth_core::Locale::EnUs => {
                                format!("DECK LIST · {}/{required_size}", draft.cards.len())
                            }
                            hearth_core::Locale::ZhCn => {
                                format!("套牌列表 · {}/{required_size}", draft.cards.len())
                            }
                            hearth_core::Locale::ZhTw => {
                                format!("牌組列表 · {}/{required_size}", draft.cards.len())
                            }
                        }
                    },
                ),
                text_font(18.0),
                TextColor(CARD_SELECTED),
                Pickable::IGNORE,
            ));
            column.spawn((
                Text::new(deck_summary(library, draft, locale)),
                text_font(12.0),
                TextColor(MUTED_TEXT),
                Pickable::IGNORE,
            ));
            for (card_id, count) in rows.iter().skip(page * DRAFT_ROWS).take(DRAFT_ROWS) {
                let (name, cost) = library
                    .cards()
                    .iter()
                    .find(|card| card.id == *card_id)
                    .map(|card| (card.name.as_str(), card.cost))
                    .unwrap_or((card_id.as_str(), 0));
                column
                    .spawn((
                        InspectableCard(card_id.clone()),
                        Node {
                            width: percent(100),
                            min_height: px(40),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(6),
                            padding: UiRect::all(px(5)),
                            ..default()
                        },
                        BackgroundColor(ACTION),
                        Pickable::default(),
                    ))
                    .observe(show_card_preview)
                    .observe(hide_card_preview)
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!("[{cost}] {name}  ×{count}")),
                            text_font(13.0),
                            TextColor(TEXT),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        spawn_small_button(
                            row,
                            "−",
                            UiAction::RemoveDraftCard(card_id.clone()),
                            ENEMY,
                        );
                        if state.draft_sideboard_owner.is_none()
                            && library
                                .cards()
                                .iter()
                                .find(|card| card.id == *card_id)
                                .is_some_and(|card| card.sideboard_size > 0)
                        {
                            spawn_small_button(
                                row,
                                pick(locale, "BAND", "乐队", "樂團"),
                                UiAction::EditDraftSideboard(card_id.clone()),
                                FRIENDLY,
                            );
                        }
                    });
            }
            spawn_page_controls(
                column,
                page,
                pages,
                UiAction::PreviousDraftPage,
                UiAction::NextDraftPage,
                locale,
            );
        });
}

fn spawn_scene_header(parent: &mut ChildSpawnerCommands, title: &str, subtitle: &str) {
    parent.spawn((
        Text::new(title),
        text_font(34.0),
        TextColor(CARD_SELECTED),
        Pickable::IGNORE,
    ));
    parent.spawn((
        Text::new(subtitle),
        text_font(15.0),
        TextColor(MUTED_TEXT),
        Pickable::IGNORE,
    ));
}

fn spawn_catalog_filters(parent: &mut ChildSpawnerCommands, state: &FrontendState) {
    let locale = state.config.locale;
    parent
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(pick(locale, "Search", "搜索", "搜尋")),
                text_font(12.0),
                TextColor(MUTED_TEXT),
                Pickable::IGNORE,
            ));
            let mut input = EditableText::new(state.catalog_query.clone());
            input.max_characters = Some(80);
            row.spawn((
                input,
                CatalogSearchInput,
                TabIndex(1),
                text_font(12.0),
                TextColor(TEXT),
                TextCursorStyle {
                    color: CARD_SELECTED,
                    selected_text_color: Some(BACKGROUND),
                    ..default()
                },
                Node {
                    flex_grow: 1.0,
                    min_height: px(30),
                    padding: UiRect::axes(px(7), px(4)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(BACKGROUND),
                BorderColor::all(ACTION_HOVER),
            ))
            .observe(sync_catalog_search_input);
            spawn_filter_button(
                row,
                pick(locale, "Apply", "应用", "套用"),
                UiAction::ApplyCatalogSearch,
                !state.catalog_query.trim().is_empty(),
            );
            if !state.catalog_query.is_empty() {
                spawn_filter_button(
                    row,
                    pick(locale, "Clear", "清除", "清除"),
                    UiAction::ClearCatalogSearch,
                    false,
                );
            }
        });
    parent
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(pick(locale, "Cost", "费用", "消耗")),
                text_font(12.0),
                TextColor(MUTED_TEXT),
                Pickable::IGNORE,
            ));
            spawn_filter_button(
                row,
                pick(locale, "All", "全部", "全部"),
                UiAction::FilterCatalogCost(None),
                state.catalog_cost.is_none(),
            );
            for cost in 0..=6 {
                spawn_filter_button(
                    row,
                    &cost.to_string(),
                    UiAction::FilterCatalogCost(Some(cost)),
                    state.catalog_cost == Some(cost),
                );
            }
            spawn_filter_button(
                row,
                "7+",
                UiAction::FilterCatalogCost(Some(7)),
                state.catalog_cost == Some(7),
            );
        });
    parent
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(5),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(pick(locale, "Type", "类型", "類型")),
                text_font(12.0),
                TextColor(MUTED_TEXT),
                Pickable::IGNORE,
            ));
            spawn_filter_button(
                row,
                pick(locale, "All", "全部", "全部"),
                UiAction::FilterCatalogKind(None),
                state.catalog_kind.is_none(),
            );
            for kind in [
                hearth_core::CardKind::Minion,
                hearth_core::CardKind::Spell,
                hearth_core::CardKind::Weapon,
                hearth_core::CardKind::Location,
                hearth_core::CardKind::Hero,
            ] {
                spawn_filter_button(
                    row,
                    kind_label(locale, kind),
                    UiAction::FilterCatalogKind(Some(kind)),
                    state.catalog_kind == Some(kind),
                );
            }
        });
}

fn sync_catalog_search_input(
    event: On<TextEditChange>,
    query: Query<&EditableText, With<CatalogSearchInput>>,
    mut state: ResMut<FrontendState>,
) {
    if let Ok(input) = query.get(event.event_target()) {
        state.catalog_query = editable_text_value(input);
    }
}

fn sync_deck_name_input(
    event: On<TextEditChange>,
    query: Query<&EditableText, With<DeckNameInput>>,
    mut state: ResMut<FrontendState>,
) {
    let Ok(input) = query.get(event.event_target()) else {
        return;
    };
    if let Some(draft) = state.draft.as_mut() {
        draft.name = editable_text_value(input);
    }
}

fn sync_deck_code_input(
    event: On<TextEditChange>,
    query: Query<&EditableText, With<DeckCodeInput>>,
    mut state: ResMut<FrontendState>,
) {
    if let Ok(input) = query.get(event.event_target()) {
        state.deck_code = editable_text_value(input);
    }
}

fn editable_text_value(input: &EditableText) -> String {
    input.value().to_string()
}

fn spawn_selected_deck(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    stored: Option<&hearth_app::StoredDeck>,
    color: Color,
    locale: hearth_core::Locale,
) {
    let value = stored
        .map(|stored| {
            format!(
                "{label}\n{}\n{} · {} {}",
                stored.deck.name,
                class_label(locale, &stored.deck.class),
                stored.deck.cards.len(),
                pick(locale, "cards", "张卡牌", "張卡牌")
            )
        })
        .unwrap_or_else(|| {
            format!(
                "{label}\n{}",
                pick(locale, "No deck selected", "尚未选择套牌", "尚未選擇牌組")
            )
        });
    parent.spawn((
        Text::new(value),
        text_font(15.0),
        TextColor(TEXT),
        Node {
            width: percent(50),
            min_height: px(70),
            padding: UiRect::all(px(10)),
            ..default()
        },
        BackgroundColor(color),
        Pickable::IGNORE,
    ));
}

fn spawn_page_controls(
    parent: &mut ChildSpawnerCommands,
    page: usize,
    pages: usize,
    previous: UiAction,
    next: UiAction,
    locale: hearth_core::Locale,
) {
    parent
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            ..default()
        })
        .with_children(|controls| {
            if page > 0 {
                spawn_small_button(
                    controls,
                    pick(locale, "← Previous", "← 上一页", "← 上一頁"),
                    previous,
                    ACTION,
                );
            }
            controls.spawn((
                Text::new(match locale {
                    hearth_core::Locale::EnUs => format!("Page {}/{}", page + 1, pages),
                    hearth_core::Locale::ZhCn => format!("第 {}/{} 页", page + 1, pages),
                    hearth_core::Locale::ZhTw => format!("第 {}/{} 頁", page + 1, pages),
                }),
                text_font(13.0),
                TextColor(MUTED_TEXT),
                Pickable::IGNORE,
            ));
            if page + 1 < pages {
                spawn_small_button(
                    controls,
                    pick(locale, "Next →", "下一页 →", "下一頁 →"),
                    next,
                    ACTION,
                );
            }
        });
}

fn spawn_frontend_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: UiAction,
    normal: Color,
    width: f32,
) {
    parent
        .spawn((
            Button,
            action,
            ButtonColors {
                normal,
                hovered: ACTION_HOVER,
                pressed: CARD_SELECTED,
            },
            Node {
                width: px(width),
                min_height: px(48),
                padding: UiRect::all(px(10)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(8)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::srgb(0.62, 0.58, 0.42)),
            BackgroundColor(normal),
        ))
        .observe(handle_ui_click)
        .with_child((
            Text::new(label),
            text_font(16.0),
            TextColor(TEXT),
            Pickable::IGNORE,
        ));
}

fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: UiAction,
    normal: Color,
) {
    parent
        .spawn((
            Button,
            action,
            ButtonColors {
                normal,
                hovered: ACTION_HOVER,
                pressed: CARD_SELECTED,
            },
            Node {
                min_width: px(76),
                min_height: px(36),
                padding: UiRect::axes(px(9), px(6)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::srgb(0.34, 0.46, 0.56)),
            BackgroundColor(normal),
        ))
        .observe(handle_ui_click)
        .with_child((
            Text::new(label),
            text_font(13.0),
            TextColor(TEXT),
            Pickable::IGNORE,
        ));
}

fn spawn_filter_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: UiAction,
    active: bool,
) {
    let normal = if active { CARD_SELECTED } else { ACTION };
    parent
        .spawn((
            Button,
            action,
            ButtonColors {
                normal,
                hovered: ACTION_HOVER,
                pressed: CARD_SELECTED,
            },
            Node {
                min_width: px(54),
                min_height: px(28),
                padding: UiRect::axes(px(7), px(3)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(4)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::srgb(0.34, 0.46, 0.56)),
            BackgroundColor(normal),
        ))
        .observe(handle_ui_click)
        .with_child((
            Text::new(label),
            text_font(11.0),
            TextColor(if active { BACKGROUND } else { TEXT }),
            Pickable::IGNORE,
        ));
}

fn spawn_status(parent: &mut ChildSpawnerCommands, status: &str, locale: hearth_core::Locale) {
    let error_label = pick(locale, "Error", "错误", "錯誤");
    parent.spawn((
        Text::new(status),
        text_font(14.0),
        TextColor(
            if status.starts_with("Error") || status.starts_with(error_label) {
                Color::srgb(1.0, 0.42, 0.34)
            } else {
                CARD_SELECTED
            },
        ),
        Pickable::IGNORE,
    ));
}

fn deck_counts(deck: &DeckList) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for card in &deck.cards {
        *counts.entry(card.clone()).or_default() += 1;
    }
    counts
}

fn draft_area_counts(state: &FrontendState, deck: &DeckList) -> BTreeMap<String, usize> {
    let Some(owner) = state.draft_sideboard_owner.as_deref() else {
        return deck_counts(deck);
    };
    let mut counts = BTreeMap::new();
    if let Some(sideboard) = deck
        .sideboards
        .iter()
        .find(|sideboard| sideboard.owner == owner)
    {
        for card in &sideboard.cards {
            *counts.entry(card.clone()).or_default() += 1;
        }
    }
    counts
}

fn card_allowed(card: &CardCatalogEntry, draft: &DeckList) -> bool {
    draft.unrestricted
        || card.class == "neutral"
        || card.class == draft.class
        || card.classes.iter().any(|class| class == &draft.class)
        || draft.cards.contains(&card.id)
}

fn card_matches_filters(
    card: &CardCatalogEntry,
    draft: &DeckList,
    state: &FrontendState,
    library: &DeckLibrary,
) -> bool {
    let already_present = draft.cards.contains(&card.id)
        || draft
            .sideboards
            .iter()
            .any(|sideboard| sideboard.cards.contains(&card.id));
    card_allowed(card, draft)
        && (already_present || library.card_fits_deck_runes(draft, card))
        && catalog_filter_matches(card, state.catalog_cost, state.catalog_kind)
        && card_matches_query(card, &state.catalog_query)
}

fn card_matches_query(card: &CardCatalogEntry, query: &str) -> bool {
    let haystack = format!(
        "{} {} {} {} {} {} {} {}",
        card.name,
        card.text,
        card.id,
        card.set,
        card.class,
        card.classes.join(" "),
        card.keywords.join(" "),
        card_rune_query_terms(card.rune_cost)
    )
    .to_lowercase();
    query
        .split_whitespace()
        .all(|term| haystack.contains(&term.to_lowercase()))
}

fn card_rune_query_terms(runes: hearth_core::RuneCost) -> String {
    if runes.is_empty() {
        return String::new();
    }
    format!(
        "{} {} {} runes:{}",
        "blood ".repeat(usize::from(runes.blood)),
        "frost ".repeat(usize::from(runes.frost)),
        "unholy ".repeat(usize::from(runes.unholy)),
        card_rune_label(runes).to_lowercase()
    )
}

fn card_rune_label(runes: hearth_core::RuneCost) -> String {
    format!(
        "{}{}{}",
        "B".repeat(usize::from(runes.blood)),
        "F".repeat(usize::from(runes.frost)),
        "U".repeat(usize::from(runes.unholy))
    )
}

fn card_rune_suffix(runes: hearth_core::RuneCost) -> String {
    if runes.is_empty() {
        String::new()
    } else {
        format!(" · {}", card_rune_label(runes))
    }
}

fn deck_rune_label(runes: hearth_core::RuneCost) -> String {
    let mut label = card_rune_label(runes);
    label.push_str(&"-".repeat(usize::from(
        hearth_core::RuneCost::SLOTS.saturating_sub(runes.total()),
    )));
    label
}

fn catalog_filter_matches(
    card: &CardCatalogEntry,
    cost: Option<u8>,
    kind: Option<hearth_core::CardKind>,
) -> bool {
    let cost_matches = match cost {
        None => true,
        Some(7) => card.cost >= 7,
        Some(cost) => card.cost == cost,
    };
    cost_matches && kind.is_none_or(|kind| card.kind == kind)
}

fn deck_summary(library: &DeckLibrary, deck: &DeckList, locale: hearth_core::Locale) -> String {
    let mut minions = 0;
    let mut spells = 0;
    let mut weapons = 0;
    let mut locations = 0;
    let mut other = 0;
    let mut curve = [0usize; 8];
    for card_id in &deck.cards {
        let Some(card) = library.cards().iter().find(|card| card.id == *card_id) else {
            continue;
        };
        curve[usize::from(card.cost.min(7))] += 1;
        match card.kind {
            hearth_core::CardKind::Minion => minions += 1,
            hearth_core::CardKind::Spell => spells += 1,
            hearth_core::CardKind::Weapon => weapons += 1,
            hearth_core::CardKind::Location => locations += 1,
            _ => other += 1,
        }
    }
    match locale {
        hearth_core::Locale::EnUs => format!(
            "Minions {minions} · Spells {spells} · Weapons {weapons} · Locations {locations} · Other {other}\nMana 0:{}  1:{}  2:{}  3:{}  4:{}  5:{}  6:{}  7+:{}",
            curve[0], curve[1], curve[2], curve[3], curve[4], curve[5], curve[6], curve[7]
        ),
        hearth_core::Locale::ZhCn => format!(
            "随从 {minions} · 法术 {spells} · 武器 {weapons} · 地标 {locations} · 其他 {other}\n法力曲线 0:{}  1:{}  2:{}  3:{}  4:{}  5:{}  6:{}  7+:{}",
            curve[0], curve[1], curve[2], curve[3], curve[4], curve[5], curve[6], curve[7]
        ),
        hearth_core::Locale::ZhTw => format!(
            "手下 {minions} · 法術 {spells} · 武器 {weapons} · 地標 {locations} · 其他 {other}\n法力曲線 0:{}  1:{}  2:{}  3:{}  4:{}  5:{}  6:{}  7+:{}",
            curve[0], curve[1], curve[2], curve[3], curve[4], curve[5], curve[6], curve[7]
        ),
    }
}

fn card_sort_key(library: &DeckLibrary, card_id: &str) -> (u8, String, String) {
    library
        .cards()
        .iter()
        .find(|card| card.id == card_id)
        .map(|card| (card.cost, card.name.to_lowercase(), card.id.clone()))
        .unwrap_or((0, card_id.to_owned(), card_id.to_owned()))
}

fn shorten_text(value: &str, max_chars: usize) -> String {
    let plain = value
        .replace(['\n', '\r'], " ")
        .replace(['<', '>', '$'], "");
    let mut characters = plain.chars();
    let shortened = characters.by_ref().take(max_chars).collect::<String>();
    if characters.next().is_some() {
        format!("{shortened}…")
    } else {
        shortened
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_filter_keeps_neutral_class_and_existing_cards() {
        let draft = DeckList {
            name: "Mage".to_owned(),
            format: None,
            class: "mage".to_owned(),
            cards: vec!["existing".to_owned()],
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };
        let entry = |id: &str, class: &str| CardCatalogEntry {
            id: id.to_owned(),
            name: id.to_owned(),
            text: String::new(),
            set: String::new(),
            kind: hearth_core::CardKind::Minion,
            collectible: true,
            class: class.to_owned(),
            classes: Vec::new(),
            sideboard_size: 0,
            deck_size: None,
            starting_health: None,
            rune_cost: hearth_core::RuneCost::default(),
            rarity: None,
            cost: 1,
            attack: 1,
            health: 1,
            armor: 0,
            keywords: Vec::new(),
        };
        assert!(card_allowed(&entry("neutral", "neutral"), &draft));
        assert!(card_allowed(&entry("mage", "mage"), &draft));
        assert!(card_allowed(&entry("existing", "warrior"), &draft));
        assert!(!card_allowed(&entry("other", "warrior"), &draft));
    }

    #[test]
    fn death_knight_catalog_hides_cards_that_exceed_the_current_runes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), config.locale).unwrap();
        let state = FrontendState::new(config, &library, false);
        let draft = DeckList {
            name: "Blood".to_owned(),
            format: Some("wild".to_owned()),
            class: "death_knight".to_owned(),
            cards: vec!["RLK_067".to_owned()],
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };
        let unholy = library.definition("RLK_048").unwrap();
        let triple_frost = library.definition("RLK_063").unwrap();

        assert!(card_matches_filters(unholy, &draft, &state, &library));
        assert!(!card_matches_filters(
            triple_frost,
            &draft,
            &state,
            &library
        ));
        assert_eq!(deck_rune_label(library.deck_rune_cost(&draft)), "BB-");
    }

    #[test]
    fn deck_counts_group_duplicates() {
        let deck = DeckList {
            name: "Deck".to_owned(),
            format: None,
            class: "mage".to_owned(),
            cards: vec!["a".to_owned(), "b".to_owned(), "a".to_owned()],
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };
        let counts = deck_counts(&deck);
        assert_eq!(counts["a"], 2);
        assert_eq!(counts["b"], 1);
    }

    #[test]
    fn editable_text_value_preserves_multiline_deck_exports() {
        let input = EditableText::new("### Named Deck\n# Format: Standard\nAAECAAA=");
        assert_eq!(
            editable_text_value(&input),
            "### Named Deck\n# Format: Standard\nAAECAAA="
        );
    }

    #[test]
    fn catalog_filters_support_exact_cost_seven_plus_and_kind() {
        let card = CardCatalogEntry {
            id: "card".to_owned(),
            name: "Card".to_owned(),
            text: String::new(),
            set: String::new(),
            kind: hearth_core::CardKind::Minion,
            collectible: true,
            class: "neutral".to_owned(),
            classes: Vec::new(),
            sideboard_size: 0,
            deck_size: None,
            starting_health: None,
            rune_cost: hearth_core::RuneCost::default(),
            rarity: None,
            cost: 9,
            attack: 1,
            health: 1,
            armor: 0,
            keywords: Vec::new(),
        };
        assert!(catalog_filter_matches(&card, None, None));
        assert!(catalog_filter_matches(&card, Some(7), None));
        assert!(!catalog_filter_matches(&card, Some(6), None));
        assert!(catalog_filter_matches(
            &card,
            None,
            Some(hearth_core::CardKind::Minion)
        ));
        assert!(!catalog_filter_matches(
            &card,
            None,
            Some(hearth_core::CardKind::Spell)
        ));
    }

    #[test]
    fn catalog_search_matches_multiple_localized_card_fields() {
        let card = CardCatalogEntry {
            id: "EX1_008".to_owned(),
            name: "银色侍从".to_owned(),
            text: "圣盾".to_owned(),
            set: "EXPERT1".to_owned(),
            kind: hearth_core::CardKind::Minion,
            collectible: true,
            class: "neutral".to_owned(),
            classes: Vec::new(),
            sideboard_size: 0,
            deck_size: None,
            starting_health: None,
            rune_cost: hearth_core::RuneCost::default(),
            rarity: Some("common".to_owned()),
            cost: 1,
            attack: 1,
            health: 1,
            armor: 0,
            keywords: vec!["divine_shield".to_owned()],
        };
        assert!(card_matches_query(&card, ""));
        assert!(card_matches_query(&card, "银色"));
        assert!(card_matches_query(&card, "ex1_008 shield"));
        assert!(card_matches_query(&card, "expert1 圣盾"));
        assert!(!card_matches_query(&card, "mage"));
    }

    #[test]
    fn a_new_deck_can_choose_class_only_while_empty_and_reset_to_its_baseline() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library = DeckLibrary::load(root.join("decks"), root.join("data"), config.locale)
            .expect("repository deck library should load");
        let mut state = FrontendState::new(config, &library, false);

        state.open_new_deck();
        assert_eq!(state.scene, ClientScene::DeckBuilder);
        assert_eq!(state.draft_source, None);
        assert_eq!(
            state.draft.as_ref().map(|deck| deck.class.as_str()),
            Some("mage")
        );
        assert!(state.set_draft_class("warrior"));
        let draft = state.draft.as_mut().expect("new deck exists");
        assert_eq!(draft.class, "warrior");
        assert_eq!(draft.hero_power, None);
        draft.name = "Renamed".to_owned();
        draft.cards.push("EX1_008".to_owned());
        assert!(!state.set_draft_class("rogue"));

        state.reset_draft();
        let reset = state.draft.as_ref().expect("baseline remains available");
        assert_eq!(reset.name, "New Custom Deck");
        assert_eq!(reset.class, "mage");
        assert!(reset.cards.is_empty());
    }

    #[test]
    fn repository_decks_open_as_copies_and_saved_paths_restore_both_selections() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library = DeckLibrary::load(root.join("decks"), root.join("data"), config.locale)
            .expect("repository deck library should load");
        let mut state = FrontendState::new(config, &library, false);

        state.open_builder(&library);
        assert_eq!(state.draft_source, None);
        assert!(
            state
                .draft
                .as_ref()
                .expect("repository deck draft exists")
                .name
                .ends_with(" (Custom)")
        );

        let source = library.deck(0).expect("source deck exists").path.clone();
        let saved = library.deck(1).expect("saved deck exists").path.clone();
        let other = library.deck(2).expect("other deck exists").path.clone();
        state.restore_deck_selections_after_save(&library, &saved, Some(&source), Some(&source));
        assert_eq!(state.player_deck, 1);
        assert_eq!(state.opponent_deck, 1);

        state.restore_deck_selections_after_save(&library, &source, Some(&saved), Some(&other));
        assert_eq!(state.player_deck, 0);
        assert_eq!(state.opponent_deck, 2);
    }

    #[test]
    fn deck_selection_indices_remain_valid_after_deletion() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library = DeckLibrary::load(root.join("decks"), root.join("data"), config.locale)
            .expect("repository deck library should load");
        let mut state = FrontendState::new(config, &library, false);

        state.player_deck = 9;
        state.opponent_deck = 4;
        state.deck_page = 3;
        state.pending_delete_deck = Some(4);
        state.repair_deck_indices_after_removal(4, 10);
        assert_eq!(state.player_deck, 8);
        assert_eq!(state.opponent_deck, 4);
        assert_eq!(state.deck_page, 1);
        assert_eq!(state.pending_delete_deck, None);

        state.player_deck = 9;
        state.repair_deck_indices_after_removal(9, 9);
        assert_eq!(state.player_deck, 8);
        state.repair_deck_indices_after_removal(0, 0);
        assert_eq!(state.player_deck, 0);
        assert_eq!(state.opponent_deck, 0);
    }

    #[test]
    fn hotseat_assigns_selected_decks_to_player_one_and_player_two() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut config = MatchConfig::demo(&root);
        config.match_mode = MatchMode::Hotseat;
        config.human_player = hearth_core::PlayerId::TWO;
        let library = DeckLibrary::load(root.join("decks"), root.join("data"), config.locale)
            .expect("repository deck library should load");
        let mut state = FrontendState::new(config, &library, false);
        state.player_deck = 1;
        state.opponent_deck = 2;

        state.apply_selected_decks(&library).unwrap();

        assert_eq!(state.config.deck_one, library.deck(1).unwrap().path);
        assert_eq!(state.config.deck_two, library.deck(2).unwrap().path);
    }

    #[test]
    fn an_imported_deck_opens_as_an_unsaved_builder_draft() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library = DeckLibrary::load(root.join("decks"), root.join("data"), config.locale)
            .expect("repository deck library should load");
        let mut state = FrontendState::new(config, &library, false);
        state.scene = ClientScene::DeckSelect;
        state.open_deck_code("code".to_owned());
        assert_eq!(state.deck_code_return, ClientScene::DeckSelect);

        let imported = DeckList {
            name: "Imported Quest Rogue".to_owned(),
            format: Some("standard".to_owned()),
            class: "rogue".to_owned(),
            cards: vec!["UNG_067".to_owned(); 30],
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        };
        state.open_imported_deck(imported.clone());

        assert_eq!(state.scene, ClientScene::DeckBuilder);
        let draft = state.draft.as_ref().expect("imported draft exists");
        let baseline = state
            .draft_baseline
            .as_ref()
            .expect("imported baseline exists");
        assert_eq!(draft.name, imported.name);
        assert_eq!(draft.format, imported.format);
        assert_eq!(draft.class, imported.class);
        assert_eq!(draft.cards, imported.cards);
        assert_eq!(baseline.name, draft.name);
        assert_eq!(baseline.cards, draft.cards);
        assert_eq!(state.draft_source, None);
    }

    #[test]
    fn match_progress_pauses_in_the_overlay_and_its_settings_round_trip() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library = DeckLibrary::load(root.join("decks"), root.join("data"), config.locale)
            .expect("repository deck library should load");
        let mut state = FrontendState::new(config, &library, false);

        state.scene = ClientScene::Match;
        assert!(!state.pauses_match_progress());
        state.match_menu_open = true;
        assert!(state.pauses_match_progress());

        state.scene = ClientScene::Settings;
        state.settings_return = ClientScene::Match;
        state.match_menu_open = false;
        assert!(state.pauses_match_progress());

        state.settings_return = ClientScene::MainMenu;
        assert!(!state.pauses_match_progress());
    }
}
