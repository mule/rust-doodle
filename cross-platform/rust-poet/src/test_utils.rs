//! Test utilities for downstream consumers. Gated behind the `test-utils` feature.

use async_trait::async_trait;
use std::sync::Mutex;

use crate::provider::{LlmProvider, LlmRequest, LlmResponse, ProviderError, Usage};

pub struct MockLlmProvider {
    text: String,
    model: String,
    last_request: Mutex<Option<LlmRequest>>,
}

impl MockLlmProvider {
    pub fn with_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            model: "mock-model".into(),
            last_request: Mutex::new(None),
        }
    }

    pub fn with_text_and_model(text: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            model: model.into(),
            last_request: Mutex::new(None),
        }
    }

    pub fn last_request(&self) -> Option<LlmRequest> {
        self.last_request.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    async fn generate(&self, req: &LlmRequest) -> Result<LlmResponse, ProviderError> {
        *self.last_request.lock().unwrap() = Some(req.clone());
        Ok(LlmResponse {
            text: self.text.clone(),
            model: self.model.clone(),
            usage: Some(Usage { prompt_tokens: 0, completion_tokens: 0 }),
        })
    }
}
