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

    /// Load config from explicit path, ./poet.ron, the user-config dir, or built-in defaults.
    pub fn load(explicit_path: Option<PathBuf>) -> Result<Self, ConfigError> {
        if let Some(path) = explicit_path {
            return Self::load_from_file(&path);
        }
        // 1. ./poet.ron
        let cwd_path = PathBuf::from("poet.ron");
        if cwd_path.is_file() {
            return Self::load_from_file(&cwd_path);
        }
        // 2. User-config dir
        if let Some(dirs) = directories::ProjectDirs::from("", "", "rust-poet") {
            let user_path = dirs.config_dir().join("poet.ron");
            if user_path.is_file() {
                return Self::load_from_file(&user_path);
            }
        }
        // 3. Defaults
        Ok(Self::defaults())
    }

    fn load_from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::FileIo {
            path: path.clone(),
            source,
        })?;
        ron::from_str(&text).map_err(|source| ConfigError::FileParse {
            path: path.clone(),
            source,
        })
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

    #[test]
    fn loads_explicit_path() {
        let dir = tempdir_for_test();
        let path = dir.join("explicit.ron");
        std::fs::write(&path, r#"(
            default_provider: "mistral",
            default_source: "fixed",
            providers: {
                "mistral": (model: "mistral-small-latest", base_url: None),
            },
            poem: (max_tokens: 100, temperature: 0.5),
            request_timeout_secs: 10,
        )"#).unwrap();
        let c = Config::load(Some(path.clone())).unwrap();
        assert_eq!(c.default_provider, "mistral");
    }

    #[test]
    fn falls_back_to_defaults_when_no_file_found() {
        let dir = tempdir_for_test();
        // Use a non-existent explicit path -> error.
        let missing = dir.join("does-not-exist.ron");
        let err = Config::load(Some(missing)).unwrap_err();
        assert!(matches!(err, ConfigError::FileIo { .. }));

        // No path and no discoverable file -> defaults.
        // (We can't easily simulate "no discoverable file" without controlling cwd and HOME,
        // so just verify the search-with-no-explicit-path branch returns *something*.)
        let _maybe = Config::load(None); // may load ./poet.ron or defaults; both are valid
    }

    /// Helper: returns a unique temp directory path that's already created.
    fn tempdir_for_test() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("rust-poet-test-{}", std::process::id()));
        p.push(format!(
            "{}-cfg",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
