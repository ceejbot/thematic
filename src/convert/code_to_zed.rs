//! Trait implementation to convert VS Code themes (secretly TextMate themes)
//! into Zed themes.

use std::collections::HashMap;

use crate::editors::vscode::{TokenColorRule, TokenColors, TokenScope};
use crate::editors::zed::{Appearance, FontStyle, HighlightStyle, PlayerColor, ZedThemeStyle};
use crate::{VsCodeTheme, ZedTheme};

impl From<&VsCodeTheme> for ZedTheme {
    fn from(value: &VsCodeTheme) -> Self {
        let appearance = if value.is_dark_theme() {
            Appearance::Dark
        } else {
            Appearance::Light
        };

        let mut style = ZedThemeStyle::default();

        // Map UI colors from VSCode to Zed
        if let Some(colors) = &value.colors {
            map_ui_colors(colors, &mut style);
        }

        // Map syntax highlighting
        let syntax = map_syntax_highlighting(value);

        // Set up player colors with some defaults based on theme colors
        let players = create_default_players(&style);

        style.syntax = Some(syntax);
        style.players = Some(players);

        ZedTheme {
            name: value.name.clone(),
            appearance,
            style,
        }
    }
}

/// Maps VSCode UI colors to Zed theme style
fn map_ui_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
    // Background and basic colors
    if let Some(color) = vscode_colors.get("editor.background") {
        zed_style.editor_background = Some(color.clone());
        zed_style.background = Some(color.clone());
        zed_style.panel_background = Some(color.clone());
    }

    if let Some(color) = vscode_colors.get("editor.foreground") {
        zed_style.editor_foreground = Some(color.clone());
        zed_style.text = Some(color.clone());
    }

    // Status bar
    if let Some(color) = vscode_colors.get("statusBar.background") {
        zed_style.status_bar_background = Some(color.clone());
    }

    // Title bar
    if let Some(color) = vscode_colors.get("titleBar.activeBackground") {
        zed_style.title_bar_background = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("titleBar.inactiveBackground") {
        zed_style.title_bar_inactive_background = Some(color.clone());
    }

    // Tabs
    if let Some(color) = vscode_colors.get("tab.activeBackground") {
        zed_style.tab_active_background = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("tab.inactiveBackground") {
        zed_style.tab_inactive_background = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("editorGroupHeader.tabsBackground") {
        zed_style.tab_bar_background = Some(color.clone());
    }

    // Terminal colors
    if let Some(color) = vscode_colors.get("terminal.foreground") {
        zed_style.terminal_foreground = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("editor.background") {
        zed_style.terminal_background = Some(color.clone());
    }

    // Terminal ANSI colors
    map_terminal_colors(vscode_colors, zed_style);

    // Editor line highlighting
    if let Some(color) = vscode_colors.get("editor.lineHighlightBackground") {
        zed_style.editor_active_line_background = Some(color.clone());
    }

    // Line numbers
    if let Some(color) = vscode_colors.get("editorLineNumber.foreground") {
        zed_style.editor_line_number = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("editorLineNumber.activeForeground") {
        zed_style.editor_active_line_number = Some(color.clone());
    }

    // Gutter
    if let Some(color) = vscode_colors.get("editorGutter.background") {
        zed_style.editor_gutter_background = Some(color.clone());
    }

    // Selection
    if let Some(color) = vscode_colors.get("editor.selectionBackground") {
        // Zed doesn't have a direct equivalent, but we can use it for other selection-like colors
        zed_style.element_selected = Some(color.clone());
    }

    // Sidebar
    if let Some(color) = vscode_colors.get("sideBar.background") {
        zed_style.surface_background = Some(color.clone());
        zed_style.elevated_surface_background = Some(color.clone());
    }

    // Focus and borders
    if let Some(color) = vscode_colors.get("focusBorder") {
        zed_style.border_focused = Some(color.clone());
    }

    // Scrollbar
    if let Some(color) = vscode_colors.get("scrollbarSlider.background") {
        zed_style.scrollbar_thumb_background = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("scrollbarSlider.hoverBackground") {
        zed_style.scrollbar_thumb_hover_background = Some(color.clone());
    }

    // Set some reasonable defaults for Zed-specific colors
    set_zed_defaults(zed_style);
}

/// Maps terminal ANSI colors from VSCode to Zed
fn map_terminal_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
    let terminal_color_map = [
        ("terminal.ansiBlack", &mut zed_style.terminal_ansi_black),
        ("terminal.ansiRed", &mut zed_style.terminal_ansi_red),
        ("terminal.ansiGreen", &mut zed_style.terminal_ansi_green),
        ("terminal.ansiYellow", &mut zed_style.terminal_ansi_yellow),
        ("terminal.ansiBlue", &mut zed_style.terminal_ansi_blue),
        ("terminal.ansiMagenta", &mut zed_style.terminal_ansi_magenta),
        ("terminal.ansiCyan", &mut zed_style.terminal_ansi_cyan),
        ("terminal.ansiWhite", &mut zed_style.terminal_ansi_white),
        ("terminal.ansiBrightBlack", &mut zed_style.terminal_ansi_bright_black),
        ("terminal.ansiBrightRed", &mut zed_style.terminal_ansi_bright_red),
        ("terminal.ansiBrightGreen", &mut zed_style.terminal_ansi_bright_green),
        ("terminal.ansiBrightYellow", &mut zed_style.terminal_ansi_bright_yellow),
        ("terminal.ansiBrightBlue", &mut zed_style.terminal_ansi_bright_blue),
        (
            "terminal.ansiBrightMagenta",
            &mut zed_style.terminal_ansi_bright_magenta,
        ),
        ("terminal.ansiBrightCyan", &mut zed_style.terminal_ansi_bright_cyan),
        ("terminal.ansiBrightWhite", &mut zed_style.terminal_ansi_bright_white),
    ];

    for (vscode_key, zed_field) in terminal_color_map {
        if let Some(color) = vscode_colors.get(vscode_key) {
            *zed_field = Some(color.clone());
        }
    }
}

/// Sets reasonable defaults for Zed-specific properties
fn set_zed_defaults(zed_style: &mut ZedThemeStyle) {
    // Set default borders if not already set
    if zed_style.border.is_none() {
        if let Some(bg) = &zed_style.background {
            // Create a slightly lighter/darker border color
            zed_style.border = Some(bg.clone());
        }
    }

    // Set element states based on background
    if let Some(bg) = &zed_style.surface_background {
        if zed_style.element_background.is_none() {
            zed_style.element_background = Some(bg.clone());
        }
        if zed_style.element_hover.is_none() {
            zed_style.element_hover = Some(bg.clone());
        }
    }

    // Set text muted if not set
    if zed_style.text_muted.is_none() && zed_style.text.is_some() {
        zed_style.text_muted = zed_style.text.clone();
    }
}

/// Maps VSCode token colors to Zed syntax highlighting
fn map_syntax_highlighting(vscode_theme: &VsCodeTheme) -> HashMap<String, HighlightStyle> {
    let mut syntax_map = HashMap::new();

    if let Some(TokenColors::Rules(rules)) = &vscode_theme.token_colors {
        for rule in rules {
            map_token_rule_to_zed(rule, &mut syntax_map);
        }
    }

    // Ensure we have some basic syntax highlighting
    add_default_syntax_colors(&mut syntax_map);

    syntax_map
}

/// Maps a single VSCode token color rule to Zed syntax entries
fn map_token_rule_to_zed(rule: &TokenColorRule, syntax_map: &mut HashMap<String, HighlightStyle>) {
    let highlight_style = HighlightStyle {
        color: rule.settings.foreground.clone(),
        background_color: rule.settings.background.clone(),
        font_style: parse_font_style(&rule.settings.font_style),
        font_weight: None, // VSCode doesn't have direct font weight in token colors
    };

    if let Some(scope) = &rule.scope {
        let scopes = match scope {
            TokenScope::Single(s) => vec![s.as_str()],
            TokenScope::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
        };

        for scope_str in scopes {
            // Map VSCode TextMate scopes to Zed syntax keys
            if let Some(zed_key) = map_textmate_scope_to_zed(scope_str) {
                syntax_map.insert(zed_key, highlight_style.clone());
            }
        }
    }
}

/// Maps TextMate scopes to Zed syntax keys
fn map_textmate_scope_to_zed(scope: &str) -> Option<String> {
    match scope {
        "comment" | "comment.line" | "comment.block" => Some("comment".to_string()),
        "comment.block.documentation" => Some("comment.doc".to_string()),
        "keyword" | "keyword.control" => Some("keyword".to_string()),
        "string" | "string.quoted" => Some("string".to_string()),
        "string.regexp" => Some("string.regex".to_string()),
        "constant.numeric" => Some("number".to_string()),
        "constant" => Some("constant".to_string()),
        "entity.name.function" | "support.function" => Some("function".to_string()),
        "entity.name.type" | "support.type" | "storage.type" => Some("type".to_string()),
        "variable" => Some("variable".to_string()),
        "variable.language" | "variable.other.special" => Some("variable.special".to_string()),
        "punctuation" => Some("punctuation".to_string()),
        "punctuation.definition" => Some("punctuation.delimiter".to_string()),
        "keyword.operator" => Some("operator".to_string()),
        "entity.name.class" => Some("type".to_string()),
        "entity.name.tag" => Some("tag".to_string()),
        "markup.heading" => Some("title".to_string()),
        "markup.bold" => Some("emphasis.strong".to_string()),
        "markup.italic" => Some("emphasis".to_string()),
        "invalid" | "invalid.illegal" => Some("error".to_string()),
        _ => {
            // For unmapped scopes, try to extract a reasonable key
            if scope.contains("comment") {
                Some("comment".to_string())
            } else if scope.contains("string") {
                Some("string".to_string())
            } else if scope.contains("keyword") {
                Some("keyword".to_string())
            } else if scope.contains("function") {
                Some("function".to_string())
            } else if scope.contains("type") {
                Some("type".to_string())
            } else if scope.contains("variable") {
                Some("variable".to_string())
            } else {
                None
            }
        }
    }
}

/// Parses VSCode font style string to Zed FontStyle
fn parse_font_style(font_style: &Option<String>) -> Option<FontStyle> {
    match font_style.as_deref() {
        Some("italic") => Some(FontStyle::Italic),
        Some("oblique") => Some(FontStyle::Oblique),
        _ => None,
    }
}

/// Adds default syntax colors if they're missing
fn add_default_syntax_colors(syntax_map: &mut HashMap<String, HighlightStyle>) {
    let defaults = [
        ("comment", "#6A9955"),
        ("keyword", "#569CD6"),
        ("string", "#CE9178"),
        ("number", "#B5CEA8"),
        ("function", "#DCDCAA"),
        ("type", "#4EC9B0"),
        ("variable", "#9CDCFE"),
        ("operator", "#D4D4D4"),
        ("punctuation", "#D4D4D4"),
    ];

    for (key, color) in defaults {
        if !syntax_map.contains_key(key) {
            syntax_map.insert(
                key.to_string(),
                HighlightStyle {
                    color: Some(color.to_string()),
                    font_style: None,
                    font_weight: None,
                    background_color: None,
                },
            );
        }
    }
}

/// Creates default player colors for collaboration
fn create_default_players(style: &ZedThemeStyle) -> Vec<PlayerColor> {
    let base_color = style.text.as_deref().unwrap_or("#FFFFFF");

    // Create 5 default player colors with some variety
    vec![
        PlayerColor {
            cursor: Some(base_color.to_string()),
            background: Some(base_color.to_string()),
            selection: Some(format!("{base_color}22")), // Add alpha
        },
        PlayerColor {
            cursor: Some("#FF6B6B".to_string()),
            background: Some("#FF6B6B".to_string()),
            selection: Some("#FF6B6B44".to_string()),
        },
        PlayerColor {
            cursor: Some("#4ECDC4".to_string()),
            background: Some("#4ECDC4".to_string()),
            selection: Some("#4ECDC444".to_string()),
        },
        PlayerColor {
            cursor: Some("#45B7D1".to_string()),
            background: Some("#45B7D1".to_string()),
            selection: Some("#45B7D144".to_string()),
        },
        PlayerColor {
            cursor: Some("#FFA07A".to_string()),
            background: Some("#FFA07A".to_string()),
            selection: Some("#FFA07A44".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editors::ThemeFile;

    #[test]
    fn vscode_to_zed_conversion() {
        let vscode_theme =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode theme");

        let thematic: ZedTheme = (&vscode_theme).into();

        assert_eq!(thematic.name, "Rosé Pine Moon");
        assert!(matches!(thematic.appearance, Appearance::Dark));
        assert!(thematic.style.editor_background.is_some());
        assert!(thematic.style.syntax.is_some());
    }

    #[test]
    fn textmate_scope_mapping() {
        assert_eq!(map_textmate_scope_to_zed("comment"), Some("comment".to_string()));
        assert_eq!(
            map_textmate_scope_to_zed("keyword.control"),
            Some("keyword".to_string())
        );
        assert_eq!(map_textmate_scope_to_zed("string.quoted"), Some("string".to_string()));
        assert_eq!(
            map_textmate_scope_to_zed("constant.numeric"),
            Some("number".to_string())
        );
    }
}
