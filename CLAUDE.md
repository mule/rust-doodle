# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Sandbox repo for Rust experiments. Each project is a standalone crate — there is no workspace.

## Build Commands

Each project must be built individually via `--manifest-path`:

```bash
cargo build --manifest-path cross-platform/hello-bevy/Cargo.toml
cargo build --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
cargo build --manifest-path cross-platform/hello-rust/Cargo.toml
cargo run --manifest-path cross-platform/hello-bevy/Cargo.toml
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
cargo build --manifest-path cross-platform/rust-poet/Cargo.toml
cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- --topic rain
cargo test --manifest-path cross-platform/rust-poet/Cargo.toml
```

## Architecture

- **No Cargo workspace** — projects are independent crates under topic directories (e.g., `cross-platform/`).
- **Linker**: `.cargo/config.toml` uses `rust-lld.exe` for faster linking on Windows MSVC.
- **Bevy projects** use `opt-level = 1` for dev builds and `opt-level = 3` for dependencies to balance compile time vs runtime performance.
- **Assets** (fonts, textures, shaders) go in each project's `assets/` directory, loaded via Bevy's `AssetServer`.
- **Custom shaders** (WGSL) go in `assets/shaders/` and are loaded as asset paths (e.g., `"shaders/spotlight.wgsl"`).
- **VS Code** launch configs are in `.vscode/launch.json` using CodeLLDB for debugging.
