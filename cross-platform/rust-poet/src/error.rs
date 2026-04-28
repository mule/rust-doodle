use thiserror::Error;

#[derive(Debug, Error)]
pub enum PoetError {
    #[error("config: {0}")]
    Config(String),
}
