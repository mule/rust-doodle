# rust-doodle

A sandbox for Rust experiments. Each project under `cross-platform/` is an independent crate — there is no Cargo workspace, so commands are run per crate via `--manifest-path`.

## Prerequisites

- **Rust** (2024 edition) — install via [rustup](https://rustup.rs/).
- **Windows + MSVC**: `.cargo/config.toml` pins the linker to `rust-lld.exe` for faster linking. If your build complains it can't find `rust-lld.exe`, either install LLVM (which provides it) and put it on `PATH`, or simply delete `.cargo/config.toml` to fall back to the default MSVC linker. On Linux/macOS the setting is ignored.

## Projects

| Project | What it is |
|---|---|
| [`cross-platform/hello-rust`](cross-platform/hello-rust) | Minimal `println!` — sanity check that the toolchain works. |
| [`cross-platform/hello-bevy`](cross-platform/hello-bevy) | Bevy 0.18 hello-world: centered text, custom font, RON-loaded config. |
| [`cross-platform/hello-bevy-advanced`](cross-platform/hello-bevy-advanced) | Bevy 0.18 with a custom WGSL spotlight shader, particles, and animated text wave. |
| [`cross-platform/rust-poet`](cross-platform/rust-poet) | CLI that generates poems via OpenAI / Anthropic / Mistral, with pluggable topic sources (fixed text, random seeds, Wikipedia "On this day"). Loads API keys from `.env` or shell. See its [README](cross-platform/rust-poet/README.md). |

## Running the experiments

### hello-rust

```bash
cargo run --manifest-path cross-platform/hello-rust/Cargo.toml
```

### hello-bevy

```bash
cargo run --manifest-path cross-platform/hello-bevy/Cargo.toml
```

Assets (font, RON config) live in `cross-platform/hello-bevy/assets/` and are loaded via Bevy's `AssetServer`.

### hello-bevy-advanced

```bash
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
```

Adds a WGSL shader at `assets/shaders/spotlight.wgsl`, a particle system, and a text-wave effect. First build is slow because of Bevy; subsequent builds use the dev profile's `opt-level = 1` for the crate plus `opt-level = 3` for dependencies.

### rust-poet

```bash
# Build / test
cargo build --manifest-path cross-platform/rust-poet/Cargo.toml
cargo test  --manifest-path cross-platform/rust-poet/Cargo.toml

# Run with a fixed topic. Set OPENAI_API_KEY in the shell or in a .env file
# in the current working directory (dotenvy auto-loads it).
cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- \
    --source fixed --topic "rain on a tin roof"
```

Full CLI flags, config-file format, env vars, and feature flags are documented in [`cross-platform/rust-poet/README.md`](cross-platform/rust-poet/README.md).

## Repository layout

```
.cargo/config.toml          # global cargo settings (Windows linker)
.vscode/launch.json         # CodeLLDB debug configs per project
cross-platform/             # standalone crates
docs/superpowers/           # design specs and implementation plans
CLAUDE.md                   # guidance for Claude Code working in this repo
```

## Adding a new experiment

1. `cargo new --bin cross-platform/<name>` (or `--lib`).
2. Add the project to the table above.
3. If it needs runtime config (env vars, asset paths, large CLI surface), add a per-crate `README.md`.
4. Bevy projects: copy the `[profile.dev]` block from `hello-bevy/Cargo.toml` so dev iteration stays fast.

## License

[MIT](LICENSE)
