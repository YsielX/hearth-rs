use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode, WindowPlugin, WindowResolution};
use hearth_app::{
    AppError, BotDifficulty, DeckLibrary, DeckList, GameSession, GameSessionSnapshot, MatchConfig,
    MatchMode, export_deckstring, import_deckstring,
};
use hearth_core::{
    CardKind, EntityId, EntityView, LegalAction, PlayerCommand, PlayerId, PlayerView,
};
use serde::{Deserialize, Serialize};

use crate::battlefield_status::spawn_battlefield_status;
use crate::bot_playback::{BotPlaybackState, update_bot_playback};
use crate::card_preview::{
    CardPreviewState, InspectableCard, hide_card_preview, show_card_preview, spawn_card_preview,
    update_card_preview,
};
use crate::choice_overlay::spawn_choice_overlay;
use crate::combat_feedback::{CombatFeedbackState, update_combat_feedback};
use crate::emotes::{EmoteKind, EmoteState, update_emotes};
use crate::event_animation::{EventAnimationState, spawn_event_toast, update_event_toast};
use crate::event_log::recent_event_lines;
use crate::frontend::{ClientCatalog, ClientScene, FrontendState, spawn_frontend};
use crate::game_art::{GameArt, GameArtPlugin};
use crate::i18n::{
    bot_difficulty_label, class_label, game_over_label, interaction_error, opening_mulligan_prompt,
    opening_order_label, pick,
};
use crate::interaction::{
    ActionSource, BoardPlacement, ClickOutcome, InteractionState, activate_hero_power,
    choose_board_placement, click_entity, command_placement, command_source, drag_to_board,
    drag_to_board_placement, drag_to_entity, is_candidate_target, is_legal_source,
    selection_matches,
};
use crate::opponent_hand::spawn_opponent_hand;
use crate::player_resources::spawn_player_resources;
use crate::targeting_guide::{
    HeroPowerTargetingSource, spawn_targeting_guide, update_targeting_guide,
};
use crate::turn_timer::{TurnClock, TurnTimerConfig, spawn_turn_timer, update_turn_timer};

mod battlefield_status;
mod bot_playback;
mod card_preview;
mod choice_overlay;
mod combat_feedback;
mod emotes;
mod event_animation;
mod event_log;
mod frontend;
mod game_art;
mod i18n;
mod interaction;
mod opponent_hand;
mod player_resources;
mod targeting_guide;
mod turn_timer;

const BACKGROUND: Color = Color::srgb(0.055, 0.075, 0.105);
const PANEL: Color = Color::srgb(0.09, 0.12, 0.17);
const BOARD: Color = Color::srgb(0.24, 0.17, 0.10);
const CARD: Color = Color::srgb(0.78, 0.70, 0.52);
const FROZEN_CARD: Color = Color::srgb(0.48, 0.70, 0.80);
const CARD_SELECTED: Color = Color::srgb(0.95, 0.73, 0.20);
const SOURCE_HINT: Color = Color::srgb(0.24, 0.67, 0.39);
const TARGET_HINT: Color = Color::srgb(0.94, 0.40, 0.24);
const REPLACE_HINT: Color = Color::srgb(0.75, 0.25, 0.22);
const FRIENDLY: Color = Color::srgb(0.15, 0.32, 0.48);
const ENEMY: Color = Color::srgb(0.48, 0.18, 0.18);
const ACTION: Color = Color::srgb(0.16, 0.27, 0.38);
const ACTION_HOVER: Color = Color::srgb(0.24, 0.42, 0.57);
const TEXT: Color = Color::srgb(0.94, 0.93, 0.88);
const MUTED_TEXT: Color = Color::srgb(0.68, 0.71, 0.75);
const ACTIONS_PER_PAGE: usize = 7;

#[derive(Component)]
struct GameUiRoot;

#[derive(Component, Clone, Copy)]
struct GameEntity(EntityId);

#[derive(Component)]
struct DraggableGameEntity;

#[derive(Component)]
struct BoardDropZone;

#[derive(Component, Clone, Copy)]
struct BoardDropSlot(BoardPlacement);

#[derive(Component, Clone)]
enum UiAction {
    Dispatch(PlayerCommand),
    Entity(EntityId),
    BoardPlacement(BoardPlacement),
    HeroPower,
    ConfirmMulligan,
    ClearSelection,
    PreviousPage,
    NextPage,
    OpenMainMenu,
    QuitApplication,
    OpenSettings,
    OpenDeckSelect,
    OpenDeckBuilder,
    ContinueMatch,
    PauseMatch,
    OpenMatchMenu,
    CloseMatchMenu,
    OpenMatchSettings,
    CloseSettings,
    RequestConcede,
    ConfirmConcede,
    CancelConcede,
    ToggleEmoteMenu,
    EmitEmote(EmoteKind),
    ToggleSquelch,
    CloseEmoteMenu,
    AbandonMatch,
    CancelAbandonMatch,
    SetMatchMode(MatchMode),
    SetBotDifficulty(BotDifficulty),
    ConfirmHandoff,
    OpenDeckCode,
    CloseDeckCode,
    ExportDeckCode,
    ImportDeckCode,
    NewDeck,
    SelectPlayerDeck(usize),
    SelectOpponentDeck(usize),
    EditDeck(usize),
    DeleteDeck(usize),
    CancelDeckDelete,
    PreviousDeckPage,
    NextDeckPage,
    StartMatch,
    Rematch,
    AddDraftCard(String),
    RemoveDraftCard(String),
    EditDraftSideboard(String),
    CloseDraftSideboard,
    PreviousCatalogPage,
    NextCatalogPage,
    PreviousDraftPage,
    NextDraftPage,
    FilterCatalogCost(Option<u8>),
    FilterCatalogKind(Option<CardKind>),
    SetDraftClass(String),
    ApplyCatalogSearch,
    ClearCatalogSearch,
    ResetDraft,
    SaveDraft,
    SetLocale(hearth_core::Locale),
    SetTurnSeconds(u64),
    SetFullscreen(bool),
    SetUiScale(u16),
}

#[derive(Component, Clone, Copy)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
    pressed: Color,
}

#[derive(Resource, Default)]
struct UiState {
    interaction: InteractionState,
    dragged: Option<EntityId>,
    drag_origin: Option<Vec2>,
    page: usize,
    error: Option<String>,
    dirty: bool,
}

#[derive(Resource)]
struct ScreenshotRequest(PathBuf);

const CLIENT_SETTINGS_VERSION: u32 = 3;
const PREVIOUS_CLIENT_SETTINGS_VERSION: u32 = 2;
const LEGACY_CLIENT_SETTINGS_VERSION: u32 = 1;
const UI_SCALE_OPTIONS: [u16; 3] = [80, 100, 120];

const fn default_ui_scale_percent() -> u16 {
    100
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct PersistedClientSettings {
    version: u32,
    locale: hearth_core::Locale,
    turn_seconds: u64,
    #[serde(default)]
    bot_difficulty: BotDifficulty,
    #[serde(default)]
    fullscreen: bool,
    #[serde(default = "default_ui_scale_percent")]
    ui_scale_percent: u16,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DisplaySettings {
    pub(crate) fullscreen: bool,
    pub(crate) ui_scale_percent: u16,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            ui_scale_percent: default_ui_scale_percent(),
        }
    }
}

impl DisplaySettings {
    fn ui_scale(self) -> f32 {
        f32::from(self.ui_scale_percent) / 100.0
    }
}

#[derive(Resource)]
struct ClientSettingsStore {
    path: Option<PathBuf>,
}

#[derive(Resource)]
pub(crate) struct MatchResumeStore {
    path: Option<PathBuf>,
}

#[derive(SystemParam)]
struct FrontendOptions<'w, 's> {
    timer: ResMut<'w, TurnTimerConfig>,
    animations: ResMut<'w, EventAnimationState>,
    settings: Res<'w, ClientSettingsStore>,
    resume: ResMut<'w, MatchResumeStore>,
    display: ResMut<'w, DisplaySettings>,
    ui_scale: ResMut<'w, UiScale>,
    emotes: ResMut<'w, EmoteState>,
    primary_window: Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
}

#[derive(SystemParam)]
struct RebuildResources<'w> {
    frontend: Res<'w, FrontendState>,
    catalog: Res<'w, ClientCatalog>,
    timer: Res<'w, TurnTimerConfig>,
    art: Res<'w, GameArt>,
    display: Res<'w, DisplaySettings>,
    emotes: Res<'w, EmoteState>,
}

struct LaunchOptions {
    config: MatchConfig,
    screenshot: Option<PathBuf>,
    quick_start: bool,
    turn_seconds: u64,
    display: DisplaySettings,
    settings_path: Option<PathBuf>,
    resume_path: Option<PathBuf>,
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let options = match parse_config(&root) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let LaunchOptions {
        config,
        screenshot,
        quick_start,
        turn_seconds,
        display,
        settings_path,
        resume_path,
    } = options;
    let library = match DeckLibrary::load(root.join("decks"), &config.data_dir, config.locale) {
        Ok(library) => library,
        Err(error) => {
            eprintln!("failed to load deck library: {error}");
            std::process::exit(1);
        }
    };
    let mut frontend = FrontendState::new(config.clone(), &library, quick_start);
    let mut session = match GameSession::load(&config) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("failed to start graphical client: {error}");
            std::process::exit(1);
        }
    };
    if !quick_start && let Some(path) = resume_path.as_deref() {
        match load_match_resume(path).and_then(|snapshot| {
            snapshot
                .map(|snapshot| {
                    GameSession::from_snapshot(&config.data_dir, config.locale, &snapshot)
                        .map(Some)
                        .map_err(|error| {
                            format!("failed to restore match {}: {error}", path.display())
                        })
                })
                .unwrap_or(Ok(None))
        }) {
            Ok(Some(restored)) => {
                session = restored;
                sync_frontend_to_restored_match(&mut frontend, &library, &session);
                frontend.resume_available = true;
                frontend.status = Some(
                    pick(
                        config.locale,
                        "An unfinished match is ready to continue.",
                        "有一场未完成的对局可以继续。",
                        "有一場未完成的對戰可以繼續。",
                    )
                    .to_owned(),
                );
            }
            Ok(None) => {}
            Err(error) => frontend.status = Some(error),
        }
    }
    if quick_start && session.is_hotseat() {
        frontend.handoff_player = Some(session.human_player());
    }
    let resume_store = MatchResumeStore { path: resume_path };
    if quick_start {
        frontend.resume_available = true;
        if let Err(error) = save_match_resume(&resume_store, &session) {
            frontend.status = Some(error);
        }
    }

    let mut app = App::new();
    app.insert_non_send(session)
        .insert_resource(ClientCatalog(library))
        .insert_resource(frontend)
        .insert_resource(CardPreviewState::default())
        .insert_resource(TurnClock::default())
        .insert_resource(TurnTimerConfig {
            default_seconds: turn_seconds,
        })
        .insert_resource(UiScale(display.ui_scale()))
        .insert_resource(display)
        .insert_resource(ClientSettingsStore {
            path: settings_path,
        })
        .insert_resource(resume_store)
        .insert_resource(EventAnimationState::default())
        .insert_resource(CombatFeedbackState::default())
        .insert_resource(BotPlaybackState::default())
        .insert_resource(EmoteState::default())
        .insert_resource(UiState {
            dirty: true,
            ..default()
        })
        .insert_resource(ClearColor(BACKGROUND))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: pick(
                        config.locale,
                        "hearth-rs — Bevy client",
                        "hearth-rs — Bevy 图形客户端",
                        "hearth-rs — Bevy 圖形客戶端",
                    )
                    .to_owned(),
                    resolution: WindowResolution::new(1440, 900),
                    resizable: true,
                    mode: window_mode(display.fullscreen),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(GameArtPlugin)
        .add_plugins(TabNavigationPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_match_menu_shortcut,
                toggle_fullscreen_shortcut,
                update_bot_playback,
                update_turn_timer,
                update_emotes,
                style_buttons,
                rebuild_ui,
                update_board_drop_slots,
                update_targeting_guide,
                update_combat_feedback,
                update_card_preview,
                update_event_toast,
                capture_requested_screenshot,
            )
                .chain(),
        );
    if let Some(path) = screenshot {
        app.insert_resource(ScreenshotRequest(path));
    }
    app.run();
}

fn parse_config(root: &Path) -> Result<LaunchOptions, String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    parse_config_from(root, &args, default_settings_path(), default_resume_path())
}

fn parse_config_from(
    root: &Path,
    arguments: &[String],
    default_settings: Option<PathBuf>,
    default_resume: Option<PathBuf>,
) -> Result<LaunchOptions, String> {
    let settings_path = resolve_settings_path(arguments, default_settings)?;
    let resume_path = resolve_resume_path(arguments, default_resume)?;
    let persisted = settings_path
        .as_deref()
        .map(load_client_settings)
        .transpose()?
        .flatten();
    let mut config = MatchConfig::demo(root);
    let mut screenshot = None;
    let mut quick_start = false;
    let mut turn_seconds = persisted
        .as_ref()
        .map_or(75, |settings| settings.turn_seconds);
    let mut display = persisted
        .as_ref()
        .map_or_else(DisplaySettings::default, |settings| DisplaySettings {
            fullscreen: settings.fullscreen,
            ui_scale_percent: settings.ui_scale_percent,
        });
    if let Some(settings) = persisted.as_ref() {
        config.locale = settings.locale;
        config.bot_difficulty = settings.bot_difficulty;
    }
    let mut args = arguments.iter().cloned();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--data" => config.data_dir = required_path(&mut args, "--data")?,
            "--deck-one" => config.deck_one = required_path(&mut args, "--deck-one")?,
            "--deck-two" => config.deck_two = required_path(&mut args, "--deck-two")?,
            "--seed" => {
                config.seed = args
                    .next()
                    .ok_or_else(|| "--seed requires a value".to_owned())?
                    .parse()
                    .map_err(|_| "--seed must be a non-negative integer".to_owned())?;
            }
            "--locale" => {
                config.locale = args
                    .next()
                    .ok_or_else(|| "--locale requires a value".to_owned())?
                    .parse()?;
            }
            "--human" => {
                config.human_player = match args.next().as_deref() {
                    Some("1" | "p1" | "P1") => PlayerId::ONE,
                    Some("2" | "p2" | "P2") => PlayerId::TWO,
                    _ => return Err("--human expects 1 or 2".to_owned()),
                };
            }
            "--hotseat" => config.match_mode = MatchMode::Hotseat,
            "--bot-difficulty" => {
                config.bot_difficulty = args
                    .next()
                    .ok_or_else(|| "--bot-difficulty requires a value".to_owned())?
                    .parse()?;
            }
            "--screenshot" => screenshot = Some(required_path(&mut args, "--screenshot")?),
            "--quick-start" => quick_start = true,
            "--settings" => {
                required_path(&mut args, "--settings")?;
            }
            "--no-settings" => {}
            "--resume" => {
                required_path(&mut args, "--resume")?;
            }
            "--no-resume" => {}
            "--turn-seconds" => {
                turn_seconds = args
                    .next()
                    .ok_or_else(|| "--turn-seconds requires a value".to_owned())?
                    .parse()
                    .map_err(|_| "--turn-seconds must be a non-negative integer".to_owned())?;
            }
            "--fullscreen" => display.fullscreen = true,
            "--windowed" => display.fullscreen = false,
            "--ui-scale" => {
                display.ui_scale_percent = parse_ui_scale_percent(
                    &args
                        .next()
                        .ok_or_else(|| "--ui-scale requires a value".to_owned())?,
                )?;
            }
            "--help" | "-h" => {
                println!(
                    "hearth-client-bevy [--data PATH] [--deck-one PATH] [--deck-two PATH] \
                     [--seed N] [--locale enUS|zhCN|zhTW] [--human 1|2] [--screenshot PATH] \
                     [--quick-start] [--hotseat] [--bot-difficulty easy|normal|hard] \
                     [--turn-seconds N] \
                     [--fullscreen|--windowed] [--ui-scale 80|100|120] \
                     [--settings PATH|--no-settings] [--resume PATH|--no-resume]"
                );
                std::process::exit(0);
            }
            value => return Err(format!("unknown option {value}; use --help")),
        }
    }
    Ok(LaunchOptions {
        config,
        screenshot,
        quick_start,
        turn_seconds,
        display,
        settings_path,
        resume_path,
    })
}

