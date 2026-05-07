use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "rust-poet", about = "Generate poems using configurable LLM providers")]
pub struct Cli {
    /// Provider name (overrides config default)
    #[arg(long)]
    pub provider: Option<String>,

    /// Model name (overrides per-provider default)
    #[arg(long)]
    pub model: Option<String>,

    /// Topic source: wikipedia | random | fixed (overrides config default)
    #[arg(long)]
    pub source: Option<String>,

    /// Topic string (required when --source=fixed)
    #[arg(long)]
    pub topic: Option<String>,

    /// Sampling temperature (overrides config default)
    #[arg(long)]
    pub temperature: Option<f32>,

    /// Max tokens for the response (overrides config default)
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Path to a poet.ron config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Print the effective merged config and exit
    #[arg(long)]
    pub print_config: bool,

    /// Enable debug-level tracing
    #[arg(short, long)]
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_args() {
        let cli = Cli::try_parse_from(["rust-poet"]).unwrap();
        assert!(cli.provider.is_none());
        assert!(!cli.print_config);
    }

    #[test]
    fn parses_provider_and_topic() {
        let cli = Cli::try_parse_from([
            "rust-poet",
            "--provider", "openai",
            "--source", "fixed",
            "--topic", "rain",
            "--max-tokens", "200",
            "--temperature", "0.5",
        ]).unwrap();
        assert_eq!(cli.provider.as_deref(), Some("openai"));
        assert_eq!(cli.source.as_deref(), Some("fixed"));
        assert_eq!(cli.topic.as_deref(), Some("rain"));
        assert_eq!(cli.max_tokens, Some(200));
        assert!((cli.temperature.unwrap() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_print_config_flag() {
        let cli = Cli::try_parse_from(["rust-poet", "--print-config"]).unwrap();
        assert!(cli.print_config);
    }
}
