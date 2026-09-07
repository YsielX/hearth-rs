use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};

use crate::LlmError;

/// Chat Completions connection settings. Credentials are never serialized.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    #[serde(skip)]
    pub api_key: String,
    pub timeout_seconds: u64,
    /// Disable for compatible services that do not accept response_format.
    pub json_mode: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            model: String::new(),
            api_key: String::new(),
            timeout_seconds: 120,
            json_mode: true,
        }
    }
}

impl fmt::Debug for LlmConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LlmConfig")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &"[redacted]")
            .field("timeout_seconds", &self.timeout_seconds)
            .field("json_mode", &self.json_mode)
            .finish()
    }
}

impl LlmConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.apply_env();
        config
    }

    /// Environment values override persisted settings; CLI flags can override these.
    pub fn apply_env(&mut self) {
        for (name, field) in [
            ("HEARTH_LLM_URL", &mut self.base_url),
            ("HEARTH_LLM_MODEL", &mut self.model),
            ("HEARTH_LLM_API_KEY", &mut self.api_key),
        ] {
            if let Ok(value) = std::env::var(name) {
                *field = value;
            }
        }
    }

    pub fn endpoint(&self) -> Result<String, LlmError> {
        let base = self.base_url.trim().trim_end_matches('/');
        let uri: ureq::http::Uri = base
            .parse()
            .map_err(|_| LlmError::Config("invalid LLM URL"))?;
        if !matches!(uri.scheme_str(), Some("http" | "https"))
            || uri.host().is_none()
            || uri.authority().is_some_and(|a| a.as_str().contains('@'))
            || uri.query().is_some()
            || base.contains('#')
        {
            return Err(LlmError::Config(
                "LLM URL must be an HTTP(S) URL without credentials, query or fragment",
            ));
        }
        Ok(if base.ends_with("/chat/completions") {
            base.to_owned()
        } else {
            format!("{base}/chat/completions")
        })
    }

    pub fn validate(&self) -> Result<(), LlmError> {
        self.endpoint()?;
        if self.model.trim().is_empty() {
            return Err(LlmError::Config("LLM model is required"));
        }
        if !(1..=600).contains(&self.timeout_seconds) {
            return Err(LlmError::Config(
                "LLM timeout must be between 1 and 600 seconds",
            ));
        }
        if !self.api_key.is_empty() {
            ureq::http::HeaderValue::from_str(&format!("Bearer {}", self.api_key.trim()))
                .map_err(|_| LlmError::Config("invalid API key header"))?;
        }
        Ok(())
    }

    pub(crate) fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
}
