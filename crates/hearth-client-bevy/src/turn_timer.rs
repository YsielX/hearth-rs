use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use hearth_app::GameSession;
use hearth_core::{LegalAction, Locale, PlayerCommand, PlayerId};

use crate::frontend::{ClientScene, FrontendState};
use crate::i18n::pick;
use crate::{
    ACTION, CARD_SELECTED, MatchResumeStore, TEXT, UiState, hotseat_handoff_after_action,
    sync_match_resume, text_font,
};

const MAX_TIMEOUT_ACTIONS: usize = 64;

#[derive(Resource, Clone, Copy)]
pub struct TurnTimerConfig {
    pub default_seconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TurnClockKey {
    match_number: u64,
    turn: u32,
    player: PlayerId,
    seconds: u64,
}

#[derive(Resource)]
pub struct TurnClock {
    key: Option<TurnClockKey>,
    timer: Timer,
}

impl Default for TurnClock {
    fn default() -> Self {
        Self {
            key: None,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub(crate) struct TurnTimerRoot;

#[derive(Component)]
pub(crate) struct TurnTimerFill;

#[derive(Component)]
pub(crate) struct TurnTimerLabel;

#[derive(SystemParam)]
pub(crate) struct TimerFrontend<'w> {
    frontend: ResMut<'w, FrontendState>,
    resume: Res<'w, MatchResumeStore>,
}

type TurnTimerRootQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Node, &'static mut Visibility),
    (With<TurnTimerRoot>, Without<TurnTimerFill>),
>;

type TurnTimerFillQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Node, &'static mut BackgroundColor),
    (With<TurnTimerFill>, Without<TurnTimerRoot>),
>;

#[derive(SystemParam)]
pub(crate) struct TurnTimerUi<'w, 's> {
    root: TurnTimerRootQuery<'w, 's>,
    fill: TurnTimerFillQuery<'w, 's>,
    label: Query<'w, 's, &'static mut Text, With<TurnTimerLabel>>,
}

pub fn spawn_turn_timer(parent: &mut ChildSpawnerCommands, locale: Locale) {
    parent
        .spawn((
            TurnTimerRoot,
            Node {
                display: Display::None,
                width: px(230),
                height: px(28),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
                padding: UiRect::axes(px(8), px(4)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(9)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.055, 0.075, 0.105, 0.96)),
            BorderColor::all(Color::srgb(0.62, 0.52, 0.25)),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|timer| {
            timer.spawn((
                TurnTimerLabel,
                Text::new(format!("{} 75s", pick(locale, "TURN", "回合", "回合"))),
                text_font(13.0),
                TextColor(TEXT),
                Node {
                    width: px(78),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            timer
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        height: px(14),
                        border_radius: BorderRadius::all(px(7)),
                        overflow: Overflow::clip_x(),
                        ..default()
                    },
                    BackgroundColor(ACTION),
                    Pickable::IGNORE,
                ))
                .with_child((
                    TurnTimerFill,
                    Node {
                        width: percent(100),
                        height: percent(100),
                        border_radius: BorderRadius::all(px(7)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.60, 0.83)),
                    Pickable::IGNORE,
                ));
        });
}

pub fn update_turn_timer(
    time: Res<Time>,
    config: Res<TurnTimerConfig>,
    client: TimerFrontend,
    mut session: NonSendMut<GameSession>,
    mut clock: ResMut<TurnClock>,
    mut ui: ResMut<UiState>,
    mut display: TurnTimerUi,
) {
    let TimerFrontend {
        mut frontend,
        resume,
    } = client;
    if frontend.pauses_match_progress() {
        pause_clock(&mut display);
        return;
    }
    if frontend.scene != ClientScene::Match || frontend.handoff_player.is_some() {
        hide_clock(&mut clock, &mut display);
        return;
    }
    let view = session.view();
    if view.outcome.is_some()
        || !view.mulligan_eligible.is_empty()
        || view.input_player != session.human_player()
    {
        hide_clock(&mut clock, &mut display);
        return;
    }
    let rule_seconds = match session.turn_time_limit_seconds() {
        Ok(limit) => limit,
        Err(error) => {
            ui.error = Some(error.to_string());
            hide_clock(&mut clock, &mut display);
            return;
        }
    };
    let Some(seconds) = effective_seconds(rule_seconds, config.default_seconds) else {
        hide_clock(&mut clock, &mut display);
        return;
    };
    let key = TurnClockKey {
        match_number: frontend.match_number,
        turn: view.turn,
        player: view.input_player,
        seconds,
    };
    if clock.key != Some(key) {
        clock.key = Some(key);
        clock.timer = Timer::new(Duration::from_secs(seconds), TimerMode::Once);
    }
    let expired = clock.timer.tick(time.delta()).just_finished();
    render_clock(&clock, session.locale(), &mut display);
    if expired {
        let acting_player = session.human_player();
        let starting_turn = view.turn;
        if let Err(message) = dispatch_timeout_actions(&mut session, starting_turn) {
            ui.error = Some(message);
        } else {
            ui.error = None;
            ui.interaction.reset_after_dispatch();
            ui.page = 0;
            frontend.handoff_player = hotseat_handoff_after_action(&session, acting_player);
            if let Err(message) = sync_match_resume(&resume, &session, &mut frontend) {
                ui.error = Some(message);
            }
        }
        ui.dirty = true;
        clock.key = None;
    }
}

fn effective_seconds(rule_seconds: Option<u64>, default_seconds: u64) -> Option<u64> {
    rule_seconds.or_else(|| (default_seconds > 0).then_some(default_seconds))
}

fn dispatch_timeout_actions(session: &mut GameSession, starting_turn: u32) -> Result<(), String> {
    for _ in 0..MAX_TIMEOUT_ACTIONS {
        let legal = session.legal_actions().map_err(|error| error.to_string())?;
        let Some(command) = timeout_command(&legal) else {
            return Err("turn timer expired with no non-concede legal action".to_owned());
        };
        let ends_turn = matches!(command, PlayerCommand::EndTurn);
        session
            .dispatch_human_only(command)
            .map_err(|error| error.to_string())?;
        let view = session.view();
        if ends_turn || view.outcome.is_some() || view.turn != starting_turn {
            return Ok(());
        }
    }
    Err(format!(
        "turn timer exceeded {MAX_TIMEOUT_ACTIONS} forced actions"
    ))
}

fn timeout_command(legal: &[LegalAction]) -> Option<PlayerCommand> {
    legal
        .iter()
        .map(|action| &action.command)
        .find(|command| matches!(command, PlayerCommand::EndTurn))
        .or_else(|| {
            legal
                .iter()
                .map(|action| &action.command)
                .find(|command| !matches!(command, PlayerCommand::Concede))
        })
        .cloned()
}

fn hide_clock(clock: &mut TurnClock, display: &mut TurnTimerUi<'_, '_>) {
    clock.key = None;
    if let Ok((mut node, mut visibility)) = display.root.single_mut() {
        node.display = Display::None;
        *visibility = Visibility::Hidden;
    }
}

fn pause_clock(display: &mut TurnTimerUi<'_, '_>) {
    if let Ok((mut node, mut visibility)) = display.root.single_mut() {
        node.display = Display::None;
        *visibility = Visibility::Hidden;
    }
}

fn render_clock(clock: &TurnClock, locale: Locale, display: &mut TurnTimerUi<'_, '_>) {
    if let Ok((mut node, mut visibility)) = display.root.single_mut() {
        node.display = Display::Flex;
        *visibility = Visibility::Inherited;
    }
    let remaining = clock.timer.remaining_secs().ceil() as u64;
    if let Ok(mut label) = display.label.single_mut() {
        **label = format!("{} {remaining}s", pick(locale, "TURN", "回合", "回合"));
    }
    if let Ok((mut node, mut background)) = display.fill.single_mut() {
        node.width = percent(clock.timer.fraction_remaining() * 100.0);
        background.0 = if remaining <= 15 {
            Color::srgb(0.86, 0.19, 0.13)
        } else if remaining <= 30 {
            CARD_SELECTED
        } else {
            Color::srgb(0.18, 0.60, 0.83)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legal(command: PlayerCommand) -> LegalAction {
        LegalAction {
            command,
            mana_cost: 0,
        }
    }

    #[test]
    fn runtime_rule_overrides_the_default_and_zero_disables_only_the_default() {
        assert_eq!(effective_seconds(None, 75), Some(75));
        assert_eq!(effective_seconds(Some(15), 75), Some(15));
        assert_eq!(effective_seconds(None, 0), None);
        assert_eq!(effective_seconds(Some(15), 0), Some(15));
    }

    #[test]
    fn timeout_prefers_end_turn_and_never_concedes() {
        let choices = vec![
            legal(PlayerCommand::Concede),
            legal(PlayerCommand::Choose { index: 0 }),
            legal(PlayerCommand::EndTurn),
        ];
        assert_eq!(timeout_command(&choices), Some(PlayerCommand::EndTurn));
        assert_eq!(
            timeout_command(&choices[..2]),
            Some(PlayerCommand::Choose { index: 0 })
        );
        assert_eq!(timeout_command(&choices[..1]), None);
    }
}
