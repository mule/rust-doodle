use async_trait::async_trait;

use crate::topic::{Topic, TopicError, TopicSource};

pub struct FixedTopic {
    seed: String,
}

impl FixedTopic {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[async_trait]
impl TopicSource for FixedTopic {
    fn name(&self) -> &'static str {
        "fixed"
    }

    async fn next_topic(&self) -> Result<Topic, TopicError> {
        Ok(Topic { seed: self.seed.clone(), context: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_seed_unchanged() {
        let s = FixedTopic::new("rain");
        let t = s.next_topic().await.unwrap();
        assert_eq!(t.seed, "rain");
        assert!(t.context.is_none());
    }
}
