# A guided tour of `rust-poet` for new Rustaceans

This document walks through every source file in `rust-poet`, using the codebase as an excuse to introduce idiomatic Rust patterns. It's not a from-scratch Rust tutorial — keep [The Rust Book](https://doc.rust-lang.org/book/) open in another tab and follow the "Read more" links at the end of each section when you want to dig deeper.

**How to read this:**

- Files are presented in *learning order*, not alphabetical order. Cargo manifest first, then the simplest module, then the more interesting ones.
- Each section starts with **what the file does** in one paragraph, then walks through the most teaching-rich excerpts.
- "Rust concept spotlight" callouts highlight idioms you'll see in nearly every Rust project.
- Each section ends with a **"Try it"** exercise — usually a small temporary edit you make, run, and revert. The point isn't to keep the change; it's to *see* the compiler / runtime react. Skipping the exercises is fine on a first read; doing them is how the concepts stick.

If you've never compiled this app, run [the Quick start](README.md#quick-start) first so you can match each file against actual behavior.

---

## 0. The big picture

```
                    ┌──────────┐
   command-line ──▶ │   CLI    │  cli.rs (clap derive)
                    └────┬─────┘
                         ▼
                   ┌───────────┐
   poet.ron ─────▶ │  Config   │  config.rs (serde + RON + thiserror)
                   └─────┬─────┘
                         │ chooses
              ┌──────────┴──────────┐
              ▼                     ▼
       ┌────────────┐        ┌────────────┐
       │TopicSource │        │LlmProvider │  traits in topic/mod.rs and provider/mod.rs
       └─────┬──────┘        └─────┬──────┘
             │   topic              │   poem
             └──────────┬───────────┘
                        ▼
                   ┌──────────┐
                   │   Poet   │  poet.rs (orchestrator)
                   └────┬─────┘
                        ▼
                  poem text on stdout
```

The shape is: **CLI args + config file → choose provider + source → orchestrate them → print poem.** Everything else is plumbing for those four steps.

---

## 1. `Cargo.toml` — the package manifest

Cargo is Rust's build tool, package manager, and dependency resolver in one. Every Rust project has a `Cargo.toml` describing what to build and what it depends on.

Open [`Cargo.toml`](Cargo.toml). The interesting parts:

```toml
[package]
name = "rust-poet"
version = "0.1.0"
edition = "2024"
```

The **edition** controls which language idioms the compiler uses. `2024` is the most recent edition and enables modern syntax (e.g. `let-else`).

```toml
[lib]
path = "src/lib.rs"

[[bin]]
name = "rust-poet"
path = "src/main.rs"
```

This crate is **both a library and a binary**. `src/lib.rs` exposes the `rust_poet::*` API for tests and downstream users; `src/main.rs` is the executable. The integration tests in `tests/` link against the library, not the binary.

```toml
[features]
test-utils = []
live-tests = []
```

**Cargo features** are compile-time flags. Enabling `test-utils` makes `MockLlmProvider` available; enabling `live-tests` compiles the real-API integration tests. Both default to *off*.

```toml
[dev-dependencies]
wiremock = "0.6"
```

`[dev-dependencies]` are only compiled for tests, examples, and benchmarks. Production builds skip them.

> **Try it:**
> 1. Run `cargo tree --manifest-path Cargo.toml` and find which dependencies pull in `tokio`. More than one does — that's why Cargo's resolver picks one shared version of `tokio` for the whole build.
> 2. Compare a plain `cargo build --manifest-path Cargo.toml` against `cargo build --manifest-path Cargo.toml --features test-utils`. The second one compiles `src/test_utils.rs`; the first skips it. To prove it, drop a `compile_error!("included");` line inside `test_utils.rs` and re-run both — only the feature-on build fails. (Remove the line afterward.)

