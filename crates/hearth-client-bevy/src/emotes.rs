use bevy::prelude::*;
use hearth_app::GameSession;
use hearth_core::{Locale, PlayerId};

use crate::UiState;
use crate::frontend::{ClientScene, FrontendState};
use crate::i18n::pick;

const EMOTE_COOLDOWN_SECONDS: f32 = 1.5;
const EMOTE_DISPLAY_SECONDS: f32 = 3.0;
const BOT_REPLY_DELAY_SECONDS: f32 = 1.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EmoteKind {
    Thanks,
    WellPlayed,
    Greetings,
    Wow,
    Oops,
    Threaten,
}

impl EmoteKind {
    pub(crate) const ALL: [Self; 6] = [
        Self::Thanks,
        Self::WellPlayed,
        Self::Greetings,
        Self::Wow,
        Self::Oops,
        Self::Threaten,
    ];

    pub(crate) fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::Thanks => pick(locale, "Thanks", "感谢", "感謝"),
            Self::WellPlayed => pick(locale, "Well Played", "打得不错", "打得不錯"),
            Self::Greetings => pick(locale, "Greetings", "问候", "問候"),
            Self::Wow => pick(locale, "Wow", "惊叹", "驚嘆"),
            Self::Oops => pick(locale, "Oops", "失误", "失誤"),
            Self::Threaten => pick(locale, "Threaten", "威胁", "威脅"),
        }
    }

    pub(crate) fn phrase(self, locale: Locale) -> &'static str {
        match self {
            Self::Thanks => pick(locale, "Thanks!", "谢谢！", "謝謝！"),
            Self::WellPlayed => pick(locale, "Well played.", "打得不错。", "打得不錯。"),
            Self::Greetings => pick(locale, "Greetings.", "你好。", "你好。"),
            Self::Wow => pick(locale, "Wow!", "哇！", "哇！"),
            Self::Oops => pick(locale, "Oops.", "失误了。", "失誤了。"),
            Self::Threaten => pick(
                locale,
                "You will regret this.",
                "你会后悔的。",
                "你會後悔的。",
            ),
        }
    }

    fn bot_reply(self) -> Self {
        match self {
            Self::Thanks | Self::Greetings => Self::Greetings,
            Self::WellPlayed => Self::WellPlayed,
            Self::Oops => Self::Wow,
            Self::Wow => Self::Wow,
            Self::Threaten => Self::Threaten,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveEmote {
    player: PlayerId,
    kind: EmoteKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingBotReply {
    player: PlayerId,
    kind: EmoteKind,
    remaining: f32,
}

#[derive(Resource, Debug)]
pub(crate) struct EmoteState {
    match_number: Option<u64>,
    menu_open: bool,
    squelched_by_viewer: [bool; 2],
    cooldown_remaining: [f32; 2],
    active: Option<ActiveEmote>,
    display_remaining: f32,
    pending_bot_reply: Option<PendingBotReply>,
}

impl Default for EmoteState {
    fn default() -> Self {
        Self {
            match_number: None,
            menu_open: false,
            squelched_by_viewer: [false; 2],
            cooldown_remaining: [0.0; 2],
            active: None,
            display_remaining: 0.0,
            pending_bot_reply: None,
        }
    }
}

impl EmoteState {
    pub(crate) fn menu_open(&self) -> bool {
        self.menu_open
    }

    pub(crate) fn toggle_menu(&mut self) {
        self.menu_open = !self.menu_open;
    }

    pub(crate) fn close_menu(&mut self) -> bool {
        let changed = self.menu_open;
        self.menu_open = false;
        changed
    }

    pub(crate) fn is_squelched(&self, viewer: PlayerId) -> bool {
        self.squelched_by_viewer[viewer.index()]
    }

    pub(crate) fn toggle_squelch(&mut self, viewer: PlayerId) -> bool {
        let squelched = &mut self.squelched_by_viewer[viewer.index()];
        *squelched = !*squelched;
        if *squelched && self.active.is_some_and(|active| active.player != viewer) {
            self.active = None;
            self.display_remaining = 0.0;
        }
        *squelched
    }

    pub(crate) fn cooldown_remaining(&self, player: PlayerId) -> f32 {
        self.cooldown_remaining[player.index()]
    }

    pub(crate) fn emit(
        &mut self,
        player: PlayerId,
        kind: EmoteKind,
        bot_opponent: Option<PlayerId>,
    ) -> bool {
        if self.cooldown_remaining(player) > 0.0 {
            return false;
        }
        self.menu_open = false;
        self.cooldown_remaining[player.index()] = EMOTE_COOLDOWN_SECONDS;
        self.active = Some(ActiveEmote { player, kind });
        self.display_remaining = EMOTE_DISPLAY_SECONDS;
        self.pending_bot_reply = bot_opponent.map(|opponent| PendingBotReply {
            player: opponent,
            kind: kind.bot_reply(),
            remaining: BOT_REPLY_DELAY_SECONDS,
        });
        true
    }

    pub(crate) fn visible_for(&self, player: PlayerId, viewer: PlayerId) -> Option<EmoteKind> {
        self.active
            .filter(|active| {
                active.player == player
                    && (player == viewer || !self.squelched_by_viewer[viewer.index()])
            })
            .map(|active| active.kind)
    }

    fn sync_match(&mut self, match_number: u64) -> bool {
        if self.match_number == Some(match_number) {
            return false;
        }
        *self = Self {
            match_number: Some(match_number),
            ..default()
        };
        true
    }

    fn clear_for_frontend(&mut self) -> bool {
        let changed = self.menu_open || self.active.is_some() || self.pending_bot_reply.is_some();
        self.menu_open = false;
        self.active = None;
        self.display_remaining = 0.0;
        self.pending_bot_reply = None;
        changed
    }

    fn advance(&mut self, delta_seconds: f32, viewer: PlayerId) -> bool {
        let mut changed = false;
        for cooldown in &mut self.cooldown_remaining {
            let before = *cooldown;
            *cooldown = (*cooldown - delta_seconds).max(0.0);
            changed |= before > 0.0 && *cooldown == 0.0 && self.menu_open;
        }

        if self.active.is_some() {
            self.display_remaining = (self.display_remaining - delta_seconds).max(0.0);
            if self.display_remaining == 0.0 {
                self.active = None;
                changed = true;
            }
        }

        if let Some(reply) = &mut self.pending_bot_reply {
            reply.remaining = (reply.remaining - delta_seconds).max(0.0);
            if reply.remaining == 0.0 {
                let reply = self.pending_bot_reply.take().unwrap();
                if !self.squelched_by_viewer[viewer.index()] {
                    self.active = Some(ActiveEmote {
                        player: reply.player,
                        kind: reply.kind,
                    });
                    self.display_remaining = EMOTE_DISPLAY_SECONDS;
                    changed = true;
                }
            }
        }
        changed
    }
}

pub(crate) fn update_emotes(
    time: Res<Time>,
    session: NonSend<GameSession>,
    frontend: Res<FrontendState>,
    mut emotes: ResMut<EmoteState>,
    mut ui: ResMut<UiState>,
) {
    let mut changed = emotes.sync_match(frontend.match_number);
    if frontend.scene != ClientScene::Match {
        if frontend.pauses_match_progress() {
            return;
        }
        changed |= emotes.clear_for_frontend();
        if changed {
            ui.dirty = true;
        }
        return;
    }
    if frontend.handoff_player.is_some() {
        changed |= emotes.close_menu();
        if changed {
            ui.dirty = true;
        }
        return;
    }
    if frontend.pauses_match_progress() {
        return;
    }
    if session.view().outcome.is_some() {
        changed |= emotes.close_menu();
    }
    changed |= emotes.advance(time.delta_secs(), session.human_player());
    if changed {
        ui.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emotes_cool_down_expire_and_receive_a_delayed_bot_reply() {
        let mut state = EmoteState::default();
        assert!(state.emit(PlayerId::ONE, EmoteKind::WellPlayed, Some(PlayerId::TWO)));
        assert!(!state.emit(PlayerId::ONE, EmoteKind::Wow, Some(PlayerId::TWO)));
        assert_eq!(
            state.visible_for(PlayerId::ONE, PlayerId::ONE),
            Some(EmoteKind::WellPlayed)
        );

        assert!(state.advance(BOT_REPLY_DELAY_SECONDS, PlayerId::ONE));
        assert_eq!(
            state.visible_for(PlayerId::TWO, PlayerId::ONE),
            Some(EmoteKind::WellPlayed)
        );
        assert!(state.cooldown_remaining(PlayerId::ONE) > 0.0);
        state.advance(EMOTE_DISPLAY_SECONDS, PlayerId::ONE);
        assert_eq!(state.visible_for(PlayerId::TWO, PlayerId::ONE), None);
        assert!(state.emit(PlayerId::ONE, EmoteKind::Wow, Some(PlayerId::TWO)));
    }

    #[test]
    fn squelch_is_private_to_each_hotseat_viewer() {
        let mut state = EmoteState {
            active: Some(ActiveEmote {
                player: PlayerId::TWO,
                kind: EmoteKind::Threaten,
            }),
            display_remaining: EMOTE_DISPLAY_SECONDS,
            ..default()
        };

        assert!(state.toggle_squelch(PlayerId::ONE));
        assert_eq!(state.visible_for(PlayerId::TWO, PlayerId::ONE), None);
        assert!(!state.is_squelched(PlayerId::TWO));

        state.active = Some(ActiveEmote {
            player: PlayerId::TWO,
            kind: EmoteKind::Greetings,
        });
        assert_eq!(
            state.visible_for(PlayerId::TWO, PlayerId::TWO),
            Some(EmoteKind::Greetings)
        );
    }

    #[test]
    fn all_emotes_have_localized_labels_and_phrases() {
        for kind in EmoteKind::ALL {
            assert!(!kind.label(Locale::EnUs).is_empty());
            assert!(!kind.label(Locale::ZhCn).is_empty());
            assert!(!kind.phrase(Locale::ZhTw).is_empty());
        }
    }
}
