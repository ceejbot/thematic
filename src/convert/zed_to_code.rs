use std::collections::HashMap;

use crate::editors::vscode::{TokenColorRule, TokenColorSettings, TokenColors, TokenScope};
use crate::editors::zed::{Appearance, FontStyle, ZedThemeStyle};
use crate::{VsCodeTheme, ZedTheme, ZedThemeFamily};

impl From<&ZedThemeFamily> for Vec<VsCodeTheme> {
    fn from(value: &ZedThemeFamily) -> Self {
        value.themes.iter().map(|xs| xs.into()).collect()
    }
}

impl From<&ZedTheme> for VsCodeTheme {
    fn from(value: &ZedTheme) -> Self {
        let theme_type = match value.appearance {
            Appearance::Dark => "dark",
            Appearance::Light => "light",
        };

        let mut theme = VsCodeTheme::new(value.name.clone());
        theme.theme_type = Some(theme_type.to_string());

        // Map UI colors from Zed to VSCode
        let colors = map_zed_to_vscode_colors(&value.style);
        theme.colors = Some(colors);

        // Map syntax highlighting
        let token_colors = map_zed_syntax_to_vscode(&value.style);
        theme.token_colors = Some(TokenColors::Rules(token_colors));

        theme
    }
}

/// Maps Zed theme colors to VSCode color format
fn map_zed_to_vscode_colors(zed_style: &ZedThemeStyle) -> HashMap<String, String> {
    let mut colors = HashMap::new();

    // Editor colors
    if let Some(color) = &zed_style.editor_background {
        colors.insert("editor.background".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.editor_foreground {
        colors.insert("editor.foreground".to_string(), color.clone());
    }

    // Status bar
    if let Some(color) = &zed_style.status_bar_background {
        colors.insert("statusBar.background".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.text {
        colors.insert("statusBar.foreground".to_string(), color.clone());
    }

    // Title bar
    if let Some(color) = &zed_style.title_bar_background {
        colors.insert("titleBar.activeBackground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.title_bar_inactive_background {
        colors.insert("titleBar.inactiveBackground".to_string(), color.clone());
    }

    // Tabs
    if let Some(color) = &zed_style.tab_active_background {
        colors.insert("tab.activeBackground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.tab_inactive_background {
        colors.insert("tab.inactiveBackground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.tab_bar_background {
        colors.insert("editorGroupHeader.tabsBackground".to_string(), color.clone());
    }

    // Terminal
    if let Some(color) = &zed_style.terminal_foreground {
        colors.insert("terminal.foreground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.terminal_background {
        colors.insert("terminal.background".to_string(), color.clone());
    }

    // Terminal ANSI colors
    map_zed_terminal_colors(zed_style, &mut colors);

    // Line numbers
    if let Some(color) = &zed_style.editor_line_number {
        colors.insert("editorLineNumber.foreground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.editor_active_line_number {
        colors.insert("editorLineNumber.activeForeground".to_string(), color.clone());
    }

    // Editor highlighting
    if let Some(color) = &zed_style.editor_active_line_background {
        colors.insert("editor.lineHighlightBackground".to_string(), color.clone());
    }

    // Gutter
    if let Some(color) = &zed_style.editor_gutter_background {
        colors.insert("editorGutter.background".to_string(), color.clone());
    }

    // Basic UI colors
    if let Some(color) = &zed_style.text {
        colors.insert("foreground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.border_focused {
        colors.insert("focusBorder".to_string(), color.clone());
    }

    // Sidebar
    if let Some(color) = &zed_style.surface_background {
        colors.insert("sideBar.background".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.text {
        colors.insert("sideBar.foreground".to_string(), color.clone());
    }

    // Panel
    if let Some(color) = &zed_style.panel_background {
        colors.insert("panel.background".to_string(), color.clone());
    }

    // Selection
    if let Some(color) = &zed_style.element_selected {
        colors.insert("editor.selectionBackground".to_string(), color.clone());
    }

    // Scrollbar
    if let Some(color) = &zed_style.scrollbar_thumb_background {
        colors.insert("scrollbarSlider.background".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.scrollbar_thumb_hover_background {
        colors.insert("scrollbarSlider.hoverBackground".to_string(), color.clone());
    }

    // Search match highlighting
    if let Some(color) = &zed_style.search_match_background {
        colors.insert("editor.findMatchHighlightBackground".to_string(), color.clone());
    }

    // Text accent / link colors
    if let Some(color) = &zed_style.text_accent {
        colors.insert("textLink.foreground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.text_disabled {
        colors.insert("disabledForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.text_placeholder {
        colors.insert("input.placeholderForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.link_text_hover {
        colors.insert("textLink.activeForeground".to_string(), color.clone());
    }

    // Border variant
    if let Some(color) = &zed_style.border_variant {
        colors.insert("editorWidget.border".to_string(), color.clone());
    }

    // Active element
    if let Some(color) = &zed_style.element_active {
        colors.insert("list.activeSelectionBackground".to_string(), color.clone());
    }

    // Diagnostic colors
    if let Some(color) = &zed_style.error {
        colors.insert("editorError.foreground".to_string(), color.clone());
        colors.insert("errorForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.warning {
        colors.insert("editorWarning.foreground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.info {
        colors.insert("editorInfo.foreground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.hint {
        colors.insert("editorHint.foreground".to_string(), color.clone());
    }

    // Git decoration colors
    if let Some(color) = &zed_style.modified {
        colors.insert("gitDecoration.modifiedResourceForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.deleted {
        colors.insert("gitDecoration.deletedResourceForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.created {
        colors.insert("gitDecoration.untrackedResourceForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.conflict {
        colors.insert("gitDecoration.conflictingResourceForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.renamed {
        colors.insert("gitDecoration.renamedResourceForeground".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.ignored {
        colors.insert("gitDecoration.ignoredResourceForeground".to_string(), color.clone());
    }

    // Indent guides
    if let Some(color) = &zed_style.editor_indent_guide {
        colors.insert("editorIndentGuide.background".to_string(), color.clone());
    }
    if let Some(color) = &zed_style.editor_indent_guide_active {
        colors.insert("editorIndentGuide.activeBackground".to_string(), color.clone());
    }

    colors
}

/// Maps Zed terminal colors to VSCode format
fn map_zed_terminal_colors(zed_style: &ZedThemeStyle, colors: &mut HashMap<String, String>) {
    let terminal_color_map = [
        (&zed_style.terminal_ansi_black, "terminal.ansiBlack"),
        (&zed_style.terminal_ansi_red, "terminal.ansiRed"),
        (&zed_style.terminal_ansi_green, "terminal.ansiGreen"),
        (&zed_style.terminal_ansi_yellow, "terminal.ansiYellow"),
        (&zed_style.terminal_ansi_blue, "terminal.ansiBlue"),
        (&zed_style.terminal_ansi_magenta, "terminal.ansiMagenta"),
        (&zed_style.terminal_ansi_cyan, "terminal.ansiCyan"),
        (&zed_style.terminal_ansi_white, "terminal.ansiWhite"),
        (&zed_style.terminal_ansi_bright_black, "terminal.ansiBrightBlack"),
        (&zed_style.terminal_ansi_bright_red, "terminal.ansiBrightRed"),
        (&zed_style.terminal_ansi_bright_green, "terminal.ansiBrightGreen"),
        (&zed_style.terminal_ansi_bright_yellow, "terminal.ansiBrightYellow"),
        (&zed_style.terminal_ansi_bright_blue, "terminal.ansiBrightBlue"),
        (&zed_style.terminal_ansi_bright_magenta, "terminal.ansiBrightMagenta"),
        (&zed_style.terminal_ansi_bright_cyan, "terminal.ansiBrightCyan"),
        (&zed_style.terminal_ansi_bright_white, "terminal.ansiBrightWhite"),
    ];

    for (zed_color, vscode_key) in terminal_color_map {
        if let Some(color) = zed_color {
            colors.insert(vscode_key.to_string(), color.clone());
        }
    }
}

/// Maps Zed syntax highlighting to VSCode token colors
fn map_zed_syntax_to_vscode(zed_style: &ZedThemeStyle) -> Vec<TokenColorRule> {
    let mut rules = Vec::new();

    if let Some(syntax) = &zed_style.syntax {
        for (zed_key, highlight_style) in syntax {
            if let Some(vscode_scopes) = map_zed_key_to_textmate_scopes(zed_key) {
                let settings = TokenColorSettings {
                    foreground: highlight_style.color.clone(),
                    background: highlight_style.background_color.clone(),
                    font_style: highlight_style.font_style.as_ref().map(|fs| match fs {
                        FontStyle::Italic => "italic".to_string(),
                        FontStyle::Oblique => "oblique".to_string(),
                        FontStyle::Normal => "normal".to_string(),
                    }),
                };

                let scope = if vscode_scopes.len() == 1 {
                    TokenScope::Single(vscode_scopes[0].clone())
                } else {
                    TokenScope::Multiple(vscode_scopes)
                };

                rules.push(TokenColorRule {
                    name: Some(zed_key.clone()),
                    scope: Some(scope),
                    settings,
                });
            }
        }
    }

    rules
}

/// Maps Zed syntax keys to VSCode TextMate scopes
fn map_zed_key_to_textmate_scopes(zed_key: &str) -> Option<Vec<String>> {
    let scopes = match zed_key {
        "comment" => vec!["comment"],
        "comment.doc" => vec!["comment.block.documentation"],
        "keyword" => vec!["keyword", "keyword.control"],
        "string" => vec!["string", "string.quoted"],
        "string.regex" => vec!["string.regexp"],
        "number" => vec!["constant.numeric"],
        "constant" => vec!["constant"],
        "function" => vec!["entity.name.function", "support.function"],
        "type" => vec!["entity.name.type", "support.type", "storage.type"],
        "variable" => vec!["variable"],
        "variable.special" => vec!["variable.language", "variable.other.special"],
        "punctuation" => vec!["punctuation"],
        "punctuation.delimiter" => vec!["punctuation.definition"],
        "punctuation.bracket" => vec!["punctuation.definition.bracket"],
        "operator" => vec!["keyword.operator"],
        "tag" => vec!["entity.name.tag"],
        "title" => vec!["markup.heading"],
        "emphasis.strong" => vec!["markup.bold"],
        "emphasis" => vec!["markup.italic"],
        "error" => vec!["invalid", "invalid.illegal"],
        "constructor" => vec!["entity.name.function.constructor"],
        "property" => vec!["variable.other.property"],
        "label" => vec!["entity.name.label"],
        _ => return None,
    };

    Some(scopes.into_iter().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editors::ThemeFile;

    #[test]
    fn zed_to_vscode_conversion() {
        let zed_family = ZedThemeFamily::read("fixtures/zed/rose-pine-theme/themes/rose-pine-moon.json")
            .expect("Failed to load Zed theme");

        let thematic = &zed_family.themes[0];
        let vscode_theme: VsCodeTheme = thematic.into();

        assert_eq!(vscode_theme.name, "Rosé Pine Moon");
        assert_eq!(vscode_theme.theme_type, Some("dark".to_string()));
        assert!(vscode_theme.colors.is_some());
        assert!(vscode_theme.token_colors.is_some());
    }

    #[test]
    fn zed_family_to_vscode_themes() {
        let zed_family = ZedThemeFamily::read("fixtures/zed/rose-pine-theme/themes/rose-pine-moon.json")
            .expect("Failed to load Zed theme");

        let vscode_themes: Vec<VsCodeTheme> = (&zed_family).into();

        assert_eq!(vscode_themes.len(), zed_family.themes.len());
        assert_eq!(vscode_themes[0].name, zed_family.themes[0].name);
    }

    #[test]
    fn zed_key_to_textmate_mapping() {
        let scopes = map_zed_key_to_textmate_scopes("comment").unwrap();
        assert!(scopes.contains(&"comment".to_string()));

        let scopes = map_zed_key_to_textmate_scopes("function").unwrap();
        assert!(scopes.contains(&"entity.name.function".to_string()));
    }
}
