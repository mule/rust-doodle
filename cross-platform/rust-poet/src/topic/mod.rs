use async_trait::async_trait;
use thiserror::Error;

pub mod fixed;
pub mod random;
pub mod wikipedia;

#[async_trait]
pub trait TopicSource: Send + Sync {
    fn name(&self) -> &'static str;
    async fn next_topic(&self) -> Result<Topic, TopicError>;
}

#[derive(Debug, Clone)]
pub struct Topic {
    pub seed: String,
    pub context: Option<String>,
}

#[derive(Debug, Error)]
pub enum TopicError {
    #[error("network: {0}")]
    Network(#[from] reqwest::Error),

    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("no events available for date")]
    NoEventsForDate,

    #[error("bad input: {0}")]
    BadInput(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_source_is_object_safe() {
        fn _assert(_: Box<dyn TopicSource>) {}
    }
}
