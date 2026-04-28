use rust_poet::provider::{LlmProvider, LlmRequest, Message, ProviderError, Role};
use rust_poet::provider::anthropic::Anthropic;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_request() -> LlmRequest {
    LlmRequest {
        model: "claude-haiku-4-5".into(),
        messages: vec![
            Message { role: Role::System, content: "you are a poet".into() },
            Message { role: Role::User, content: "fog".into() },
        ],
        max_tokens: 64,
        temperature: 0.8,
    }
}

#[tokio::test]
async fn anthropic_lifts_system_to_top_level() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "anth-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .and(body_partial_json(serde_json::json!({
            "model": "claude-haiku-4-5",
            "max_tokens": 64,
            "system": "you are a poet",
            "messages": [{"role": "user", "content": "fog"}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "content": [{"type": "text", "text": "thick gray drift"}],
            "usage": {"input_tokens": 8, "output_tokens": 3}
        })))
        .mount(&server)
        .await;

    let provider = Anthropic::with_base_url("anth-key".into(), server.uri());
    let resp = provider.generate(&sample_request()).await.unwrap();
    assert_eq!(resp.text, "thick gray drift");
    assert_eq!(resp.model, "claude-haiku-4-5-20251001");
}

#[tokio::test]
async fn anthropic_maps_401_to_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {"type": "authentication_error", "message": "bad key"}
        })))
        .mount(&server)
        .await;

    let provider = Anthropic::with_base_url("bad".into(), server.uri());
    let err = provider.generate(&sample_request()).await.unwrap_err();
    assert!(matches!(err, ProviderError::Auth { provider: "anthropic" }));
}