> **Read more:** [The Cargo Book — Manifest format](https://doc.rust-lang.org/cargo/reference/manifest.html)

---

## 2. `src/lib.rs` — declaring modules

A library crate's `lib.rs` is the **root of its module tree**. It declares which top-level modules exist and what's publicly visible.

```rust
pub mod cli;
pub mod config;
pub mod error;
pub mod poet;
pub mod prompt;
pub mod provider;
pub mod topic;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub use config::Config;
pub use error::PoetError;
pub use poet::{Poem, PoemSettings, Poet};
```

Three things to notice:

1. **`pub mod foo;`** tells the compiler "load `foo.rs` (or `foo/mod.rs`) and make it part of the public API." Without `pub`, the module exists but external users can't reach it.
2. **`#[cfg(any(test, feature = "test-utils"))]`** — a *conditional compilation attribute*. The `test_utils` module only exists when running unit tests *or* when someone enables the `test-utils` feature. In any other build it's as if the file weren't there.
3. **`pub use config::Config;`** is a *re-export*. It lets users write `rust_poet::Config` instead of `rust_poet::config::Config`. Re-exports flatten the public API and let you reorganize internals without breaking callers.

> **Rust concept spotlight: the module system.**
> Every `.rs` file or directory is a *module*. Modules form a tree rooted at `lib.rs` (or `main.rs` for binaries). Items default to private; `pub` exposes them one level up.

> **Try it:** Comment out the line `pub use poet::{Poem, PoemSettings, Poet};` in `lib.rs` and run `cargo check --manifest-path Cargo.toml`. The compiler will pinpoint exactly where in `main.rs` the re-export was being used (`Poet`, `PoemSettings`). That's the re-export earning its keep — without it, every caller would have to write the longer path `rust_poet::poet::Poet`. Restore the line when done.

> **Read more:** [The Book — Managing growing projects](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)

---

## 3. `src/main.rs` — the binary entry point

This is the file that gets executed when you run `cargo run` or the compiled binary. Open [`src/main.rs`](src/main.rs).

### Imports

```rust
use anyhow::{Context, Result};
use clap::Parser;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use rust_poet::cli::Cli;
use rust_poet::{Config, PoemSettings, Poet};
use rust_poet::{provider, topic};
```

`use` brings names into scope. `rust_poet::` is the *library half* of this crate — the binary uses its own library, just like an external user would. Notice the asymmetry: `Cli` is reached via the long path `rust_poet::cli::Cli` because it's not re-exported in `lib.rs`, while `Config`, `PoemSettings`, and `Poet` use the short form thanks to the `pub use` lines at the top of the library. (See §2's "Try it" — comment out a re-export and watch this very file fail to compile.)

### The async entry point

```rust
#[tokio::main]
async fn main() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) if e.not_found() => {}
        Err(e) => eprintln!("warning: ignoring .env: {e}"),
    }
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let mut cfg = Config::load(cli.config.clone()).context("loading config")?;
    ...
}
```

The `dotenvy::dotenv()` call has three outcomes worth distinguishing: success (loaded), "no `.env` here" (perfectly normal — silent), and "something is wrong with the `.env` you have" (warn so the user isn't surprised when their key isn't there). The match makes those three buckets explicit. We use `eprintln!` rather than `tracing::warn!` because `init_tracing` hasn't run yet.

Several Rust-flavored things in three lines:

- **`#[tokio::main]`** is a *procedural macro* that wraps your `async fn main` in code that starts a Tokio runtime and runs the future to completion. Without it, `async fn main` doesn't compile — Rust's standard library doesn't ship an async executor.
- **`Result<()>`** uses `anyhow::Result`, which is a type alias for `Result<T, anyhow::Error>`. The `()` (unit type) means "succeeds with no value." On failure, `main` returns an error and Cargo prints it.
- **The `?` operator** at the end of `Config::load(...).context(...)?` says "if this is `Err`, return early from this function with that error." It's the cornerstone of Rust error handling.
- **`.context("loading config")`** wraps the inner error with extra context. When the program fails, you'll see both layers: `Error: loading config / Caused by: ...`.

### CLI overrides on top of config

```rust
let provider_name = cli.provider.clone().unwrap_or_else(|| cfg.default_provider.clone());
let source_name = cli.source.clone().unwrap_or_else(|| cfg.default_source.clone());
if let Some(t) = cli.temperature {
    cfg.poem.temperature = t;
}
```

- **`Option::unwrap_or_else(|| ...)`** — if `cli.provider` is `Some(x)`, return `x`; else call the closure. Closures are written `|args| body`. The closure is only evaluated on the `None` branch, so we don't clone the config string unnecessarily.
- **`if let Some(t) = ...`** is the destructuring shorthand for "if it's `Some`, bind the inner value to `t` and run the block." It's equivalent to a one-arm `match`.

### Building the components

```rust
let provider = provider::build(&provider_name, &cfg).context("building provider")?;
let source = topic::build(&source_name, cli.topic.as_deref()).context("building source")?;

let settings = PoemSettings { model, max_tokens: cfg.poem.max_tokens, temperature: cfg.poem.temperature };
let poet = Poet::new(provider, source, settings);
let poem = poet.generate().await.context("generating poem")?;
println!("{}", poem.text);
Ok(())
```

`provider::build` and `topic::build` are factory functions that return `Box<dyn LlmProvider>` / `Box<dyn TopicSource>` — heap-allocated trait objects. We'll see how those work in the provider section.

> **Rust concept spotlight: the `?` operator.**
> Rust has no exceptions. Errors are values returned in `Result<T, E>`. `?` is sugar for "if `Err`, propagate it; if `Ok`, unwrap it." Combined with `From` conversions, it lets multi-step pipelines stay flat: `let x = step1()?; let y = step2(x)?;` instead of nested `match`.

> **Try it:** See `.context()` build a real error chain. Run:
> ```bash
> cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- --config does-not-exist.ron
> ```
> You should see something like:
> ```
> Error: loading config
> Caused by:
>     0: config file IO error at does-not-exist.ron: ...
>     1: The system cannot find the file specified. (os error 2)
> ```
> The top frame came from `main.rs`'s `.context("loading config")`. The middle frame came from `ConfigError::FileIo`. The bottom frame came from `std::io::Error`. Three layers, zero `try`/`catch` boilerplate — that's what `?` and `.context()` buy you.

