//! VSCode theme data structures
//!
//! This module contains the Rust structures that correspond to VSCode color
//! theme files. Based on the VSCode color theme schema and example files.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::skip_serializing_none;

use crate::ThemeError;
use crate::editors::ThemeFile;

/// A complete VSCode color theme
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsCodeTheme {
    pub name: String,
    #[serde(rename = "type")]
    pub theme_type: Option<String>,
    pub colors: Option<HashMap<String, String>>,
    #[serde(rename = "tokenColors")]
    pub token_colors: Option<TokenColors>,
    #[serde(rename = "semanticHighlighting")]
    pub semantic_highlighting: Option<bool>,
    #[serde(
        rename = "semanticTokenColors",
        deserialize_with = "deserialize_semantic_token_colors",
        default
    )]
    pub semantic_token_colors: Option<HashMap<String, TokenColorSettings>>,
    #[serde(skip)]
    pub filename: String,
}

/// Custom deserializer for semanticTokenColors that handles both old and new
/// formats
fn deserialize_semantic_token_colors<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, TokenColorSettings>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;

    let value: Option<Value> = Option::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    // Try to deserialize as new format first (HashMap<String, TokenColorSettings>)
    if let Ok(new_format) = HashMap::<String, TokenColorSettings>::deserialize(value.clone()) {
        return Ok(Some(new_format));
    }

    // Try to deserialize as old format (HashMap<String, String>)
    if let Ok(old_format) = HashMap::<String, String>::deserialize(value) {
        let converted = old_format
            .into_iter()
            .map(|(key, color)| {
                (
                    key,
                    TokenColorSettings {
                        foreground: Some(color),
                        background: None,
                        font_style: None,
                    },
                )
            })
            .collect();
        return Ok(Some(converted));
    }

    // If both fail, return None (ignore the field)
    Ok(None)
}

impl ThemeFile for VsCodeTheme {
    // cheat! cheat!
    type T = VsCodeTheme;

    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref)?;
        let mut theme: VsCodeTheme = serde_json::from_str(&content)?;
        theme.filename = path_ref.display().to_string();
        Ok(theme)
    }

    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError> {
        Ok(serde_json::from_slice::<VsCodeTheme>(bytes)?)
    }
}

/// Token colors can be either a path to a tmTheme file or an array of token
/// color rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenColors {
    /// Path to a tmTheme file (relative to the current file)
    Path(String),
    /// Array of token color rules
    Rules(Vec<TokenColorRule>),
}

/// A single token color rule
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenColorRule {
    pub name: Option<String>,
    pub scope: Option<TokenScope>,
    pub settings: TokenColorSettings,
}

/// Token scope can be a single string or an array of strings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenScope {
    Single(String),
    Multiple(Vec<String>),
}

/// Settings for token colors
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenColorSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    #[serde(rename = "fontStyle")]
    pub font_style: Option<String>,
}

impl VsCodeTheme {
    /// Create a new VSCode theme with the given name
    pub fn new(name: String) -> Self {
        let filename = format!("{}.json", slug::slugify(&name));
        Self {
            name,
            theme_type: None,
            colors: None,
            token_colors: None,
            semantic_highlighting: None,
            semantic_token_colors: None,
            filename,
        }
    }

    /// Get the theme type (light or dark)
    pub fn get_theme_type(&self) -> Option<&str> {
        self.theme_type.as_deref()
    }

    /// Check if this is a dark theme
    pub fn is_dark_theme(&self) -> bool {
        self.theme_type.as_ref().map(|t| t == "dark").unwrap_or(false)
    }

    /// Check if this is a light theme
    pub fn is_light_theme(&self) -> bool {
        self.theme_type.as_ref().map(|t| t == "light").unwrap_or(false)
    }

    /// Get a color by key from the colors map
    pub fn get_color(&self, key: &str) -> Option<&str> {
        self.colors
            .as_ref()
            .and_then(|colors| colors.get(key))
            .map(|s| s.as_str())
    }

