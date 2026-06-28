//! Thematic
//!
//! A library for converting Zed color & icon themes to VSCode,
//! and vice versa. We do our best to translate equivalent concepts
//! so a theme in one editor looks like a theme in another. There might
//! be subtleties that only the human designer will be able to get right,
//! but this converter does a pretty good job.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(unsafe_code)]

pub mod convert;
pub mod editors;
pub mod errors;
pub mod grouping;
pub mod icon_files;

#[cfg(test)]
mod conversion_tests;

// Internal ergonomics: editor types stay reachable as `crate::Foo` (plus the
// `crate::vscode` / `crate::zed` module shortcuts) throughout the crate. Kept
// crate-private so the wildcard does not widen the public API surface.
pub(crate) use editors::*;
// Public façade — the curated surface external callers depend on. Lower-level
// theme representation types remain reachable under `thematic::editors::…`.
pub use editors::{Extension, ThemeFile, VsCodeExtension, VsCodeTheme, ZedExtension, ZedTheme, ZedThemeFamily};
pub use errors::ThemeError;
pub use grouping::{find_extension_name, group_families};
pub use icon_files::IconFileManager;