> **Read more:** [The Book — Recoverable errors with `Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html), [Asynchronous programming in Rust](https://rust-lang.github.io/async-book/)

---

## 4. `src/cli.rs` — declarative argument parsing with clap

This file defines the command-line interface. [Clap](https://docs.rs/clap)'s `derive` feature generates the parser from a struct.

```rust
#[derive(Parser, Debug, Clone)]
#[command(name = "rust-poet", about = "Generate poems using configurable LLM providers")]
pub struct Cli {
    #[arg(long)]
    pub provider: Option<String>,

    #[arg(long)]
    pub topic: Option<String>,

    #[arg(short, long)]
    pub verbose: bool,
    // ...
}
```

- **`#[derive(Parser)]`** generates a full argument parser at compile time. The struct *is* the spec; there's no separate config language.
- **`Option<String>`** flags are optional; if the user doesn't pass `--provider`, the field is `None`. Required flags would be `String`.
- **`#[arg(short, long)]`** says "accept both `-v` and `--verbose`."

### Why this pattern is nice

The struct doubles as documentation. Anyone reading `Cli` immediately knows what flags exist, their types, and (with rustdoc comments) their meaning. There's no string-based "argument lookup" at runtime — every flag is a typed field.

> **Rust concept spotlight: derive macros.**
> `#[derive(...)]` runs a procedural macro at compile time that generates code (impls, structs) based on your type. Common derives: `Debug` for `{:?}` formatting, `Clone`, `Default`, `PartialEq`. Library macros like `Parser`, `Deserialize`, `Error` extend this mechanism for whole subsystems.

> **Try it:** Add a `--dry-run` flag to `Cli`:
> ```rust
> /// Skip the LLM call; just print what would have happened.
> #[arg(long)]
> pub dry_run: bool,
> ```
> Run `cargo run --manifest-path Cargo.toml -- --help` and confirm `--dry-run` shows up in the auto-generated help text, with your doc comment as its description. You don't have to wire the flag to do anything yet — the point is to feel how clap picks up new fields automatically. Revert when done, or wire it into `main.rs` and skip the `poet.generate()` call when `cli.dry_run` is true.

> **Read more:** [Clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)

---

## 5. `src/prompt.rs` — pure functions and string building

Small file, easy entry point. Open [`src/prompt.rs`](src/prompt.rs).

```rust
const SYSTEM_PROMPT: &str = "You are a poet. ...";

pub fn build(topic: &Topic) -> Vec<Message> {
    let mut user = String::new();
    user.push_str("Write a short poem about: ");
    user.push_str(&topic.seed);

    if let Some(ctx) = &topic.context {
        user.push_str("\n\nBackground (use as inspiration, do not quote):\n");
        user.push_str(ctx);
    }

    vec![
        Message { role: Role::System, content: SYSTEM_PROMPT.to_string() },
        Message { role: Role::User, content: user },
    ]
}
```

Things to notice:

- **`&str` vs `String`.** `&str` is a borrowed string slice (a view into UTF-8 bytes someone else owns). `String` is an owned, heap-allocated, growable buffer. `SYSTEM_PROMPT` is `&str` because it's baked into the binary. `user` is `String` because we're building it up at runtime. The `.to_string()` on `SYSTEM_PROMPT` allocates because `Message::content` wants ownership.
- **`vec![...]`** is a macro that builds a `Vec<T>`. The element type `Message` is inferred.
- **`if let Some(ctx) = &topic.context`** borrows the optional context. If we wrote `if let Some(ctx) = topic.context` without the `&`, we'd be trying to *move* `ctx` out of `topic`, which the compiler would reject because `topic` is a shared reference (`&Topic`).

This file teaches a useful baseline: **most Rust functions look like functions in any other language.** Borrow what you can read, allocate (`String`, `Vec`) what you need to mutate or return.

> **Try it:** Add a third test to `mod tests` in `prompt.rs`:
> ```rust
> #[test]
> fn omits_background_when_no_context() {
>     let topic = Topic { seed: "stars".into(), context: None };
>     let msgs = build(&topic);
>     assert!(!msgs[1].content.contains("Background"));
> }
> ```
> Run `cargo test --manifest-path Cargo.toml prompt::`. The `prompt::` filter restricts execution to tests whose path contains "prompt", so you don't run the whole suite while iterating.

> **Read more:** [The Book — Strings](https://doc.rust-lang.org/book/ch08-02-strings.html)

---

## 6. `src/config.rs` — serde, RON, and `thiserror`

This file does three teaching-rich things: defines configuration data types, deserializes them from a config file, and defines its own error type. Open [`src/config.rs`](src/config.rs).

### Data types with `serde`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub default_provider: String,
    pub default_source: String,
    pub providers: BTreeMap<String, ProviderConfig>,
    pub poem: PoemConfig,
    pub request_timeout_secs: u64,
}
```

- **`Deserialize` and `Serialize`** are derive macros from [serde](https://serde.rs/). With those four lines, the struct can be read from RON, JSON, YAML, TOML, etc., and written back out.
- **`BTreeMap<K, V>`** is an ordered map (a B-tree). We use it instead of `HashMap` because it serializes in deterministic order — useful for `--print-config` output.

### Errors with `thiserror`

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing API key: set environment variable {env_var}")]
    MissingApiKey { env_var: &'static str },

    #[error("config file parse error at {path}: {source}")]
    FileParse {
        path: PathBuf,
        #[source]
        source: Box<ron::de::SpannedError>,
    },

    #[error("unknown provider '{0}'")]
    UnknownProvider(String),
    // ...
}
```

- **`#[derive(Error)]`** from `thiserror` generates `impl std::error::Error` plus `Display` based on your `#[error("...")]` strings. Field names in `{braces}` interpolate.
- **Tuple variants** like `UnknownProvider(String)` use `{0}` to refer to the first field.
- **`#[source]`** marks a field as the underlying cause. Tools like `anyhow` walk this chain to print "Caused by: ..." trails.
- **`Box<...>`** wraps a large field on the heap so the enum's overall size stays small. This keeps `Result<Config, ConfigError>` cheap to return on the success path.

> **Rust concept spotlight: `thiserror` for libraries, `anyhow` for binaries.**
> The convention: library crates define typed errors with `thiserror::Error` so callers can match on variants. Binaries use `anyhow::Result` for ergonomics — it accepts any error and adds `.context(...)` chains. Look back at `main.rs`: it imports `anyhow::Result`, while `config.rs` defines a typed `ConfigError`.

### Config file discovery

```rust
pub fn load(explicit_path: Option<PathBuf>) -> Result<Self, ConfigError> {
    if let Some(path) = explicit_path {
        return Self::load_from_file(&path);
    }
    let cwd_path = PathBuf::from("poet.ron");
    if cwd_path.is_file() {
        return Self::load_from_file(&cwd_path);
    }
    if let Some(dirs) = directories::ProjectDirs::from("", "", "rust-poet") {
        let user_path = dirs.config_dir().join("poet.ron");
        if user_path.is_file() {
            return Self::load_from_file(&user_path);
        }
    }
    Ok(Self::defaults())
}
```

A clean example of the **fall-through pattern**: try increasingly fuzzy lookups, return as soon as one hits, fall back to a default if none do. Each branch returns early with `return`, so the final `Ok(Self::defaults())` is the "no file found" case.

> **Try it:** Make a malformed config and watch the layered error fire. Create `bad.ron` in the repo root with the literal content:
> ```
> (missing_brace
> ```
> Then run:
> ```bash
> cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- --config bad.ron
> ```
> You'll see *three* layers chained: `loading config` (from `main.rs`), `config file parse error at bad.ron: ...` (from `ConfigError::FileParse`'s `#[error("...")]` template), and the underlying RON parser's complaint. Notice how the path field interpolates into the `{path}` placeholder — that's `thiserror` doing the work. Delete `bad.ron` when done.

