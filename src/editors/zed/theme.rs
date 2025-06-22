//! Zed theme data structures
//!
//! This module contains the Rust structures that correspond to the Zed theme schema.
//! Based on the Zed v0.2.0 theme schema and example files.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::ThemeError;
use crate::editors::ThemeFile;

/// A complete Zed theme family containing metadata and one or more themes
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZedThemeFamily {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub author: String,
    pub name: String,
    pub themes: Vec<ZedTheme>,
}

impl ThemeFile for ZedThemeFamily {
    // cheat! cheat!
    type T = ZedThemeFamily;

    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError> {
        let content = fs::read_to_string(path)?;
        let theme: ZedThemeFamily = serde_json::from_str(&content)?;
        Ok(theme)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError> {
        Ok(serde_json::from_slice::<ZedThemeFamily>(bytes)?)
    }
}

/// A single theme within a theme family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZedTheme {
    pub name: String,
    pub appearance: Appearance,
    pub style: ZedThemeStyle,
}

/// Theme appearance - light or dark
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Light,
    Dark,
}

/// Main theme style containing all color and styling definitions
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZedThemeStyle {
    pub accents: Option<Vec<String>>,

    // Background appearance
    #[serde(rename = "background.appearance")]
    pub background_appearance: Option<BackgroundAppearance>,

    // Base colors
    pub background: Option<String>,
    pub border: Option<String>,
    #[serde(rename = "border.disabled")]
    pub border_disabled: Option<String>,
    #[serde(rename = "border.focused")]
    pub border_focused: Option<String>,
    #[serde(rename = "border.selected")]
    pub border_selected: Option<String>,
    #[serde(rename = "border.transparent")]
    pub border_transparent: Option<String>,
    #[serde(rename = "border.variant")]
    pub border_variant: Option<String>,

    // Surface colors
    #[serde(rename = "elevated_surface.background")]
    pub elevated_surface_background: Option<String>,
    #[serde(rename = "surface.background")]
    pub surface_background: Option<String>,
    #[serde(rename = "drop_target.background")]
    pub drop_target_background: Option<String>,

    // Element colors
    #[serde(rename = "element.active")]
    pub element_active: Option<String>,
    #[serde(rename = "element.background")]
    pub element_background: Option<String>,
    #[serde(rename = "element.disabled")]
    pub element_disabled: Option<String>,
    #[serde(rename = "element.hover")]
    pub element_hover: Option<String>,
    #[serde(rename = "element.selected")]
    pub element_selected: Option<String>,

    // Ghost element colors
    #[serde(rename = "ghost_element.active")]
    pub ghost_element_active: Option<String>,
    #[serde(rename = "ghost_element.background")]
    pub ghost_element_background: Option<String>,
    #[serde(rename = "ghost_element.disabled")]
    pub ghost_element_disabled: Option<String>,
    #[serde(rename = "ghost_element.hover")]
    pub ghost_element_hover: Option<String>,
    #[serde(rename = "ghost_element.selected")]
    pub ghost_element_selected: Option<String>,

    // Text colors
    pub text: Option<String>,
    #[serde(rename = "text.accent")]
    pub text_accent: Option<String>,
    #[serde(rename = "text.disabled")]
    pub text_disabled: Option<String>,
    #[serde(rename = "text.muted")]
    pub text_muted: Option<String>,
    #[serde(rename = "text.placeholder")]
    pub text_placeholder: Option<String>,

    // Icon colors
    pub icon: Option<String>,
    #[serde(rename = "icon.accent")]
    pub icon_accent: Option<String>,
    #[serde(rename = "icon.disabled")]
    pub icon_disabled: Option<String>,
    #[serde(rename = "icon.muted")]
    pub icon_muted: Option<String>,
    #[serde(rename = "icon.placeholder")]
    pub icon_placeholder: Option<String>,

    // UI component backgrounds
    #[serde(rename = "status_bar.background")]
    pub status_bar_background: Option<String>,
    #[serde(rename = "title_bar.background")]
    pub title_bar_background: Option<String>,
    #[serde(rename = "title_bar.inactive_background")]
    pub title_bar_inactive_background: Option<String>,
    #[serde(rename = "toolbar.background")]
    pub toolbar_background: Option<String>,
    #[serde(rename = "tab_bar.background")]
    pub tab_bar_background: Option<String>,
    #[serde(rename = "tab.active_background")]
    pub tab_active_background: Option<String>,
    #[serde(rename = "tab.inactive_background")]
    pub tab_inactive_background: Option<String>,

    // Search and panel colors
    #[serde(rename = "search.match_background")]
    pub search_match_background: Option<String>,
    #[serde(rename = "panel.background")]
    pub panel_background: Option<String>,
    #[serde(rename = "panel.focused_border")]
    pub panel_focused_border: Option<String>,
    #[serde(rename = "panel.indent_guide")]
    pub panel_indent_guide: Option<String>,
    #[serde(rename = "panel.indent_guide_active")]
    pub panel_indent_guide_active: Option<String>,
    #[serde(rename = "panel.indent_guide_hover")]
    pub panel_indent_guide_hover: Option<String>,

    // Pane colors
    #[serde(rename = "pane.focused_border")]
    pub pane_focused_border: Option<String>,
    #[serde(rename = "pane_group.border")]
    pub pane_group_border: Option<String>,

    // Scrollbar colors
    #[serde(rename = "scrollbar.thumb.background")]
    pub scrollbar_thumb_background: Option<String>,
    #[serde(rename = "scrollbar.thumb.border")]
    pub scrollbar_thumb_border: Option<String>,
    #[serde(rename = "scrollbar.thumb.hover_background")]
    pub scrollbar_thumb_hover_background: Option<String>,
    #[serde(rename = "scrollbar.track.background")]
    pub scrollbar_track_background: Option<String>,
    #[serde(rename = "scrollbar.track.border")]
    pub scrollbar_track_border: Option<String>,

    // Editor colors
    #[serde(rename = "editor.active_line.background")]
    pub editor_active_line_background: Option<String>,
    #[serde(rename = "editor.active_line_number")]
    pub editor_active_line_number: Option<String>,
    #[serde(rename = "editor.active_wrap_guide")]
    pub editor_active_wrap_guide: Option<String>,
    #[serde(rename = "editor.background")]
    pub editor_background: Option<String>,
    #[serde(rename = "editor.document_highlight.bracket_background")]
    pub editor_document_highlight_bracket_background: Option<String>,
    #[serde(rename = "editor.document_highlight.read_background")]
    pub editor_document_highlight_read_background: Option<String>,
    #[serde(rename = "editor.document_highlight.write_background")]
    pub editor_document_highlight_write_background: Option<String>,
    #[serde(rename = "editor.foreground")]
    pub editor_foreground: Option<String>,
    #[serde(rename = "editor.gutter.background")]
    pub editor_gutter_background: Option<String>,
    #[serde(rename = "editor.highlighted_line.background")]
    pub editor_highlighted_line_background: Option<String>,
    #[serde(rename = "editor.indent_guide")]
    pub editor_indent_guide: Option<String>,
    #[serde(rename = "editor.indent_guide_active")]
    pub editor_indent_guide_active: Option<String>,
    #[serde(rename = "editor.invisible")]
    pub editor_invisible: Option<String>,
    #[serde(rename = "editor.line_number")]
    pub editor_line_number: Option<String>,
    #[serde(rename = "editor.subheader.background")]
    pub editor_subheader_background: Option<String>,
    #[serde(rename = "editor.wrap_guide")]
    pub editor_wrap_guide: Option<String>,

    // Terminal colors
    #[serde(rename = "terminal.ansi.background")]
    pub terminal_ansi_background: Option<String>,
    #[serde(rename = "terminal.ansi.black")]
    pub terminal_ansi_black: Option<String>,
    #[serde(rename = "terminal.ansi.blue")]
    pub terminal_ansi_blue: Option<String>,
    #[serde(rename = "terminal.ansi.bright_black")]
    pub terminal_ansi_bright_black: Option<String>,
    #[serde(rename = "terminal.ansi.bright_blue")]
    pub terminal_ansi_bright_blue: Option<String>,
    #[serde(rename = "terminal.ansi.bright_cyan")]
    pub terminal_ansi_bright_cyan: Option<String>,
    #[serde(rename = "terminal.ansi.bright_green")]
    pub terminal_ansi_bright_green: Option<String>,
    #[serde(rename = "terminal.ansi.bright_magenta")]
    pub terminal_ansi_bright_magenta: Option<String>,
    #[serde(rename = "terminal.ansi.bright_red")]
    pub terminal_ansi_bright_red: Option<String>,
    #[serde(rename = "terminal.ansi.bright_white")]
    pub terminal_ansi_bright_white: Option<String>,
    #[serde(rename = "terminal.ansi.bright_yellow")]
    pub terminal_ansi_bright_yellow: Option<String>,
    #[serde(rename = "terminal.ansi.cyan")]
    pub terminal_ansi_cyan: Option<String>,
    #[serde(rename = "terminal.ansi.dim_black")]
    pub terminal_ansi_dim_black: Option<String>,
    #[serde(rename = "terminal.ansi.dim_blue")]
    pub terminal_ansi_dim_blue: Option<String>,
    #[serde(rename = "terminal.ansi.dim_cyan")]
    pub terminal_ansi_dim_cyan: Option<String>,
    #[serde(rename = "terminal.ansi.dim_green")]
    pub terminal_ansi_dim_green: Option<String>,
    #[serde(rename = "terminal.ansi.dim_magenta")]
    pub terminal_ansi_dim_magenta: Option<String>,
    #[serde(rename = "terminal.ansi.dim_red")]
    pub terminal_ansi_dim_red: Option<String>,
    #[serde(rename = "terminal.ansi.dim_white")]
    pub terminal_ansi_dim_white: Option<String>,
    #[serde(rename = "terminal.ansi.dim_yellow")]
    pub terminal_ansi_dim_yellow: Option<String>,
    #[serde(rename = "terminal.ansi.green")]
    pub terminal_ansi_green: Option<String>,
    #[serde(rename = "terminal.ansi.magenta")]
    pub terminal_ansi_magenta: Option<String>,
    #[serde(rename = "terminal.ansi.red")]
    pub terminal_ansi_red: Option<String>,
    #[serde(rename = "terminal.ansi.white")]
    pub terminal_ansi_white: Option<String>,
    #[serde(rename = "terminal.ansi.yellow")]
    pub terminal_ansi_yellow: Option<String>,
    #[serde(rename = "terminal.background")]
    pub terminal_background: Option<String>,
    #[serde(rename = "terminal.bright_foreground")]
    pub terminal_bright_foreground: Option<String>,
    #[serde(rename = "terminal.dim_foreground")]
    pub terminal_dim_foreground: Option<String>,
    #[serde(rename = "terminal.foreground")]
    pub terminal_foreground: Option<String>,

    // Link colors
    #[serde(rename = "link_text.hover")]
    pub link_text_hover: Option<String>,

    // Status colors
    pub conflict: Option<String>,
    #[serde(rename = "conflict.background")]
    pub conflict_background: Option<String>,
    #[serde(rename = "conflict.border")]
    pub conflict_border: Option<String>,
    pub created: Option<String>,
    #[serde(rename = "created.background")]
    pub created_background: Option<String>,
    #[serde(rename = "created.border")]
    pub created_border: Option<String>,
    pub deleted: Option<String>,
    #[serde(rename = "deleted.background")]
    pub deleted_background: Option<String>,
    #[serde(rename = "deleted.border")]
    pub deleted_border: Option<String>,
    pub error: Option<String>,
    #[serde(rename = "error.background")]
    pub error_background: Option<String>,
    #[serde(rename = "error.border")]
    pub error_border: Option<String>,
    pub hidden: Option<String>,
    #[serde(rename = "hidden.background")]
    pub hidden_background: Option<String>,
    #[serde(rename = "hidden.border")]
    pub hidden_border: Option<String>,
    pub hint: Option<String>,
    #[serde(rename = "hint.background")]
    pub hint_background: Option<String>,
    #[serde(rename = "hint.border")]
    pub hint_border: Option<String>,
    pub ignored: Option<String>,
    #[serde(rename = "ignored.background")]
    pub ignored_background: Option<String>,
    #[serde(rename = "ignored.border")]
    pub ignored_border: Option<String>,
    pub info: Option<String>,
    #[serde(rename = "info.background")]
    pub info_background: Option<String>,
    #[serde(rename = "info.border")]
    pub info_border: Option<String>,
    pub modified: Option<String>,
    #[serde(rename = "modified.background")]
    pub modified_background: Option<String>,
    #[serde(rename = "modified.border")]
    pub modified_border: Option<String>,
    pub predictive: Option<String>,
    #[serde(rename = "predictive.background")]
    pub predictive_background: Option<String>,
    #[serde(rename = "predictive.border")]
    pub predictive_border: Option<String>,
    pub renamed: Option<String>,
    #[serde(rename = "renamed.background")]
    pub renamed_background: Option<String>,
    #[serde(rename = "renamed.border")]
    pub renamed_border: Option<String>,
    pub success: Option<String>,
    #[serde(rename = "success.background")]
    pub success_background: Option<String>,
    #[serde(rename = "success.border")]
    pub success_border: Option<String>,
    pub unreachable: Option<String>,
    #[serde(rename = "unreachable.background")]
    pub unreachable_background: Option<String>,
    #[serde(rename = "unreachable.border")]
    pub unreachable_border: Option<String>,
    pub warning: Option<String>,
    #[serde(rename = "warning.background")]
    pub warning_background: Option<String>,
    #[serde(rename = "warning.border")]
    pub warning_border: Option<String>,

    // Player colors for collaboration
    pub players: Option<Vec<PlayerColor>>,

    // Syntax highlighting
    pub syntax: Option<HashMap<String, HighlightStyle>>,
}

