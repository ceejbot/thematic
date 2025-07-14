//! Trait implementation to convert VS Code themes (secretly TextMate themes)
//! into Zed themes.

use std::collections::HashMap;

use crate::editors::vscode::{TokenColorRule, TokenColors, TokenScope};
use crate::editors::zed::{Appearance, FontStyle, FontWeight, HighlightStyle, PlayerColor, ZedThemeStyle};
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

    // Enhance Zed-specific color mapping
    derive_zed_specific_colors(zed_style);
}

/// Derives intelligent color mappings for Zed-specific UI elements that don't exist in VSCode
fn derive_zed_specific_colors(zed_style: &mut ZedThemeStyle) {
    // Set ghost element colors based on regular elements with reduced opacity
    if let Some(element_bg) = &zed_style.element_background {
        if zed_style.ghost_element_background.is_none() {
            zed_style.ghost_element_background = Some(format!("{}66", element_bg.trim_start_matches('#')));
        }
    }
    if let Some(element_hover) = &zed_style.element_hover {
        if zed_style.ghost_element_hover.is_none() {
            zed_style.ghost_element_hover = Some(format!("{}44", element_hover.trim_start_matches('#')));
        }
    }
    if let Some(element_active) = &zed_style.element_active {
        if zed_style.ghost_element_active.is_none() {
            zed_style.ghost_element_active = Some(format!("{}55", element_active.trim_start_matches('#')));
        }
    }

    // Set icon colors based on text colors
    if let Some(text) = &zed_style.text {
        if zed_style.icon.is_none() {
            zed_style.icon = Some(text.clone());
        }
    }
    if let Some(text_muted) = &zed_style.text_muted {
        if zed_style.icon_muted.is_none() {
            zed_style.icon_muted = Some(text_muted.clone());
        }
    }
    if let Some(text_disabled) = &zed_style.text_disabled {
        if zed_style.icon_disabled.is_none() {
            zed_style.icon_disabled = Some(text_disabled.clone());
        }
    }
    if let Some(text_placeholder) = &zed_style.text_placeholder {
        if zed_style.icon_placeholder.is_none() {
            zed_style.icon_placeholder = Some(text_placeholder.clone());
        }
    }

    // Set editor-specific colors based on general background
    if let Some(bg) = &zed_style.background {
        if zed_style.editor_subheader_background.is_none() {
            zed_style.editor_subheader_background = Some(bg.clone());
        }
        if zed_style.drop_target_background.is_none() {
            zed_style.drop_target_background = Some(format!("{}22", bg.trim_start_matches('#')));
        }
    }

    // Set pane borders based on border color
    if let Some(border) = &zed_style.border {
        if zed_style.pane_focused_border.is_none() {
            zed_style.pane_focused_border = Some(border.clone());
        }
        if zed_style.pane_group_border.is_none() {
            zed_style.pane_group_border = Some(border.clone());
        }
        if zed_style.panel_focused_border.is_none() {
            zed_style.panel_focused_border = Some(border.clone());
        }
    }

    // Set indent guides based on border with reduced opacity
    if let Some(border) = &zed_style.border {
        if zed_style.editor_indent_guide.is_none() {
            zed_style.editor_indent_guide = Some(format!("{}33", border.trim_start_matches('#')));
        }
        if zed_style.editor_indent_guide_active.is_none() {
            zed_style.editor_indent_guide_active = Some(format!("{}66", border.trim_start_matches('#')));
        }
        if zed_style.panel_indent_guide.is_none() {
            zed_style.panel_indent_guide = Some(format!("{}33", border.trim_start_matches('#')));
        }
        if zed_style.panel_indent_guide_active.is_none() {
            zed_style.panel_indent_guide_active = Some(format!("{}66", border.trim_start_matches('#')));
        }
        if zed_style.panel_indent_guide_hover.is_none() {
            zed_style.panel_indent_guide_hover = Some(format!("{}55", border.trim_start_matches('#')));
        }
    }

    // Set editor wrap guides
    if let Some(line_number) = &zed_style.editor_line_number {
        if zed_style.editor_wrap_guide.is_none() {
            zed_style.editor_wrap_guide = Some(format!("{}44", line_number.trim_start_matches('#')));
        }
        if zed_style.editor_active_wrap_guide.is_none() {
            zed_style.editor_active_wrap_guide = Some(format!("{}77", line_number.trim_start_matches('#')));
        }
    }

    // Set invisible characters color
    if let Some(text_muted) = &zed_style.text_muted {
        if zed_style.editor_invisible.is_none() {
            zed_style.editor_invisible = Some(format!("{}33", text_muted.trim_start_matches('#')));
        }
    }

    // Set document highlight colors based on selection
    if let Some(selected) = &zed_style.element_selected {
        if zed_style.editor_document_highlight_read_background.is_none() {
            zed_style.editor_document_highlight_read_background =
                Some(format!("{}33", selected.trim_start_matches('#')));
        }
        if zed_style.editor_document_highlight_write_background.is_none() {
            zed_style.editor_document_highlight_write_background =
                Some(format!("{}55", selected.trim_start_matches('#')));
        }
    }

    // Set status colors with semantic defaults
    set_semantic_status_colors(zed_style);
}