> **Read more:** [The Book — Defining error types](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html), [serde](https://serde.rs/)

---

## 7. `src/error.rs` — composing error types

```rust
#[derive(Debug, Error)]
pub enum PoetError {
    #[error(transparent)]
    Provider(#[from] ProviderError),

    #[error(transparent)]
    TopicSource(#[from] TopicError),

    #[error(transparent)]
    Config(#[from] ConfigError),
}
```

This is the *top-level* library error — what `Poet::generate` returns. Two new things:

- **`#[error(transparent)]`** says "use the inner error's `Display` and `source` directly." `PoetError::Provider(err)` prints exactly like `err`.
- **`#[from]`** generates an `impl From<ProviderError> for PoetError`. That's what makes the `?` operator work across error types: any function returning `Result<_, PoetError>` can use `?` on a `Result<_, ProviderError>` and the conversion happens automatically.

So in `poet.rs`:

```rust
let topic = self.source.next_topic().await?;        // TopicError → PoetError
let resp = self.provider.generate(&req).await?;     // ProviderError → PoetError
```

Both `?`s do an automatic `From` conversion. Without the `#[from]` derive you'd have to call `.map_err(PoetError::Provider)?` manually.

> **Try it:** Temporarily delete the `#[from]` attribute on the `Provider` variant in `error.rs`. Run `cargo check --manifest-path Cargo.toml`. The compiler error will say something like *"the trait `From<ProviderError>` is not implemented for `PoetError`"* and point at the `?` in `poet.rs`'s `generate()`. Now you have two ways to fix it: put `#[from]` back, or change the call site to `.map_err(PoetError::Provider)?`. Both work. The derive is just less typing — and the compiler error showed you what code it was generating.

> **Read more:** [thiserror docs](https://docs.rs/thiserror/), [The `?` operator and `From`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator)

---

## 8. `src/provider/mod.rs` — traits and trait objects

This is the big one for new Rustaceans. Open [`src/provider/mod.rs`](src/provider/mod.rs).

### The trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn generate(&self, req: &LlmRequest) -> Result<LlmResponse, ProviderError>;
}
```

A **trait** is Rust's interface mechanism: a set of method signatures that types can implement. Every detail here is teaching-worthy:

- **`#[async_trait]`** is a macro that lets us write `async fn` in a trait. Bare async-trait methods got stabilized recently but `async-trait` is still common, especially for object-safe traits (next bullet).
- **`Send + Sync`** are *auto traits* the compiler implements for you. `Send` means "safe to move to another thread"; `Sync` means "safe to share `&T` across threads." Tokio's executor needs both. Constraining the trait to require both means anything implementing `LlmProvider` is automatically thread-safe.
- **`&self`** means the method only reads from `self`. The trait deliberately uses `&self`, not `&mut self`, so that callers can hold an `Arc<dyn LlmProvider>` shared across tasks. If state needs to mutate (like the mock recording the last request), the impl uses *interior mutability* (e.g. `Mutex`).
- **`-> Result<LlmResponse, ProviderError>`** — typed errors. Callers can match on variants like `ProviderError::RateLimited { retry_after }`.

### The factory function

```rust
pub fn build(name: &str, cfg: &Config) -> Result<Box<dyn LlmProvider>, ConfigError> {
    let provider_cfg = cfg.providers.get(name)
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
        // ...
    }
}
```

This is a **runtime polymorphism** pattern in Rust. `Box<dyn LlmProvider>` is a *trait object*: a heap-allocated value plus a hidden vtable pointer. The caller doesn't know whether they got an `OpenAi`, `Anthropic`, or `Mistral` — only that it implements `LlmProvider`.

Compare this to:
- **Java/C#**: `LlmProvider provider = new OpenAi(...)` — interface dispatch is implicit.
- **Go**: `var provider LlmProvider = &OpenAi{...}` — interface dispatch is implicit.
- **Rust**: you must opt in by writing `Box<dyn LlmProvider>` (or `&dyn LlmProvider`, or `Arc<dyn LlmProvider>`). The compiler wants you to choose: heap-allocated and owned (`Box`), shared and reference-counted (`Arc`), or a temporary borrow (`&dyn`).

> **Rust concept spotlight: traits and trait objects.**
> Traits are how Rust expresses "any type that can do X." When the *type* matters, you use generics: `fn process<P: LlmProvider>(p: P)`. When the type must be chosen at runtime — like here, where `--provider` decides — you use trait objects: `Box<dyn LlmProvider>`. Generics are zero-cost; trait objects cost one extra pointer dereference per method call.

> **Try it:** Try removing `: Send + Sync` from the `LlmProvider` trait declaration. Run `cargo build --manifest-path Cargo.toml`. You'll get a long compiler error pointing at `Box::new(provider)` in `provider::build` (or further upstream). The message will say something like *"`(dyn LlmProvider + 'static)` cannot be sent between threads safely"*. That's Tokio's executor — its `spawn` requires futures to be `Send`, and a non-`Send` provider would poison the whole future. Those bounds aren't decorative; remove them and the async pipeline collapses.

> **Read more:** [The Book — Trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)

---

## 9. `src/provider/openai.rs` and `openai_like.rs` — composition over inheritance

A common question for newcomers: **"Where's `extends` in Rust?"** Answer: there isn't one. Rust has no class inheritance. To share behavior, you compose.

Open [`src/provider/openai.rs`](src/provider/openai.rs):

```rust
pub struct OpenAi {
    inner: OpenAiLikeClient,
}

