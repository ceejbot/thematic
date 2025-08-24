# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

**Thematic** is a bidirectional theme converter between VSCode and Zed editors (v0.1.3). Currently macOS-only.

### Implemented Features
✅ VSCode → Zed color theme conversion (`vz` command)
✅ Zed → VSCode color theme conversion (`zv` command)
✅ Theme listing and fuzzy search for both editors
✅ Automatic theme family grouping for Zed
✅ Extension manifest generation for both formats
✅ Installation to default editor directories
⚠️  Partial icon theme support (file detection works, copying not yet implemented)

### Known Limitations
- macOS only (Linux/Windows paths not implemented)
- Icon theme conversion incomplete (file copying needed)
- Some Zed-specific theme features not fully mapped to VSCode
- Code needs cleanup and refactoring (see TODOs in code)

## Build/Lint/Test Commands

**Note:** There's a `.justfile` (with leading dot!) containing convenience recipes.

### Just Recipes
- `just test` - Run all tests using nextest
- `just ci` - Run tests, clippy, and formatting (requires nightly)
- `just lint` - Run clippy fixes and format code
- `just setup` - Install required development tools
- `just version BUMP` - Tag a new version (patch/minor/major)
- `just release` - Build and release for both Apple architectures

### Direct Cargo Commands
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
- `src/main.rs` - CLI entry point with clap commands
- `src/lib.rs` - Library interface (docs TODO)
- `src/editors/` - Contains VSCode and Zed specific implementations
  - `vscode/` - VSCode extension, theme, manifest, and icon theme modules
  - `zed/` - Zed extension, theme, manifest, icon theme, and installed extensions modules
- `src/convert/` - Bidirectional conversion logic between theme formats
  - `code_to_zed.rs` - VSCode to Zed conversion logic
  - `zed_to_code.rs` - Zed to VSCode conversion logic
- `src/errors.rs` - Centralized error types using `thiserror`
- `src/icon_files.rs` - Icon theme file management utilities
- `src/conversion_tests.rs` - Integration tests for conversions

**Key Types:**
- `VsCodeExtension` and `ZedExtension` - Main extension containers implementing `Extension` trait
- `ThemeError` - Unified error handling across all operations
- Conversion functions in `convert/` module handle format translation

**CLI Interface:**
- Binary uses `clap` for command parsing with aliases:
  - `vz` / `vscode-to-zed` - Convert VSCode themes to Zed
  - `zv` / `zed-to-vscode` - Convert Zed themes to VSCode
  - `zed` / `zed-list` - List Zed themes
  - `vsc` / `vscode-list` - List VSCode themes
- Supports fuzzy search for themes using string similarity matching
- Automatic installation to editor-specific directories

## Codebase Orientation

### Key Files to Start With
1. `src/main.rs` - CLI command handling and orchestration
2. `src/editors/mod.rs` - Extension trait definition and common logic
3. `src/convert/mod.rs` - Entry point for conversion logic
4. `src/errors.rs` - All error types used throughout

### Directory Layout
```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library interface
├── errors.rs               # Error types
├── icon_files.rs           # Icon file utilities
├── conversion_tests.rs     # Integration tests
├── editors/
│   ├── mod.rs             # Extension trait and common code
│   ├── vscode/            # VSCode-specific implementation
│   └── zed/               # Zed-specific implementation
└── convert/
    ├── mod.rs             # Conversion module interface
    ├── code_to_zed.rs     # VSCode → Zed conversion
    └── zed_to_code.rs     # Zed → VSCode conversion
```

### Testing Approach
- Unit tests colocated with implementation files
- Integration tests in `src/conversion_tests.rs`
- Test fixtures in `fixtures/` directory
- Schema files in `schemas/` directory for reference

## Outstanding TODOs (from codebase)

1. **Library Documentation** - Add usage docs to `src/lib.rs:9`
2. **Family Groupings** - Add more assertions about family groupings in `src/editors/mod.rs:397`
3. **Extension Consideration** - Consider approach in `src/editors/vscode/extension.rs:83`
4. **Manifest Sub-pieces** - Write sub-pieces in `src/editors/zed/manifest.rs:125`
5. **Extension Types** - Look up full list of extension types in `src/editors/zed/installed.rs:25`

## Development Tasks (from README)

- [ ] Handle some parts of Zed themes that VSCode doesn't do
- [ ] Convert icon themes fully (copy files etc.)
- [ ] Polish up the user-visible output
- [ ] Clean up code architecture
- [ ] Detect Linux and use appropriate paths
- [ ] Maybe try the Zed Windows beta too
