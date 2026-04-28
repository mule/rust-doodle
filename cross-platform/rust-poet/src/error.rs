use thiserror::Error;

use crate::config::ConfigError;
use crate::provider::ProviderError;
use crate::topic::TopicError;

#[derive(Debug, Error)]
pub enum PoetError {
    #[error(transparent)]
    Provider(#[from] ProviderError),

    #[error(transparent)]
    TopicSource(#[from] TopicError),

    #[error(transparent)]
    Config(#[from] ConfigError),
}
