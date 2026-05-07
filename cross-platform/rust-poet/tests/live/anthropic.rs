use rust_poet::provider::anthropic::Anthropic;
use rust_poet::provider::{LlmProvider, LlmRequest, Message, Role};

#[tokio::test]
async fn live_anthropic_generates_short_response() {
    let key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("ANTHROPIC_API_KEY not set; skipping live test");
            return;
        }
    };
    let provider = Anthropic::new(key);
    let req = LlmRequest {
        model: "claude-haiku-4-5".into(),
        messages: vec![Message { role: Role::User, content: "Reply with one word.".into() }],
        max_tokens: 50,
        temperature: 0.0,
    };
    let resp = provider.generate(&req).await.expect("anthropic live call failed");
    assert!(!resp.text.is_empty());
}
