use std::path::{Path, PathBuf};

use thiserror::Error;

pub mod deck;
pub mod match_session;
pub mod presentation;

pub use deck::{
    CardCatalogEntry, DeckLibrary, DeckList, DeckSideboard, DeckstringError, StoredDeck,
    export_deckstring, import_deckstring,
};
pub use hearth_bot::BotDifficulty;
pub use match_session::{
    GAME_SESSION_SNAPSHOT_VERSION, GameSession, GameSessionSnapshot, MatchConfig, MatchMode,
    MatchSession, MatchSetup, starting_player_for_seed, timeout_command,
};

// Compatibility module aliases for callers that used the first shared-text
// extraction before presentation became its own namespace.
pub use presentation::{command_text, event_text};

/// Locates the repository-shaped runtime bundle used by both native clients.
///
/// Release archives place the executables beside `data/` and `decks/`. During
/// development and tests those directories live at the workspace root, so the
/// compile-time path remains a fallback rather than leaking into release use.
pub fn runtime_root() -> PathBuf {
    runtime_root_from(std::env::current_exe().ok().as_deref())
}

fn runtime_root_from(executable: Option<&Path>) -> PathBuf {
    if let Some(root) = executable.and_then(Path::parent)
        && root.join("data").is_dir()
        && root.join("decks").is_dir()
    {
        return root.to_owned();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse deck {path}: {source}")]
    ParseDeck {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to parse card metadata {path}: {source}")]
    ParseCardMetadata {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid card metadata {path}: {message}")]
    InvalidCardMetadata { path: PathBuf, message: String },
    #[error("failed to serialize deck {path}: {source}")]
    SerializeDeck {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to delete {path}: {source}")]
    Delete {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("deck index {0} does not exist")]
    UnknownDeckIndex(usize),
    #[error("deck path is not loaded: {0}")]
    UnknownDeckPath(PathBuf),
    #[error("refusing to delete protected repository deck {0}")]
    ProtectedDeck(PathBuf),
    #[error("another custom deck already uses {0}")]
    DeckNameConflict(PathBuf),
    #[error("invalid deck {deck}: {message}")]
    InvalidDeck { deck: String, message: String },
    #[error("failed to load card scripts: {0}")]
    Script(#[from] hearth_script::ScriptLoadError),
    #[error("game error: {0}")]
    Game(#[from] hearth_core::GameError),
    #[error("controller error: {0}")]
    Controller(String),
    #[error("runtime rule error: {0}")]
    RuntimeRule(String),
    #[error("automated opponent exceeded {0} consecutive actions")]
    BotActionLimit(usize),
    #[error("turn timer expired with no non-concede legal action")]
    NoTimeoutAction,
    #[error("turn timer exceeded {0} forced actions")]
    TimeoutActionLimit(usize),
    #[error("unsupported saved session format {0}")]
    UnsupportedSessionSnapshot(u32),
}
