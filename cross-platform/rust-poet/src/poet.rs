use crate::error::PoetError;
use crate::prompt;
use crate::provider::{LlmProvider, LlmRequest};
use crate::topic::{Topic, TopicSource};

pub struct Poet {
    provider: Box<dyn LlmProvider>,
    source: Box<dyn TopicSource>,
    settings: PoemSettings,
}

#[derive(Debug, Clone)]
pub struct PoemSettings {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
pub struct Poem {
    pub text: String,
    pub topic: Topic,
    pub provider: &'static str,
    pub model: String,
}

impl Poet {
    pub fn new(
        provider: Box<dyn LlmProvider>,
        source: Box<dyn TopicSource>,
        settings: PoemSettings,
    ) -> Self {
        Self { provider, source, settings }
    }

    pub async fn generate(&self) -> Result<Poem, PoetError> {
        let topic = self.source.next_topic().await?;
        let messages = prompt::build(&topic);
        let req = LlmRequest {
            messages,
            model: self.settings.model.clone(),
            max_tokens: self.settings.max_tokens,
            temperature: self.settings.temperature,
        };
        let resp = self.provider.generate(&req).await?;
        Ok(Poem {
            text: resp.text,
            topic,
            provider: self.provider.name(),
            model: resp.model,
        })
    }
}