    /// Set a color in the colors map
    pub fn set_color(&mut self, key: String, value: String) {
        if self.colors.is_none() {
            self.colors = Some(HashMap::new());
        }
        if let Some(colors) = &mut self.colors {
            colors.insert(key, value);
        }
    }

    /// Get all token color rules (if using rules format)
    pub fn get_token_rules(&self) -> Option<&Vec<TokenColorRule>> {
        match &self.token_colors {
            Some(TokenColors::Rules(rules)) => Some(rules),
            _ => None,
        }
    }

    /// Set token color rules
    pub fn set_token_rules(&mut self, rules: Vec<TokenColorRule>) {
        self.token_colors = Some(TokenColors::Rules(rules));
    }

    /// Add a token color rule
    pub fn add_token_rule(&mut self, rule: TokenColorRule) {
        match &mut self.token_colors {
            Some(TokenColors::Rules(rules)) => {
                rules.push(rule);
            }
            _ => {
                self.token_colors = Some(TokenColors::Rules(vec![rule]));
            }
        }
    }
}

impl TokenColorRule {
    /// Create a new token color rule
    pub fn new(scope: TokenScope, settings: TokenColorSettings) -> Self {
        Self {
            name: None,
            scope: Some(scope),
            settings,
        }
    }

    /// Create a new token color rule with a name
    pub fn with_name(name: String, scope: TokenScope, settings: TokenColorSettings) -> Self {
        Self {
            name: Some(name),
            scope: Some(scope),
            settings,
        }
    }

    /// Create a rule for a single scope
    pub fn single_scope(scope: String, settings: TokenColorSettings) -> Self {
        Self::new(TokenScope::Single(scope), settings)
    }

    /// Create a rule for multiple scopes
    pub fn multiple_scopes(scopes: Vec<String>, settings: TokenColorSettings) -> Self {
        Self::new(TokenScope::Multiple(scopes), settings)
    }
}

impl TokenColorSettings {
    /// Create new token color settings with just a foreground color
    pub fn foreground(color: String) -> Self {
        Self {
            foreground: Some(color),
            background: None,
            font_style: None,
        }
    }

    /// Create new token color settings with foreground and font style
    pub fn foreground_with_style(color: String, font_style: String) -> Self {
        Self {
            foreground: Some(color),
            background: None,
            font_style: Some(font_style),
        }
    }

    /// Create new token color settings with all properties
    pub fn new(foreground: Option<String>, background: Option<String>, font_style: Option<String>) -> Self {
        Self {
            foreground,
            background,
            font_style,
        }
    }
}

/// Common VSCode workbench color keys
pub mod workbench_colors {
    // Activity Bar
    pub const ACTIVITY_BAR_BACKGROUND: &str = "activityBar.background";
    pub const ACTIVITY_BAR_FOREGROUND: &str = "activityBar.foreground";
    pub const ACTIVITY_BAR_INACTIVE_FOREGROUND: &str = "activityBar.inactiveForeground";
    pub const ACTIVITY_BAR_BORDER: &str = "activityBar.border";
    pub const ACTIVITY_BAR_ACTIVE_BORDER: &str = "activityBar.activeBorder";

    // Status Bar
    pub const STATUS_BAR_BACKGROUND: &str = "statusBar.background";
    pub const STATUS_BAR_FOREGROUND: &str = "statusBar.foreground";
    pub const STATUS_BAR_BORDER: &str = "statusBar.border";
    pub const STATUS_BAR_DEBUGGING_BACKGROUND: &str = "statusBar.debuggingBackground";
    pub const STATUS_BAR_DEBUGGING_FOREGROUND: &str = "statusBar.debuggingForeground";
    pub const STATUS_BAR_NO_FOLDER_BACKGROUND: &str = "statusBar.noFolderBackground";
    pub const STATUS_BAR_NO_FOLDER_FOREGROUND: &str = "statusBar.noFolderForeground";