fn parse_ui_scale_percent(value: &str) -> Result<u16, String> {
    let percent = value
        .parse::<u16>()
        .map_err(|_| "--ui-scale expects 80, 100, or 120".to_owned())?;
    if UI_SCALE_OPTIONS.contains(&percent) {
        Ok(percent)
    } else {
        Err("--ui-scale expects 80, 100, or 120".to_owned())
    }
}

fn window_mode(fullscreen: bool) -> WindowMode {
    if fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    }
}

fn resolve_settings_path(
    arguments: &[String],
    default_settings: Option<PathBuf>,
) -> Result<Option<PathBuf>, String> {
    let mut explicit = None;
    let mut disabled = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--settings" => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| "--settings requires a value".to_owned())?;
                explicit = Some(PathBuf::from(value));
                index += 2;
            }
            "--no-settings" => {
                disabled = true;
                index += 1;
            }
            "--quick-start" | "--hotseat" | "--no-resume" | "--fullscreen" | "--windowed"
            | "--help" | "-h" => index += 1,
            "--data" | "--deck-one" | "--deck-two" | "--seed" | "--locale" | "--human"
            | "--screenshot" | "--turn-seconds" | "--resume" | "--ui-scale"
            | "--bot-difficulty" => index += 2,
            _ => index += 1,
        }
    }
    if disabled && explicit.is_some() {
        return Err("--settings and --no-settings cannot be used together".to_owned());
    }
    Ok(if disabled {
        None
    } else {
        explicit.or(default_settings)
    })
}

fn resolve_resume_path(
    arguments: &[String],
    default_resume: Option<PathBuf>,
) -> Result<Option<PathBuf>, String> {
    let mut explicit = None;
    let mut disabled = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--resume" => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| "--resume requires a value".to_owned())?;
                explicit = Some(PathBuf::from(value));
                index += 2;
            }
            "--no-resume" => {
                disabled = true;
                index += 1;
            }
            "--quick-start" | "--hotseat" | "--no-settings" | "--fullscreen" | "--windowed"
            | "--help" | "-h" => index += 1,
            "--data" | "--deck-one" | "--deck-two" | "--seed" | "--locale" | "--human"
            | "--screenshot" | "--turn-seconds" | "--settings" | "--ui-scale"
            | "--bot-difficulty" => index += 2,
            _ => index += 1,
        }
    }
    if disabled && explicit.is_some() {
        return Err("--resume and --no-resume cannot be used together".to_owned());
    }
    Ok(if disabled {
        None
    } else {
        explicit.or(default_resume)
    })
}

fn default_settings_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".config"))
        })
        .map(|directory| directory.join("hearth-rs/client.json"))
}

fn default_resume_path() -> Option<PathBuf> {
    std::env::var_os("XDG_STATE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".local/state"))
        })
        .map(|directory| directory.join("hearth-rs/active-match.json"))
}

fn load_client_settings(path: &Path) -> Result<Option<PersistedClientSettings>, String> {
    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to read settings {}: {error}",
                path.display()
            ));
        }
    };
    let mut settings = serde_json::from_str::<PersistedClientSettings>(&json)
        .map_err(|error| format!("failed to parse settings {}: {error}", path.display()))?;
    if !matches!(
        settings.version,
        LEGACY_CLIENT_SETTINGS_VERSION | PREVIOUS_CLIENT_SETTINGS_VERSION | CLIENT_SETTINGS_VERSION
    ) {
        return Err(format!(
            "unsupported settings version {} in {}; expected {}, {}, or {}",
            settings.version,
            path.display(),
            LEGACY_CLIENT_SETTINGS_VERSION,
            PREVIOUS_CLIENT_SETTINGS_VERSION,
            CLIENT_SETTINGS_VERSION
        ));
    }
    if !UI_SCALE_OPTIONS.contains(&settings.ui_scale_percent) {
        return Err(format!(
            "unsupported UI scale {}% in {}; expected 80, 100, or 120",
            settings.ui_scale_percent,
            path.display()
        ));
    }
    settings.version = CLIENT_SETTINGS_VERSION;
    Ok(Some(settings))
}

fn save_client_settings(
    store: &ClientSettingsStore,
    locale: hearth_core::Locale,
    turn_seconds: u64,
    bot_difficulty: BotDifficulty,
    display: DisplaySettings,
) -> Result<(), String> {
    let Some(path) = store.path.as_deref() else {
        return Ok(());
    };
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create settings directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let settings = PersistedClientSettings {
        version: CLIENT_SETTINGS_VERSION,
        locale,
        turn_seconds,
        bot_difficulty,
        fullscreen: display.fullscreen,
        ui_scale_percent: display.ui_scale_percent,
    };
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("failed to serialize settings: {error}"))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("client.json");
    let temporary = path.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()));
    fs::write(&temporary, format!("{json}\n")).map_err(|error| {
        format!(
            "failed to write temporary settings {}: {error}",
            temporary.display()
        )
    })?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "failed to replace settings {}: {error}",
            path.display()
        ));
    }
    Ok(())
}

fn load_match_resume(path: &Path) -> Result<Option<GameSessionSnapshot>, String> {
    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to read saved match {}: {error}",
                path.display()
            ));
        }
    };
    serde_json::from_str(&json)
        .map(Some)
        .map_err(|error| format!("failed to parse saved match {}: {error}", path.display()))
}

fn save_match_resume(store: &MatchResumeStore, session: &GameSession) -> Result<(), String> {
    let Some(path) = store.path.as_deref() else {
        return Ok(());
    };
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create saved-match directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let json = serde_json::to_string(&session.snapshot())
        .map_err(|error| format!("failed to serialize saved match: {error}"))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("active-match.json");
    let temporary = path.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()));
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary).map_err(|error| {
        format!(
            "failed to write temporary saved match {}: {error}",
            temporary.display()
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| {
                format!(
                    "failed to protect temporary saved match {}: {error}",
                    temporary.display()
                )
            })?;
    }
    if let Err(error) = file
        .write_all(json.as_bytes())
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "failed to write temporary saved match {}: {error}",
            temporary.display()
        ));
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "failed to replace saved match {}: {error}",
            path.display()
        ));
    }
    Ok(())
}

fn clear_match_resume(store: &MatchResumeStore) -> Result<(), String> {
    let Some(path) = store.path.as_deref() else {
        return Ok(());
    };
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to remove completed saved match {}: {error}",
            path.display()
        )),
    }
}

pub(crate) fn sync_match_resume(
    store: &MatchResumeStore,
    session: &GameSession,
    frontend: &mut FrontendState,
) -> Result<(), String> {
    if session.view().outcome.is_some() {
        frontend.resume_available = false;
        frontend.pending_abandon_match = false;
        clear_match_resume(store)
    } else {
        frontend.resume_available = true;
        save_match_resume(store, session)
    }
}

fn sync_frontend_to_restored_match(
    frontend: &mut FrontendState,
    library: &DeckLibrary,
    session: &GameSession,
) {
    frontend.config.match_mode = session.match_mode();
    frontend.config.human_player = session.human_player();
    frontend.config.bot_difficulty = session.bot_difficulty();
    let deck_index = |player| {
        library
            .decks()
            .iter()
            .position(|stored| stored.deck.name == session.deck_name(player))
    };
    let first = deck_index(PlayerId::ONE);
    let second = deck_index(PlayerId::TWO);
    if session.is_hotseat() || session.human_player() == PlayerId::ONE {
        if let Some(index) = first {
            frontend.player_deck = index;
        }
        if let Some(index) = second {
            frontend.opponent_deck = index;
        }
    } else {
        if let Some(index) = second {
            frontend.player_deck = index;
        }
        if let Some(index) = first {
            frontend.opponent_deck = index;
        }
    }
}

fn required_path(args: &mut impl Iterator<Item = String>, option: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{option} requires a path"))
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.insert_resource(GameArt::load(&asset_server));
    spawn_event_toast(&mut commands);
    spawn_card_preview(&mut commands);
    spawn_targeting_guide(&mut commands);
}

fn capture_requested_screenshot(
    mut commands: Commands,
    request: Option<Res<ScreenshotRequest>>,
    mut frames: Local<u8>,
) {
    let Some(request) = request else {
        return;
    };
    *frames = frames.saturating_add(1);
    if *frames == 8 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(request.0.clone()));
    }
}

fn style_buttons(
    mut query: Query<(&Interaction, &ButtonColors, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, colors, mut background) in &mut query {
        match *interaction {
            Interaction::Pressed => *background = colors.pressed.into(),
            Interaction::Hovered => *background = colors.hovered.into(),
            Interaction::None => *background = colors.normal.into(),
        }
    }
}

fn update_board_drop_slots(
    session: NonSend<GameSession>,
    frontend: Res<FrontendState>,
    ui: Res<UiState>,
    mut slots: Query<(
        &BoardDropSlot,
        &Interaction,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    let view = session.view();
    let source = ui
        .dragged
        .map(ActionSource::Entity)
        .or(ui.interaction.source);
    let active = frontend.scene == ClientScene::Match
        && frontend.handoff_player.is_none()
        && !frontend.match_menu_open
        && view.pending_input.is_none()
        && view.outcome.is_none()
        && is_board_placement_source(&view, source);
    let legal = active.then(|| session.legal_actions().unwrap_or_default());
    for (slot, interaction, mut node, mut background, mut border) in &mut slots {
        let valid = legal.as_ref().is_some_and(|legal| {
            source.is_some_and(|source| {
                legal.iter().any(|action| {
                    command_source(&action.command) == Some(source)
                        && command_placement(&action.command) == Some(slot.0)
                })
            })
        });
        let selected = valid && ui.interaction.placement == Some(slot.0);
        node.width = px(if valid { 28 } else { 12 });
        if !valid {
            background.0 = Color::NONE;
            border.set_all(Color::NONE);
        } else if selected {
            background.0 = CARD_SELECTED.with_alpha(0.72);
            border.set_all(Color::WHITE);
        } else if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            background.0 = TARGET_HINT.with_alpha(0.76);
            border.set_all(Color::WHITE);
        } else {
            background.0 = SOURCE_HINT.with_alpha(0.52);
            border.set_all(CARD_SELECTED);
        }
    }
}

fn is_board_placement_source(view: &PlayerView, source: Option<ActionSource>) -> bool {
    let Some(ActionSource::Entity(source)) = source else {
        return false;
    };
    view.entity(source).is_some_and(|entity| {
        matches!(entity.kind, CardKind::Minion | CardKind::Location)
            && view.player(entity.controller).hand.contains(&source)
    })
}

