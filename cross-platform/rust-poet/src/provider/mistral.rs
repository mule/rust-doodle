use async_trait::async_trait;
use reqwest::Client;

use crate::provider::openai_like::OpenAiLikeClient;
use crate::provider::{LlmProvider, LlmRequest, LlmResponse, ProviderError};

const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1";
pub const PROVIDER_NAME: &str = "mistral";

pub struct Mistral {
    inner: OpenAiLikeClient,
}

impl Mistral {
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL.into())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAiLikeClient {
                http: Client::new(),
                base_url,
                bearer_token: api_key,
                provider_name: PROVIDER_NAME,
            },
        }
    }
}

#[async_trait]
impl LlmProvider for Mistral {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn generate(&self, req: &LlmRequest) -> Result<LlmResponse, ProviderError> {
        self.inner.send_chat_completion(req).await
    }
}
