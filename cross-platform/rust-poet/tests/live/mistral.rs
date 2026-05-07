use rust_poet::provider::mistral::Mistral;
use rust_poet::provider::{LlmProvider, LlmRequest, Message, Role};

#[tokio::test]
async fn live_mistral_generates_short_response() {
    let key = match std::env::var("MISTRAL_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("MISTRAL_API_KEY not set; skipping live test");
            return;
        }
    };
    let provider = Mistral::new(key);
    let req = LlmRequest {
        model: "mistral-small-latest".into(),
        messages: vec![Message { role: Role::User, content: "Reply with one word.".into() }],
        max_tokens: 50,
        temperature: 0.0,
    };
    let resp = provider.generate(&req).await.expect("mistral live call failed");
    assert!(!resp.text.is_empty());
}
