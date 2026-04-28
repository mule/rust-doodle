use thiserror::Error;

use crate::provider::ProviderError;

#[derive(Debug, Error)]
pub enum PoetError {
    #[error(transparent)]
    Provider(#[from] ProviderError),

    #[error("config: {0}")]
    Config(String),
}
