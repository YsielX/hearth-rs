use std::collections::VecDeque;
use std::time::Duration;

use bevy::prelude::*;
use hearth_app::GameSession;
use hearth_core::{PlayerId, PublicEvent};

use crate::event_log::event_summary;
use crate::frontend::{ClientScene, FrontendState};

const TOAST_SECONDS: f32 = 1.15;
const MAX_PENDING_CUES: usize = 32;

#[derive(Component)]
pub struct EventToast;

#[derive(Component)]
pub struct EventToastText;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CueTone {
    Info,
    Friendly,
    Enemy,
    Combat,
    Healing,
    Result,
}

struct EventCue {
    text: String,
    tone: CueTone,
}

#[derive(Resource)]
pub struct EventAnimationState {
    cursor: usize,
    pending: VecDeque<EventCue>,
    current: Option<EventCue>,
    timer: Timer,
}

impl Default for EventAnimationState {
    fn default() -> Self {
        Self {
            cursor: 0,
            pending: VecDeque::new(),
            current: None,
            timer: Timer::from_seconds(TOAST_SECONDS, TimerMode::Once),
        }
    }
}

pub fn spawn_event_toast(commands: &mut Commands) {
    commands
        .spawn((
            EventToast,
            Node {
                position_type: PositionType::Absolute,
                top: px(78),
                left: percent(23),
                width: percent(38),
                min_height: px(48),
                padding: UiRect::axes(px(18), px(9)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.12, 0.18, 0.94)),
            BorderColor::all(Color::srgb(0.40, 0.58, 0.74)),
            GlobalZIndex(100),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_child((
            EventToastText,
            Text::new(""),
            TextFont {
                font: FontSource::SansSerif,
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
}

pub fn update_event_toast(
    time: Res<Time>,
    session: NonSend<GameSession>,
    frontend: Res<FrontendState>,
    mut state: ResMut<EventAnimationState>,
    mut toast: Query<(&mut BackgroundColor, &mut BorderColor, &mut Visibility), With<EventToast>>,
    mut text: Query<&mut Text, With<EventToastText>>,
) {
    if frontend.match_menu_open {
        if let Ok((_, _, mut visibility)) = toast.single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    if frontend.scene != ClientScene::Match || frontend.handoff_player.is_some() {
        state.cursor = session.view().history.len();
        state.pending.clear();
        state.current = None;
        if let Ok((_, _, mut visibility)) = toast.single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    let view = session.view();
    if state.cursor > view.history.len() {
        state.cursor = 0;
        state.pending.clear();
        state.current = None;
    }
    for record in view.history.iter().skip(state.cursor) {
        if let Some(summary) = event_summary(&session, view.viewer, &record.event) {
            state.pending.push_back(EventCue {
                text: summary,
                tone: cue_tone(&record.event, view.viewer),
            });
        }
    }
    state.cursor = view.history.len();
    while state.pending.len() > MAX_PENDING_CUES {
        state.pending.pop_front();
    }

    if state.current.is_some() && state.timer.tick(time.delta()).just_finished() {
        state.current = None;
    }
    if state.current.is_none()
        && let Some(next) = state.pending.pop_front()
    {
        state
            .timer
            .set_duration(Duration::from_secs_f32(TOAST_SECONDS));
        state.timer.reset();
        state.current = Some(next);
    }

    let Ok((mut background, mut border, mut visibility)) = toast.single_mut() else {
        return;
    };
    let Ok(mut label) = text.single_mut() else {
        return;
    };
    if let Some(cue) = &state.current {
        **label = cue.text.clone();
        let color = tone_color(cue.tone);
        background.0 = color.with_alpha(0.94);
        border.set_all(color.lighter(0.22));
        *visibility = Visibility::Inherited;
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn cue_tone(event: &PublicEvent, viewer: PlayerId) -> CueTone {
    match event {
        PublicEvent::TurnStarted { player, .. } if *player == viewer => CueTone::Friendly,
        PublicEvent::TurnStarted { .. } => CueTone::Enemy,
        PublicEvent::Damaged { .. }
        | PublicEvent::DamagePrevented { .. }
        | PublicEvent::Attack { .. }
        | PublicEvent::EntityDied { .. }
        | PublicEvent::CardBurned { .. } => CueTone::Combat,
        PublicEvent::Healed { .. } | PublicEvent::ArmorGained { .. } => CueTone::Healing,
        PublicEvent::Conceded { .. } | PublicEvent::GameEnded { .. } => CueTone::Result,
        PublicEvent::CardPlayed { player, .. }
        | PublicEvent::SpellCast { player, .. }
        | PublicEvent::MinionPlayed { player, .. }
        | PublicEvent::WeaponPlayed { player, .. }
        | PublicEvent::LocationPlayed { player, .. }
        | PublicEvent::HeroPowerUsed { player, .. }
            if *player == viewer =>
        {
            CueTone::Friendly
        }
        PublicEvent::CardPlayed { .. }
        | PublicEvent::SpellCast { .. }
        | PublicEvent::MinionPlayed { .. }
        | PublicEvent::WeaponPlayed { .. }
        | PublicEvent::LocationPlayed { .. }
        | PublicEvent::HeroPowerUsed { .. } => CueTone::Enemy,
        _ => CueTone::Info,
    }
}

fn tone_color(tone: CueTone) -> Color {
    match tone {
        CueTone::Info => Color::srgb(0.14, 0.23, 0.34),
        CueTone::Friendly => Color::srgb(0.10, 0.35, 0.50),
        CueTone::Enemy => Color::srgb(0.48, 0.16, 0.16),
        CueTone::Combat => Color::srgb(0.60, 0.20, 0.12),
        CueTone::Healing => Color::srgb(0.12, 0.43, 0.25),
        CueTone::Result => Color::srgb(0.55, 0.39, 0.08),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_cues_are_relative_to_the_viewer() {
        let event = PublicEvent::TurnStarted {
            player: PlayerId::ONE,
            turn: 1,
        };
        assert_eq!(cue_tone(&event, PlayerId::ONE), CueTone::Friendly);
        assert_eq!(cue_tone(&event, PlayerId::TWO), CueTone::Enemy);
    }

    #[test]
    fn damage_and_healing_have_distinct_tones() {
        let target = hearth_core::PublicEntity {
            id: hearth_core::EntityId(1),
            card_id: "target".to_owned(),
        };
        assert_eq!(
            cue_tone(
                &PublicEvent::Damaged {
                    source: None,
                    target: target.clone(),
                    amount: 3,
                },
                PlayerId::ONE,
            ),
            CueTone::Combat
        );
        assert_eq!(
            cue_tone(
                &PublicEvent::Healed {
                    source: None,
                    target,
                    amount: 3,
                },
                PlayerId::ONE,
            ),
            CueTone::Healing
        );
    }
}
