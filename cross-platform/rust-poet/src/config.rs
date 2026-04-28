use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub default_provider: String,
    pub default_source: String,
    pub providers: BTreeMap<String, ProviderConfig>,
    pub poem: PoemConfig,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub model: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoemConfig {
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing API key: set environment variable {env_var}")]
    MissingApiKey { env_var: &'static str },

    #[error("config file parse error at {path}: {source}")]
    FileParse {
        path: PathBuf,
        #[source]
        source: ron::de::SpannedError,
    },

    #[error("config file IO error at {path}: {source}")]
    FileIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("unknown provider '{0}'")]
    UnknownProvider(String),

    #[error("unknown source '{0}'")]
    UnknownSource(String),

    #[error("invalid value for {field}: {message}")]
    InvalidValue { field: &'static str, message: String },
}

impl Config {
    pub fn defaults() -> Self {
        let mut providers = BTreeMap::new();
        providers.insert(
            "openai".into(),
            ProviderConfig { model: "gpt-4o-mini".into(), base_url: None },
        );
        providers.insert(
            "anthropic".into(),
            ProviderConfig { model: "claude-haiku-4-5".into(), base_url: None },
        );
        providers.insert(
            "mistral".into(),
            ProviderConfig { model: "mistral-small-latest".into(), base_url: None },
        );
        Config {
            default_provider: "openai".into(),
            default_source: "wikipedia".into(),
            providers,
            poem: PoemConfig { max_tokens: 400, temperature: 0.9 },
            request_timeout_secs: 30,
        }
    }

    pub fn from_ron_str(s: &str) -> Result<Self, ron::de::SpannedError> {
        ron::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_three_providers() {
        let c = Config::defaults();
        assert_eq!(c.default_provider, "openai");
        assert_eq!(c.providers.len(), 3);
        assert!(c.providers.contains_key("openai"));
        assert!(c.providers.contains_key("anthropic"));
        assert!(c.providers.contains_key("mistral"));
    }

    #[test]
    fn parses_sample_ron() {
        let s = r#"(
            default_provider: "anthropic",
            default_source: "random",
            providers: {
                "openai":    (model: "gpt-4o-mini",          base_url: None),
                "anthropic": (model: "claude-haiku-4-5",     base_url: None),
                "mistral":   (model: "mistral-small-latest", base_url: None),
            },
            poem: (max_tokens: 200, temperature: 1.1),
            request_timeout_secs: 45,
        )"#;
        let c = Config::from_ron_str(s).unwrap();
        assert_eq!(c.default_provider, "anthropic");
        assert_eq!(c.default_source, "random");
        assert_eq!(c.poem.max_tokens, 200);
        assert_eq!(c.request_timeout_secs, 45);
    }
}
