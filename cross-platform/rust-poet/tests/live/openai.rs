use rust_poet::provider::openai::OpenAi;
use rust_poet::provider::{LlmProvider, LlmRequest, Message, Role};

#[tokio::test]
async fn live_openai_generates_short_response() {
    let key = match std::env::var("OPENAI_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("OPENAI_API_KEY not set; skipping live test");
            return;
        }
    };
    let provider = OpenAi::new(key);
    let req = LlmRequest {
        model: "gpt-4o-mini".into(),
        messages: vec![Message { role: Role::User, content: "Reply with one word.".into() }],
        max_tokens: 50,
        temperature: 0.0,
    };
    let resp = provider.generate(&req).await.expect("openai live call failed");
    assert!(!resp.text.is_empty());
}