/// Sets semantic status colors for Git, diagnostics, etc.
fn set_semantic_status_colors(zed_style: &mut ZedThemeStyle) {
    // Error colors (red family)
    if zed_style.error.is_none() {
        zed_style.error = Some("#FF6B6B".to_string());
    }
    if let Some(error) = &zed_style.error {
        if zed_style.error_background.is_none() {
            zed_style.error_background = Some(format!("{}22", error.trim_start_matches('#')));
        }
        if zed_style.error_border.is_none() {
            zed_style.error_border = Some(error.clone());
        }
    }

    // Warning colors (yellow/orange family)
    if zed_style.warning.is_none() {
        zed_style.warning = Some("#FFB347".to_string());
    }
    if let Some(warning) = &zed_style.warning {
        if zed_style.warning_background.is_none() {
            zed_style.warning_background = Some(format!("{}22", warning.trim_start_matches('#')));
        }
        if zed_style.warning_border.is_none() {
            zed_style.warning_border = Some(warning.clone());
        }
    }

    // Success colors (green family)
    if zed_style.success.is_none() {
        zed_style.success = Some("#51CF66".to_string());
    }
    if let Some(success) = &zed_style.success {
        if zed_style.success_background.is_none() {
            zed_style.success_background = Some(format!("{}22", success.trim_start_matches('#')));
        }
        if zed_style.success_border.is_none() {
            zed_style.success_border = Some(success.clone());
        }
    }

    // Info colors (blue family)
    if zed_style.info.is_none() {
        zed_style.info = Some("#4DABF7".to_string());
    }
    if let Some(info) = &zed_style.info {
        if zed_style.info_background.is_none() {
            zed_style.info_background = Some(format!("{}22", info.trim_start_matches('#')));
        }
        if zed_style.info_border.is_none() {
            zed_style.info_border = Some(info.clone());
        }
    }

    // Hint colors (purple family)
    if zed_style.hint.is_none() {
        zed_style.hint = Some("#9775FA".to_string());
    }
    if let Some(hint) = &zed_style.hint {
        if zed_style.hint_background.is_none() {
            zed_style.hint_background = Some(format!("{}22", hint.trim_start_matches('#')));
        }
        if zed_style.hint_border.is_none() {
            zed_style.hint_border = Some(hint.clone());
        }
    }

    // Git status colors
    if zed_style.created.is_none() {
        zed_style.created = zed_style.success.clone();
    }
    if zed_style.modified.is_none() {
        zed_style.modified = zed_style.warning.clone();
    }
    if zed_style.deleted.is_none() {
        zed_style.deleted = zed_style.error.clone();
    }
    if zed_style.conflict.is_none() {
        zed_style.conflict = Some("#FF8C42".to_string());
    }
    if zed_style.renamed.is_none() {
        zed_style.renamed = zed_style.info.clone();
    }

    // Set corresponding background and border colors for Git status
    for (color, bg_field, border_field) in [
        (
            &zed_style.created,
            &mut zed_style.created_background,
            &mut zed_style.created_border,
        ),
        (
            &zed_style.modified,
            &mut zed_style.modified_background,
            &mut zed_style.modified_border,
        ),
        (
            &zed_style.deleted,
            &mut zed_style.deleted_background,
            &mut zed_style.deleted_border,
        ),
        (
            &zed_style.conflict,
            &mut zed_style.conflict_background,
            &mut zed_style.conflict_border,
        ),
        (
            &zed_style.renamed,
            &mut zed_style.renamed_background,
            &mut zed_style.renamed_border,
        ),
    ] {
        if let Some(color_val) = color {
            if bg_field.is_none() {
                *bg_field = Some(format!("{}22", color_val.trim_start_matches('#')));
            }
            if border_field.is_none() {
                *border_field = Some(color_val.clone());
            }
        }
    }

    // Predictive and other special states
    if zed_style.predictive.is_none() {
        zed_style.predictive = Some("#8E8E93".to_string());
    }
    if let Some(predictive) = &zed_style.predictive {
        if zed_style.predictive_background.is_none() {
            zed_style.predictive_background = Some(format!("{}22", predictive.trim_start_matches('#')));
        }
        if zed_style.predictive_border.is_none() {
            zed_style.predictive_border = Some(predictive.clone());
        }
    }

    if zed_style.unreachable.is_none() {
        zed_style.unreachable = Some("#6C6C70".to_string());
    }
    if zed_style.ignored.is_none() {
        zed_style.ignored = Some("#8E8E93".to_string());
    }
    if zed_style.hidden.is_none() {
        zed_style.hidden = Some("#48484A".to_string());
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
        // Comments
        "comment" | "comment.line" | "comment.block" => Some("comment".to_string()),
        "comment.block.documentation" => Some("comment.doc".to_string()),

        // Keywords and storage
        "keyword" | "keyword.control" | "keyword.control.flow" => Some("keyword".to_string()),
        "keyword.operator" => Some("operator".to_string()),
        "storage.type" | "storage.modifier" => Some("keyword".to_string()),

        // Strings and literals
        "string" | "string.quoted" | "string.quoted.single" | "string.quoted.double" => Some("string".to_string()),
        "string.regexp" | "string.regex" => Some("string.regex".to_string()),
        "string.unquoted" => Some("string".to_string()),
        "string.interpolated" => Some("string".to_string()),

        // Numbers and constants
        "constant.numeric" | "constant.numeric.integer" | "constant.numeric.float" => Some("number".to_string()),
        "constant.character" | "constant.character.escape" => Some("string".to_string()),
        "constant" | "constant.other" => Some("constant".to_string()),
        "constant.language" => Some("boolean".to_string()),
        "constant.language.boolean" => Some("boolean".to_string()),
        "constant.language.null" => Some("constant".to_string()),

        // Functions and methods
        "entity.name.function" | "entity.name.function.member" => Some("function".to_string()),
        "entity.name.function.constructor" => Some("constructor".to_string()),
        "support.function" => Some("function".to_string()),
        "meta.function-call" => Some("function".to_string()),

        // Types and classes
        "entity.name.type" | "entity.name.class" => Some("type".to_string()),
        "entity.name.type.class" => Some("type".to_string()),
        "entity.name.enum" => Some("enum".to_string()),
        "entity.name.interface" => Some("type".to_string()),
        "entity.name.struct" => Some("type".to_string()),
        "support.type" => Some("type".to_string()),
        "storage.type.class" => Some("type".to_string()),

        // Variables and parameters
        "variable" | "variable.other" => Some("variable".to_string()),
        "variable.language" | "variable.language.this" | "variable.language.self" => {
            Some("variable.special".to_string())
        }
        "variable.parameter" => Some("variable".to_string()),
        "variable.other.member" => Some("property".to_string()),
        "variable.other.property" => Some("property".to_string()),

        // Properties and attributes
        "entity.other.attribute-name" => Some("attribute".to_string()),
        "entity.other.attribute-name.class" => Some("attribute".to_string()),
        "entity.other.attribute-name.id" => Some("attribute".to_string()),

        // Punctuation
        "punctuation" => Some("punctuation".to_string()),
        "punctuation.definition" => Some("punctuation.delimiter".to_string()),
        "punctuation.definition.string" => Some("punctuation.delimiter".to_string()),
        "punctuation.definition.comment" => Some("punctuation.delimiter".to_string()),
        "punctuation.separator" => Some("punctuation.delimiter".to_string()),
        "punctuation.terminator" => Some("punctuation.delimiter".to_string()),
        "punctuation.accessor" => Some("punctuation.delimiter".to_string()),
        "punctuation.section.brackets" | "punctuation.section.brackets.begin" | "punctuation.section.brackets.end" => {
            Some("punctuation.bracket".to_string())
        }
        "punctuation.section.parens" | "punctuation.section.parens.begin" | "punctuation.section.parens.end" => {
            Some("punctuation.bracket".to_string())
        }
        "punctuation.section.braces" | "punctuation.section.braces.begin" | "punctuation.section.braces.end" => {
            Some("punctuation.bracket".to_string())
        }

        // Markup (Markdown, etc.)
        "markup.heading" | "markup.heading.1" | "markup.heading.2" | "markup.heading.3" => Some("title".to_string()),
        "markup.bold" => Some("emphasis.strong".to_string()),
        "markup.italic" => Some("emphasis".to_string()),
        "markup.underline" => Some("emphasis".to_string()),
        "markup.strikethrough" => Some("emphasis".to_string()),
        "markup.list" => Some("punctuation.list_marker".to_string()),
        "markup.list.numbered" => Some("punctuation.list_marker".to_string()),
        "markup.list.unnumbered" => Some("punctuation.list_marker".to_string()),
        "markup.quote" => Some("string".to_string()),
        "markup.raw" | "markup.raw.inline" | "markup.raw.block" => Some("string".to_string()),
        "markup.fenced_code" => Some("string".to_string()),

        // Links
        "markup.underline.link" => Some("link_uri".to_string()),
        "string.other.link" => Some("link_uri".to_string()),
        "meta.link" => Some("link_text".to_string()),

        // Tags (HTML, XML)
        "entity.name.tag" => Some("tag".to_string()),
        "entity.name.tag.open" | "entity.name.tag.close" => Some("tag".to_string()),

        // Preprocessor
        "meta.preprocessor" => Some("preproc".to_string()),
        "keyword.control.directive" => Some("preproc".to_string()),
        "entity.name.function.preprocessor" => Some("preproc".to_string()),

        // Labels and goto
        "entity.name.label" => Some("label".to_string()),

        // Errors and invalid
        "invalid" | "invalid.illegal" | "invalid.deprecated" => Some("error".to_string()),

        // Language-specific mappings
        "support.class" => Some("type".to_string()),
        "support.constant" => Some("constant".to_string()),
        "support.variable" => Some("variable".to_string()),
        "entity.name.module" | "entity.name.namespace" => Some("type".to_string()),

        _ => {
            // Enhanced fallback logic with more specific pattern matching
            if scope.contains("comment.doc") || scope.contains("documentation") {
                Some("comment.doc".to_string())
            } else if scope.contains("comment") {
                Some("comment".to_string())
            } else if scope.contains("string.regex") || scope.contains("string.regexp") {
                Some("string.regex".to_string())
            } else if scope.contains("string") {
                Some("string".to_string())
            } else if scope.contains("keyword.operator") || scope.contains("operator") {
                Some("operator".to_string())
            } else if scope.contains("keyword") {
                Some("keyword".to_string())
            } else if scope.contains("function.constructor") || scope.contains("constructor") {
                Some("constructor".to_string())
            } else if scope.contains("function") {
                Some("function".to_string())
            } else if scope.contains("class") || scope.contains("interface") || scope.contains("struct") {
                Some("type".to_string())
            } else if scope.contains("enum") {
                Some("enum".to_string())
            } else if scope.contains("type") {
                Some("type".to_string())
            } else if scope.contains("property") || scope.contains("member") {
                Some("property".to_string())
            } else if scope.contains("variable.special") || scope.contains("variable.language") {
                Some("variable.special".to_string())
            } else if scope.contains("variable") {
                Some("variable".to_string())
            } else if scope.contains("attribute") {
                Some("attribute".to_string())
            } else if scope.contains("constant.numeric") || scope.contains("numeric") {
                Some("number".to_string())
            } else if scope.contains("constant.language.boolean") || scope.contains("boolean") {
                Some("boolean".to_string())
            } else if scope.contains("constant") {
                Some("constant".to_string())
            } else if scope.contains("punctuation.bracket") || scope.contains("bracket") {
                Some("punctuation.bracket".to_string())
            } else if scope.contains("punctuation.list") {
                Some("punctuation.list_marker".to_string())
            } else if scope.contains("punctuation") {
                Some("punctuation".to_string())
            } else if scope.contains("markup.heading") || scope.contains("heading") {
                Some("title".to_string())
            } else if scope.contains("markup.bold") || scope.contains("bold") {
                Some("emphasis.strong".to_string())
            } else if scope.contains("markup.italic") || scope.contains("italic") {
                Some("emphasis".to_string())
            } else if scope.contains("link") {
                Some("link_text".to_string())
            } else if scope.contains("tag") {
                Some("tag".to_string())
            } else if scope.contains("preprocessor") || scope.contains("directive") {
                Some("preproc".to_string())
            } else if scope.contains("label") {
                Some("label".to_string())
            } else if scope.contains("invalid") || scope.contains("illegal") {
                Some("error".to_string())
            } else {
                // If we can't classify it, return None to skip
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
    // Enhanced defaults that cover more Zed syntax keys with more modern colors
    let defaults = [
        // Core syntax elements
        ("comment", "#6A9955"),
        ("comment.doc", "#7C9F07"),
        ("keyword", "#569CD6"),
        ("string", "#CE9178"),
        ("string.regex", "#D16969"),
        ("number", "#B5CEA8"),
        ("boolean", "#569CD6"),
        ("constant", "#4FC1FF"),
        ("function", "#DCDCAA"),
        ("constructor", "#4EC9B0"),
        ("type", "#4EC9B0"),
        ("enum", "#4EC9B0"),
        ("variable", "#9CDCFE"),
        ("variable.special", "#C586C0"),
        ("property", "#9CDCFE"),
        ("attribute", "#92C5F7"),
        ("operator", "#D4D4D4"),
        // Punctuation variants
        ("punctuation", "#D4D4D4"),
        ("punctuation.delimiter", "#D4D4D4"),
        ("punctuation.bracket", "#FFD700"),
        ("punctuation.list_marker", "#6A9955"),
        // Markup elements
        ("title", "#4EC9B0"),
        ("emphasis", "#C586C0"),
        ("emphasis.strong", "#569CD6"),
        ("link_text", "#3794FF"),
        ("link_uri", "#3794FF"),
        ("tag", "#569CD6"),
        // Special elements
        ("preproc", "#C586C0"),
        ("label", "#C586C0"),
        ("error", "#F44747"),
        ("hint", "#3794FF"),
        ("predictive", "#6A9955"),
        ("primary", "#569CD6"),
        ("embedded", "#9CDCFE"),
    ];

    for (key, color) in defaults {
        if !syntax_map.contains_key(key) {
            let mut style = HighlightStyle {
                color: Some(color.to_string()),
                font_style: None,
                font_weight: None,
                background_color: None,
            };

            // Add appropriate font styles for certain elements
            match key {
                "comment" | "comment.doc" => {
                    style.font_style = Some(FontStyle::Italic);
                }
                "emphasis" => {
                    style.font_style = Some(FontStyle::Italic);
                }
                "emphasis.strong" => {
                    style.font_weight = Some(FontWeight::Number(700));
                }
                _ => {}
            }

            syntax_map.insert(key.to_string(), style);
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
