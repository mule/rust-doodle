use thiserror::Error;

use crate::provider::ProviderError;
use crate::topic::TopicError;

#[derive(Debug, Error)]
pub enum PoetError {
    #[error(transparent)]
    Provider(#[from] ProviderError),

    #[error(transparent)]
    TopicSource(#[from] TopicError),

    #[error("config: {0}")]
    Config(String),
}
