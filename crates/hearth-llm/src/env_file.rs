use std::{collections::BTreeMap, path::Path};

use crate::{LlmConfig, LlmError};

impl LlmConfig {
    /// Import connection variables without shell execution or process-wide
    /// environment changes. File values override current settings. Unknown
    /// names are ignored; errors never contain file contents.
    pub fn apply_env_file(&mut self, path: impl AsRef<Path>) -> Result<(), LlmError> {
        let input = std::fs::read_to_string(path)
            .map_err(|_| LlmError::Config("cannot read LLM env file"))?;
        self.apply_env_text(&input)
    }

    fn apply_env_text(&mut self, input: &str) -> Result<(), LlmError> {
        let mut values = BTreeMap::new();
        for line in input.trim_start_matches('\u{feff}').lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let line = line.strip_prefix("export ").unwrap_or(line).trim_start();
            let Some((name, raw)) = line.split_once('=') else {
                return Err(LlmError::Config(
                    "invalid LLM env file; expected NAME=value",
                ));
            };
            let name = name.trim();
            if !matches!(
                name,
                "HEARTH_LLM_URL"
                    | "OPENAI_API_URL"
                    | "OPENAI_BASE_URL"
                    | "HEARTH_LLM_API_KEY"
                    | "OPENAI_API_KEY"
                    | "HEARTH_LLM_MODEL"
                    | "OPENAI_MODEL"
            ) {
                continue;
            }
            let raw = raw.trim();
            let value = if raw.starts_with(['\'', '"']) {
                let quote = raw.chars().next().unwrap();
                let end = raw[1..]
                    .find(quote)
                    .ok_or(LlmError::Config("unclosed quote in LLM env file"))?
                    + 1;
                let tail = raw[end + 1..].trim();
                if !tail.is_empty() && !tail.starts_with('#') {
                    return Err(LlmError::Config(
                        "unexpected text after quoted LLM env value",
                    ));
                }
                &raw[1..end]
            } else {
                raw.find(" #").map_or(raw, |end| raw[..end].trim_end())
            };
            values.insert(name, value);
        }
        for (names, field) in [
            (
                &["HEARTH_LLM_URL", "OPENAI_API_URL", "OPENAI_BASE_URL"][..],
                &mut self.base_url,
            ),
            (&["HEARTH_LLM_MODEL", "OPENAI_MODEL"][..], &mut self.model),
            (
                &["HEARTH_LLM_API_KEY", "OPENAI_API_KEY"][..],
                &mut self.api_key,
            ),
        ] {
            if let Some(value) = names.iter().find_map(|name| values.get(name)) {
                *field = (*value).to_owned();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_accepts_openai_names_and_preserves_literal_credentials() {
        let mut config = LlmConfig::default();
        config.apply_env_text("\u{feff}# test\nexport OPENAI_API_URL = 'https://example.com/v1' # URL\nOPENAI_API_KEY=\"literal$KEY`command`=#value\"\nOPENAI_MODEL=test-model\n").unwrap();
        assert_eq!(config.base_url, "https://example.com/v1");
        assert_eq!(config.model, "test-model");
        assert_eq!(config.api_key, "literal$KEY`command`=#value");
        config.apply_env_text("HEARTH_LLM_MODEL=preferred\nOPENAI_MODEL=other\nOPENAI_API_KEY=unquoted=key # comment\n").unwrap();
        assert_eq!(config.model, "preferred");
        assert_eq!(config.api_key, "unquoted=key");
    }

    #[test]
    fn invalid_import_is_atomic_and_does_not_echo_secrets() {
        let mut config = LlmConfig {
            model: "original".into(),
            ..Default::default()
        };
        let original = config.clone();
        let error = config
            .apply_env_text("OPENAI_MODEL=changed\nOPENAI_API_KEY=\"secret-unclosed")
            .unwrap_err();
        assert_eq!(config, original);
        assert!(!error.to_string().contains("secret-unclosed"));
    }
}
