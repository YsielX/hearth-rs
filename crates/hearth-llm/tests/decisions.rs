use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    thread,
    time::Duration,
};

use hearth_core::{CardRuntime, Game, PlayerCommand, PlayerId};
use hearth_llm::{DecisionRequest, ExpertLabel, LlmBot, LlmConfig, LlmError};
use hearth_script::LuaCardRuntime;
use serde_json::{Value, json};

fn game() -> Game<LuaCardRuntime> {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    Game::new_unrestricted(
        LuaCardRuntime::load_dir(data).unwrap(),
        vec!["CS2_120".into(); 20],
        vec!["EX1_560".into(); 20],
        7,
    )
    .unwrap()
}

fn request(game: &Game<LuaCardRuntime>) -> DecisionRequest {
    let player = game.state().input_player();
    DecisionRequest::prepare(
        &game.state().player_view(player),
        &game.legal_action_options().unwrap(),
        &game.state().player(player).starting_deck,
        game.runtime().pack_hash(),
        |id| game.runtime().definition(id),
    )
    .unwrap()
}

#[test]
fn requests_preserve_own_knowledge_and_reject_stale_or_invalid_labels() {
    let mut game = game();
    let req = request(&game);
    let own = game.state().input_player();
    let hidden_card = if own == PlayerId::ONE {
        "EX1_560"
    } else {
        "CS2_120"
    };
    assert!(!serde_json::to_string(&req).unwrap().contains(hidden_card));
    assert_eq!(
        req.position()["players"][own.opponent().index()]["hand"],
        json!([])
    );
    assert!(
        !req.position()["own_starting_deck"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    let mut label = ExpertLabel {
        request_id: req.request_id().into(),
        action_index: 0,
        acceptable_actions: vec![],
        reason: "test".into(),
    };
    assert_eq!(
        req.resolve(&label).unwrap(),
        game.legal_action_options().unwrap()[0].command
    );
    label.acceptable_actions.push(usize::MAX);
    assert!(matches!(req.resolve(&label), Err(LlmError::Response(_))));
    label.acceptable_actions.clear();
    game.dispatch(PlayerCommand::Mulligan { replace: vec![] })
        .unwrap();
    assert!(matches!(
        request(&game).resolve(&label),
        Err(LlmError::StaleDecision)
    ));
    let wrong_view = game
        .state()
        .player_view(game.state().input_player().opponent());
    assert!(
        DecisionRequest::prepare(
            &wrong_view,
            &game.legal_action_options().unwrap(),
            &[],
            "x",
            |_| None
        )
        .is_err()
    );
}

fn mock(
    response: impl FnOnce(Value) -> (u16, Value) + Send + 'static,
    json_mode: bool,
) -> (LlmBot, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        let (header_end, length) = loop {
            let n = stream.read(&mut buffer).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buffer[..n]);
            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                assert!(headers.starts_with("post /v1/chat/completions http/1.1"));
                assert!(headers.contains("authorization: bearer test-secret"));
                let length = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .unwrap()
                    .trim()
                    .parse::<usize>()
                    .unwrap();
                break (end + 4, length);
            }
        };
        while bytes.len() < header_end + length {
            let n = stream.read(&mut buffer).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buffer[..n]);
        }
        let body: Value = serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap();
        assert_eq!(body["model"], "test-model");
        assert_eq!(body.get("response_format").is_some(), json_mode);
        let (status, result) = response(body);
        let result = result.to_string();
        write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{result}", result.len()).unwrap();
    });
    let config = LlmConfig {
        base_url: format!("http://{address}/v1"),
        model: "test-model".into(),
        api_key: "test-secret".into(),
        timeout_seconds: 5,
        json_mode,
    };
    (LlmBot::new(config).unwrap(), worker)
}

#[test]
fn http_contract_validates_labels_and_returns_training_metadata() {
    let req = request(&game());
    let (bot, worker) = mock(
        |body| {
            let prompt: Value =
                serde_json::from_str(body["messages"][1]["content"].as_str().unwrap()).unwrap();
            let label = json!({"request_id": prompt["request_id"], "action_index": 0, "reason": "keep hand"});
            (
                200,
                json!({"id": "response-1", "model": "resolved-model", "usage": {"total_tokens": 42},
            "choices": [{"finish_reason": "stop", "message": {"content": format!("```json\n{label}\n```")}}]}),
            )
        },
        true,
    );
    let decision = bot.decide(&req).unwrap();
    worker.join().unwrap();
    assert_eq!(decision.label.acceptable_actions, vec![0]);
    assert_eq!(decision.usage["total_tokens"], 42);
    assert_eq!(decision.response_model.as_deref(), Some("resolved-model"));
    req.resolve(&decision.label).unwrap();

    for (status, response) in [
        (401, json!({"error": "test-secret"})),
        (307, json!({})),
        (
            200,
            json!({"choices": [{"finish_reason": "length", "message": {"content": "{}"}}]}),
        ),
        (
            200,
            json!({"choices": [{"message": {"content": "not JSON"}}]}),
        ),
        (
            200,
            json!({"choices": [{"message": {"content": json!({"request_id": req.request_id(), "action_index": 999999}).to_string()}}]}),
        ),
        (
            200,
            json!({"choices": [{"message": {"content": json!({"request_id": "old", "action_index": 0}).to_string()}}]}),
        ),
    ] {
        let (bot, worker) = mock(move |_| (status, response), false);
        let error = bot.decide(&req).unwrap_err();
        assert!(!error.to_string().contains("test-secret"));
        worker.join().unwrap();
    }
}

#[test]
fn connection_config_normalizes_urls_and_never_serializes_credentials() {
    let mut config = LlmConfig {
        base_url: "https://example.com/v1/".into(),
        model: "model".into(),
        api_key: "private-test-key".into(),
        ..Default::default()
    };
    assert_eq!(
        config.endpoint().unwrap(),
        "https://example.com/v1/chat/completions"
    );
    config.base_url = config.endpoint().unwrap();
    assert_eq!(config.endpoint().unwrap(), config.base_url);
    let encoded = serde_json::to_string(&config).unwrap();
    assert!(!encoded.contains("private-test-key"));
    assert!(!format!("{config:?}").contains("private-test-key"));
    assert!(
        serde_json::from_str::<LlmConfig>(&encoded)
            .unwrap()
            .api_key
            .is_empty()
    );
    for url in [
        "file:///tmp/key",
        "https://user:pass@example.com/v1",
        "https://example.com/v1?key=x",
        "https://example.com/v1#fragment",
        "",
    ] {
        config.base_url = url.into();
        assert!(config.validate().is_err(), "{url}");
    }
}
