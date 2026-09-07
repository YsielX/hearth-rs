//! Player-visible LLM decisions, reusable by clients and training schedulers.
//!
//! Prepare requests on the engine thread, send them on a worker, then validate
//! their position fingerprint before dispatch. No engine or Lua value crosses
//! the HTTP boundary. The caller owns retry, fallback and training policy.

mod config;
mod env_file;
mod request;
mod transport;

pub use config::LlmConfig;
pub use request::{DecisionRequest, ExpertLabel, PROMPT_VERSION};
pub use transport::{LlmBot, LlmDecision};

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("{0}")]
    Config(&'static str),
    #[error("invalid decision input: {0}")]
    Input(&'static str),
    #[error("LLM connection failed or timed out; check the URL and connection")]
    Transport,
    #[error("LLM service returned HTTP {0}")]
    Http(u16),
    #[error("invalid LLM response: {0}")]
    Response(&'static str),
    #[error("LLM answer belongs to a different position")]
    StaleDecision,
}
