use anyhow::{Context, Result};
use clap::Parser;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use rust_poet::cli::Cli;
use rust_poet::{Config, PoemSettings, Poet};
use rust_poet::{provider, topic};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env from cwd (or any ancestor) before anything reads env. Existing process
    // env wins — `export FOO=bar` overrides `FOO=baz` in .env. Missing .env is fine;
    // anything else (malformed syntax, permission denied) gets a one-line warning so
    // the user isn't silently running with stale env. Runs before init_tracing, so we
    // use eprintln! rather than tracing::warn!.
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) if e.not_found() => {}
        Err(e) => eprintln!("warning: ignoring .env: {e}"),
    }

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let mut cfg = Config::load(cli.config.clone()).context("loading config")?;

    // Apply per-call CLI overrides on top of the loaded config.
    let provider_name = cli.provider.clone().unwrap_or_else(|| cfg.default_provider.clone());
    let source_name = cli.source.clone().unwrap_or_else(|| cfg.default_source.clone());
    if let Some(t) = cli.temperature {
        cfg.poem.temperature = t;
    }
    if let Some(mt) = cli.max_tokens {
        cfg.poem.max_tokens = mt;
    }
    let model = match cli.model.clone() {
        Some(m) => m,
        None => cfg
            .providers
            .get(&provider_name)
            .map(|p| p.model.clone())
            .ok_or_else(|| {
                anyhow::anyhow!("unknown provider '{provider_name}' (no entry in config)")
            })?,
    };

    if cli.print_config {
        // Print the effective config (with overrides applied) and exit.
        let s = ron::ser::to_string_pretty(&cfg, ron::ser::PrettyConfig::default())
            .context("serializing config")?;
        println!("{s}");
        println!("# effective provider: {provider_name}");
        println!("# effective source:   {source_name}");
        println!("# effective model:    {model}");
        return Ok(());
    }

    debug!(provider = %provider_name, source = %source_name, model = %model, "resolved selection");

    let provider = provider::build(&provider_name, &cfg).context("building provider")?;
    let source = topic::build(&source_name, cli.topic.as_deref()).context("building source")?;

    let settings = PoemSettings {
        model,
        max_tokens: cfg.poem.max_tokens,
        temperature: cfg.poem.temperature,
    };

    let poet = Poet::new(provider, source, settings);
    let poem = poet.generate().await.context("generating poem")?;

    info!(provider = %poem.provider, model = %poem.model, "poem generated");
    println!("{}", poem.text);
    Ok(())
}

fn init_tracing(verbose: bool) {
    let default_level = if verbose { "debug" } else { "warn" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    tracing_subscriber::fmt().with_env_filter(filter).with_target(false).init();
}
