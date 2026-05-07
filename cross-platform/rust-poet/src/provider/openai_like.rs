//! Shared request/response shapes for OpenAI-compatible providers (OpenAI, Mistral, Ollama-OpenAI-mode, Groq, ...).

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::provider::{LlmRequest, LlmResponse, Message, ProviderError, Role, Usage};

#[derive(Debug, Serialize)]
pub(crate) struct ChatRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<ChatMessage<'a>>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChatMessage<'a> {
    pub role: &'static str,
    pub content: &'a str,
}

impl<'a> From<&'a Message> for ChatMessage<'a> {
    fn from(m: &'a Message) -> Self {
        let role = match m.role {
            Role::System => "system",
            Role::User => "user",
        };
        ChatMessage { role, content: &m.content }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponse {
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<UsageDto>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Choice {
    pub message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChoiceMessage {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UsageDto {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ErrorBody {
    pub message: String,
}

pub(crate) struct OpenAiLikeClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) bearer_token: String,
    pub(crate) provider_name: &'static str,
}

impl OpenAiLikeClient {
    pub(crate) async fn send_chat_completion(
        &self,
        req: &LlmRequest,
    ) -> Result<LlmResponse, ProviderError> {
        let body = ChatRequest {
            model: &req.model,
            messages: req.messages.iter().map(ChatMessage::from).collect(),
            max_tokens: req.max_tokens,
            temperature: req.temperature,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.bearer_token)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(map_http_error(self.provider_name, status, resp).await);
        }

        let parsed: ChatResponse = resp.json().await?;
        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or(ProviderError::EmptyResponse { provider: self.provider_name })?;

        if choice.message.content.is_empty() {
            return Err(ProviderError::EmptyResponse { provider: self.provider_name });
        }

        Ok(LlmResponse {
            text: choice.message.content,
            model: parsed.model,
            usage: parsed.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            }),
        })
    }
}

async fn map_http_error(
    provider: &'static str,
    status: StatusCode,
    resp: reqwest::Response,
) -> ProviderError {
    match status.as_u16() {
        401 | 403 => ProviderError::Auth { provider },
        429 => {
            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs);
            ProviderError::RateLimited { retry_after }
        }
        500..=599 => ProviderError::ServerError { provider, status: status.as_u16() },
        _ => {
            let message = resp
                .json::<ErrorEnvelope>()
                .await
                .map(|e| e.error.message)
                .unwrap_or_else(|_| format!("HTTP {}", status.as_u16()));
            ProviderError::BadRequest { provider, message }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_request_with_required_fields() {
        let req = ChatRequest {
            model: "gpt-4o-mini",
            messages: vec![
                ChatMessage { role: "system", content: "be brief" },
                ChatMessage { role: "user", content: "hi" },
            ],
            max_tokens: 128,
            temperature: 0.7,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o-mini");
        assert_eq!(json["max_tokens"], 128);
        let temp = json["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 1e-6, "temperature {temp} not close to 0.7");
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(json["messages"][1]["content"], "hi");
    }

    #[test]
    fn parses_response_choices() {
        let body = r#"{
            "model": "gpt-4o-mini-2024-07-18",
            "choices": [
                {"message": {"role": "assistant", "content": "hello there"}}
            ],
            "usage": {"prompt_tokens": 10, "completion_tokens": 4, "total_tokens": 14}
        }"#;
        let parsed: ChatResponse = serde_json::from_str(body).unwrap();
        assert_eq!(parsed.choices[0].message.content, "hello there");
        assert_eq!(parsed.usage.unwrap().completion_tokens, 4);
    }

    #[test]
    fn parses_error_envelope() {
        let body = r#"{"error": {"message": "model not found", "type": "invalid_request"}}"#;
        let env: ErrorEnvelope = serde_json::from_str(body).unwrap();
        assert_eq!(env.error.message, "model not found");
    }
}
