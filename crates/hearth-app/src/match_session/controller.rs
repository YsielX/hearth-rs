use std::path::Path;

use hearth_bot::DifficultyBot;
use hearth_core::{LegalAction, Locale, PlayerCommand, PlayerController, PlayerId, PlayerView};

use crate::{AppError, BotDifficulty};

use super::config::{MatchConfig, MatchMode};
use super::session::MatchSession;
use super::snapshot::{GAME_SESSION_SNAPSHOT_VERSION, GameSessionSnapshot};

/// Local human/bot policy layered on top of MatchSession.
pub struct GameSession {
    pub(super) session: MatchSession,
    pub(super) human_player: PlayerId,
    pub(super) match_mode: MatchMode,
    pub(super) bot: DifficultyBot,
}

impl GameSession {
    pub fn load(config: &MatchConfig) -> Result<Self, AppError> {
        Ok(Self {
            session: MatchSession::load(&config.match_setup())?,
            human_player: config.human_player,
            match_mode: config.match_mode,
            bot: DifficultyBot::new(config.bot_difficulty),
        })
    }

    pub fn snapshot(&self) -> GameSessionSnapshot {
        GameSessionSnapshot {
            format_version: GAME_SESSION_SNAPSHOT_VERSION,
            game: self.session.snapshot(),
            human_player: self.human_player,
            match_mode: self.match_mode,
            bot_difficulty: self.bot.difficulty(),
            deck_names: self.session.deck_names.clone(),
        }
    }

    pub fn from_snapshot(
        data_dir: impl AsRef<Path>,
        locale: Locale,
        snapshot: &GameSessionSnapshot,
    ) -> Result<Self, AppError> {
        if snapshot.format_version != GAME_SESSION_SNAPSHOT_VERSION {
            return Err(AppError::UnsupportedSessionSnapshot(
                snapshot.format_version,
            ));
        }
        Ok(Self {
            session: MatchSession::from_snapshot_with_deck_names(
                data_dir,
                locale,
                &snapshot.game,
                snapshot.deck_names.clone(),
            )?,
            human_player: snapshot.human_player,
            match_mode: snapshot.match_mode,
            bot: DifficultyBot::new(snapshot.bot_difficulty),
        })
    }

    pub fn human_player(&self) -> PlayerId {
        if self.match_mode == MatchMode::Hotseat {
            self.session.state().input_player()
        } else {
            self.human_player
        }
    }

    pub fn match_mode(&self) -> MatchMode {
        self.match_mode
    }

    pub fn bot_difficulty(&self) -> BotDifficulty {
        self.bot.difficulty()
    }

    pub fn starting_player(&self) -> PlayerId {
        self.session.state().starting_player
    }

    pub fn is_hotseat(&self) -> bool {
        self.match_mode == MatchMode::Hotseat
    }

    pub fn locale(&self) -> Locale {
        self.session.locale()
    }

    pub fn deck_name(&self, player: PlayerId) -> &str {
        self.session.deck_name(player)
    }

    pub fn view(&self) -> PlayerView {
        self.session.view_for(self.human_player())
    }

    pub fn legal_actions(&self) -> Result<Vec<LegalAction>, AppError> {
        if self.session.state().input_player() != self.human_player() {
            return Ok(Vec::new());
        }
        self.session.legal_action_options()
    }

    pub fn dispatch_human(&mut self, command: PlayerCommand) -> Result<(), AppError> {
        self.dispatch_human_only(command)?;
        self.advance_bot(10_000)
    }

    /// Dispatches one human command without consuming any following bot input.
    ///
    /// Interactive frontends can use this to present automated actions one at a
    /// time. Batch frontends should normally keep using dispatch_human.
    pub fn dispatch_human_only(&mut self, command: PlayerCommand) -> Result<(), AppError> {
        let human_player = self.human_player();
        if self.session.state().input_player() != human_player {
            return Err(AppError::Controller(format!(
                "{} cannot act while {} has input",
                human_player,
                self.session.state().input_player()
            )));
        }
        let legal = self.session.legal_actions()?;
        if !legal.contains(&command) {
            return Err(AppError::Controller(format!(
                "the selected command is no longer legal: {command:?}"
            )));
        }
        self.session.dispatch(command)?;
        Ok(())
    }

    /// Concedes the locally controlled side even when another controller owns
    /// the current input (for example while the built-in AI is acting).
    pub fn concede_human(&mut self) -> Result<(), AppError> {
        let player = self.human_player();
        self.session
            .dispatch(PlayerCommand::ConcedePlayer { player })?;
        Ok(())
    }

    pub fn is_bot_turn(&self) -> bool {
        !self.is_hotseat()
            && self.session.state().outcome.is_none()
            && self.session.state().input_player() != self.human_player
    }

    /// Advances at most one automated action, returning whether one was
    /// dispatched. This is intentionally deterministic and uses the same bot
    /// policy as advance_bot.
    pub fn advance_bot_once(&mut self) -> Result<bool, AppError> {
        if !self.is_bot_turn() {
            return Ok(false);
        }
        let player = self.session.state().input_player();
        let view = self.session.view_for(player);
        let legal = self.session.legal_action_options()?;
        let command = self
            .bot
            .choose_action(&view, &legal)
            .map_err(AppError::Controller)?;
        self.session.dispatch(command)?;
        Ok(true)
    }

    pub fn advance_bot(&mut self, action_limit: usize) -> Result<(), AppError> {
        if self.is_hotseat() {
            return Ok(());
        }
        let mut actions = 0usize;
        while self.session.state().outcome.is_none()
            && self.session.state().input_player() != self.human_player
        {
            actions += 1;
            if actions > action_limit {
                return Err(AppError::BotActionLimit(action_limit));
            }
            let advanced = self.advance_bot_once()?;
            debug_assert!(advanced, "bot loop condition requires one action");
        }
        Ok(())
    }

    pub fn card_name(&self, card_id: &str) -> String {
        self.session.card_name(card_id)
    }

    pub fn card_text(&self, card_id: &str) -> String {
        self.session.card_text(card_id)
    }

    pub fn turn_time_limit_seconds(&self) -> Result<Option<u64>, AppError> {
        self.session.turn_time_limit_seconds()
    }
}
