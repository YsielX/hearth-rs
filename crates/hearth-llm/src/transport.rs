use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{DecisionRequest, ExpertLabel, LlmConfig, LlmError, PROMPT_VERSION};

/// A cloneable blocking HTTP client. Call on a worker thread in interactive apps.
#[derive(Clone, Debug)]
pub struct LlmBot {
    config: LlmConfig,
    agent: ureq::Agent,
}

/// A validated expert label plus metadata for training and request accounting.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmDecision {
    pub label: ExpertLabel,
    pub prompt_version: String,
    pub requested_model: String,
    pub response_model: Option<String>,
    pub response_id: Option<String>,
    pub usage: Value,
    pub elapsed_ms: u64,
}

impl LlmBot {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        config.validate()?;
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(config.timeout()))
            .max_redirects(0)
            .http_status_as_error(false)
            .build()
            .into();
        Ok(Self { config, agent })
    }

    /// One bounded request, no silent heuristic fallback or implicit paid retry.
    pub fn decide(&self, request: &DecisionRequest) -> Result<LlmDecision, LlmError> {
        let started = Instant::now();
        let mut body = json!({
            "model": self.config.model.trim(), "messages": request.messages(), "stream": false,
        });
        if self.config.json_mode {
            body["response_format"] = json!({"type": "json_object"});
        }
        let mut http = self
            .agent
            .post(self.config.endpoint()?)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !self.config.api_key.trim().is_empty() {
            http = http.header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.trim()),
            );
        }
        let mut response = http
            .send(body.to_string())
            .map_err(|_| LlmError::Transport)?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            // Provider error bodies can echo credentials; never surface them in client logs.
            return Err(LlmError::Http(status));
        }
        let text = response
            .body_mut()
            .with_config()
            .limit(2 * 1024 * 1024)
            .read_to_string()
            .map_err(|_| LlmError::Response("unreadable or oversized response body"))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|_| LlmError::Response("service did not return JSON"))?;
        let choice = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|c| c.first())
            .ok_or(LlmError::Response("missing completion choice"))?;
        if choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .is_some_and(|r| r != "stop")
        {
            return Err(LlmError::Response(
                "completion was interrupted or did not return a final answer",
            ));
        }
        if choice
            .pointer("/message/refusal")
            .is_some_and(|v| !v.is_null())
        {
            return Err(LlmError::Response("service declined the request"));
        }
        let content = choice
            .pointer("/message/content")
            .and_then(Value::as_str)
            .ok_or(LlmError::Response("missing answer text"))?
            .trim();
        let content = content
            .strip_prefix("```json")
            .or_else(|| content.strip_prefix("```"))
            .and_then(|s| s.trim().strip_suffix("```"))
            .unwrap_or(content)
            .trim();
        let mut label: ExpertLabel = serde_json::from_str(content).map_err(|_| {
            LlmError::Response(
                "expected request_id, integer action_index and optional acceptable_actions/reason",
            )
        })?;
        request.resolve(&label)?;
        label.acceptable_actions.push(label.action_index);
        label.acceptable_actions.sort_unstable();
        label.acceptable_actions.dedup();
        Ok(LlmDecision {
            label,
            prompt_version: PROMPT_VERSION.to_owned(),
            requested_model: self.config.model.clone(),
            response_model: value
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_owned),
            response_id: value.get("id").and_then(Value::as_str).map(str::to_owned),
            usage: value.get("usage").cloned().unwrap_or(Value::Null),
            elapsed_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
        })
    }
}