    // Title Bar
    pub const TITLE_BAR_ACTIVE_BACKGROUND: &str = "titleBar.activeBackground";
    pub const TITLE_BAR_ACTIVE_FOREGROUND: &str = "titleBar.activeForeground";
    pub const TITLE_BAR_INACTIVE_BACKGROUND: &str = "titleBar.inactiveBackground";
    pub const TITLE_BAR_INACTIVE_FOREGROUND: &str = "titleBar.inactiveForeground";
    pub const TITLE_BAR_BORDER: &str = "titleBar.border";

    // Editor
    pub const EDITOR_BACKGROUND: &str = "editor.background";
    pub const EDITOR_FOREGROUND: &str = "editor.foreground";
    pub const EDITOR_LINE_HIGHLIGHT_BACKGROUND: &str = "editor.lineHighlightBackground";
    pub const EDITOR_LINE_HIGHLIGHT_BORDER: &str = "editor.lineHighlightBorder";
    pub const EDITOR_SELECTION_BACKGROUND: &str = "editor.selectionBackground";
    pub const EDITOR_SELECTION_FOREGROUND: &str = "editor.selectionForeground";
    pub const EDITOR_CURSOR_FOREGROUND: &str = "editorCursor.foreground";
    pub const EDITOR_CURSOR_BACKGROUND: &str = "editorCursor.background";

    // Editor Line Numbers
    pub const EDITOR_LINE_NUMBER_FOREGROUND: &str = "editorLineNumber.foreground";
    pub const EDITOR_LINE_NUMBER_ACTIVE_FOREGROUND: &str = "editorLineNumber.activeForeground";

    // Editor Gutter
    pub const EDITOR_GUTTER_BACKGROUND: &str = "editorGutter.background";
    pub const EDITOR_GUTTER_ADDED_BACKGROUND: &str = "editorGutter.addedBackground";
    pub const EDITOR_GUTTER_DELETED_BACKGROUND: &str = "editorGutter.deletedBackground";
    pub const EDITOR_GUTTER_MODIFIED_BACKGROUND: &str = "editorGutter.modifiedBackground";

    // Sidebar
    pub const SIDEBAR_BACKGROUND: &str = "sideBar.background";
    pub const SIDEBAR_FOREGROUND: &str = "sideBar.foreground";
    pub const SIDEBAR_BORDER: &str = "sideBar.border";

    // Panel
    pub const PANEL_BACKGROUND: &str = "panel.background";
    pub const PANEL_BORDER: &str = "panel.border";

    // Terminal
    pub const TERMINAL_FOREGROUND: &str = "terminal.foreground";
    pub const TERMINAL_BACKGROUND: &str = "terminal.background";
    pub const TERMINAL_CURSOR_FOREGROUND: &str = "terminalCursor.foreground";
    pub const TERMINAL_CURSOR_BACKGROUND: &str = "terminalCursor.background";
    pub const TERMINAL_SELECTION_BACKGROUND: &str = "terminal.selectionBackground";

    // Terminal ANSI Colors
    pub const TERMINAL_ANSI_BLACK: &str = "terminal.ansiBlack";
    pub const TERMINAL_ANSI_RED: &str = "terminal.ansiRed";
    pub const TERMINAL_ANSI_GREEN: &str = "terminal.ansiGreen";
    pub const TERMINAL_ANSI_YELLOW: &str = "terminal.ansiYellow";
    pub const TERMINAL_ANSI_BLUE: &str = "terminal.ansiBlue";
    pub const TERMINAL_ANSI_MAGENTA: &str = "terminal.ansiMagenta";
    pub const TERMINAL_ANSI_CYAN: &str = "terminal.ansiCyan";
    pub const TERMINAL_ANSI_WHITE: &str = "terminal.ansiWhite";
    pub const TERMINAL_ANSI_BRIGHT_BLACK: &str = "terminal.ansiBrightBlack";
    pub const TERMINAL_ANSI_BRIGHT_RED: &str = "terminal.ansiBrightRed";
    pub const TERMINAL_ANSI_BRIGHT_GREEN: &str = "terminal.ansiBrightGreen";
    pub const TERMINAL_ANSI_BRIGHT_YELLOW: &str = "terminal.ansiBrightYellow";
    pub const TERMINAL_ANSI_BRIGHT_BLUE: &str = "terminal.ansiBrightBlue";
    pub const TERMINAL_ANSI_BRIGHT_MAGENTA: &str = "terminal.ansiBrightMagenta";
    pub const TERMINAL_ANSI_BRIGHT_CYAN: &str = "terminal.ansiBrightCyan";
    pub const TERMINAL_ANSI_BRIGHT_WHITE: &str = "terminal.ansiBrightWhite";

