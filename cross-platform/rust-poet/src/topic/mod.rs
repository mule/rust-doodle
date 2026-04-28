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

use crate::config::ConfigError;

pub fn build(name: &str, topic_arg: Option<&str>) -> Result<Box<dyn TopicSource>, ConfigError> {
    match name {
        "fixed" => {
            let seed = topic_arg.ok_or(ConfigError::InvalidValue {
                field: "topic",
                message: "--topic is required when --source=fixed".into(),
            })?;
            Ok(Box::new(fixed::FixedTopic::new(seed)))
        }
        "random" => Ok(Box::new(random::RandomTopic::new())),
        "wikipedia" => Ok(Box::new(wikipedia::WikipediaOnThisDay::new())),
        other => Err(ConfigError::UnknownSource(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_source_is_object_safe() {
        fn _assert(_: Box<dyn TopicSource>) {}
    }

    use crate::config::ConfigError;

    #[test]
    fn build_fixed_requires_topic() {
        let Err(err) = build("fixed", None) else { panic!("expected Err"); };
        assert!(matches!(err, ConfigError::InvalidValue { field: "topic", .. }));
    }

    #[test]
    fn build_random_ignores_topic() {
        let s = build("random", None).unwrap();
        assert_eq!(s.name(), "random");
    }

    #[test]
    fn build_unknown_errors() {
        let Err(err) = build("notreal", None) else { panic!("expected Err"); };
        assert!(matches!(err, ConfigError::UnknownSource(_)));
    }
}