/// Background appearance mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundAppearance {
    Opaque,
    Transparent,
    Blurred,
}

/// Player color for collaboration features
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerColor {
    pub cursor: Option<String>,
    pub selection: Option<String>,
    pub background: Option<String>,
}

/// Syntax highlighting style
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightStyle {
    pub color: Option<String>,
    pub font_style: Option<FontStyle>,
    pub font_weight: Option<FontWeight>,
    pub background_color: Option<String>,
}

/// Font style options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Font weight - can be a number or named weight
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FontWeight {
    Number(u16),
    Named(NamedFontWeight),
}

/// Named font weights
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NamedFontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_zed_fixture() {
        let result = ZedThemeFamily::read("fixtures/zed/catppuccin/themes/catppuccin-mauve.json");
        assert!(result.is_ok());
        let theme_family = result.unwrap();
        assert_eq!(theme_family.name, "Catppuccin");
        assert_eq!(theme_family.author, "Catppuccin <releases@catppuccin.com>");
        assert!(!theme_family.themes.is_empty());

        let first_theme = &theme_family.themes[0];
        assert_eq!(first_theme.name, "Catppuccin Latte");
        assert!(matches!(first_theme.appearance, Appearance::Light));
    }

    #[test]
    fn test_zed_theme_round_trip() {
        // Load a Zed theme
        let original_theme = ZedThemeFamily::read("fixtures/zed/catppuccin/themes/catppuccin-mauve.json")
            .expect("Failed to load Zed theme");

        // Serialize it back to JSON
        let serialized = serde_json::to_string_pretty(&original_theme).expect("Failed to serialize Zed theme");

        // Deserialize it again
        let round_trip_theme: ZedThemeFamily =
            serde_json::from_str(&serialized).expect("Failed to deserialize Zed theme");

        // Check that key properties are preserved
        assert_eq!(original_theme.name, round_trip_theme.name);
        assert_eq!(original_theme.author, round_trip_theme.author);
        assert_eq!(original_theme.themes.len(), round_trip_theme.themes.len());

        for (orig, rt) in original_theme.themes.iter().zip(round_trip_theme.themes.iter()) {
            assert_eq!(orig.name, rt.name);
            assert_eq!(
                std::mem::discriminant(&orig.appearance),
                std::mem::discriminant(&rt.appearance)
            );
        }
    }

    #[test]
    fn test_zed_theme_creation() {
        let theme = ZedTheme {
            name: "Test Theme".to_string(),
            appearance: Appearance::Dark,
            style: ZedThemeStyle {
                background: Some("#000000".to_string()),
                text: Some("#ffffff".to_string()),
                ..Default::default()
            },
        };

        assert_eq!(theme.name, "Test Theme");
        assert!(matches!(theme.appearance, Appearance::Dark));
    }

    #[test]
    fn test_clean_serialization() {
        // Create a minimal theme to test that None fields are not serialized
        let theme = ZedTheme {
            name: "Minimal Theme".to_string(),
            appearance: Appearance::Dark,
            style: ZedThemeStyle {
                background: Some("#1e1e1e".to_string()),
                text: Some("#ffffff".to_string()),
                // All other fields are None and should not appear in JSON
                ..Default::default()
            },
        };

        let theme_family = ZedThemeFamily {
            schema: None, // This should not appear in JSON
            name: "Test Family".to_string(),
            author: "Test Author".to_string(),
            themes: vec![theme],
        };

        let json = serde_json::to_string_pretty(&theme_family).expect("Serialization failed");

        // Verify that None fields are not present in the output
        assert!(!json.contains("\"schema\""));
        assert!(!json.contains("\"accents\""));
        assert!(!json.contains("\"border\""));
        assert!(!json.contains("\"editor_foreground\""));

        // Verify that Some fields are present
        assert!(json.contains("\"background\": \"#1e1e1e\""));
        assert!(json.contains("\"text\": \"#ffffff\""));
        assert!(json.contains("\"name\": \"Test Family\""));
        assert!(json.contains("\"author\": \"Test Author\""));
    }
}