    // Tabs
    pub const TAB_ACTIVE_BACKGROUND: &str = "tab.activeBackground";
    pub const TAB_ACTIVE_FOREGROUND: &str = "tab.activeForeground";
    pub const TAB_INACTIVE_BACKGROUND: &str = "tab.inactiveBackground";
    pub const TAB_INACTIVE_FOREGROUND: &str = "tab.inactiveForeground";
    pub const TAB_BORDER: &str = "tab.border";

    // General
    pub const FOREGROUND: &str = "foreground";
    pub const FOCUS_BORDER: &str = "focusBorder";
    pub const SELECTION_BACKGROUND: &str = "selection.background";
    pub const ERROR_FOREGROUND: &str = "errorForeground";
    pub const DESCRIPTION_FOREGROUND: &str = "descriptionForeground";
    pub const ICON_FOREGROUND: &str = "icon.foreground";
}

/// Common TextMate scope names for token colors
pub mod token_scopes {
    // Comments
    pub const COMMENT: &str = "comment";
    pub const COMMENT_LINE: &str = "comment.line";
    pub const COMMENT_BLOCK: &str = "comment.block";
    pub const COMMENT_DOCUMENTATION: &str = "comment.block.documentation";

    // Keywords
    pub const KEYWORD: &str = "keyword";
    pub const KEYWORD_CONTROL: &str = "keyword.control";
    pub const KEYWORD_OPERATOR: &str = "keyword.operator";
    pub const KEYWORD_OTHER: &str = "keyword.other";

    // Strings
    pub const STRING: &str = "string";
    pub const STRING_QUOTED: &str = "string.quoted";
    pub const STRING_QUOTED_SINGLE: &str = "string.quoted.single";
    pub const STRING_QUOTED_DOUBLE: &str = "string.quoted.double";
    pub const STRING_TEMPLATE: &str = "string.template";
    pub const STRING_REGEXP: &str = "string.regexp";

    // Numbers
    pub const CONSTANT_NUMERIC: &str = "constant.numeric";
    pub const CONSTANT_NUMERIC_INTEGER: &str = "constant.numeric.integer";
    pub const CONSTANT_NUMERIC_FLOAT: &str = "constant.numeric.float";

    // Functions
    pub const ENTITY_NAME_FUNCTION: &str = "entity.name.function";
    pub const SUPPORT_FUNCTION: &str = "support.function";
    pub const META_FUNCTION_CALL: &str = "meta.function-call";

    // Variables
    pub const VARIABLE: &str = "variable";
    pub const VARIABLE_LANGUAGE: &str = "variable.language";
    pub const VARIABLE_PARAMETER: &str = "variable.parameter";
    pub const VARIABLE_OTHER: &str = "variable.other";

    // Types
    pub const ENTITY_NAME_TYPE: &str = "entity.name.type";
    pub const SUPPORT_TYPE: &str = "support.type";
    pub const STORAGE_TYPE: &str = "storage.type";

    // Classes
    pub const ENTITY_NAME_CLASS: &str = "entity.name.class";
    pub const SUPPORT_CLASS: &str = "support.class";

