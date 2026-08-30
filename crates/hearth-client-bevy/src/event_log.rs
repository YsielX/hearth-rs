use hearth_app::GameSession;
use hearth_core::{PlayerId, PlayerView, PublicEvent};

pub fn recent_event_lines(session: &GameSession, view: &PlayerView, limit: usize) -> Vec<String> {
    hearth_app::presentation::event_text::recent_event_lines(session, view, limit)
}

pub fn event_summary(
    session: &GameSession,
    viewer: PlayerId,
    event: &PublicEvent,
) -> Option<String> {
    hearth_app::presentation::event_text::event_summary(session, viewer, event)
}
