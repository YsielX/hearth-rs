use std::{
    sync::{Mutex, mpsc},
    time::Duration,
};

use bevy::prelude::*;
use hearth_app::{GameSession, LlmDecision, LlmError};

use crate::frontend::{ClientScene, FrontendState};
use crate::{MatchResumeStore, UiState, sync_match_resume};

const BOT_ACTION_DELAY_SECONDS: f32 = 0.72;

#[derive(Resource)]
pub(crate) struct BotPlaybackState {
    match_number: Option<u64>,
    timer: Timer,
    armed: bool,
    failed: bool,
    pending: Option<Mutex<mpsc::Receiver<Result<LlmDecision, LlmError>>>>,
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
            pending: None,
        }
    }
}

impl BotPlaybackState {
    pub(crate) fn retry(&mut self) {
        self.pending = None;
        self.armed = false;
        self.failed = false;
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
        playback.retry();
    }
    if frontend.pauses_match_progress() {
        return;
    }
    let active = frontend.scene == ClientScene::Match
        && frontend.handoff_player.is_none()
        && session.is_bot_turn();
    if !active {
        playback.retry();
        return;
    }
    if playback.failed {
        return;
    }
    if let Some(pending) = &playback.pending {
        let result = pending.lock().expect("LLM receiver lock").try_recv();
        let result = match result {
            Ok(result) => result.map_err(|e| e.to_string()).and_then(|decision| {
                session
                    .apply_llm_decision(&decision)
                    .map_err(|e| e.to_string())
            }),
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("LLM worker stopped; retry from the match menu".into())
            }
        };
        playback.pending = None;
        playback.armed = false;
        ui.dirty = true;
        match result {
            Ok(()) => {
                ui.interaction.reset_after_dispatch();
                ui.page = 0;
                ui.error = sync_match_resume(&resume, &session, &mut frontend).err();
            }
            Err(error) => {
                ui.error = Some(error);
                playback.failed = true;
            }
        }
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

    if session.is_llm_turn() {
        match session.prepare_llm_turn() {
            Ok((bot, request)) => {
                let (sender, receiver) = mpsc::channel();
                match std::thread::Builder::new()
                    .name("hearth-llm".into())
                    .spawn(move || {
                        let _ = sender.send(bot.decide(&request));
                    }) {
                    Ok(_) => playback.pending = Some(Mutex::new(receiver)),
                    Err(_) => {
                        ui.error = Some("Unable to start LLM worker".into());
                        playback.failed = true;
                    }
                }
            }
            Err(error) => {
                ui.error = Some(error.to_string());
                playback.failed = true;
            }
        }
        ui.dirty = true;
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

    #[test]
    fn retry_discards_in_flight_results_and_recovers_from_failure() {
        let (sender, receiver) = mpsc::channel();
        let mut state = BotPlaybackState {
            pending: Some(Mutex::new(receiver)),
            failed: true,
            armed: true,
            ..Default::default()
        };
        state.retry();
        assert!(state.pending.is_none());
        assert!(!state.failed);
        assert!(!state.armed);
        assert!(sender.send(Err(LlmError::Transport)).is_err());
    }
}