impl OpenAi {
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
impl LlmProvider for OpenAi {
    fn name(&self) -> &'static str { PROVIDER_NAME }

    async fn generate(&self, req: &LlmRequest) -> Result<LlmResponse, ProviderError> {
        self.inner.send_chat_completion(req).await
    }
}
```

`OpenAi` is a **wrapper** around `OpenAiLikeClient`. The shared HTTP/serialization logic lives in `openai_like.rs` and is used by anyone whose API speaks the OpenAI dialect (Mistral does; Anthropic doesn't). To share it, the concrete type `OpenAi` *contains* an `OpenAiLikeClient` and forwards the work.

If you peek at `mistral.rs` you'll see the same pattern. `anthropic.rs` does its own thing because the Anthropic API has a different request shape — the lack of inheritance forces you to be explicit about what's actually shared.

### Lifetimes in `openai_like.rs`

```rust
#[derive(Debug, Serialize)]
pub(crate) struct ChatRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<ChatMessage<'a>>,
    pub max_tokens: u32,
    pub temperature: f32,
}
```

The **`<'a>`** is a *lifetime parameter*. It says "this struct borrows data, and `'a` is a name for the scope of that borrow." Every `&str` or `&T` field then refers to that same lifetime. The compiler uses this to enforce: the borrowed data lives at least as long as the struct.

In practice, lifetimes are how Rust avoids needless copying when serializing: we send borrowed pointers to the message strings instead of cloning every string into a temporary. Most beginners can get away with letting the compiler infer lifetimes — they only become explicit on struct fields and public function signatures with multiple references.

### Visibility: `pub(crate)`

```rust
pub(crate) struct OpenAiLikeClient { ... }
```

`pub(crate)` means "public *within this crate*, but invisible to external users." Plain `pub` would expose this type as part of `rust_poet`'s API. We don't want that — `OpenAiLikeClient` is an implementation detail.

> **Try it:** Add a stub for an OpenAI-compatible provider — say, Groq. About 30 lines total:
> 1. Copy `src/provider/mistral.rs` to `src/provider/groq.rs`. Change `PROVIDER_NAME` to `"groq"` and `DEFAULT_BASE_URL` to `"https://api.groq.com/openai/v1"`.
> 2. Add `pub mod groq;` in `src/provider/mod.rs` next to the others.
> 3. In the `build` function, mirror the Mistral arm — read `GROQ_API_KEY`, build a `groq::Groq`, box it.
> 4. Add `"groq": (model: "llama-3.3-70b-versatile", base_url: None)` to your `poet.example.ron`'s `providers` map.
> 5. Run `cargo run --manifest-path Cargo.toml -- --provider groq --print-config`. (`--print-config` exits before reading the API key, so you don't need a real Groq account to verify wiring.)
>
> Notice how almost nothing in `openai_like.rs` had to change: composition + a thin wrapper struct is the entire cost of a new provider when its API speaks the OpenAI dialect.

> **Read more:** [The Book — Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html), [Rust API guidelines on visibility](https://rust-lang.github.io/api-guidelines/predictability.html)

---

## 10. `src/topic/mod.rs` and submodules — mirroring the provider pattern

Topic sources mirror providers exactly: a trait, three implementations, a factory. Open [`src/topic/mod.rs`](src/topic/mod.rs).

```rust
#[async_trait]
pub trait TopicSource: Send + Sync {
    fn name(&self) -> &'static str;
    async fn next_topic(&self) -> Result<Topic, TopicError>;
}

