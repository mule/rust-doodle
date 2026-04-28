use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::provider::{LlmProvider, LlmRequest, LlmResponse, ProviderError, Role, Usage};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const PROVIDER_NAME: &str = "anthropic";

pub struct Anthropic {
    http: Client,
    base_url: String,
    api_key: String,
}

impl Anthropic {
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL.into())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self { http: Client::new(), base_url, api_key }
    }
}

#[derive(Debug, Serialize)]
struct MsgRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<UserMsg<'a>>,
}

#[derive(Debug, Serialize)]
struct UserMsg<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct MsgResponse {
    model: String,
    content: Vec<ContentBlock>,
    #[serde(default)]
    usage: Option<UsageDto>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct UsageDto {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    message: String,
}

#[async_trait]
impl LlmProvider for Anthropic {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn generate(&self, req: &LlmRequest) -> Result<LlmResponse, ProviderError> {
        // Anthropic: system is a top-level field, not part of messages.
        let mut system_buf: Vec<&str> = Vec::new();
        let mut user_messages: Vec<UserMsg> = Vec::new();
        for m in &req.messages {
            match m.role {
                Role::System => system_buf.push(&m.content),
                Role::User => user_messages.push(UserMsg { role: "user", content: &m.content }),
            }
        }
        let system_joined: Option<String> =
            (!system_buf.is_empty()).then(|| system_buf.join("\n\n"));

        let body = MsgRequest {
            model: &req.model,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            system: system_joined.as_deref(),
            messages: user_messages,
        };

        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(map_error(status, resp).await);
        }

        let parsed: MsgResponse = resp.json().await?;
        let text = parsed
            .content
            .into_iter()
            .find(|b| b.kind == "text")
            .map(|b| b.text)
            .ok_or(ProviderError::EmptyResponse { provider: PROVIDER_NAME })?;

        if text.is_empty() {
            return Err(ProviderError::EmptyResponse { provider: PROVIDER_NAME });
        }

        Ok(LlmResponse {
            text,
            model: parsed.model,
            usage: parsed.usage.map(|u| Usage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
            }),
        })
    }
}

async fn map_error(status: StatusCode, resp: reqwest::Response) -> ProviderError {
    match status.as_u16() {
        401 | 403 => ProviderError::Auth { provider: PROVIDER_NAME },
        429 => {
            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs);
            ProviderError::RateLimited { retry_after }
        }
        500..=599 => ProviderError::ServerError { provider: PROVIDER_NAME, status: status.as_u16() },
        _ => {
            let message = resp
                .json::<ErrorEnvelope>()
                .await
                .map(|e| e.error.message)
                .unwrap_or_else(|_| format!("HTTP {}", status.as_u16()));
            ProviderError::BadRequest { provider: PROVIDER_NAME, message }
        }
    }
}