fn handle_ui_click(
    mut event: On<Pointer<Click>>,
    mut commands: Commands,
    actions: Query<&UiAction>,
    mut session: NonSendMut<GameSession>,
    mut ui: ResMut<UiState>,
    mut frontend: ResMut<FrontendState>,
    mut catalog: ResMut<ClientCatalog>,
    mut options: FrontendOptions,
) {
    let Ok(action) = actions.get(event.event_target()) else {
        return;
    };
    let action = action.clone();
    if options.emotes.menu_open() && !is_emote_action(&action) {
        options.emotes.close_menu();
        ui.dirty = true;
        event.propagate(false);
        return;
    }
    if frontend.scene == ClientScene::Match
        && frontend.match_menu_open
        && !is_match_menu_action(&action)
    {
        event.propagate(false);
        return;
    }
    let outcome = match action {
        UiAction::Dispatch(command) => Some(ClickOutcome::Dispatch(command)),
        UiAction::Entity(entity) => match session.legal_actions() {
            Ok(legal) => {
                let view = session.view();
                if view.mulligan_eligible.is_empty() {
                    Some(click_entity(&mut ui.interaction, &legal, entity))
                } else {
                    Some(
                        ui.interaction
                            .toggle_mulligan(entity, &view.mulligan_eligible),
                    )
                }
            }
            Err(error) => Some(ClickOutcome::Invalid(error.to_string())),
        },
        UiAction::BoardPlacement(placement) => match session.legal_actions() {
            Ok(legal) if is_board_placement_source(&session.view(), ui.interaction.source) => Some(
                choose_board_placement(&mut ui.interaction, &legal, placement),
            ),
            Ok(_) => Some(ClickOutcome::Invalid(
                "choose a playable Minion or Location before choosing a board position".to_owned(),
            )),
            Err(error) => Some(ClickOutcome::Invalid(error.to_string())),
        },
        UiAction::HeroPower => match session.legal_actions() {
            Ok(legal) => Some(activate_hero_power(&mut ui.interaction, &legal)),
            Err(error) => Some(ClickOutcome::Invalid(error.to_string())),
        },
        UiAction::ConfirmMulligan => {
            let view = session.view();
            Some(ClickOutcome::Dispatch(
                ui.interaction.mulligan_command(&view.mulligan_eligible),
            ))
        }
        UiAction::ClearSelection => {
            ui.interaction.clear_selection();
            ui.page = 0;
            None
        }
        UiAction::PreviousPage => {
            ui.page = ui.page.saturating_sub(1);
            None
        }
        UiAction::NextPage => {
            ui.page = ui.page.saturating_add(1);
            None
        }
        UiAction::OpenMainMenu => {
            frontend.scene = ClientScene::MainMenu;
            frontend.handoff_player = None;
            frontend.match_menu_open = false;
            frontend.pending_concede = false;
            frontend.settings_return = ClientScene::MainMenu;
            frontend.status = None;
            frontend.pending_delete_deck = None;
            ui.interaction = InteractionState::default();
            ui.error = None;
            *options.animations = EventAnimationState::default();
            None
        }
        UiAction::QuitApplication => {
            commands.write_message(AppExit::Success);
            None
        }
        UiAction::ContinueMatch => {
            frontend.scene = ClientScene::Match;
            frontend.handoff_player = session.is_hotseat().then(|| session.human_player());
            frontend.match_menu_open = false;
            frontend.pending_concede = false;
            frontend.pending_abandon_match = false;
            frontend.status = None;
            frontend.pending_delete_deck = None;
            ui.interaction = InteractionState::default();
            ui.error = None;
            *options.animations = EventAnimationState::default();
            None
        }
        UiAction::PauseMatch => {
            let save_error = sync_match_resume(&options.resume, &session, &mut frontend).err();
            frontend.scene = ClientScene::MainMenu;
            frontend.handoff_player = None;
            frontend.match_menu_open = false;
            frontend.pending_concede = false;
            frontend.settings_return = ClientScene::MainMenu;
            frontend.pending_delete_deck = None;
            frontend.pending_abandon_match = false;
            frontend.status = Some(save_error.unwrap_or_else(|| {
                pick(
                    frontend.config.locale,
                    "Match paused. You can continue from this device.",
                    "对局已暂停；你可以在此设备上继续。",
                    "對戰已暫停；你可以在此裝置上繼續。",
                )
                .to_owned()
            }));
            ui.interaction = InteractionState::default();
            ui.error = None;
            *options.animations = EventAnimationState::default();
            None
        }
        UiAction::OpenMatchMenu => {
            if frontend.scene == ClientScene::Match
                && frontend.handoff_player.is_none()
                && session.view().outcome.is_none()
            {
                frontend.match_menu_open = true;
                frontend.pending_concede = false;
                ui.interaction = InteractionState::default();
                ui.dragged = None;
                ui.drag_origin = None;
                ui.error = None;
            }
            None
        }
        UiAction::CloseMatchMenu => {
            frontend.match_menu_open = false;
            frontend.pending_concede = false;
            ui.error = None;
            None
        }
        UiAction::OpenMatchSettings => {
            frontend.scene = ClientScene::Settings;
            frontend.settings_return = ClientScene::Match;
            frontend.pending_concede = false;
            frontend.status = None;
            ui.error = None;
            None
        }
        UiAction::CloseSettings => {
            let destination = frontend.settings_return;
            frontend.scene = destination;
            frontend.settings_return = ClientScene::MainMenu;
            frontend.status = None;
            if destination != ClientScene::Match {
                frontend.match_menu_open = false;
                frontend.pending_concede = false;
            }
            ui.error = None;
            None
        }
        UiAction::RequestConcede => {
            frontend.pending_concede = true;
            None
        }
        UiAction::ConfirmConcede => {
            if frontend.pending_concede {
                match session.concede_human() {
                    Ok(()) => {
                        frontend.match_menu_open = false;
                        frontend.pending_concede = false;
                        frontend.handoff_player = None;
                        ui.interaction.reset_after_dispatch();
                        ui.page = 0;
                        ui.error =
                            sync_match_resume(&options.resume, &session, &mut frontend).err();
                    }
                    Err(error) => ui.error = Some(error.to_string()),
                }
            }
            None
        }
        UiAction::CancelConcede => {
            frontend.pending_concede = false;
            None
        }
        UiAction::ToggleEmoteMenu => {
            if frontend.scene == ClientScene::Match
                && frontend.handoff_player.is_none()
                && !frontend.match_menu_open
                && session.view().outcome.is_none()
            {
                options.emotes.toggle_menu();
                ui.error = None;
            }
            None
        }
        UiAction::EmitEmote(kind) => {
            let player = session.human_player();
            let bot_opponent = (!session.is_hotseat()).then_some(player.opponent());
            if options.emotes.emit(player, kind, bot_opponent) {
                ui.error = None;
            } else {
                ui.error = Some("emotes are cooling down".to_owned());
            }
            None
        }
        UiAction::ToggleSquelch => {
            let viewer = session.human_player();
            options.emotes.toggle_squelch(viewer);
            options.emotes.close_menu();
            ui.error = None;
            None
        }
        UiAction::CloseEmoteMenu => {
            options.emotes.close_menu();
            None
        }
        UiAction::AbandonMatch => {
            if !frontend.pending_abandon_match {
                frontend.pending_abandon_match = true;
                frontend.status = Some(
                    pick(
                        frontend.config.locale,
                        "Abandon this unfinished match? This cannot be undone.",
                        "要放弃这场未完成的对局吗？此操作无法撤销。",
                        "要放棄這場未完成的對戰嗎？此操作無法復原。",
                    )
                    .to_owned(),
                );
            } else {
                match clear_match_resume(&options.resume) {
                    Ok(()) => {
                        frontend.resume_available = false;
                        frontend.pending_abandon_match = false;
                        frontend.status = Some(
                            pick(
                                frontend.config.locale,
                                "Saved match abandoned.",
                                "已放弃保存的对局。",
                                "已放棄儲存的對戰。",
                            )
                            .to_owned(),
                        );
                    }
                    Err(error) => frontend.status = Some(error),
                }
            }
            None
        }
        UiAction::CancelAbandonMatch => {
            frontend.pending_abandon_match = false;
            frontend.status = None;
            None
        }
        UiAction::OpenSettings => {
            frontend.settings_return = frontend.scene;
            frontend.scene = ClientScene::Settings;
            frontend.handoff_player = None;
            frontend.match_menu_open = false;
            frontend.pending_concede = false;
            frontend.status = None;
            frontend.pending_delete_deck = None;
            frontend.pending_abandon_match = false;
            ui.error = None;
            None
        }
        UiAction::OpenDeckSelect => {
            frontend.scene = ClientScene::DeckSelect;
            frontend.handoff_player = None;
            frontend.status = None;
            frontend.pending_delete_deck = None;
            frontend.pending_abandon_match = false;
            ui.error = None;
            None
        }
        UiAction::OpenDeckBuilder => {
            frontend.open_builder(&catalog.0);
            None
        }
        UiAction::SetMatchMode(mode) => {
            frontend.config.match_mode = mode;
            frontend.handoff_player = None;
            frontend.pending_delete_deck = None;
            frontend.status = Some(match (frontend.config.locale, mode) {
                (hearth_core::Locale::EnUs, MatchMode::VsBot) => {
                    "Match mode set to built-in AI.".to_owned()
                }
                (hearth_core::Locale::ZhCn, MatchMode::VsBot) => {
                    "对局模式已设为内置 AI。".to_owned()
                }
                (hearth_core::Locale::ZhTw, MatchMode::VsBot) => {
                    "對戰模式已設為內建 AI。".to_owned()
                }
                (hearth_core::Locale::EnUs, MatchMode::Hotseat) => {
                    "Local two-player mode enabled. The screen is hidden between turns.".to_owned()
                }
                (hearth_core::Locale::ZhCn, MatchMode::Hotseat) => {
                    "已启用本地双人模式；输入权切换时会隐藏画面。".to_owned()
                }
                (hearth_core::Locale::ZhTw, MatchMode::Hotseat) => {
                    "已啟用本機雙人模式；輸入權切換時會隱藏畫面。".to_owned()
                }
            });
            None
        }
        UiAction::SetBotDifficulty(difficulty) => {
            frontend.config.bot_difficulty = difficulty;
            let label = bot_difficulty_label(frontend.config.locale, difficulty);
            frontend.status = Some(match frontend.config.locale {
                hearth_core::Locale::EnUs => format!("AI difficulty set to {label}."),
                hearth_core::Locale::ZhCn => format!("AI 难度已设为{label}。"),
                hearth_core::Locale::ZhTw => format!("AI 難度已設為{label}。"),
            });
            if let Err(error) = save_client_settings(
                &options.settings,
                frontend.config.locale,
                options.timer.default_seconds,
                frontend.config.bot_difficulty,
                *options.display,
            ) {
                frontend.status = Some(settings_save_error(frontend.config.locale, &error));
            }
            None
        }
        UiAction::ConfirmHandoff => {
            frontend.handoff_player = None;
            ui.interaction = InteractionState::default();
            ui.error = None;
            None
        }
        UiAction::OpenDeckCode => {
            let locale = frontend.config.locale;
            match deck_for_export(&frontend, &catalog.0)
                .ok_or_else(|| {
                    pick(
                        locale,
                        "no deck is selected",
                        "尚未选择套牌",
                        "尚未選擇牌組",
                    )
                    .to_owned()
                })
                .and_then(|deck| {
                    export_deckstring(&catalog.0, &deck).map_err(|error| error.to_string())
                }) {
                Ok(code) => frontend.open_deck_code(code),
                Err(error) => {
                    frontend.status = Some(format!(
                        "{}: {error}",
                        pick(locale, "Error", "错误", "錯誤")
                    ));
                }
            }
            None
        }
        UiAction::CloseDeckCode => {
            frontend.scene = frontend.deck_code_return;
            frontend.status = None;
            None
        }
        UiAction::ExportDeckCode => {
            let locale = frontend.config.locale;
            match deck_for_export(&frontend, &catalog.0)
                .ok_or_else(|| {
                    pick(
                        locale,
                        "no deck is selected",
                        "尚未选择套牌",
                        "尚未選擇牌組",
                    )
                    .to_owned()
                })
                .and_then(|deck| {
                    export_deckstring(&catalog.0, &deck).map_err(|error| error.to_string())
                }) {
                Ok(code) => {
                    frontend.deck_code = code;
                    frontend.status = Some(
                        pick(
                            locale,
                            "Deck code refreshed. Use Ctrl+A and Ctrl+C in the field to copy it.",
                            "套牌代码已刷新；在输入框中按 Ctrl+A、Ctrl+C 即可复制。",
                            "牌組代碼已重新匯出；在輸入框中按 Ctrl+A、Ctrl+C 即可複製。",
                        )
                        .to_owned(),
                    );
                }
                Err(error) => {
                    frontend.status = Some(format!(
                        "{}: {error}",
                        pick(locale, "Error", "错误", "錯誤")
                    ));
                }
            }
            None
        }
        UiAction::ImportDeckCode => {
            let locale = frontend.config.locale;
            let default_name = pick(locale, "Imported Deck", "导入套牌", "匯入牌組");
            match import_deckstring(&catalog.0, &frontend.deck_code, default_name) {
                Ok(deck) => {
                    frontend.open_imported_deck(deck);
                    frontend.status = Some(
                        pick(
                            locale,
                            "Deck code imported. Review the deck, then save it as a custom deck.",
                            "套牌代码已导入；检查后保存为自定义套牌。",
                            "牌組代碼已匯入；檢查後儲存為自訂牌組。",
                        )
                        .to_owned(),
                    );
                }
                Err(error) => {
                    frontend.status = Some(format!(
                        "{}: {error}",
                        pick(locale, "Import failed", "导入失败", "匯入失敗")
                    ));
                }
            }
            None
        }
        UiAction::NewDeck => {
            frontend.open_new_deck();
            None
        }
        UiAction::SelectPlayerDeck(index) => {
            let locale = frontend.config.locale;
            let mode = frontend.config.match_mode;
            frontend.player_deck = index;
            frontend.pending_delete_deck = None;
            frontend.status = catalog.0.deck(index).map(|stored| match (locale, mode) {
                (hearth_core::Locale::EnUs, MatchMode::Hotseat) => {
                    format!("Selected {} for Player 1.", stored.deck.name)
                }
                (hearth_core::Locale::ZhCn, MatchMode::Hotseat) => {
                    format!("已选择 {} 作为玩家 1 的套牌。", stored.deck.name)
                }
                (hearth_core::Locale::ZhTw, MatchMode::Hotseat) => {
                    format!("已選擇 {} 作為玩家 1 的牌組。", stored.deck.name)
                }
                (hearth_core::Locale::EnUs, MatchMode::VsBot) => {
                    format!("Selected {} for you.", stored.deck.name)
                }
                (hearth_core::Locale::ZhCn, MatchMode::VsBot) => {
                    format!("已选择 {} 作为你的套牌。", stored.deck.name)
                }
                (hearth_core::Locale::ZhTw, MatchMode::VsBot) => {
                    format!("已選擇 {} 作為你的牌組。", stored.deck.name)
                }
            });
            None
        }
        UiAction::SelectOpponentDeck(index) => {
            let locale = frontend.config.locale;
            let mode = frontend.config.match_mode;
            frontend.opponent_deck = index;
            frontend.pending_delete_deck = None;
            frontend.status = catalog.0.deck(index).map(|stored| match (locale, mode) {
                (hearth_core::Locale::EnUs, MatchMode::Hotseat) => {
                    format!("Selected {} for Player 2.", stored.deck.name)
                }
                (hearth_core::Locale::ZhCn, MatchMode::Hotseat) => {
                    format!("已选择 {} 作为玩家 2 的套牌。", stored.deck.name)
                }
                (hearth_core::Locale::ZhTw, MatchMode::Hotseat) => {
                    format!("已選擇 {} 作為玩家 2 的牌組。", stored.deck.name)
                }
                (hearth_core::Locale::EnUs, MatchMode::VsBot) => {
                    format!("Selected {} for the AI.", stored.deck.name)
                }
                (hearth_core::Locale::ZhCn, MatchMode::VsBot) => {
                    format!("已选择 {} 作为 AI 的套牌。", stored.deck.name)
                }
                (hearth_core::Locale::ZhTw, MatchMode::VsBot) => {
                    format!("已選擇 {} 作為 AI 的牌組。", stored.deck.name)
                }
            });
            None
        }
        UiAction::EditDeck(index) => {
            frontend.player_deck = index;
            frontend.open_builder(&catalog.0);
            None
        }
        UiAction::DeleteDeck(index) => {
            let locale = frontend.config.locale;
            if frontend.pending_delete_deck != Some(index) {
                frontend.pending_delete_deck = Some(index);
                frontend.status = catalog.0.deck(index).map(|stored| match locale {
                    hearth_core::Locale::EnUs => format!(
                        "Delete {}? Confirm to permanently remove this custom deck.",
                        stored.deck.name
                    ),
                    hearth_core::Locale::ZhCn => format!(
                        "要删除 {} 吗？请再次确认以永久移除此自定义套牌。",
                        stored.deck.name
                    ),
                    hearth_core::Locale::ZhTw => format!(
                        "要刪除 {} 嗎？請再次確認以永久移除此自訂牌組。",
                        stored.deck.name
                    ),
                });
            } else {
                match catalog.0.delete_custom(index) {
                    Ok(deleted) => {
                        let remaining = catalog.0.decks().len();
                        frontend.repair_deck_indices_after_removal(index, remaining);
                        frontend.status = Some(match locale {
                            hearth_core::Locale::EnUs => {
                                format!("Deleted custom deck {}.", deleted.deck.name)
                            }
                            hearth_core::Locale::ZhCn => {
                                format!("已删除自定义套牌 {}。", deleted.deck.name)
                            }
                            hearth_core::Locale::ZhTw => {
                                format!("已刪除自訂牌組 {}。", deleted.deck.name)
                            }
                        });
                    }
                    Err(error) => {
                        frontend.pending_delete_deck = None;
                        frontend.status = Some(format!(
                            "{}: {error}",
                            pick(locale, "Error", "错误", "錯誤")
                        ));
                    }
                }
            }
            None
        }
        UiAction::CancelDeckDelete => {
            frontend.pending_delete_deck = None;
            frontend.status = None;
            None
        }
        UiAction::PreviousDeckPage => {
            frontend.pending_delete_deck = None;
            frontend.deck_page = frontend.deck_page.saturating_sub(1);
            None
        }
        UiAction::NextDeckPage => {
            frontend.pending_delete_deck = None;
            frontend.deck_page = frontend.deck_page.saturating_add(1);
            None
        }
        UiAction::StartMatch | UiAction::Rematch => {
            if let Err(message) = start_selected_match(
                &mut frontend,
                &catalog,
                &mut session,
                &mut ui,
                &mut options.animations,
            ) {
                frontend.status = Some(format!(
                    "{}: {message}",
                    pick(frontend.config.locale, "Error", "错误", "錯誤")
                ));
            } else if let Err(message) = sync_match_resume(&options.resume, &session, &mut frontend)
            {
                ui.error = Some(message);
            }
            None
        }
        UiAction::AddDraftCard(card_id) => {
            add_draft_card(&mut frontend, &catalog, &card_id);
            None
        }
        UiAction::RemoveDraftCard(card_id) => {
            remove_draft_card(&mut frontend, &card_id);
            None
        }
        UiAction::EditDraftSideboard(owner) => {
            if let Some(draft) = frontend.draft.as_mut()
                && draft.cards.contains(&owner)
            {
                if !draft
                    .sideboards
                    .iter()
                    .any(|sideboard| sideboard.owner == owner)
                {
                    draft.sideboards.push(hearth_app::DeckSideboard {
                        owner: owner.clone(),
                        cards: Vec::new(),
                    });
                }
                frontend.draft_sideboard_owner = Some(owner);
                frontend.catalog_page = 0;
                frontend.draft_page = 0;
                frontend.status = None;
            }
            None
        }
        UiAction::CloseDraftSideboard => {
            frontend.draft_sideboard_owner = None;
            frontend.catalog_page = 0;
            frontend.draft_page = 0;
            None
        }
        UiAction::PreviousCatalogPage => {
            frontend.catalog_page = frontend.catalog_page.saturating_sub(1);
            None
        }
        UiAction::NextCatalogPage => {
            frontend.catalog_page = frontend.catalog_page.saturating_add(1);
            None
        }
        UiAction::PreviousDraftPage => {
            frontend.draft_page = frontend.draft_page.saturating_sub(1);
            None
        }
        UiAction::NextDraftPage => {
            frontend.draft_page = frontend.draft_page.saturating_add(1);
            None
        }
        UiAction::FilterCatalogCost(cost) => {
            frontend.catalog_cost = cost;
            frontend.catalog_page = 0;
            None
        }
        UiAction::FilterCatalogKind(kind) => {
            frontend.catalog_kind = kind;
            frontend.catalog_page = 0;
            None
        }
        UiAction::SetDraftClass(class) => {
            let locale = frontend.config.locale;
            if frontend.set_draft_class(&class) {
                frontend.status = None;
            } else {
                frontend.status = Some(
                    pick(
                        locale,
                        "Error: remove all cards before changing class.",
                        "错误：移除全部卡牌后才能更改职业。",
                        "錯誤：移除全部卡牌後才能變更職業。",
                    )
                    .to_owned(),
                );
            }
            None
        }
        UiAction::ApplyCatalogSearch => {
            frontend.catalog_page = 0;
            frontend.status = None;
            None
        }
        UiAction::ClearCatalogSearch => {
            frontend.catalog_query.clear();
            frontend.catalog_page = 0;
            frontend.status = None;
            None
        }
        UiAction::ResetDraft => {
            frontend.reset_draft();
            None
        }
        UiAction::SaveDraft => {
            save_draft(&mut frontend, &mut catalog);
            None
        }
        UiAction::SetLocale(locale) => {
            match set_frontend_locale(&mut frontend, &mut catalog, locale) {
                Ok(()) => {
                    if frontend.resume_available {
                        match GameSession::from_snapshot(
                            &frontend.config.data_dir,
                            locale,
                            &session.snapshot(),
                        ) {
                            Ok(localized) => {
                                *session = localized;
                                if let Err(error) =
                                    sync_match_resume(&options.resume, &session, &mut frontend)
                                {
                                    frontend.status = Some(error);
                                }
                            }
                            Err(error) => {
                                frontend.status = Some(format!(
                                    "{}: {error}",
                                    pick(locale, "Error", "错误", "錯誤")
                                ));
                            }
                        }
                    }
                    if let Err(error) = save_client_settings(
                        &options.settings,
                        frontend.config.locale,
                        options.timer.default_seconds,
                        frontend.config.bot_difficulty,
                        *options.display,
                    ) {
                        frontend.status = Some(settings_save_error(locale, &error));
                    }
                }
                Err(error) => {
                    frontend.status = Some(format!(
                        "{}: {error}",
                        pick(frontend.config.locale, "Error", "错误", "錯誤")
                    ));
                }
            }
            None
        }
        UiAction::SetTurnSeconds(seconds) => {
            options.timer.default_seconds = seconds;
            frontend.status = Some(match frontend.config.locale {
                hearth_core::Locale::EnUs if seconds == 0 => {
                    "Default turn timer disabled.".to_owned()
                }
                hearth_core::Locale::ZhCn if seconds == 0 => "默认回合计时已关闭。".to_owned(),
                hearth_core::Locale::ZhTw if seconds == 0 => "預設回合計時已關閉。".to_owned(),
                hearth_core::Locale::EnUs => {
                    format!("Default turn timer set to {seconds} seconds.")
                }
                hearth_core::Locale::ZhCn => {
                    format!("默认回合计时已设为 {seconds} 秒。")
                }
                hearth_core::Locale::ZhTw => {
                    format!("預設回合計時已設為 {seconds} 秒。")
                }
            });
            if let Err(error) = save_client_settings(
                &options.settings,
                frontend.config.locale,
                options.timer.default_seconds,
                frontend.config.bot_difficulty,
                *options.display,
            ) {
                frontend.status = Some(settings_save_error(frontend.config.locale, &error));
            }
            None
        }
        UiAction::SetFullscreen(fullscreen) => {
            match options.primary_window.single_mut() {
                Ok(mut window) => {
                    window.mode = window_mode(fullscreen);
                    options.display.fullscreen = fullscreen;
                    frontend.status = Some(fullscreen_status(frontend.config.locale, fullscreen));
                    if let Err(error) = save_client_settings(
                        &options.settings,
                        frontend.config.locale,
                        options.timer.default_seconds,
                        frontend.config.bot_difficulty,
                        *options.display,
                    ) {
                        frontend.status = Some(settings_save_error(frontend.config.locale, &error));
                    }
                }
                Err(error) => {
                    frontend.status = Some(format!(
                        "{}: {error}",
                        pick(frontend.config.locale, "Error", "错误", "錯誤")
                    ));
                }
            }
            None
        }
        UiAction::SetUiScale(percent) => {
            if UI_SCALE_OPTIONS.contains(&percent) {
                options.display.ui_scale_percent = percent;
                options.ui_scale.0 = options.display.ui_scale();
                frontend.status = Some(ui_scale_status(frontend.config.locale, percent));
                if let Err(error) = save_client_settings(
                    &options.settings,
                    frontend.config.locale,
                    options.timer.default_seconds,
                    frontend.config.bot_difficulty,
                    *options.display,
                ) {
                    frontend.status = Some(settings_save_error(frontend.config.locale, &error));
                }
            } else {
                frontend.status = Some(
                    pick(
                        frontend.config.locale,
                        "Error: unsupported UI scale.",
                        "错误：不支持的界面缩放比例。",
                        "錯誤：不支援的介面縮放比例。",
                    )
                    .to_owned(),
                );
            }
            None
        }
    };
    if let Some(outcome) = outcome {
        apply_click_outcome(
            outcome,
            &mut session,
            &mut ui,
            &mut frontend,
            &options.resume,
        );
    }
    ui.dirty = true;
    event.propagate(false);
}