pub fn build(name: &str, topic_arg: Option<&str>) -> Result<Box<dyn TopicSource>, ConfigError> {
    match name {
        "fixed" => {
            let seed = topic_arg.ok_or(ConfigError::InvalidValue { ... })?;
            Ok(Box::new(fixed::FixedTopic::new(seed)))
        }
        "random" => Ok(Box::new(random::RandomTopic::new())),
        "wikipedia" => {
            // WikipediaOnThisDay::new() returns Result because it builds an HTTP
            // client (which can fail at TLS/DNS init). Convert into ConfigError
            // so the factory's signature stays uniform.
            let source = wikipedia::WikipediaOnThisDay::new()
                .map_err(|e| ConfigError::InvalidValue {
                    field: "source 'wikipedia'",
                    message: e.to_string(),
                })?;
            Ok(Box::new(source))
        }
        other => Err(ConfigError::UnknownSource(other.to_string())),
    }
}
```

The most interesting impl is [`src/topic/wikipedia.rs`](src/topic/wikipedia.rs):

```rust
pub struct WikipediaOnThisDay {
    http: Client,
    base_url: String,
    date: NaiveDate,
    rng: Mutex<StdRng>,
}
```

Why is the RNG behind a `Mutex`? Because `next_topic(&self)` takes `&self` (shared reference), but generating a random number requires mutating the RNG's internal state. **`Mutex` provides interior mutability** — the ability to mutate through a shared reference, with runtime locking to keep things safe.

```rust
let idx = {
    let mut rng = self.rng.lock().expect("rng poisoned");
    rng.random_range(0..feed.events.len())
};
```

The braces form a *block expression* whose value is the last expression. We acquire the lock, grab the index, and the `MutexGuard` is dropped when the block ends — releasing the lock as quickly as possible.

> **Rust concept spotlight: interior mutability.**
> Rust's default rule is "either many shared references *or* one mutable reference, never both." For genuinely shared mutable state, escape hatches exist: `RefCell` (single-threaded, runtime-checked), `Mutex`/`RwLock` (multi-threaded, blocking), `Atomic*` (lock-free primitives). They all work *through* `&self`, by checking borrow rules at runtime instead of compile time.

> **Try it:** Open `src/topic/wikipedia.rs` and try changing the field `rng: Mutex<StdRng>` to `rng: StdRng`. Then in `next_topic()`, replace the entire `let idx = { ... };` block (the one that calls `self.rng.lock()`) with a single line: `let idx = self.rng.random_range(0..feed.events.len());`. Run `cargo build --manifest-path Cargo.toml`. The error you'll get is *"cannot borrow `self.rng` as mutable, as `self` is a `&` reference"*. That's the borrow checker enforcing the rule mentioned above. Two ways out:
> - **(a)** Put the `Mutex` back — interior mutability through `&self`. ✓ Works for shared instances.
> - **(b)** Change the trait method to `async fn next_topic(&mut self)` — but that breaks the `Box<dyn TopicSource>` sharing model and requires every caller to hold an exclusive reference.
>
> Option (a) is what real concurrent code reaches for. The borrow checker just refused to let you fake it.

> **Read more:** [The Book — `RefCell` and interior mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)

---

## 11. `src/poet.rs` — the orchestrator

This is the smallest "interesting" file: it's the glue between provider and topic source. Open [`src/poet.rs`](src/poet.rs).

```rust
pub struct Poet {
    provider: Box<dyn LlmProvider>,
    source: Box<dyn TopicSource>,
    settings: PoemSettings,
}

