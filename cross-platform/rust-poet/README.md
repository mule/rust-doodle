# rust-poet

A small CLI + library that generates short poems by combining an **LLM provider** with a **topic source**.

- **Providers**: OpenAI, Anthropic, Mistral (each with optional `base_url` override for OpenAI-compatible endpoints).
- **Topic sources**: `fixed` (your text), `random` (a built-in seed list), `wikipedia` ("On this day" events).

> **New to Rust?** Read the [**guided walkthrough**](WALKTHROUGH.md) — a tour of every source file that uses this codebase to introduce idiomatic Rust patterns.

Design rationale lives in [`docs/superpowers/specs/2026-04-27-rust-poet-design.md`](../../docs/superpowers/specs/2026-04-27-rust-poet-design.md).

## Quick start

You can supply your API key either via the shell or a `.env` file. Pick one:

```bash
# Option A: shell env (transient)
export OPENAI_API_KEY=sk-...   # Windows PowerShell: $env:OPENAI_API_KEY = "sk-..."

# Option B: .env file in the directory you run from (loaded automatically by dotenvy)
echo "OPENAI_API_KEY=sk-..." > .env

# Then:
cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- \
    --source fixed --topic "rain on a tin roof"
```

You should see a few lines of poem on stdout. If no key is found in either place, the CLI exits with a clear error.

> **Note:** `.env` is in the repo `.gitignore`. Never commit it.

## Build, test, lint

```bash
cargo build --manifest-path cross-platform/rust-poet/Cargo.toml
cargo test  --manifest-path cross-platform/rust-poet/Cargo.toml
cargo clippy --manifest-path cross-platform/rust-poet/Cargo.toml --all-targets
```

The default `cargo test` run uses mocked HTTP via `wiremock` — **no API calls, no keys needed**.

## CLI flags

| Flag | Purpose |
|---|---|
| `--provider <name>` | `openai` \| `anthropic` \| `mistral`. Overrides `default_provider` in config. |
| `--model <id>` | Model id (e.g. `gpt-4o-mini`, `claude-haiku-4-5`). Overrides per-provider default. |
| `--source <name>` | `fixed` \| `random` \| `wikipedia`. Overrides `default_source`. |
| `--topic <text>` | Required when `--source=fixed`. Ignored otherwise. |
| `--temperature <f>` | Sampling temperature (e.g. `0.9`). |
| `--max-tokens <n>` | Response length cap. |
| `--config <path>` | Explicit RON config file. Overrides discovery. |
| `--print-config` | Print the effective merged config (defaults + file + CLI overrides) and exit. Great for debugging. |
| `-v`, `--verbose` | Enable `debug`-level tracing. |

Set `RUST_LOG=rust_poet=trace` to override the log filter directly.

## API keys

Provider keys are **only** read from the process environment — they are never read from the RON config file. Only the key for the selected provider is required.

| Provider | Env var |
|---|---|
| `openai` | `OPENAI_API_KEY` |
| `anthropic` | `ANTHROPIC_API_KEY` |
| `mistral` | `MISTRAL_API_KEY` |

### `.env` support

At startup, the CLI calls `dotenvy::dotenv()` which loads `.env` from the current working directory (or any ancestor). Both shell exports and `.env` work; if both supply the same variable, **the shell wins** — `dotenvy` does not overwrite already-set variables. This is how you can safely keep a `.env` for everyday use while overriding it ad-hoc from the shell.

A typical `.env` looks like:

```
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
# RUST_LOG=rust_poet=debug   # uncomment for verbose logs without -v
```

`.env` is ignored by `.gitignore` at the repo root — keep it that way.

## Configuration file

The CLI looks for `poet.ron` in this order, stopping at the first hit:

1. `--config <path>` if given.
2. `./poet.ron` in the current working directory.
3. The user-config directory:
   - Linux: `$XDG_CONFIG_HOME/rust-poet/poet.ron` (or `~/.config/rust-poet/poet.ron`)
   - macOS: `~/Library/Application Support/rust-poet/poet.ron`
   - Windows: `%APPDATA%\rust-poet\poet.ron`
4. Built-in defaults (no file needed).

A documented sample is at [`poet.example.ron`](poet.example.ron). Copy it to one of the locations above and edit:

```ron
(
    default_provider: "openai",
    default_source:   "wikipedia",
    providers: {
        "openai":    (model: "gpt-4o-mini",          base_url: None),
        "anthropic": (model: "claude-haiku-4-5",     base_url: None),
        "mistral":   (model: "mistral-small-latest", base_url: None),
    },
    poem: (max_tokens: 400, temperature: 0.9),
    request_timeout_secs: 30,
)
```

The `base_url` field lets you point any provider at an OpenAI-compatible proxy (Ollama, LiteLLM, an internal gateway, …) instead of the vendor endpoint.

## Topic sources

| Name | Behavior |
|---|---|
| `fixed` | Uses `--topic <text>` verbatim as the seed. |
| `random` | Picks a seed from a built-in list (no network). |
| `wikipedia` | Fetches today's events from Wikipedia's "On this day" feed and samples one. Network required. |

## Examples

All commands below use `--manifest-path` so they run from the repo root. If you `cd cross-platform/rust-poet` first, you can drop the flag and just write `cargo run -- ...`.

```bash
# Default provider (from config), Wikipedia topic
cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- --source wikipedia

# Anthropic with a fixed topic and lower temperature
cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- \
    --provider anthropic --source fixed \
    --topic "the smell of an old book" --temperature 0.5

# Inspect what config the CLI will actually use
cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- --provider mistral --print-config
```

## Cargo features

| Feature | What it enables |
|---|---|
| `test-utils` | Exposes `rust_poet::test_utils::MockLlmProvider` for downstream tests. Also gates the `tests/poet_pipeline.rs` integration test. |
| `live-tests` | Runs `tests/live_tests.rs` against the real OpenAI/Anthropic/Mistral APIs. Each test individually skips if its env var is unset, so you can run with any subset of keys. |

```bash
# Run live tests (will hit real APIs and consume tokens)
cargo test --manifest-path cross-platform/rust-poet/Cargo.toml --features live-tests

# Default test run (mocked, no keys needed)
cargo test --manifest-path cross-platform/rust-poet/Cargo.toml
```

## Library use

The crate is also a library. The top-level types are re-exported for convenience:

```rust
use rust_poet::{Config, Poem, Poet, PoemSettings};
use rust_poet::{provider, topic};

let cfg = Config::load(None)?;
let provider = provider::build("openai", &cfg)?;
let source   = topic::build("fixed", Some("autumn light"))?;
let settings = PoemSettings { model: "gpt-4o-mini".into(), max_tokens: 200, temperature: 0.8 };

let poem = Poet::new(provider, source, settings).generate().await?;
println!("{}", poem.text);
```

## Layout

```
src/
  cli.rs           # clap CLI definition
  config.rs        # RON config + discovery + ConfigError
  error.rs         # PoetError (top-level error type)
  poet.rs          # Poet orchestrator: source -> prompt -> provider -> Poem
  prompt.rs        # Prompt construction
  provider/        # LlmProvider trait + OpenAI / Anthropic / Mistral impls
  topic/           # TopicSource trait + fixed / random / wikipedia impls
  test_utils.rs    # MockLlmProvider (gated by `test-utils` feature)
tests/
  poet_pipeline.rs # end-to-end test using MockLlmProvider
  live_tests.rs    # real-API tests (gated by `live-tests` feature)
poet.example.ron   # sample config
```