fn deck_for_export(frontend: &FrontendState, library: &DeckLibrary) -> Option<DeckList> {
    let source_scene = if frontend.scene == ClientScene::DeckCode {
        frontend.deck_code_return
    } else {
        frontend.scene
    };
    if source_scene == ClientScene::DeckBuilder {
        frontend.draft.clone()
    } else {
        library
            .deck(frontend.player_deck)
            .map(|stored| stored.deck.clone())
    }
}

fn settings_save_error(locale: hearth_core::Locale, error: &str) -> String {
    match locale {
        hearth_core::Locale::EnUs => {
            format!("Setting applied, but could not save it: {error}")
        }
        hearth_core::Locale::ZhCn => {
            format!("设置已应用，但无法保存：{error}")
        }
        hearth_core::Locale::ZhTw => {
            format!("設定已套用，但無法儲存：{error}")
        }
    }
}

fn is_match_menu_action(action: &UiAction) -> bool {
    matches!(
        action,
        UiAction::CloseMatchMenu
            | UiAction::OpenMatchSettings
            | UiAction::PauseMatch
            | UiAction::RequestConcede
            | UiAction::ConfirmConcede
            | UiAction::CancelConcede
    )
}

fn is_emote_action(action: &UiAction) -> bool {
    matches!(
        action,
        UiAction::ToggleEmoteMenu
            | UiAction::EmitEmote(_)
            | UiAction::ToggleSquelch
            | UiAction::CloseEmoteMenu
    )
}

fn set_frontend_locale(
    frontend: &mut FrontendState,
    catalog: &mut ClientCatalog,
    locale: hearth_core::Locale,
) -> Result<(), String> {
    let player_path = catalog
        .0
        .deck(frontend.player_deck)
        .map(|stored| stored.path.clone());
    let opponent_path = catalog
        .0
        .deck(frontend.opponent_deck)
        .map(|stored| stored.path.clone());
    catalog
        .0
        .reload_locale(&frontend.config.data_dir, locale)
        .map_err(|error| error.to_string())?;
    frontend.config.locale = locale;
    frontend.restore_deck_selections_by_path(
        &catalog.0,
        player_path.as_deref(),
        opponent_path.as_deref(),
    );
    frontend.catalog_page = 0;
    frontend.status = Some(
        pick(
            locale,
            "Language changed to English.",
            "语言已切换为简体中文。",
            "語言已切換為繁體中文。",
        )
        .to_owned(),
    );
    Ok(())
}

fn start_selected_match(
    frontend: &mut FrontendState,
    catalog: &ClientCatalog,
    session: &mut GameSession,
    ui: &mut UiState,
    animations: &mut EventAnimationState,
) -> Result<(), String> {
    frontend.apply_selected_decks(&catalog.0)?;
    let next = GameSession::load(&frontend.config).map_err(|error| error.to_string())?;
    frontend.handoff_player = next.is_hotseat().then(|| next.human_player());
    frontend.match_menu_open = false;
    frontend.pending_concede = false;
    frontend.settings_return = ClientScene::MainMenu;
    *session = next;
    *animations = EventAnimationState::default();
    ui.interaction = InteractionState::default();
    ui.page = 0;
    ui.error = None;
    frontend.status = None;
    frontend.pending_delete_deck = None;
    frontend.pending_abandon_match = false;
    frontend.scene = ClientScene::Match;
    Ok(())
}

fn add_draft_card(frontend: &mut FrontendState, catalog: &ClientCatalog, card_id: &str) {
    let locale = frontend.config.locale;
    let sideboard_owner = frontend.draft_sideboard_owner.clone();
    let Some(draft) = frontend.draft.as_mut() else {
        frontend.status = Some(
            pick(
                locale,
                "Error: no deck is being edited.",
                "错误：当前没有正在编辑的套牌。",
                "錯誤：目前沒有正在編輯的牌組。",
            )
            .to_owned(),
        );
        return;
    };
    let Some(card) = catalog.0.cards().iter().find(|card| card.id == card_id) else {
        frontend.status = Some(match locale {
            hearth_core::Locale::EnUs => format!("Error: unknown card {card_id}."),
            hearth_core::Locale::ZhCn => format!("错误：未知卡牌 {card_id}。"),
            hearth_core::Locale::ZhTw => format!("錯誤：未知卡牌 {card_id}。"),
        });
        return;
    };
    if !catalog.0.card_fits_deck_runes(draft, card) {
        let runes = catalog.0.deck_rune_cost(draft).combined(card.rune_cost);
        frontend.status = Some(match locale {
            hearth_core::Locale::EnUs => format!(
                "Error: {} would require {} Death Knight rune slots (Blood {}, Frost {}, Unholy {}).",
                card.name,
                runes.total(),
                runes.blood,
                runes.frost,
                runes.unholy
            ),
            hearth_core::Locale::ZhCn => format!(
                "错误：加入{}后需要 {} 个死亡骑士符文槽（鲜血 {}、冰霜 {}、邪恶 {}）。",
                card.name,
                runes.total(),
                runes.blood,
                runes.frost,
                runes.unholy
            ),
            hearth_core::Locale::ZhTw => format!(
                "錯誤：加入{}後需要 {} 個死亡騎士符文欄位（血魄 {}、冰霜 {}、穢邪 {}）。",
                card.name,
                runes.total(),
                runes.blood,
                runes.frost,
                runes.unholy
            ),
        });
        return;
    }
    if let Some(owner) = sideboard_owner.as_deref() {
        let capacity = catalog
            .0
            .cards()
            .iter()
            .find(|card| card.id == owner)
            .map(|card| usize::from(card.sideboard_size))
            .unwrap_or_default();
        let Some(sideboard) = draft
            .sideboards
            .iter_mut()
            .find(|sideboard| sideboard.owner == owner)
        else {
            frontend.status = Some(
                pick(
                    locale,
                    "Error: this sideboard is unavailable.",
                    "错误：此边牌区不可用。",
                    "錯誤：此備牌區無法使用。",
                )
                .to_owned(),
            );
            return;
        };
        if card_id == owner || sideboard.cards.len() >= capacity {
            frontend.status = Some(
                pick(
                    locale,
                    "Error: this band is already complete or cannot contain its owner.",
                    "错误：乐队已组满，且不能包含其所有者。",
                    "錯誤：樂團已組滿，且不能包含其擁有者。",
                )
                .to_owned(),
            );
            return;
        }
    } else {
        let capacity = catalog
            .0
            .required_deck_size(draft)
            .max(card.deck_size.map(usize::from).unwrap_or_default());
        if draft.cards.len() < capacity {
            // Continue through the shared copy-limit checks below.
        } else {
            frontend.status = Some(match locale {
                hearth_core::Locale::EnUs => {
                    format!("Error: this constructed deck contains {capacity} cards.")
                }
                hearth_core::Locale::ZhCn => {
                    format!("错误：此构筑套牌容量为 {capacity} 张。")
                }
                hearth_core::Locale::ZhTw => {
                    format!("錯誤：此構築牌組容量為 {capacity} 張。")
                }
            });
            return;
        }
    }
    let copies = draft
        .cards
        .iter()
        .chain(
            draft
                .sideboards
                .iter()
                .flat_map(|sideboard| sideboard.cards.iter()),
        )
        .filter(|id| id.as_str() == card_id)
        .count();
    let maximum = if draft.unrestricted {
        30
    } else if card.rarity.as_deref() == Some("legendary") {
        1
    } else {
        2
    };
    if copies >= maximum {
        frontend.status = Some(match locale {
            hearth_core::Locale::EnUs => {
                format!("Error: {} has a {maximum}-copy limit.", card.name)
            }
            hearth_core::Locale::ZhCn => {
                format!("错误：{} 最多可加入 {maximum} 张。", card.name)
            }
            hearth_core::Locale::ZhTw => {
                format!("錯誤：{} 最多可加入 {maximum} 張。", card.name)
            }
        });
        return;
    }
    if let Some(owner) = sideboard_owner {
        draft
            .sideboards
            .iter_mut()
            .find(|sideboard| sideboard.owner == owner)
            .unwrap()
            .cards
            .push(card_id.to_owned());
    } else {
        draft.cards.push(card_id.to_owned());
    }
    frontend.status = None;
}

fn remove_draft_card(frontend: &mut FrontendState, card_id: &str) {
    let Some(draft) = frontend.draft.as_mut() else {
        return;
    };
    if let Some(owner) = frontend.draft_sideboard_owner.as_deref() {
        if let Some(sideboard) = draft
            .sideboards
            .iter_mut()
            .find(|sideboard| sideboard.owner == owner)
            && let Some(position) = sideboard.cards.iter().rposition(|card| card == card_id)
        {
            sideboard.cards.remove(position);
            frontend.status = None;
        }
        return;
    }
    if let Some(position) = draft.cards.iter().rposition(|card| card == card_id) {
        draft.cards.remove(position);
        if !draft.cards.iter().any(|card| card == card_id) {
            draft
                .sideboards
                .retain(|sideboard| sideboard.owner != card_id);
        }
        frontend.status = None;
    }
}

