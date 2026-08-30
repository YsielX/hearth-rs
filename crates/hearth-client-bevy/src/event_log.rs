use hearth_app::GameSession;
use hearth_core::PlayerView;

pub fn recent_event_lines(session: &GameSession, view: &PlayerView, limit: usize) -> Vec<String> {
    hearth_app::presentation::event_text::recent_event_lines(session, view, limit)
}
