use std::time::Duration;

use bevy::prelude::*;
use hearth_app::GameSession;

use crate::frontend::{ClientScene, FrontendState};
use crate::{MatchResumeStore, UiState, sync_match_resume};

const BOT_ACTION_DELAY_SECONDS: f32 = 0.72;

#[derive(Resource)]
pub(crate) struct BotPlaybackState {
    match_number: Option<u64>,
    timer: Timer,
    armed: bool,
    failed: bool,
}

impl Default for BotPlaybackState {
    fn default() -> Self {
        Self {
            match_number: None,
            timer: Timer::new(
                Duration::from_secs_f32(BOT_ACTION_DELAY_SECONDS),
                TimerMode::Once,
            ),
            armed: false,
            failed: false,
        }
    }
}

pub(crate) fn update_bot_playback(
    time: Res<Time>,
    mut session: NonSendMut<GameSession>,
    mut frontend: ResMut<FrontendState>,
    resume: Res<MatchResumeStore>,
    mut ui: ResMut<UiState>,
    mut playback: ResMut<BotPlaybackState>,
) {
    if playback.match_number != Some(frontend.match_number) {
        playback.match_number = Some(frontend.match_number);
        playback.armed = false;
        playback.failed = false;
    }
    if frontend.pauses_match_progress() {
        return;
    }
    let active = frontend.scene == ClientScene::Match
        && frontend.handoff_player.is_none()
        && session.is_bot_turn();
    if !active {
        playback.armed = false;
        playback.failed = false;
        return;
    }
    if playback.failed {
        return;
    }
    if !playback.armed {
        playback.timer = Timer::new(
            Duration::from_secs_f32(BOT_ACTION_DELAY_SECONDS),
            TimerMode::Once,
        );
        playback.armed = true;
        return;
    }
    if !playback.timer.tick(time.delta()).just_finished() {
        return;
    }

    match session.advance_bot_once() {
        Ok(true) => {
            ui.interaction.reset_after_dispatch();
            ui.page = 0;
            ui.error = None;
            ui.dirty = true;
            if let Err(error) = sync_match_resume(&resume, &session, &mut frontend) {
                ui.error = Some(error);
            }
            playback.armed = false;
        }
        Ok(false) => playback.armed = false,
        Err(error) => {
            ui.error = Some(error.to_string());
            ui.dirty = true;
            playback.failed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_defaults_to_an_unarmed_recoverable_delay() {
        let state = BotPlaybackState::default();
        assert_eq!(state.match_number, None);
        assert!(!state.armed);
        assert!(!state.failed);
        assert_eq!(
            state.timer.duration(),
            Duration::from_secs_f32(BOT_ACTION_DELAY_SECONDS)
        );
    }
}