impl Poet {
    pub fn new(
        provider: Box<dyn LlmProvider>,
        source: Box<dyn TopicSource>,
        settings: PoemSettings,
    ) -> Self {
        Self { provider, source, settings }
    }

    pub async fn generate(&self) -> Result<Poem, PoetError> {
        let topic = self.source.next_topic().await?;
        let messages = prompt::build(&topic);
        let req = LlmRequest {
            messages,
            model: self.settings.model.clone(),
            max_tokens: self.settings.max_tokens,
            temperature: self.settings.temperature,
        };
        let resp = self.provider.generate(&req).await?;
        Ok(Poem {
            text: resp.text,
            topic,
            provider: self.provider.name(),
            model: resp.model,
        })
    }
}
```

`Poet` *owns* its provider and source — they're stored as `Box<dyn ...>` fields, not borrowed. That's the simplest ownership model: when the `Poet` is dropped, the boxes are dropped, which drops the underlying `OpenAi`/`Anthropic`/etc.

The `generate` method is a clean linear pipeline: get a topic → build messages → call the LLM → assemble a `Poem`. Errors short-circuit through `?`, so the happy path reads top-to-bottom with no nesting.

This is the payoff for everything in sections 7–10: the orchestrator doesn't care which provider or source it has. It just calls trait methods.

> **Try it:** Trace a deep error all the way through the orchestrator. Create a `.env` file in your current directory containing exactly:
> ```
> OPENAI_API_KEY=intentionally-bogus
> ```
> Then run:
> ```bash
> cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- \
>     --source fixed --topic "x" --max-tokens 1
> ```
> You'll see something like:
> ```
> Error: generating poem
> Caused by:
>     0: auth failed for provider 'openai'
> ```
> Map each line back to its source: "generating poem" came from `main.rs`'s `.context("generating poem")?` on the `poet.generate()` call. "auth failed for provider 'openai'" came from `openai_like.rs`'s `map_http_error` translating a 401 into `ProviderError::Auth`. The `?` in `Poet::generate` then ran `From<ProviderError> for PoetError` (free, via `#[from]`), and `?` in `main.rs` ran `From<PoetError> for anyhow::Error` (free, via `thiserror`'s blanket impl). Six lines of glue covered four error types. Delete `.env` when done.

---

## 12. `src/test_utils.rs` — feature-gated mocks

Open [`src/test_utils.rs`](src/test_utils.rs):

```rust
//! Test utilities for downstream consumers. Gated behind the `test-utils` feature.

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
    // ...
}
```

A few things to notice:

- **`//!` comments** at the top of the file are *inner doc comments* — they document the module itself, not the next item. Run `cargo doc --open` to see how they render.
- **`impl Into<String>`** is a flexible parameter type. It accepts anything convertible to `String`: a `String`, a `&str`, even a `Cow<str>`. Inside the function we call `.into()` to do the conversion. This is the standard Rust pattern for "I want a String but don't make my callers write `.to_string()` everywhere."
- **`Mutex<Option<LlmRequest>>`** combines two ideas: `Option` because the mock starts with no recorded request, `Mutex` because we need interior mutability through `&self` (same reasoning as the Wikipedia RNG).

Why feature-gate this? `MockLlmProvider` is useful for *consumers*' tests, not just our own — but they don't want it in their production builds. The `test-utils` feature makes it opt-in.

