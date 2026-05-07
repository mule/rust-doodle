use rust_poet::provider::{LlmProvider, LlmRequest, Message, ProviderError, Role};
use rust_poet::provider::openai::OpenAi;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_request() -> LlmRequest {
    LlmRequest {
        model: "gpt-4o-mini".into(),
        messages: vec![
            Message { role: Role::System, content: "you are a poet".into() },
            Message { role: Role::User, content: "rain".into() },
        ],
        max_tokens: 64,
        temperature: 0.8,
    }
}

#[tokio::test]
async fn openai_returns_text_on_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "gpt-4o-mini-2024-07-18",
            "choices": [{"message": {"role": "assistant", "content": "soft drops"}}],
            "usage": {"prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7}
        })))
        .mount(&server)
        .await;

    let provider = OpenAi::with_base_url("test-key".into(), server.uri());
    let resp = provider.generate(&sample_request()).await.unwrap();
    assert_eq!(resp.text, "soft drops");
    assert_eq!(resp.model, "gpt-4o-mini-2024-07-18");
}

#[tokio::test]
async fn openai_maps_401_to_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {"message": "bad key", "type": "auth"}
        })))
        .mount(&server)
        .await;

    let provider = OpenAi::with_base_url("bad-key".into(), server.uri());
    let err = provider.generate(&sample_request()).await.unwrap_err();
    assert!(matches!(err, ProviderError::Auth { provider: "openai" }));
}

#[tokio::test]
async fn openai_maps_429_to_rate_limited() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "30")
                .set_body_json(serde_json::json!({"error": {"message": "slow down"}})),
        )
        .mount(&server)
        .await;

    let provider = OpenAi::with_base_url("k".into(), server.uri());
    let err = provider.generate(&sample_request()).await.unwrap_err();
    match err {
        ProviderError::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(std::time::Duration::from_secs(30)));
        }
        other => panic!("expected RateLimited, got {other:?}"),
    }
}