fn save_draft(frontend: &mut FrontendState, catalog: &mut ClientCatalog) {
    let locale = frontend.config.locale;
    let Some(draft) = frontend.draft.clone() else {
        frontend.status = Some(
            pick(
                locale,
                "Error: no deck is being edited.",
                "错误：当前没有正在编辑的套牌。",
                "錯誤：目前沒有正在編輯的牌組。",
            )
            .to_owned(),
        );
        return;
    };
    let source = frontend.draft_source.clone();
    let opponent_path = catalog
        .0
        .deck(frontend.opponent_deck)
        .map(|stored| stored.path.clone());
    let result = if let Some(source) = source.as_deref() {
        catalog.0.replace_custom(source, &draft)
    } else {
        catalog.0.save_custom(&draft)
    };
    match result {
        Ok(path) => {
            frontend.draft_baseline = Some(draft.clone());
            frontend.draft_source = Some(path.clone());
            frontend.restore_deck_selections_after_save(
                &catalog.0,
                &path,
                source.as_deref(),
                opponent_path.as_deref(),
            );
            frontend.status = Some(match locale {
                hearth_core::Locale::EnUs => {
                    format!("Saved custom deck to {}.", path.display())
                }
                hearth_core::Locale::ZhCn => {
                    format!("自定义套牌已保存到 {}。", path.display())
                }
                hearth_core::Locale::ZhTw => {
                    format!("自訂牌組已儲存到 {}。", path.display())
                }
            });
        }
        Err(AppError::DeckNameConflict(_)) => {
            frontend.status = Some(
                pick(
                    locale,
                    "Error: another custom deck already uses that name.",
                    "错误：已有另一副自定义套牌使用该名称。",
                    "錯誤：已有另一副自訂牌組使用該名稱。",
                )
                .to_owned(),
            );
        }
        Err(error) => {
            frontend.status = Some(format!(
                "{}: {error}",
                pick(locale, "Error", "错误", "錯誤")
            ));
        }
    }
}

fn handle_drag_start(
    mut event: On<Pointer<DragStart>>,
    draggable: Query<(&GameEntity, &ComputedNode, &UiGlobalTransform), With<DraggableGameEntity>>,
    mut visuals: Query<(&mut Outline, &mut GlobalZIndex, &mut Pickable)>,
    mut ui: ResMut<UiState>,
) {
    let Ok((game, computed, global)) = draggable.get(event.event_target()) else {
        return;
    };
    ui.dragged = Some(game.0);
    ui.drag_origin = Some(global.affine().translation * computed.inverse_scale_factor);
    if let Ok((mut outline, mut z_index, mut pickable)) = visuals.get_mut(event.event_target()) {
        outline.color = Color::WHITE;
        z_index.0 = 50;
        *pickable = Pickable::IGNORE;
    }
    event.propagate(false);
}

fn handle_drag(
    mut event: On<Pointer<Drag>>,
    draggable: Query<(), With<DraggableGameEntity>>,
    mut transforms: Query<&mut UiTransform>,
) {
    if draggable.get(event.event_target()).is_err() {
        return;
    }
    if let Ok(mut transform) = transforms.get_mut(event.event_target()) {
        transform.translation = Val2::px(event.distance.x, event.distance.y);
    }
    event.propagate(false);
}

fn handle_drag_end(
    mut event: On<Pointer<DragEnd>>,
    draggable: Query<&GameEntity, With<DraggableGameEntity>>,
    mut visuals: Query<(
        &mut UiTransform,
        &mut Outline,
        &mut GlobalZIndex,
        &mut Pickable,
    )>,
    mut ui: ResMut<UiState>,
) {
    let Ok(game) = draggable.get(event.event_target()) else {
        return;
    };
    if ui.dragged == Some(game.0) {
        ui.dragged = None;
        ui.drag_origin = None;
    }
    if let Ok((mut transform, mut outline, mut z_index, mut pickable)) =
        visuals.get_mut(event.event_target())
    {
        transform.translation = Val2::ZERO;
        outline.color = Color::NONE;
        z_index.0 = 0;
        *pickable = Pickable::default();
    }
    event.propagate(false);
}

#[derive(SystemParam)]
struct DragDropFrontend<'w> {
    frontend: ResMut<'w, FrontendState>,
    resume: Res<'w, MatchResumeStore>,
}

#[derive(SystemParam)]
struct DragDropTargets<'w, 's> {
    draggable: Query<'w, 's, &'static GameEntity, With<DraggableGameEntity>>,
    game_entities: Query<'w, 's, &'static GameEntity>,
    board_slots: Query<'w, 's, &'static BoardDropSlot>,
    board_zones: Query<'w, 's, (), With<BoardDropZone>>,
}

fn handle_drag_drop(
    mut event: On<Pointer<DragDrop>>,
    targets: DragDropTargets,
    mut session: NonSendMut<GameSession>,
    mut ui: ResMut<UiState>,
    client: DragDropFrontend,
) {
    let DragDropFrontend {
        mut frontend,
        resume,
    } = client;
    let Ok(source) = targets.draggable.get(event.dropped) else {
        return;
    };
    let outcome = match session.legal_actions() {
        Ok(legal) => {
            if let Ok(target) = targets.game_entities.get(event.event_target()) {
                drag_to_entity(&mut ui.interaction, &legal, source.0, target.0)
            } else if let Ok(slot) = targets.board_slots.get(event.event_target()) {
                if is_board_placement_source(&session.view(), Some(ActionSource::Entity(source.0)))
                {
                    drag_to_board_placement(&mut ui.interaction, &legal, source.0, slot.0)
                } else {
                    drag_to_board(&mut ui.interaction, &legal, source.0)
                }
            } else if targets.board_zones.get(event.event_target()).is_ok() {
                drag_to_board(&mut ui.interaction, &legal, source.0)
            } else {
                return;
            }
        }
        Err(error) => ClickOutcome::Invalid(error.to_string()),
    };
    apply_click_outcome(outcome, &mut session, &mut ui, &mut frontend, &resume);
    ui.dirty = true;
    event.propagate(false);
}

fn apply_click_outcome(
    outcome: ClickOutcome,
    session: &mut GameSession,
    ui: &mut UiState,
    frontend: &mut FrontendState,
    resume: &MatchResumeStore,
) {
    match outcome {
        ClickOutcome::Changed => {
            ui.error = None;
            ui.page = 0;
        }
        ClickOutcome::Dispatch(command) => {
            let acting_player = session.human_player();
            if let Err(error) = session.dispatch_human_only(command) {
                ui.error = Some(error.to_string());
            } else {
                ui.error = None;
                ui.interaction.reset_after_dispatch();
                ui.page = 0;
                frontend.match_menu_open = false;
                frontend.pending_concede = false;
                frontend.handoff_player = hotseat_handoff_after_action(session, acting_player);
                if let Err(error) = sync_match_resume(resume, session, frontend) {
                    ui.error = Some(error);
                }
            }
        }
        ClickOutcome::Invalid(message) => ui.error = Some(message),
    }
}

pub(crate) fn hotseat_handoff_after_action(
    session: &GameSession,
    acting_player: PlayerId,
) -> Option<PlayerId> {
    let next_player = session.human_player();
    (session.is_hotseat() && session.view().outcome.is_none() && next_player != acting_player)
        .then_some(next_player)
}

fn fullscreen_status(locale: hearth_core::Locale, fullscreen: bool) -> String {
    if fullscreen {
        pick(
            locale,
            "Borderless fullscreen enabled. Press F11 to toggle.",
            "已启用无边框全屏；按 F11 可切换。",
            "已啟用無邊框全螢幕；按 F11 可切換。",
        )
    } else {
        pick(
            locale,
            "Windowed mode enabled. Press F11 to toggle.",
            "已启用窗口模式；按 F11 可切换。",
            "已啟用視窗模式；按 F11 可切換。",
        )
    }
    .to_owned()
}

fn ui_scale_status(locale: hearth_core::Locale, percent: u16) -> String {
    match locale {
        hearth_core::Locale::EnUs => format!("UI scale set to {percent}%."),
        hearth_core::Locale::ZhCn => format!("界面缩放已设为 {percent}%。"),
        hearth_core::Locale::ZhTw => format!("介面縮放已設為 {percent}%。"),
    }
}

fn handle_match_menu_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    session: NonSend<GameSession>,
    mut frontend: ResMut<FrontendState>,
    mut emotes: ResMut<EmoteState>,
    mut ui: ResMut<UiState>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    if emotes.close_menu() {
        ui.dirty = true;
        return;
    }
    if frontend.scene == ClientScene::Settings && frontend.settings_return == ClientScene::Match {
        frontend.scene = ClientScene::Match;
        frontend.settings_return = ClientScene::MainMenu;
        frontend.match_menu_open = true;
        frontend.pending_concede = false;
        frontend.status = None;
        ui.error = None;
        ui.dirty = true;
        return;
    }
    if frontend.scene != ClientScene::Match
        || frontend.handoff_player.is_some()
        || session.view().outcome.is_some()
    {
        return;
    }
    if frontend.pending_concede {
        frontend.pending_concede = false;
    } else {
        frontend.match_menu_open = !frontend.match_menu_open;
    }
    if frontend.match_menu_open {
        ui.interaction = InteractionState::default();
        ui.dragged = None;
        ui.drag_origin = None;
    }
    ui.error = None;
    ui.dirty = true;
}

fn toggle_fullscreen_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut display: ResMut<DisplaySettings>,
    settings: Res<ClientSettingsStore>,
    timer: Res<TurnTimerConfig>,
    mut frontend: ResMut<FrontendState>,
    mut ui: ResMut<UiState>,
) {
    if !keyboard.just_pressed(KeyCode::F11) {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let fullscreen = !display.fullscreen;
    window.mode = window_mode(fullscreen);
    display.fullscreen = fullscreen;
    frontend.status = Some(fullscreen_status(frontend.config.locale, fullscreen));
    if let Err(error) = save_client_settings(
        &settings,
        frontend.config.locale,
        timer.default_seconds,
        frontend.config.bot_difficulty,
        *display,
    ) {
        let message = settings_save_error(frontend.config.locale, &error);
        if frontend.scene == ClientScene::Match {
            ui.error = Some(message);
        } else {
            frontend.status = Some(message);
        }
    }
    ui.dirty = true;
}

fn rebuild_ui(
    mut commands: Commands,
    session: NonSend<GameSession>,
    mut ui: ResMut<UiState>,
    resources: RebuildResources,
    roots: Query<Entity, With<GameUiRoot>>,
) {
    let RebuildResources {
        frontend,
        catalog,
        timer,
        art,
        display,
        emotes,
    } = resources;
    if !ui.dirty {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    if frontend.scene != ClientScene::Match {
        commands
            .spawn((
                GameUiRoot,
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(BACKGROUND),
            ))
            .with_children(|root| spawn_frontend(root, &frontend, &catalog, &timer, &display));
        ui.dirty = false;
        return;
    }
    if let Some(player) = frontend.handoff_player {
        commands
            .spawn((
                GameUiRoot,
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(BACKGROUND),
            ))
            .with_children(|root| spawn_handoff_screen(root, &session, player));
        ui.dirty = false;
        return;
    }
    let view = session.view();
    let legal = match session.legal_actions() {
        Ok(actions) => actions,
        Err(error) => {
            ui.error = Some(error.to_string());
            Vec::new()
        }
    };
    if let Some(ActionSource::Entity(entity)) = ui.interaction.source
        && view.entity(entity).is_none()
    {
        ui.interaction.clear_selection();
    }
    ui.interaction
        .mulligan_replace
        .retain(|entity| view.mulligan_eligible.contains(entity));

    commands
        .spawn((
            GameUiRoot,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ImageNode::new(art.tavern_board.clone()).with_mode(NodeImageMode::Stretch),
            BackgroundColor(BACKGROUND),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(78),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(10)),
                    row_gap: px(7),
                    ..default()
                },
                BackgroundColor(BOARD.with_alpha(0.22)),
            ))
            .with_children(|board| {
                spawn_board(
                    board,
                    &session,
                    &view,
                    &legal,
                    &ui.interaction,
                    &art,
                    &emotes,
                )
            });

            root.spawn((
                Node {
                    width: percent(22),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(12)),
                    row_gap: px(8),
                    ..default()
                },
                BackgroundColor(PANEL.with_alpha(0.97)),
            ))
            .with_children(|panel| {
                spawn_action_panel(panel, &session, &view, &legal, &mut ui);
            });

            if frontend.match_menu_open {
                spawn_match_menu_overlay(root, &session, &frontend);
            } else if emotes.menu_open() {
                spawn_emote_overlay(root, &session, &emotes);
            }
        });
    ui.dirty = false;
}

fn spawn_match_menu_overlay(
    root: &mut ChildSpawnerCommands,
    session: &GameSession,
    frontend: &FrontendState,
) {
    let locale = session.locale();
    root.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(px(24)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.025, 0.045, 0.80)),
        GlobalZIndex(240),
    ))
    .with_children(|overlay| {
        overlay
            .spawn((
                Node {
                    width: px(460),
                    min_height: px(390),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(14),
                    padding: UiRect::all(px(28)),
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(18)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.055, 0.075, 0.105, 0.98)),
                BorderColor::all(Color::srgb(0.68, 0.54, 0.25)),
            ))
            .with_children(|menu| {
                menu.spawn((
                    Text::new(pick(locale, "GAME MENU", "对局菜单", "對戰選單")),
                    text_font(34.0),
                    TextColor(CARD_SELECTED),
                    Pickable::IGNORE,
                ));
                if frontend.pending_concede {
                    menu.spawn((
                        Text::new(pick(
                            locale,
                            "Concede this match? Your opponent will win immediately.",
                            "确认投降吗？对手将立即获胜。",
                            "確定投降嗎？對手將立即獲勝。",
                        )),
                        text_font(18.0),
                        TextColor(TEXT),
                        TextLayout::justify(Justify::Center),
                        Pickable::IGNORE,
                    ));
                    spawn_action_button(
                        menu,
                        pick(locale, "CONFIRM CONCEDE", "确认投降", "確認投降"),
                        UiAction::ConfirmConcede,
                    );
                    spawn_action_button(
                        menu,
                        pick(locale, "CANCEL", "取消", "取消"),
                        UiAction::CancelConcede,
                    );
                } else {
                    menu.spawn((
                        Text::new(pick(
                            locale,
                            "The turn timer and AI are paused while this menu is open.",
                            "菜单打开期间，回合计时与 AI 均会暂停。",
                            "選單開啟期間，回合計時與 AI 均會暫停。",
                        )),
                        text_font(15.0),
                        TextColor(MUTED_TEXT),
                        TextLayout::justify(Justify::Center),
                        Pickable::IGNORE,
                    ));
                    spawn_action_button(
                        menu,
                        pick(locale, "RESUME", "继续对局", "繼續對戰"),
                        UiAction::CloseMatchMenu,
                    );
                    spawn_action_button(
                        menu,
                        pick(locale, "SETTINGS", "设置", "設定"),
                        UiAction::OpenMatchSettings,
                    );
                    spawn_action_button(
                        menu,
                        pick(
                            locale,
                            "PAUSE TO MAIN MENU",
                            "暂停并返回主菜单",
                            "暫停並返回主選單",
                        ),
                        UiAction::PauseMatch,
                    );
                    spawn_action_button(
                        menu,
                        pick(locale, "CONCEDE", "投降", "投降"),
                        UiAction::RequestConcede,
                    );
                }
                menu.spawn((
                    Text::new(pick(locale, "Esc: back", "Esc：返回", "Esc：返回")),
                    text_font(13.0),
                    TextColor(MUTED_TEXT),
                    Pickable::IGNORE,
                ));
            });
    });
}

