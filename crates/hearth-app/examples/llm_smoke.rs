//! Opt-in live probe: one opening choice, then one normal turn decision.
//! No credentials or raw provider responses are printed.
use hearth_app::{LlmBot, LlmConfig, MatchConfig, MatchSession};
use hearth_core::PlayerCommand;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("usage: llm_smoke ENV_FILE MODEL")?;
    let model = args.next().ok_or("model is required")?;
    let mut config = LlmConfig::from_env();
    config.apply_env_file(path)?;
    config.model = model;
    let bot = LlmBot::new(config)?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut session = MatchSession::load(&MatchConfig::demo(root).match_setup())?;
    for phase in ["mulligan", "normal_turn"] {
        if phase == "normal_turn" {
            while !session
                .view_for(session.state().input_player())
                .mulligan_eligible
                .is_empty()
            {
                session.dispatch(PlayerCommand::Mulligan { replace: vec![] })?;
            }
        }
        let request = session.llm_request()?;
        println!(
            "{}",
            serde_json::json!({"phase": phase, "status": "requesting",
            "legal_actions": request.position()["legal_actions"].as_array().map(Vec::len),
            "request_bytes": serde_json::to_vec(&request)?.len()})
        );
        let decision = bot.decide(&request)?;
        session.dispatch_llm(&decision)?;
        // Emit only known numeric usage fields, never arbitrary server content.
        println!(
            "{}",
            serde_json::json!({"phase": phase, "status": "dispatched",
            "action_index": decision.label.action_index,
            "elapsed_ms": decision.elapsed_ms,
            "prompt_tokens": decision.usage.get("prompt_tokens").and_then(serde_json::Value::as_u64),
            "completion_tokens": decision.usage.get("completion_tokens").and_then(serde_json::Value::as_u64)})
        );
    }
    Ok(())
}
