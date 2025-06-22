# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Lint/Test Commands bash commands

- Build: `cargo build`
- Check: `cargo check`
- Format: `cargo +nightly fmt --all`
- Lint: `cargo clippy --all-targets`
- Test all: `cargo nextest run --future-incompat-report`
- Test single: `cargo nextest run <test_name>`
- Run CI checks: `just ci`
- Setup environment: `just setup`

## Code Style Guidelines

- **Imports**: Use module-level imports, grouped by std/external/crate (configured in .rustfmt.toml)
- **Types**: Prefer strong typing with proper error handling. Define message types in `src/messages.rs`.
- **Structure**: Use divergent-let for early returns when appropriate
- **Error Handling**: Avoid `unwrap()` (denied by clippy), prefer the Result pattern with crate-specific error types derived with `thiserror`. Define error types in `src/errors.rs`.
- **Formatting**: Use nightly rustfmt with project config
- **Naming**: Follow Rust conventions (snake_case for functions/variables, CamelCase for types)
- **Safety**: Unsafe code is denied by lints
- **Documentation**: Use doc comments (`//!` for module, `///` for items)
