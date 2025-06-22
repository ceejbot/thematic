//! Zed Theme Converter Library
//!
//! This library provides Rust data structures and utilities for working with both VSCode and Zed
//! color themes. It enables reading, writing, and converting between the two theme formats.
//!
//! # Overview
//!
//! The library consists of two main modules:
//! - [`vscode_theme`] - Data structures for VSCode color themes
//! - [`thematic`] - Data structures for Zed themes
//!
//! # Examples
//!
//! ## Loading a VSCode theme
//!
//! ```rust,no_run
//! use thematic::VSCodeTheme;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let theme = VSCodeTheme::load("path/to/theme.json")?;
//! println!("Theme name: {}", theme.name);
//! println!("Is dark theme: {}", theme.is_dark_theme());
//! # Ok(())
//! # }
//! ```
//!
//! ## Loading a Zed theme
//!
//! ```rust,no_run
//! use thematic::ZedThemeFamily;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let theme_family = ZedThemeFamily::load("path/to/theme.json")?;
//! println!("Theme family: {}", theme_family.name);
//! println!("Author: {}", theme_family.author);
//! for theme in &theme_family.themes {
//!     println!("  - {} ({:?})", theme.name, theme.appearance);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating a new VSCode theme
//!
//! ```rust
//! use thematic::themes::vscode::{VSCodeTheme, TokenColorRule, TokenColorSettings, TokenScope};
//!
//! let mut theme = VSCodeTheme::new("My Custom Theme".to_string());
//! theme.theme_type = Some("dark".to_string());
//! theme.set_color("editor.background".to_string(), "#1e1e1e".to_string());
//! theme.set_color("editor.foreground".to_string(), "#d4d4d4".to_string());
//!
//! // Add token colors
//! let comment_rule = TokenColorRule::single_scope(
//!     "comment".to_string(),
//!     TokenColorSettings::foreground("#6A9955".to_string())
//! );
//! theme.add_token_rule(comment_rule);
//! ```
//!
//! ## Creating a new Zed theme
//!
//! ```rust
//! use thematic::themes::zed::{ZedThemeFamily, ZedTheme, Appearance, ZedThemeStyle};
//!
//! let theme = ZedTheme {
//!     name: "My Custom Theme".to_string(),
//!     appearance: Appearance::Dark,
//!     style: ZedThemeStyle {
//!         background: Some("#1e1e1e".to_string()),
//!         text: Some("#d4d4d4".to_string()),
//!         editor_background: Some("#1e1e1e".to_string()),
//!         editor_foreground: Some("#d4d4d4".to_string()),
//!         ..Default::default()
//!     },
//! };
//!
//! let theme_family = ZedThemeFamily {
//!     schema: Some("https://zed.dev/schema/themes/v0.2.0.json".to_string()),
//!     name: "My Custom Theme Family".to_string(),
//!     author: "Your Name".to_string(),
//!     themes: vec![theme],
//! };
//! ```

pub mod convert;
pub mod editors;
pub mod errors;

#[cfg(test)]
mod conversion_tests;

pub use convert::*;
pub use editors::*;
pub use errors::ThemeError;
