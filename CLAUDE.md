# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Lint/Test Commands

- Build: `cargo build`
- Check: `cargo check`
- Format: `cargo +nightly fmt --all`
- Lint: `cargo clippy --all-targets`
- Test all: `cargo nextest run --future-incompat-report` (or fallback to `cargo test`)
- Test single: `cargo nextest run <test_name>` (or fallback to `cargo test <test_name>`)
- Run binary: `cargo run -- <args>` (e.g., `cargo run -- vz catppuccin`)
- Release build: `cargo build --release`

## Code Style Guidelines

- **Imports**: Use module-level imports, grouped by std/external/crate (configured in .rustfmt.toml)
- **Types**: Prefer strong typing with proper error handling. Define message types in `src/messages.rs`.
- **Structure**: Use divergent-let for early returns when appropriate
- **Error Handling**: Avoid `unwrap()` (denied by clippy), prefer the Result pattern with crate-specific error types derived with `thiserror`. Define error types in `src/errors.rs`.
- **Formatting**: Use nightly rustfmt with project config
- **Naming**: Follow Rust conventions (snake_case for functions/variables, CamelCase for types)
- **Safety**: Unsafe code is denied by lints
- **Documentation**: Use doc comments (`//!` for module, `///` for items)

## Project Architecture

This is a bidirectional theme converter between VSCode and Zed editors. The architecture follows:

**Core Structure:**
- `src/editors/` - Contains VSCode and Zed specific implementations (extensions, themes, manifests)
- `src/convert/` - Bidirectional conversion logic between theme formats
- `src/errors.rs` - Centralized error types using `thiserror`
- `src/icon_files.rs` - Icon theme file management utilities

**Key Types:**
- `VsCodeExtension` and `ZedExtension` - Main extension containers implementing `Extension` trait
- `ThemeError` - Unified error handling across all operations
- Conversion functions in `convert/` module handle format translation

**CLI Interface:**
- Binary uses `clap` for command parsing with aliases (`vz`, `zv`, etc.)
- Supports fuzzy search for themes using string similarity matching
- Automatic installation to editor-specific directories