    // Punctuation
    pub const PUNCTUATION: &str = "punctuation";
    pub const PUNCTUATION_DEFINITION: &str = "punctuation.definition";
    pub const PUNCTUATION_SEPARATOR: &str = "punctuation.separator";
    pub const PUNCTUATION_TERMINATOR: &str = "punctuation.terminator";

    // Markup (for Markdown, etc.)
    pub const MARKUP_HEADING: &str = "markup.heading";
    pub const MARKUP_BOLD: &str = "markup.bold";
    pub const MARKUP_ITALIC: &str = "markup.italic";
    pub const MARKUP_UNDERLINE: &str = "markup.underline";
    pub const MARKUP_QUOTE: &str = "markup.quote";
    pub const MARKUP_RAW: &str = "markup.raw";
    pub const MARKUP_LIST: &str = "markup.list";

    // Invalid
    pub const INVALID: &str = "invalid";
    pub const INVALID_ILLEGAL: &str = "invalid.illegal";
    pub const INVALID_DEPRECATED: &str = "invalid.deprecated";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_theme_format() {
        let result =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json");
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.name, "Rosé Pine Moon");
        assert_eq!(theme.get_theme_type(), Some("dark"));
        assert!(theme.is_dark_theme());
        assert!(!theme.is_light_theme());

        // Test that we can access some colors
        assert!(theme.get_color("editor.background").is_some());
        assert!(theme.get_color("editor.foreground").is_some());
    }

    #[test]
    fn older_theme_format() {
        let result =
            VsCodeTheme::read("fixtures/vscode/uloco.theme-bluloco-light-3.7.5/themes/bluloco-light-color-theme.json");
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.name, "Bluloco Light");
        assert_eq!(theme.get_theme_type(), Some("light"));
        assert!(!theme.is_dark_theme());
        assert!(theme.is_light_theme());

        // Test that we can access some colors
        assert!(theme.get_color("editor.background").is_some());
        assert!(theme.get_color("editor.foreground").is_some());
    }

    #[test]
    fn read_all_fixtures() {
        let themelist = vec![
            "fixtures/vscode/catppuccin.catppuccin-vsc-3.17.0/themes/frappe.json",
            "fixtures/vscode/catppuccin.catppuccin-vsc-3.17.0/themes/latte.json",
            "fixtures/vscode/catppuccin.catppuccin-vsc-3.17.0/themes/macchiato.json",
            "fixtures/vscode/catppuccin.catppuccin-vsc-3.17.0/themes/mocha.json",
            "fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-color-theme.json",
            "fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-dawn-color-theme.json",
            "fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json",
            "fixtures/vscode/uloco.theme-bluloco-light-3.7.5/themes/bluloco-light-color-theme.json",
            "fixtures/vscode/uloco.theme-bluloco-light-3.7.5/themes/bluloco-light-italic-color-theme.json",
        ];

        for tpath in themelist {
            eprintln!("reading {tpath}");
            let theme = VsCodeTheme::read(tpath).expect("expected to read a fixture successfully");
            assert!(!theme.name.is_empty());
        }
    }

    #[test]
    fn vscode_theme_round_trip() {
        // Load a VSCode theme
        let original_theme =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode theme");

        // Serialize it back to JSON
        let serialized = serde_json::to_string_pretty(&original_theme).expect("Failed to serialize VSCode theme");

        // Deserialize it again
        let round_trip_theme: VsCodeTheme =
            serde_json::from_str(&serialized).expect("Failed to deserialize VSCode theme");

        // Check that key properties are preserved
        assert_eq!(original_theme.name, round_trip_theme.name);
        assert_eq!(original_theme.theme_type, round_trip_theme.theme_type);

        // Check that some colors are preserved
        if let (Some(orig_colors), Some(rt_colors)) = (&original_theme.colors, &round_trip_theme.colors) {
            assert_eq!(orig_colors.len(), rt_colors.len());
            for (key, value) in orig_colors {
                assert_eq!(rt_colors.get(key), Some(value));
            }
        }
    }
}
