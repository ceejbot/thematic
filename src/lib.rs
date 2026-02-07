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

pub use editors::*;
pub use errors::ThemeError;
pub use grouping::{find_extension_name, group_families};
pub use icon_files::IconFileManager;
