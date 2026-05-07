pub(crate) mod openai_like;
pub mod openai;
pub mod mistral;
pub mod anthropic;

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

use crate::config::{Config, ConfigError};

/// Build a provider by name, reading the API key from the appropriate env var.
/// Only the selected provider's key is required.
pub fn build(name: &str, cfg: &Config) -> Result<Box<dyn LlmProvider>, ConfigError> {
    let provider_cfg = cfg
        .providers
        .get(name)
        .ok_or_else(|| ConfigError::UnknownProvider(name.to_string()))?;

    match name {
        openai::PROVIDER_NAME => {
            let key = read_key("OPENAI_API_KEY")?;
            let provider = match &provider_cfg.base_url {
                Some(url) => openai::OpenAi::with_base_url(key, url.clone()),
                None => openai::OpenAi::new(key),
            };
            Ok(Box::new(provider))
        }
        mistral::PROVIDER_NAME => {
            let key = read_key("MISTRAL_API_KEY")?;
            let provider = match &provider_cfg.base_url {
                Some(url) => mistral::Mistral::with_base_url(key, url.clone()),
                None => mistral::Mistral::new(key),
            };
            Ok(Box::new(provider))
        }
        anthropic::PROVIDER_NAME => {
            let key = read_key("ANTHROPIC_API_KEY")?;
            let provider = match &provider_cfg.base_url {
                Some(url) => anthropic::Anthropic::with_base_url(key, url.clone()),
                None => anthropic::Anthropic::new(key),
            };
            Ok(Box::new(provider))
        }
        other => Err(ConfigError::UnknownProvider(other.to_string())),
    }
}

fn read_key(env_var: &'static str) -> Result<String, ConfigError> {
    std::env::var(env_var).map_err(|_| ConfigError::MissingApiKey { env_var })
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

    use crate::config::{Config, ConfigError};

    #[test]
    fn build_unknown_provider_errors() {
        let cfg = Config::defaults();
        // SAFETY: build() reads env vars; remove all three so it can't accidentally succeed.
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::remove_var("MISTRAL_API_KEY");
        }
        // `Box<dyn LlmProvider>` isn't Debug, so use `let Err(...)` instead of `unwrap_err()`.
        let Err(err) = build("nopenope", &cfg) else { panic!("expected UnknownProvider"); };
        assert!(matches!(err, ConfigError::UnknownProvider(_)));
    }

    #[test]
    fn build_missing_key_errors() {
        let cfg = Config::defaults();
        unsafe { std::env::remove_var("OPENAI_API_KEY"); }
        let Err(err) = build("openai", &cfg) else { panic!("expected MissingApiKey"); };
        assert!(matches!(err, ConfigError::MissingApiKey { env_var: "OPENAI_API_KEY" }));
    }
}