fn spawn_emote_overlay(
    root: &mut ChildSpawnerCommands,
    session: &GameSession,
    emotes: &EmoteState,
) {
    let locale = session.locale();
    let viewer = session.human_player();
    let cooldown = emotes.cooldown_remaining(viewer);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(150),
            bottom: px(165),
            width: px(820),
            min_height: px(220),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            row_gap: px(10),
            padding: UiRect::all(px(18)),
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::all(px(18)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.055, 0.075, 0.105, 0.98)),
        BorderColor::all(Color::srgb(0.68, 0.54, 0.25)),
        GlobalZIndex(220),
    ))
    .with_children(|menu| {
        menu.spawn((
            Text::new(if cooldown > 0.0 {
                match locale {
                    hearth_core::Locale::EnUs => {
                        format!("EMOTES — ready in {:.1}s", cooldown.ceil())
                    }
                    hearth_core::Locale::ZhCn => {
                        format!("英雄表情 — {:.1} 秒后可用", cooldown.ceil())
                    }
                    hearth_core::Locale::ZhTw => {
                        format!("英雄表情 — {:.1} 秒後可用", cooldown.ceil())
                    }
                }
            } else {
                pick(locale, "CHOOSE AN EMOTE", "选择英雄表情", "選擇英雄表情").to_owned()
            }),
            text_font(22.0),
            TextColor(CARD_SELECTED),
            Node {
                width: percent(100),
                ..default()
            },
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
        for kind in EmoteKind::ALL {
            spawn_emote_choice_button(menu, kind.label(locale), UiAction::EmitEmote(kind));
        }
        spawn_emote_choice_button(
            menu,
            if emotes.is_squelched(viewer) {
                pick(locale, "UNSQUELCH OPPONENT", "取消屏蔽对手", "取消屏蔽對手")
            } else {
                pick(locale, "SQUELCH OPPONENT", "屏蔽对手", "屏蔽對手")
            },
            UiAction::ToggleSquelch,
        );
        spawn_emote_choice_button(
            menu,
            pick(locale, "CLOSE", "关闭", "關閉"),
            UiAction::CloseEmoteMenu,
        );
    });
}

fn spawn_emote_choice_button(parent: &mut ChildSpawnerCommands, label: &str, action: UiAction) {
    parent
        .spawn((
            Button,
            action,
            ButtonColors {
                normal: ACTION,
                hovered: ACTION_HOVER,
                pressed: CARD_SELECTED,
            },
            Node {
                width: px(185),
                min_height: px(44),
                padding: UiRect::all(px(7)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::srgb(0.45, 0.55, 0.64)),
            BackgroundColor(ACTION),
        ))
        .observe(handle_ui_click)
        .with_child((
            Text::new(label),
            text_font(13.0),
            TextColor(TEXT),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
}

fn spawn_handoff_screen(root: &mut ChildSpawnerCommands, session: &GameSession, player: PlayerId) {
    let locale = session.locale();
    let player_number = if player == PlayerId::ONE { 1 } else { 2 };
    let opening_order = opening_order_label(locale, session.starting_player(), player);
    root.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(22),
            padding: UiRect::all(px(32)),
            ..default()
        },
        BackgroundColor(BACKGROUND),
    ))
    .with_children(|screen| {
        screen.spawn((
            Text::new(match locale {
                hearth_core::Locale::EnUs => format!("PASS TO PLAYER {player_number}"),
                hearth_core::Locale::ZhCn => format!("请交给玩家 {player_number}"),
                hearth_core::Locale::ZhTw => format!("請交給玩家 {player_number}"),
            }),
            text_font(46.0),
            TextColor(CARD_SELECTED),
            Pickable::IGNORE,
        ));
        screen.spawn((
            Text::new(match locale {
                hearth_core::Locale::EnUs => format!(
                    "{}\n{opening_order}. The next player's hand stays hidden until they continue.",
                    session.deck_name(player),
                ),
                hearth_core::Locale::ZhCn => format!(
                    "{}\n{opening_order}。点击继续前，下一位玩家的手牌保持隐藏。",
                    session.deck_name(player),
                ),
                hearth_core::Locale::ZhTw => format!(
                    "{}\n{opening_order}。點擊繼續前，下一位玩家的手牌保持隱藏。",
                    session.deck_name(player),
                ),
            }),
            text_font(20.0),
            TextColor(TEXT),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
        screen
            .spawn(Node {
                width: px(360),
                ..default()
            })
            .with_children(|button| {
                spawn_action_button(
                    button,
                    pick(locale, "I'M READY", "我已准备好", "我已準備好"),
                    UiAction::ConfirmHandoff,
                );
            });
        screen
            .spawn(Node {
                width: px(360),
                ..default()
            })
            .with_children(|button| {
                spawn_action_button(
                    button,
                    pick(
                        locale,
                        "PAUSE TO MAIN MENU",
                        "暂停并返回主菜单",
                        "暫停並返回主選單",
                    ),
                    UiAction::PauseMatch,
                );
            });
    });
}

fn spawn_board(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    view: &PlayerView,
    legal: &[LegalAction],
    interaction: &InteractionState,
    art: &GameArt,
    emotes: &EmoteState,
) {
    let human = session.human_player();
    let enemy = human.opponent();
    let hints = InteractionHints {
        legal,
        state: interaction,
        mulligan_eligible: &view.mulligan_eligible,
    };
    spawn_player_header(parent, session, view, &hints, enemy, true, emotes);
    spawn_opponent_hand(
        parent,
        view.player(enemy).hand_size,
        session.locale(),
        &art.card_back,
    );
    spawn_zone(
        parent,
        session,
        view,
        &hints,
        &view.player(enemy).board,
        ZoneArea::EnemyBoard,
    );

    let locale = session.locale();
    let status = if let Some(outcome) = view.outcome {
        game_over_label(locale, outcome, human, view.turn)
    } else if !view.mulligan_eligible.is_empty() {
        opening_mulligan_prompt(locale, session.starting_player(), human).to_owned()
    } else if let Some(pending) = &view.pending_input {
        pending.prompt.clone()
    } else if view.input_player == human {
        match locale {
            hearth_core::Locale::EnUs => format!("Turn {} — your action", view.turn),
            hearth_core::Locale::ZhCn => format!("回合 {} — 请行动", view.turn),
            hearth_core::Locale::ZhTw => format!("回合 {} — 請行動", view.turn),
        }
    } else if view.turn == 0 {
        let order = opening_order_label(locale, session.starting_player(), human);
        match locale {
            hearth_core::Locale::EnUs => {
                format!("{order} — opponent is choosing an opening hand")
            }
            hearth_core::Locale::ZhCn => format!("{order} — 对手正在选择起手牌"),
            hearth_core::Locale::ZhTw => format!("{order} — 對手正在選擇起手牌"),
        }
    } else {
        match locale {
            hearth_core::Locale::EnUs => format!("Turn {} — opponent is acting", view.turn),
            hearth_core::Locale::ZhCn => format!("回合 {} — 对手行动中", view.turn),
            hearth_core::Locale::ZhTw => format!("回合 {} — 對手行動中", view.turn),
        }
    };
    parent
        .spawn(Node {
            width: percent(100),
            min_height: px(48),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(12),
            margin: UiRect::vertical(px(3)),
            ..default()
        })
        .with_children(|controls| {
            controls.spawn((Text::new(status), text_font(22.0), TextColor(CARD_SELECTED)));
            spawn_turn_timer(controls, locale);
            if view.outcome.is_some() {
                spawn_quick_button(
                    controls,
                    pick(locale, "REMATCH", "再来一局", "再來一局"),
                    UiAction::Rematch,
                    CARD_SELECTED,
                    None,
                );
                spawn_quick_button(
                    controls,
                    pick(locale, "MAIN MENU", "主菜单", "主選單"),
                    UiAction::OpenMainMenu,
                    ACTION,
                    None,
                );
            } else {
                if legal
                    .iter()
                    .any(|action| action.command == PlayerCommand::EndTurn)
                {
                    spawn_quick_button(
                        controls,
                        pick(locale, "END TURN", "结束回合", "結束回合"),
                        UiAction::Dispatch(PlayerCommand::EndTurn),
                        CARD_SELECTED,
                        None,
                    );
                }
                spawn_quick_button(
                    controls,
                    pick(locale, "EMOTES", "英雄表情", "英雄表情"),
                    UiAction::ToggleEmoteMenu,
                    ACTION,
                    None,
                );
                spawn_quick_button(
                    controls,
                    pick(locale, "PAUSE", "暂停", "暫停"),
                    UiAction::OpenMatchMenu,
                    ACTION,
                    None,
                );
            }
        });

    spawn_zone(
        parent,
        session,
        view,
        &hints,
        &view.player(human).board,
        ZoneArea::FriendlyBoard,
    );
    spawn_player_header(parent, session, view, &hints, human, false, emotes);
    spawn_zone(
        parent,
        session,
        view,
        &hints,
        &view.player(human).hand,
        ZoneArea::Hand,
    );
    if let Some(pending) = &view.pending_input {
        spawn_choice_overlay(parent, session, pending);
    }
}

fn spawn_player_header(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    view: &PlayerView,
    hints: &InteractionHints<'_>,
    player: PlayerId,
    enemy: bool,
    emotes: &EmoteState,
) {
    let state = view.player(player);
    let hero = view.hero(player);
    let power = view.entity(state.hero_power).unwrap();
    let title = format!(
        "{} — {}  |  {}",
        if enemy {
            pick(session.locale(), "Opponent", "对手", "對手")
        } else {
            pick(session.locale(), "You", "你", "你")
        },
        session.deck_name(player),
        class_label(session.locale(), &state.class),
    );
    parent
        .spawn((
            Node {
                width: percent(100),
                min_height: px(48),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: px(8),
                padding: UiRect::all(px(6)),
                ..default()
            },
            BackgroundColor(if enemy { ENEMY } else { FRIENDLY }),
        ))
        .with_children(|header| {
            let generic_hero_label = if enemy {
                pick(session.locale(), "Enemy Hero", "敌方英雄", "敵方英雄")
            } else {
                pick(session.locale(), "Your Hero", "你的英雄", "你的英雄")
            };
            let hero_name = if hero.card_id == "builtin_hero" {
                generic_hero_label.to_owned()
            } else {
                session.card_name(&hero.card_id)
            };
            let hero_label = format!(
                "{}\n{}/{}  +{} {}",
                hero_name,
                hero.health(),
                hero.max_health,
                hero.armor,
                pick(session.locale(), "Armor", "护甲", "護甲")
            );
            spawn_header_entity_button(
                header,
                &hero_label,
                hero.id,
                &hero.card_id,
                hints.highlight(hero.id),
            );
            if let Some(kind) = emotes.visible_for(player, session.human_player()) {
                spawn_emote_bubble(header, kind.phrase(session.locale()), enemy);
            }
            spawn_battlefield_status(header, session, view, player);
            header.spawn((
                Text::new(title),
                text_font(15.0),
                TextColor(TEXT),
                Node {
                    width: px(0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    ..default()
                },
            ));
            spawn_player_resources(header, state, session.locale());
            let power_label = format!(
                "{}\n{} {}{}",
                session.card_name(&power.card_id),
                power.cost,
                pick(session.locale(), "Mana", "法力", "法力"),
                if state.hero_power_used {
                    pick(session.locale(), " · used", " · 已使用", " · 已使用")
                } else {
                    ""
                }
            );
            if enemy {
                spawn_passive_card_button(header, &power_label, ACTION, &power.card_id);
            } else {
                spawn_quick_button(
                    header,
                    &power_label,
                    UiAction::HeroPower,
                    if hints.state.source == Some(ActionSource::HeroPower) {
                        CARD_SELECTED
                    } else if is_legal_source(hints.legal, ActionSource::HeroPower) {
                        SOURCE_HINT
                    } else {
                        ACTION
                    },
                    Some(&power.card_id),
                );
            }
        });
}

fn spawn_emote_bubble(parent: &mut ChildSpawnerCommands, phrase: &str, enemy: bool) {
    parent.spawn((
        Text::new(phrase),
        text_font(16.0),
        TextColor(TEXT),
        TextLayout::justify(Justify::Center),
        Node {
            width: px(190),
            min_height: px(42),
            padding: UiRect::all(px(8)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(14)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(if enemy {
            Color::srgba(0.28, 0.08, 0.08, 0.96)
        } else {
            Color::srgba(0.07, 0.22, 0.34, 0.96)
        }),
        BorderColor::all(CARD_SELECTED),
        Pickable::IGNORE,
    ));
}

fn spawn_zone(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    view: &PlayerView,
    hints: &InteractionHints<'_>,
    entities: &[EntityId],
    area: ZoneArea,
) {
    let mut zone = parent.spawn((
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(145),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            padding: UiRect::all(px(7)),
            ..default()
        },
        BackgroundColor(if area == ZoneArea::EnemyBoard {
            Color::srgba(0.16, 0.105, 0.08, 0.54)
        } else {
            Color::srgba(0.12, 0.16, 0.13, 0.54)
        }),
    ));
    if area != ZoneArea::Hand {
        zone.insert(BoardDropZone);
        zone.observe(handle_drag_drop);
    }
    zone.with_children(|zone| {
        if entities.is_empty() {
            zone.spawn((
                Text::new(area.label(session.locale())),
                text_font(16.0),
                TextColor(MUTED_TEXT),
                Pickable::IGNORE,
            ));
        }
        for (position, entity_id) in entities.iter().enumerate() {
            if area == ZoneArea::FriendlyBoard {
                spawn_board_drop_slot(zone, BoardPlacement::Before(position));
            }
            if let Some(entity) = view.entity(*entity_id) {
                spawn_card(zone, session, entity, hints.highlight(*entity_id));
            }
        }
        if area == ZoneArea::FriendlyBoard {
            spawn_board_drop_slot(zone, BoardPlacement::End);
        }
    });
}

fn spawn_board_drop_slot(parent: &mut ChildSpawnerCommands, placement: BoardPlacement) {
    parent
        .spawn((
            Button,
            BoardDropSlot(placement),
            UiAction::BoardPlacement(placement),
            Node {
                width: px(12),
                height: px(136),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            GlobalZIndex(15),
        ))
        .observe(handle_ui_click)
        .observe(handle_drag_drop);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ZoneArea {
    EnemyBoard,
    FriendlyBoard,
    Hand,
}

impl ZoneArea {
    fn label(self, locale: hearth_core::Locale) -> &'static str {
        match self {
            Self::EnemyBoard => pick(
                locale,
                "Enemy board — drop a card here to play it",
                "敌方战场 — 将卡牌拖到这里打出",
                "敵方戰場 — 將卡牌拖到這裡打出",
            ),
            Self::FriendlyBoard => pick(
                locale,
                "Your board — drop a card here to play it",
                "你的战场 — 将卡牌拖到这里打出",
                "你的戰場 — 將卡牌拖到這裡打出",
            ),
            Self::Hand => pick(locale, "Your hand", "你的手牌", "你的手牌"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntityHighlight {
    Normal,
    Source,
    SelectedSource,
    Target,
    SelectedTarget,
    MulliganReplace,
}

struct InteractionHints<'a> {
    legal: &'a [LegalAction],
    state: &'a InteractionState,
    mulligan_eligible: &'a [EntityId],
}

impl InteractionHints<'_> {
    fn highlight(&self, entity: EntityId) -> EntityHighlight {
        if self.state.mulligan_replace.contains(&entity) {
            EntityHighlight::MulliganReplace
        } else if self.state.source == Some(ActionSource::Entity(entity)) {
            EntityHighlight::SelectedSource
        } else if self.state.target == Some(entity) {
            EntityHighlight::SelectedTarget
        } else if is_candidate_target(self.legal, self.state.source, entity) {
            EntityHighlight::Target
        } else if self.mulligan_eligible.is_empty()
            && is_legal_source(self.legal, ActionSource::Entity(entity))
        {
            EntityHighlight::Source
        } else {
            EntityHighlight::Normal
        }
    }
}

fn highlight_color(highlight: EntityHighlight) -> Color {
    match highlight {
        EntityHighlight::Normal => PANEL,
        EntityHighlight::Source => SOURCE_HINT,
        EntityHighlight::SelectedSource => CARD_SELECTED,
        EntityHighlight::Target => TARGET_HINT,
        EntityHighlight::SelectedTarget => CARD_SELECTED,
        EntityHighlight::MulliganReplace => REPLACE_HINT,
    }
}

fn spawn_card(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    entity: &EntityView,
    highlight: EntityHighlight,
) {
    let stats = match entity.kind {
        CardKind::Minion => format!("{} / {}", entity.attack, entity.health()),
        CardKind::Location => format!(
            "{} {}",
            pick(session.locale(), "Durability", "耐久度", "耐久度"),
            entity.health()
        ),
        CardKind::Weapon => format!("{} / {}", entity.attack, entity.health()),
        _ => String::new(),
    };
    let frozen_solid = entity
        .keywords
        .iter()
        .any(|keyword| keyword == "frozen_solid");
    let keywords = runtime_keyword_labels(&entity.keywords, session.locale());
    let selected = matches!(
        highlight,
        EntityHighlight::SelectedSource
            | EntityHighlight::SelectedTarget
            | EntityHighlight::MulliganReplace
    );
    let normal = if selected {
        highlight_color(highlight)
    } else if frozen_solid {
        FROZEN_CARD
    } else {
        CARD
    };
    let border = if highlight == EntityHighlight::Normal && frozen_solid {
        Color::srgb(0.70, 0.93, 1.0)
    } else {
        highlight_color(highlight)
    };
    let mut card = parent.spawn((
        Button,
        GameEntity(entity.id),
        InspectableCard(entity.card_id.clone()),
        UiAction::Entity(entity.id),
        ButtonColors {
            normal,
            hovered: Color::srgb(0.92, 0.83, 0.62),
            pressed: CARD_SELECTED,
        },
        Node {
            width: px(128),
            height: px(154),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(7)),
            border: UiRect::all(px(if selected { 4 } else { 2 })),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BorderColor::all(border),
        BackgroundColor(normal),
        Outline {
            width: px(3),
            offset: px(2),
            color: Color::NONE,
        },
        GlobalZIndex::default(),
        Pickable::default(),
    ));
    if matches!(
        highlight,
        EntityHighlight::Source | EntityHighlight::SelectedSource
    ) {
        card.insert(DraggableGameEntity);
        card.observe(handle_drag_start)
            .observe(handle_drag)
            .observe(handle_drag_end);
    }
    card.observe(handle_ui_click)
        .observe(handle_drag_drop)
        .observe(show_card_preview)
        .observe(hide_card_preview);
    card.with_children(|card| {
        card.spawn((
            Text::new(format!(
                "{}  [{}]",
                session.card_name(&entity.card_id),
                entity.cost
            )),
            text_font(14.0),
            TextColor(BACKGROUND),
            Pickable::IGNORE,
        ));
        card.spawn((
            Text::new(shorten(&session.card_text(&entity.card_id), 95)),
            text_font(11.0),
            TextColor(Color::srgb(0.12, 0.10, 0.07)),
            Pickable::IGNORE,
        ));
        card.spawn((
            Text::new(format!("{stats}\n{keywords}\n#{}", entity.id)),
            text_font(12.0),
            TextColor(BACKGROUND),
            Pickable::IGNORE,
        ));
    });
}

fn runtime_keyword_labels(keywords: &[String], locale: hearth_core::Locale) -> String {
    keywords
        .iter()
        .map(|keyword| match keyword.as_str() {
            "frozen_solid" => pick(
                locale,
                "LOCKED THIS TURN",
                "本回合不可使用",
                "本回合無法打出",
            ),
            other => other,
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn spawn_action_panel(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    view: &PlayerView,
    legal: &[hearth_core::LegalAction],
    ui: &mut UiState,
) {
    let locale = session.locale();
    parent.spawn((
        Text::new(pick(locale, "Actions", "操作", "操作")),
        text_font(30.0),
        TextColor(TEXT),
    ));
    if view.outcome.is_none() && view.input_player != session.human_player() {
        parent.spawn((
            Text::new(pick(
                locale,
                "Opponent is thinking… Actions will play one at a time.",
                "对手正在思考……动作将逐步播放。",
                "對手正在思考……動作將逐步播放。",
            )),
            text_font(16.0),
            TextColor(CARD_SELECTED),
        ));
        return;
    }
    let mut selection = match ui.interaction.source {
        Some(ActionSource::HeroPower) => pick(
            locale,
            "Hero Power selected",
            "已选择英雄技能",
            "已選擇英雄能力",
        )
        .to_owned(),
        Some(ActionSource::Entity(entity)) => {
            let source = view
                .entity(entity)
                .map(|entity| session.card_name(&entity.card_id))
                .unwrap_or_else(|| format!("#{entity}"));
            match ui.interaction.target {
                Some(target) => {
                    let target = view
                        .entity(target)
                        .map(|entity| session.card_name(&entity.card_id))
                        .unwrap_or_else(|| format!("#{target}"));
                    format!("{source} → {target}")
                }
                None => match locale {
                    hearth_core::Locale::EnUs => {
                        format!("{source} selected — choose a target or action")
                    }
                    hearth_core::Locale::ZhCn => {
                        format!("已选择 {source} — 请选择目标或操作")
                    }
                    hearth_core::Locale::ZhTw => {
                        format!("已選擇 {source} — 請選擇目標或操作")
                    }
                },
            }
        }
        None if !view.mulligan_eligible.is_empty() => match locale {
            hearth_core::Locale::EnUs => format!(
                "{} card(s) marked for replacement",
                ui.interaction.mulligan_replace.len()
            ),
            hearth_core::Locale::ZhCn => {
                format!(
                    "已标记 {} 张待替换卡牌",
                    ui.interaction.mulligan_replace.len()
                )
            }
            hearth_core::Locale::ZhTw => {
                format!(
                    "已標記 {} 張待替換卡牌",
                    ui.interaction.mulligan_replace.len()
                )
            }
        },
        None => pick(
            locale,
            "Click or drag a green card/character to act",
            "点击或拖动绿色卡牌/角色进行操作",
            "點擊或拖動綠色卡牌/角色進行操作",
        )
        .to_owned(),
    };
    if let Some(placement) = ui.interaction.placement {
        let label = match placement {
            BoardPlacement::Before(position) => match locale {
                hearth_core::Locale::EnUs => format!("slot {}", position + 1),
                hearth_core::Locale::ZhCn => format!("第 {} 个槽位", position + 1),
                hearth_core::Locale::ZhTw => format!("第 {} 個位置", position + 1),
            },
            BoardPlacement::End => {
                pick(locale, "rightmost slot", "最右侧槽位", "最右側位置").to_owned()
            }
        };
        selection.push_str(" · ");
        selection.push_str(&label);
    }
    parent.spawn((Text::new(selection), text_font(15.0), TextColor(MUTED_TEXT)));
    if !view.mulligan_eligible.is_empty() {
        spawn_action_button(
            parent,
            &match locale {
                hearth_core::Locale::EnUs => format!(
                    "CONFIRM HAND — replace {}",
                    ui.interaction.mulligan_replace.len()
                ),
                hearth_core::Locale::ZhCn => {
                    format!(
                        "确认手牌 — 替换 {} 张",
                        ui.interaction.mulligan_replace.len()
                    )
                }
                hearth_core::Locale::ZhTw => {
                    format!(
                        "確認手牌 — 替換 {} 張",
                        ui.interaction.mulligan_replace.len()
                    )
                }
            },
            UiAction::ConfirmMulligan,
        );
    }
    if ui.interaction.source.is_some() {
        spawn_action_button(
            parent,
            pick(locale, "Show all actions", "显示全部操作", "顯示全部操作"),
            UiAction::ClearSelection,
        );
    }

    if view.mulligan_eligible.is_empty() {
        let filtered = legal
            .iter()
            .filter(|action| selection_matches(&action.command, &ui.interaction))
            .collect::<Vec<_>>();
        let pages = filtered.len().max(1).div_ceil(ACTIONS_PER_PAGE);
        ui.page = ui.page.min(pages - 1);
        let start = ui.page * ACTIONS_PER_PAGE;
        for action in filtered.iter().skip(start).take(ACTIONS_PER_PAGE) {
            let label =
                hearth_app::presentation::command_text::command_label(session, view, action);
            spawn_action_button(parent, &label, UiAction::Dispatch(action.command.clone()));
        }
        if filtered.is_empty() {
            parent.spawn((
                Text::new(pick(
                    locale,
                    "No legal action matches this selection.",
                    "没有与当前选择匹配的合法操作。",
                    "沒有與目前選擇相符的合法操作。",
                )),
                text_font(14.0),
                TextColor(MUTED_TEXT),
            ));
        }
        parent.spawn((
            Text::new(match locale {
                hearth_core::Locale::EnUs => format!("Page {}/{}", ui.page + 1, pages),
                hearth_core::Locale::ZhCn => format!("第 {}/{} 页", ui.page + 1, pages),
                hearth_core::Locale::ZhTw => format!("第 {}/{} 頁", ui.page + 1, pages),
            }),
            text_font(13.0),
            TextColor(MUTED_TEXT),
        ));
        parent
            .spawn(Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::top(px(4)),
                ..default()
            })
            .with_children(|paging| {
                if ui.page > 0 {
                    spawn_action_button(
                        paging,
                        pick(locale, "< Previous", "< 上一页", "< 上一頁"),
                        UiAction::PreviousPage,
                    );
                }
                if ui.page + 1 < pages {
                    spawn_action_button(
                        paging,
                        pick(locale, "Next >", "下一页 >", "下一頁 >"),
                        UiAction::NextPage,
                    );
                }
            });
    } else {
        ui.page = 0;
        parent.spawn((
            Text::new(pick(
                locale,
                "Selected cards use a red border. Confirm when ready.",
                "红色边框表示将替换；准备好后确认手牌。",
                "紅色邊框表示將替換；準備好後確認手牌。",
            )),
            text_font(14.0),
            TextColor(MUTED_TEXT),
        ));
    }

    if let Some(error) = &ui.error {
        parent.spawn((
            Text::new(format!(
                "{}: {}",
                pick(locale, "Error", "错误", "錯誤"),
                interaction_error(locale, error)
            )),
            text_font(14.0),
            TextColor(Color::srgb(1.0, 0.45, 0.38)),
        ));
    }
    parent.spawn((
        Text::new(pick(locale, "Battle log", "战斗日志", "戰鬥紀錄")),
        text_font(19.0),
        TextColor(TEXT),
        Node {
            margin: UiRect::top(px(7)),
            ..default()
        },
    ));
    let event_lines = recent_event_lines(session, view, 6);
    if event_lines.is_empty() {
        parent.spawn((
            Text::new(pick(
                locale,
                "Waiting for the match to start…",
                "等待对局开始……",
                "等待對戰開始……",
            )),
            text_font(12.0),
            TextColor(MUTED_TEXT),
        ));
    } else {
        for line in event_lines {
            parent.spawn((Text::new(line), text_font(12.0), TextColor(MUTED_TEXT)));
        }
    }
    parent.spawn((
        Text::new(if session.is_hotseat() {
            let first_number = if session.starting_player() == PlayerId::ONE {
                1
            } else {
                2
            };
            match locale {
                hearth_core::Locale::EnUs => format!(
                    "Turn {}\nPublic events: {}\nFirst: Player {} · Locale: {}",
                    view.turn,
                    view.history.len(),
                    first_number,
                    locale.code()
                ),
                hearth_core::Locale::ZhCn => format!(
                    "回合 {}\n公开事件：{}\n先手：玩家 {} · 语言：{}",
                    view.turn,
                    view.history.len(),
                    first_number,
                    locale.code()
                ),
                hearth_core::Locale::ZhTw => format!(
                    "回合 {}\n公開事件：{}\n先手：玩家 {} · 語言：{}",
                    view.turn,
                    view.history.len(),
                    first_number,
                    locale.code()
                ),
            }
        } else {
            let difficulty = bot_difficulty_label(locale, session.bot_difficulty());
            let order = opening_order_label(locale, session.starting_player(), view.viewer);
            match locale {
                hearth_core::Locale::EnUs => format!(
                    "Turn {}\nPublic events: {}\n{} · AI: {} · Locale: {}",
                    view.turn,
                    view.history.len(),
                    order,
                    difficulty,
                    locale.code()
                ),
                hearth_core::Locale::ZhCn => format!(
                    "回合 {}\n公开事件：{}\n{} · AI：{} · 语言：{}",
                    view.turn,
                    view.history.len(),
                    order,
                    difficulty,
                    locale.code()
                ),
                hearth_core::Locale::ZhTw => format!(
                    "回合 {}\n公開事件：{}\n{} · AI：{} · 語言：{}",
                    view.turn,
                    view.history.len(),
                    order,
                    difficulty,
                    locale.code()
                ),
            }
        }),
        text_font(13.0),
        TextColor(MUTED_TEXT),
        Node {
            margin: UiRect::top(px(8)),
            ..default()
        },
    ));
}

fn spawn_header_entity_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    entity: EntityId,
    card_id: &str,
    highlight: EntityHighlight,
) {
    let normal = match highlight {
        EntityHighlight::Normal => ACTION,
        _ => highlight_color(highlight),
    };
    let mut hero = parent.spawn((
        Button,
        GameEntity(entity),
        InspectableCard(card_id.to_owned()),
        UiAction::Entity(entity),
        ButtonColors {
            normal,
            hovered: ACTION_HOVER,
            pressed: CARD_SELECTED,
        },
        Node {
            width: px(132),
            min_height: px(42),
            padding: UiRect::all(px(5)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(highlight_color(highlight)),
        BackgroundColor(normal),
        Outline {
            width: px(3),
            offset: px(2),
            color: Color::NONE,
        },
        GlobalZIndex::default(),
        Pickable::default(),
    ));
    if matches!(
        highlight,
        EntityHighlight::Source | EntityHighlight::SelectedSource
    ) {
        hero.insert(DraggableGameEntity);
        hero.observe(handle_drag_start)
            .observe(handle_drag)
            .observe(handle_drag_end);
    }
    hero.observe(handle_ui_click)
        .observe(handle_drag_drop)
        .observe(show_card_preview)
        .observe(hide_card_preview);
    hero.with_child((
        Text::new(label),
        text_font(13.0),
        TextColor(TEXT),
        TextLayout::justify(Justify::Center),
        Pickable::IGNORE,
    ));
}

fn spawn_quick_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: UiAction,
    normal: Color,
    card_id: Option<&str>,
) {
    spawn_compact_button(parent, label, Some(action), normal, card_id);
}

fn spawn_passive_card_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    normal: Color,
    card_id: &str,
) {
    spawn_compact_button(parent, label, None, normal, Some(card_id));
}

fn spawn_compact_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: Option<UiAction>,
    normal: Color,
    card_id: Option<&str>,
) {
    let mut button = parent.spawn((
        Button,
        ButtonColors {
            normal,
            hovered: ACTION_HOVER,
            pressed: CARD_SELECTED,
        },
        Node {
            width: px(132),
            min_height: px(42),
            padding: UiRect::all(px(6)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::srgb(0.75, 0.70, 0.48)),
        BackgroundColor(normal),
    ));
    if matches!(action.as_ref(), Some(UiAction::HeroPower)) {
        button.insert(HeroPowerTargetingSource);
    }
    if let Some(action) = action {
        button.insert(action).observe(handle_ui_click);
    }
    if let Some(card_id) = card_id {
        button
            .insert(InspectableCard(card_id.to_owned()))
            .observe(show_card_preview)
            .observe(hide_card_preview);
    }
    button.with_child((
        Text::new(label),
        text_font(13.0),
        TextColor(TEXT),
        TextLayout::justify(Justify::Center),
        Pickable::IGNORE,
    ));
}

fn spawn_action_button(parent: &mut ChildSpawnerCommands, label: &str, action: UiAction) {
    parent
        .spawn((
            Button,
            action,
            ButtonColors {
                normal: ACTION,
                hovered: ACTION_HOVER,
                pressed: CARD_SELECTED,
            },
            Node {
                width: percent(100),
                min_height: px(44),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::srgb(0.32, 0.44, 0.54)),
            BackgroundColor(ACTION),
        ))
        .observe(handle_ui_click)
        .with_child((
            Text::new(label),
            text_font(13.0),
            TextColor(TEXT),
            Pickable::IGNORE,
        ));
}

fn shorten(value: &str, max_chars: usize) -> String {
    let plain = strip_markup(value).replace(['\n', '\r'], " ");
    let mut chars = plain.chars();
    let short = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{short}…")
    } else {
        short
    }
}

fn strip_markup(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            '$' if !in_tag => {}
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output
}

fn text_font(size: f32) -> TextFont {
    TextFont {
        font: FontSource::SansSerif,
        font_size: FontSize::Px(size),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempSettings(PathBuf);

    impl TempSettings {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the Unix epoch")
                .as_nanos();
            Self(std::env::temp_dir().join(format!(
                "hearth-client-settings-{}-{nonce}",
                std::process::id()
            )))
        }

        fn path(&self) -> PathBuf {
            self.0.join("nested/client.json")
        }

        fn resume_path(&self) -> PathBuf {
            self.0.join("state/active-match.json")
        }
    }

    impl Drop for TempSettings {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn settings_round_trip_and_cli_values_take_precedence() {
        let temporary = TempSettings::new();
        let path = temporary.path();
        let store = ClientSettingsStore {
            path: Some(path.clone()),
        };
        let display = DisplaySettings {
            fullscreen: true,
            ui_scale_percent: 120,
        };
        save_client_settings(
            &store,
            hearth_core::Locale::ZhCn,
            30,
            BotDifficulty::Hard,
            display,
        )
        .unwrap();
        assert_eq!(
            load_client_settings(&path).unwrap(),
            Some(PersistedClientSettings {
                version: CLIENT_SETTINGS_VERSION,
                locale: hearth_core::Locale::ZhCn,
                turn_seconds: 30,
                bot_difficulty: BotDifficulty::Hard,
                fullscreen: true,
                ui_scale_percent: 120,
            })
        );

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path_text = path.to_string_lossy().into_owned();
        let saved = parse_config_from(
            &root,
            &["--settings".to_owned(), path_text.clone()],
            None,
            None,
        )
        .unwrap();
        assert_eq!(saved.config.locale, hearth_core::Locale::ZhCn);
        assert_eq!(saved.config.bot_difficulty, BotDifficulty::Hard);
        assert_eq!(saved.turn_seconds, 30);
        assert_eq!(saved.display, display);

        let overridden = parse_config_from(
            &root,
            &[
                "--settings".to_owned(),
                path_text,
                "--locale".to_owned(),
                "zhTW".to_owned(),
                "--turn-seconds".to_owned(),
                "45".to_owned(),
                "--windowed".to_owned(),
                "--ui-scale".to_owned(),
                "80".to_owned(),
                "--bot-difficulty".to_owned(),
                "easy".to_owned(),
            ],
            None,
            None,
        )
        .unwrap();
        assert_eq!(overridden.config.locale, hearth_core::Locale::ZhTw);
        assert_eq!(overridden.config.bot_difficulty, BotDifficulty::Easy);
        assert_eq!(overridden.turn_seconds, 45);
        assert_eq!(
            overridden.display,
            DisplaySettings {
                fullscreen: false,
                ui_scale_percent: 80,
            }
        );

        let disabled = parse_config_from(
            &root,
            &["--no-settings".to_owned()],
            Some(path.clone()),
            None,
        )
        .unwrap();
        assert_eq!(disabled.config.locale, hearth_core::Locale::EnUs);
        assert_eq!(disabled.config.bot_difficulty, BotDifficulty::Normal);
        assert_eq!(disabled.turn_seconds, 75);
        assert_eq!(disabled.display, DisplaySettings::default());
        assert_eq!(disabled.settings_path, None);

        let hotseat = parse_config_from(
            &root,
            &["--no-settings".to_owned(), "--hotseat".to_owned()],
            None,
            None,
        )
        .unwrap();
        assert_eq!(hotseat.config.match_mode, MatchMode::Hotseat);
    }

    #[test]
    fn legacy_settings_migrate_and_invalid_display_values_are_rejected() {
        let temporary = TempSettings::new();
        let path = temporary.path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"version":1,"locale":"zhTW","turn_seconds":45}"#).unwrap();
        assert_eq!(
            load_client_settings(&path).unwrap(),
            Some(PersistedClientSettings {
                version: CLIENT_SETTINGS_VERSION,
                locale: hearth_core::Locale::ZhTw,
                turn_seconds: 45,
                bot_difficulty: BotDifficulty::Normal,
                fullscreen: false,
                ui_scale_percent: 100,
            })
        );

        fs::write(
            &path,
            r#"{"version":2,"locale":"enUS","turn_seconds":60,"fullscreen":true,"ui_scale_percent":120}"#,
        )
        .unwrap();
        assert_eq!(
            load_client_settings(&path).unwrap(),
            Some(PersistedClientSettings {
                version: CLIENT_SETTINGS_VERSION,
                locale: hearth_core::Locale::EnUs,
                turn_seconds: 60,
                bot_difficulty: BotDifficulty::Normal,
                fullscreen: true,
                ui_scale_percent: 120,
            })
        );

        fs::write(
            &path,
            r#"{"version":2,"locale":"enUS","turn_seconds":75,"fullscreen":false,"ui_scale_percent":101}"#,
        )
        .unwrap();
        assert!(
            load_client_settings(&path)
                .unwrap_err()
                .contains("unsupported UI scale 101%")
        );
        assert_eq!(parse_ui_scale_percent("80").unwrap(), 80);
        assert!(parse_ui_scale_percent("81").is_err());
        assert!(parse_ui_scale_percent("large").is_err());
    }

    #[test]
    fn display_settings_map_to_bevy_window_modes_and_scales() {
        assert_eq!(window_mode(false), WindowMode::Windowed);
        assert!(matches!(
            window_mode(true),
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        ));
        assert_eq!(
            DisplaySettings {
                fullscreen: false,
                ui_scale_percent: 80,
            }
            .ui_scale(),
            0.8
        );
        assert_eq!(
            fullscreen_status(hearth_core::Locale::ZhCn, true),
            "已启用无边框全屏；按 F11 可切换。"
        );
        assert_eq!(
            ui_scale_status(hearth_core::Locale::ZhTw, 120),
            "介面縮放已設為 120%。"
        );
    }

    #[test]
    fn frozen_over_hand_lock_has_a_localized_runtime_label() {
        let keywords = vec!["frozen_solid".to_owned(), "forge".to_owned()];
        assert_eq!(
            runtime_keyword_labels(&keywords, hearth_core::Locale::EnUs),
            "LOCKED THIS TURN, forge"
        );
        assert_eq!(
            runtime_keyword_labels(&keywords, hearth_core::Locale::ZhCn),
            "本回合不可使用, forge"
        );
        assert_eq!(
            runtime_keyword_labels(&keywords, hearth_core::Locale::ZhTw),
            "本回合無法打出, forge"
        );
    }

    #[test]
    fn saved_match_is_private_round_trips_and_can_be_disabled() {
        let temporary = TempSettings::new();
        let path = temporary.resume_path();
        let store = MatchResumeStore {
            path: Some(path.clone()),
        };
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let session = GameSession::load(&config).unwrap();

        save_match_resume(&store, &session).unwrap();
        let snapshot = load_match_resume(&path).unwrap().unwrap();
        let restored =
            GameSession::from_snapshot(&config.data_dir, config.locale, &snapshot).unwrap();
        assert_eq!(restored.view(), session.view());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }

        clear_match_resume(&store).unwrap();
        assert!(load_match_resume(&path).unwrap().is_none());
        let path_text = path.to_string_lossy().into_owned();
        let parsed =
            parse_config_from(&root, &["--resume".to_owned(), path_text], None, None).unwrap();
        assert_eq!(parsed.resume_path, Some(path.clone()));
        let disabled =
            parse_config_from(&root, &["--no-resume".to_owned()], None, Some(path)).unwrap();
        assert_eq!(disabled.resume_path, None);
        assert!(
            parse_config_from(
                &root,
                &[
                    "--resume".to_owned(),
                    "match.json".to_owned(),
                    "--no-resume".to_owned(),
                ],
                None,
                None,
            )
            .is_err()
        );
    }

    #[test]
    fn hotseat_handoff_is_required_only_after_input_changes_player() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut config = MatchConfig::demo(&root);
        config.match_mode = MatchMode::Hotseat;
        let mut session = GameSession::load(&config).unwrap();
        let acting = session.human_player();

        assert_eq!(hotseat_handoff_after_action(&session, acting), None);
        session
            .dispatch_human(PlayerCommand::Mulligan {
                replace: Vec::new(),
            })
            .unwrap();
        assert_eq!(
            hotseat_handoff_after_action(&session, acting),
            Some(PlayerId::TWO)
        );
    }

    #[test]
    fn changing_locale_reloads_cards_and_preserves_selected_deck_paths() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), config.locale).unwrap();
        let mut frontend = FrontendState::new(config, &library, false);
        frontend.player_deck = 1;
        frontend.opponent_deck = 2;
        let player_path = library.deck(1).unwrap().path.clone();
        let opponent_path = library.deck(2).unwrap().path.clone();
        let mut catalog = ClientCatalog(library);

        set_frontend_locale(&mut frontend, &mut catalog, hearth_core::Locale::ZhCn).unwrap();

        assert_eq!(frontend.config.locale, hearth_core::Locale::ZhCn);
        assert_eq!(catalog.0.definition("EX1_008").unwrap().name, "银色侍从");
        assert_eq!(
            catalog.0.deck(frontend.player_deck).unwrap().path,
            player_path
        );
        assert_eq!(
            catalog.0.deck(frontend.opponent_deck).unwrap().path,
            opponent_path
        );
        assert_eq!(frontend.status.as_deref(), Some("语言已切换为简体中文。"));
    }

    #[test]
    fn band_editor_enforces_capacity_and_removing_owner_cleans_sideboard() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), config.locale).unwrap();
        let catalog = ClientCatalog(library);
        let mut frontend = FrontendState::new(config, &catalog.0, false);
        frontend.draft = Some(hearth_app::DeckList {
            name: "Band".to_owned(),
            format: Some("wild".to_owned()),
            class: "mage".to_owned(),
            cards: vec!["ETC_080".to_owned()],
            sideboards: vec![hearth_app::DeckSideboard {
                owner: "ETC_080".to_owned(),
                cards: Vec::new(),
            }],
            hero_power: None,
            unrestricted: false,
        });
        frontend.draft_sideboard_owner = Some("ETC_080".to_owned());

        for card_id in ["EX1_008", "CS2_171", "CS2_120"] {
            add_draft_card(&mut frontend, &catalog, card_id);
        }
        assert_eq!(
            frontend.draft.as_ref().unwrap().sideboards[0].cards.len(),
            3
        );
        add_draft_card(&mut frontend, &catalog, "CS2_172");
        assert!(frontend.status.as_deref().unwrap().contains("complete"));

        remove_draft_card(&mut frontend, "EX1_008");
        assert_eq!(
            frontend.draft.as_ref().unwrap().sideboards[0].cards.len(),
            2
        );
        frontend.draft_sideboard_owner = None;
        remove_draft_card(&mut frontend, "ETC_080");
        let draft = frontend.draft.as_ref().unwrap();
        assert!(draft.cards.is_empty());
        assert!(draft.sideboards.is_empty());
    }

    #[test]
    fn renathal_can_expand_a_full_builder_draft_to_forty_cards() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), config.locale).unwrap();
        let mut draft = library
            .decks()
            .iter()
            .find(|stored| stored.path.ends_with("quest_rogue.json"))
            .unwrap()
            .deck
            .clone();
        assert_eq!(draft.cards.len(), 30);
        draft.cards.retain(|card| card != "REV_018");
        let catalog = ClientCatalog(library);
        let mut frontend = FrontendState::new(config, &catalog.0, false);
        frontend.draft = Some(draft);

        add_draft_card(&mut frontend, &catalog, "REV_018");

        let draft = frontend.draft.as_ref().unwrap();
        assert_eq!(draft.cards.len(), 31);
        assert_eq!(catalog.0.required_deck_size(draft), 40);
        assert!(frontend.status.is_none());
    }

    #[test]
    fn builder_rejects_a_card_that_exceeds_death_knight_rune_slots() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = MatchConfig::demo(&root);
        let library =
            DeckLibrary::load(root.join("decks"), root.join("data"), config.locale).unwrap();
        let catalog = ClientCatalog(library);
        let mut frontend = FrontendState::new(config, &catalog.0, false);
        frontend.draft = Some(hearth_app::DeckList {
            name: "Runes".to_owned(),
            format: Some("wild".to_owned()),
            class: "death_knight".to_owned(),
            cards: Vec::new(),
            sideboards: Vec::new(),
            hero_power: None,
            unrestricted: false,
        });

        add_draft_card(&mut frontend, &catalog, "RLK_067");
        assert_eq!(
            catalog.0.deck_rune_cost(frontend.draft.as_ref().unwrap()),
            hearth_core::RuneCost {
                blood: 2,
                frost: 0,
                unholy: 0,
            }
        );
        add_draft_card(&mut frontend, &catalog, "RLK_048");
        assert!(frontend.status.is_none());
        add_draft_card(&mut frontend, &catalog, "RLK_063");

        let draft = frontend.draft.as_ref().unwrap();
        assert_eq!(draft.cards, ["RLK_067", "RLK_048"]);
        assert!(
            frontend
                .status
                .as_deref()
                .is_some_and(|status| status.contains("rune slots"))
        );
    }
}
