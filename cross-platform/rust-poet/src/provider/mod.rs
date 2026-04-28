pub(crate) mod openai_like;

use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn generate(&self, req: &LlmRequest) -> Result<LlmResponse, ProviderError>;
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub model: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Copy)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("network: {0}")]
    Network(#[from] reqwest::Error),

    #[error("auth failed for provider '{provider}'")]
    Auth { provider: &'static str },

    #[error("rate limited (retry after {retry_after:?})")]
    RateLimited { retry_after: Option<Duration> },

    #[error("bad request to '{provider}': {message}")]
    BadRequest { provider: &'static str, message: String },

    #[error("server error from '{provider}': HTTP {status}")]
    ServerError { provider: &'static str, status: u16 },

    #[error("empty response from '{provider}'")]
    EmptyResponse { provider: &'static str },

    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_provider_is_object_safe() {
        fn _assert(_: Box<dyn LlmProvider>) {}
    }

    #[test]
    fn provider_error_displays_auth() {
        let e = ProviderError::Auth { provider: "openai" };
        assert_eq!(format!("{e}"), "auth failed for provider 'openai'");
    }
}
