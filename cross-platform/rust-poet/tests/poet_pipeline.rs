#![cfg(feature = "test-utils")]

use rust_poet::poet::{Poem, PoemSettings, Poet};
use rust_poet::test_utils::MockLlmProvider;
use rust_poet::topic::fixed::FixedTopic;

#[tokio::test]
async fn generates_poem_with_mock_provider_and_fixed_topic() {
    let provider = Box::new(MockLlmProvider::with_text("a soft drop / on stone"));
    let source = Box::new(FixedTopic::new("rain"));
    let settings = PoemSettings {
        model: "mock-model".into(),
        max_tokens: 64,
        temperature: 0.8,
    };

    let poet = Poet::new(provider, source, settings);
    let Poem { text, topic, provider, model } = poet.generate().await.unwrap();

    assert_eq!(text, "a soft drop / on stone");
    assert_eq!(topic.seed, "rain");
    assert_eq!(provider, "mock");
    assert_eq!(model, "mock-model");
}
