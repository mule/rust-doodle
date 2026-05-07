use rust_poet::provider::{LlmProvider, LlmRequest, Message, Role};
use rust_poet::provider::mistral::Mistral;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn mistral_returns_text_on_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer mistral-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "mistral-small-latest",
            "choices": [{"message": {"role": "assistant", "content": "windswept fields"}}]
        })))
        .mount(&server)
        .await;

    let provider = Mistral::with_base_url("mistral-key".into(), server.uri());
    let req = LlmRequest {
        model: "mistral-small-latest".into(),
        messages: vec![Message { role: Role::User, content: "wind".into() }],
        max_tokens: 64,
        temperature: 0.7,
    };
    let resp = provider.generate(&req).await.unwrap();
    assert_eq!(resp.text, "windswept fields");
    assert_eq!(provider.name(), "mistral");
}