> **Try it:** Add a test to `tests/poet_pipeline.rs` that wires `MockLlmProvider` through the full `Poet` pipeline and asserts the mock's text appears on the other side:
> ```rust
> #[tokio::test]
> async fn mock_returns_its_text_through_poet() {
>     let provider = Box::new(MockLlmProvider::with_text("expected text"));
>     let source = Box::new(rust_poet::topic::fixed::FixedTopic::new("rain"));
>     let settings = PoemSettings { model: "test-model".into(), max_tokens: 1, temperature: 0.0 };
>
>     let poem = Poet::new(provider, source, settings).generate().await.unwrap();
>
>     assert_eq!(poem.text, "expected text");
>     assert_eq!(poem.provider, "mock"); // captured from LlmProvider::name()
> }
> ```
> Run `cargo test --manifest-path Cargo.toml --features test-utils mock_returns`. If you forget `--features test-utils`, the test file is gated out (look at the `#![cfg(feature = "test-utils")]` at the top of `poet_pipeline.rs`) and the test silently doesn't run — proof the gate works.
>
> **Extension.** `MockLlmProvider` also has a `last_request()` method to interrogate what the mock saw. To use it, you'd need to keep a handle to the mock *after* `Box::new` consumes it. The standard pattern is `Arc<T>` plus a small forwarding `impl<T: LlmProvider + ?Sized> LlmProvider for Arc<T>` (about 7 lines including the `#[async_trait]` macro). Because of the orphan rule, that impl has to live inside `rust_poet` (e.g., `src/test_utils.rs` gated by the `test-utils` feature) — an integration test in `tests/` cannot define it. Try wiring this up if you want the practice.

> **Read more:** [The Cargo Book — Features](https://doc.rust-lang.org/cargo/reference/features.html)

---

## 13. Tests: unit vs integration

Rust has two kinds of tests, and `rust-poet` uses both.

### Unit tests live alongside the code

Look at the bottom of nearly every source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_three_providers() {
        let c = Config::defaults();
        assert_eq!(c.default_provider, "openai");
    }
}
```

- **`#[cfg(test)]`** says "only compile this module when running tests." Production builds skip it entirely.
- **`mod tests`** is a child module *inside* the file under test. It can see private items via `use super::*;`. That's how you test internal helpers without making them public.

### Integration tests live in `tests/`

Files under `tests/` are *separate crates* that link against your library through its public API. They cannot see anything that isn't `pub`, and they cannot see `#[cfg(test)]` items in the library — only `cfg(test)` *for the integration test itself*.

```rust
// tests/poet_pipeline.rs
#![cfg(feature = "test-utils")]

use rust_poet::poet::{Poem, PoemSettings, Poet};
use rust_poet::test_utils::MockLlmProvider;
```

The `#![cfg(feature = "test-utils")]` at the top is an *inner* attribute — it gates the whole file. We need it because `MockLlmProvider` is itself feature-gated; without this gate, a default `cargo test` would fail to compile this file.

### The wrapper-file trick for `tests/live/`

Cargo only treats top-level files in `tests/` as integration test crates. To put related tests in a subdirectory, you need a wrapper at the top level that pulls them in:

```rust
// tests/live_tests.rs
#![cfg(feature = "live-tests")]

mod live {
    pub mod openai;
    pub mod anthropic;
    pub mod mistral;
}
```

Now `tests/live/openai.rs`, `tests/live/anthropic.rs`, and `tests/live/mistral.rs` are part of the same `live_tests` integration crate, and they only compile when the feature is on.

> **Try it:** Feel the wall between integration tests and private items. Create `tests/private_helper.rs` with:
> ```rust
> use rust_poet::config::Config;
> use std::path::PathBuf;
>
> #[test]
> fn touches_private_helper() {
>     // load_from_file is `fn`, not `pub fn` — see if you can call it.
>     let _ = Config::load_from_file(&PathBuf::from("poet.ron"));
> }
> ```
> Run `cargo test --manifest-path Cargo.toml`. You'll get *"function `load_from_file` is private"*. That's the rule: integration tests link against the public API only. The fix is to move the test into `src/config.rs`'s own `#[cfg(test)] mod tests` block — unit tests share the parent module's privacy and can reach private items via `super::*`. Delete `tests/private_helper.rs` when done.

> **Read more:** [The Book — Test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

---

## 14. Where to go from here

Now that you've toured the codebase, here are good next steps depending on your interest:

- **Want to deepen Rust fundamentals?** Read [The Rust Book](https://doc.rust-lang.org/book/) chapters 4 (ownership), 10 (generics, traits, lifetimes), 13 (closures, iterators), and 15 (smart pointers).
- **Want hands-on practice?** Try [Rustlings](https://github.com/rust-lang/rustlings) — small bug-fix exercises grouped by topic.
- **Want to extend `rust-poet`?**
  - Add a new provider (e.g. Cohere, Groq). Most of OpenAI-shaped APIs can reuse `openai_like.rs`. Pattern: copy `mistral.rs`, change the URL constant, register in `provider/mod.rs`'s `build`.
  - Add a new topic source. Pattern: copy `random.rs` if it's offline, `wikipedia.rs` if it hits an HTTP API. Register in `topic/mod.rs`'s `build`.
  - Add a `--seed` flag to `cli.rs` that makes `random` and `wikipedia` deterministic. Both already accept an `rng_seed: Option<u64>` in their test constructors.
- **Curious how async actually works?** Read the [async book](https://rust-lang.github.io/async-book/) — it's short and explains what `.await` does under the hood.

When something in the code surprises you, run `cargo doc --open --manifest-path Cargo.toml` to see the rendered docs for the whole dependency tree. It's the fastest way to learn a crate's API.
