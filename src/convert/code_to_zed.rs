//! Trait implementation to convert VS Code themes (secretly TextMate themes)
//! into Zed themes.

use std::collections::HashMap;

use super::color::HexColor;
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

/// Returns true if the color string represents a fully transparent value.
/// VSCode themes commonly use `#0000` or `#00000000` for "transparent /
/// inherit", which should not be mapped to Zed fields as they'd produce
/// invisible UI elements.
fn is_transparent(color: &str) -> bool {
    let trimmed = color.trim_start_matches('#');
    trimmed == "0000" || trimmed == "00000000"
}

/// Maps VSCode UI colors to Zed theme style.
///
/// A dispatcher over single-concern helpers. Order matters: later steps
/// (diagnostics, defaults, derivations) read fields written by earlier ones.
fn map_ui_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
    map_base_and_chrome_colors(vscode_colors, zed_style);
    map_terminal_colors(vscode_colors, zed_style);
    map_editor_surface_colors(vscode_colors, zed_style);
    map_text_accent_colors(vscode_colors, zed_style);
    map_indent_guide_colors(vscode_colors, zed_style);

    // Extract diagnostic and git colors from the VSCode theme
    map_diagnostic_and_git_colors(vscode_colors, zed_style);

    // Set some reasonable defaults for Zed-specific colors
    set_zed_defaults(zed_style);
}

/// Editor base colors plus the surrounding workspace chrome: status/title bars,
/// tabs, and the terminal foreground/background.
fn map_base_and_chrome_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
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
}

/// Editor surface decorations: line highlight, line numbers, gutter, selection,
/// sidebar, focus border, and scrollbar.
fn map_editor_surface_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
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
        // Zed doesn't have a direct equivalent, but we can use it for other
        // selection-like colors
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
}

/// Text accents and links, plus assorted opaque-guarded UI accents (search
/// match, highlighted lines, bracket match, border variant, active element,
/// icon accent).
fn map_text_accent_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
    // Search match highlighting
    if let Some(color) = vscode_colors.get("editor.findMatchHighlightBackground")
        && !is_transparent(color)
    {
        zed_style.search_match_background = Some(color.clone());
    }

    // Text accent / link colors
    if let Some(color) = vscode_colors.get("textLink.foreground")
        && !is_transparent(color)
    {
        zed_style.text_accent = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("disabledForeground")
        && !is_transparent(color)
    {
        zed_style.text_disabled = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("input.placeholderForeground")
        && !is_transparent(color)
    {
        zed_style.text_placeholder = Some(color.clone());
    }
    if let Some(color) = vscode_colors.get("textLink.activeForeground")
        && !is_transparent(color)
    {
        zed_style.link_text_hover = Some(color.clone());
    }

    // Highlighted/bookmarked lines
    if let Some(color) = vscode_colors.get("editor.lineHighlightBackground")
        && !is_transparent(color)
    {
        zed_style.editor_highlighted_line_background = Some(color.clone());
    }

    // Bracket match highlighting
    if let Some(color) = vscode_colors.get("editorBracketMatch.background")
        && !is_transparent(color)
    {
        zed_style.editor_document_highlight_bracket_background = Some(color.clone());
    }

    // Border variant (deemphasized dividers)
    if let Some(color) = vscode_colors.get("editorWidget.border")
        && !is_transparent(color)
    {
        zed_style.border_variant = Some(color.clone());
    }

    // Active element background
    if let Some(color) = vscode_colors.get("list.activeSelectionBackground")
        && !is_transparent(color)
    {
        zed_style.element_active = Some(color.clone());
    }

    // Accent icon color — prefer activityBar.foreground, fall back to
    // textLink.foreground
    if zed_style.icon_accent.is_none()
        && let Some(color) = vscode_colors.get("activityBar.foreground")
        && !is_transparent(color)
    {
        zed_style.icon_accent = Some(color.clone());
    }
    if zed_style.icon_accent.is_none()
        && let Some(color) = vscode_colors.get("textLink.foreground")
        && !is_transparent(color)
    {
        zed_style.icon_accent = Some(color.clone());
    }
}

/// Indent guides, taken directly from VSCode where present, with an
/// opacity-reduced fallback derived from `tree.indentGuidesStroke`.
fn map_indent_guide_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
    // Indent guides — direct from VSCode (with background1 variant fallback)
    for key in ["editorIndentGuide.background", "editorIndentGuide.background1"] {
        if zed_style.editor_indent_guide.is_none()
            && let Some(color) = vscode_colors.get(key)
            && !is_transparent(color)
        {
            zed_style.editor_indent_guide = Some(color.clone());
        }
    }
    for key in [
        "editorIndentGuide.activeBackground",
        "editorIndentGuide.activeBackground1",
    ] {
        if zed_style.editor_indent_guide_active.is_none()
            && let Some(color) = vscode_colors.get(key)
            && !is_transparent(color)
        {
            zed_style.editor_indent_guide_active = Some(color.clone());
        }
    }
    if let Some(color) = vscode_colors.get("tree.indentGuidesStroke")
        && !is_transparent(color)
    {
        if zed_style.panel_indent_guide.is_none() {
            // Reduce opacity for non-active panel guide
            zed_style.panel_indent_guide = Some(HexColor::new(color).with_alpha("66"));
        }
        if zed_style.panel_indent_guide_active.is_none() {
            zed_style.panel_indent_guide_active = Some(color.clone());
        }
    }
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

/// Extracts diagnostic and git status colors from VSCode theme colors.
/// Called before `set_zed_defaults()` so that theme-specific values take
/// precedence over the hardcoded fallbacks in `set_semantic_status_colors()`.
fn map_diagnostic_and_git_colors(vscode_colors: &HashMap<String, String>, zed_style: &mut ZedThemeStyle) {
    // Diagnostic colors — prefer editor-specific, fall back to general
    if zed_style.error.is_none() {
        for key in ["editorError.foreground", "errorForeground"] {
            if let Some(color) = vscode_colors.get(key)
                && !is_transparent(color)
            {
                zed_style.error = Some(color.clone());
                break;
            }
        }
    }
    if zed_style.warning.is_none()
        && let Some(color) = vscode_colors.get("editorWarning.foreground")
        && !is_transparent(color)
    {
        zed_style.warning = Some(color.clone());
    }
    if zed_style.info.is_none()
        && let Some(color) = vscode_colors.get("editorInfo.foreground")
        && !is_transparent(color)
    {
        zed_style.info = Some(color.clone());
    }
    if zed_style.hint.is_none()
        && let Some(color) = vscode_colors.get("editorHint.foreground")
        && !is_transparent(color)
    {
        zed_style.hint = Some(color.clone());
    }
    // No direct VSCode "success" — use terminal.ansiGreen as a reasonable proxy
    if zed_style.success.is_none()
        && let Some(color) = vscode_colors.get("terminal.ansiGreen")
        && !is_transparent(color)
    {
        zed_style.success = Some(color.clone());
    }

    // Git status colors
    if zed_style.modified.is_none()
        && let Some(color) = vscode_colors.get("gitDecoration.modifiedResourceForeground")
        && !is_transparent(color)
    {
        zed_style.modified = Some(color.clone());
    }
    if zed_style.deleted.is_none()
        && let Some(color) = vscode_colors.get("gitDecoration.deletedResourceForeground")
        && !is_transparent(color)
    {
        zed_style.deleted = Some(color.clone());
    }
    if zed_style.created.is_none()
        && let Some(color) = vscode_colors.get("gitDecoration.untrackedResourceForeground")
        && !is_transparent(color)
    {
        zed_style.created = Some(color.clone());
    }
    if zed_style.conflict.is_none()
        && let Some(color) = vscode_colors.get("gitDecoration.conflictingResourceForeground")
        && !is_transparent(color)
    {
        zed_style.conflict = Some(color.clone());
    }
    if zed_style.renamed.is_none()
        && let Some(color) = vscode_colors.get("gitDecoration.renamedResourceForeground")
        && !is_transparent(color)
    {
        zed_style.renamed = Some(color.clone());
    }
    if zed_style.ignored.is_none()
        && let Some(color) = vscode_colors.get("gitDecoration.ignoredResourceForeground")
        && !is_transparent(color)
    {
        zed_style.ignored = Some(color.clone());
    }
}

/// Sets reasonable defaults for Zed-specific properties
fn set_zed_defaults(zed_style: &mut ZedThemeStyle) {
    // Set default borders if not already set
    if zed_style.border.is_none()
        && let Some(bg) = &zed_style.background
    {
        // Create a slightly lighter/darker border color
        zed_style.border = Some(bg.clone());
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

/// Derives intelligent color mappings for Zed-specific UI elements that don't
/// exist in VSCode.
///
/// A dispatcher over single-concern helpers. Order matters:
/// `derive_ghost_elements` runs before `derive_surface_defaults` so the
/// ghost-disabled derivation still sees `element_disabled` as `None` (its
/// default fallback is set afterward).
fn derive_zed_specific_colors(zed_style: &mut ZedThemeStyle) {
    derive_ghost_elements(zed_style);
    derive_surface_defaults(zed_style);
    derive_icon_colors(zed_style);
    derive_editor_guide_colors(zed_style);

    // Set status colors with semantic defaults
    set_semantic_status_colors(zed_style);
}

/// Ghost-element colors, derived from the regular element states with reduced
/// opacity.
fn derive_ghost_elements(zed_style: &mut ZedThemeStyle) {
    // Set ghost element colors based on regular elements with reduced opacity
    if let Some(element_bg) = &zed_style.element_background
        && zed_style.ghost_element_background.is_none()
    {
        zed_style.ghost_element_background = Some(HexColor::new(element_bg).with_alpha("66"));
    }
    if let Some(element_hover) = &zed_style.element_hover
        && zed_style.ghost_element_hover.is_none()
    {
        zed_style.ghost_element_hover = Some(HexColor::new(element_hover).with_alpha("44"));
    }
    if let Some(element_active) = &zed_style.element_active
        && zed_style.ghost_element_active.is_none()
    {
        zed_style.ghost_element_active = Some(HexColor::new(element_active).with_alpha("55"));
    }
    if let Some(element_selected) = &zed_style.element_selected
        && zed_style.ghost_element_selected.is_none()
    {
        zed_style.ghost_element_selected = Some(HexColor::new(element_selected).with_alpha("44"));
    }
    if let Some(element_disabled) = &zed_style.element_disabled
        && zed_style.ghost_element_disabled.is_none()
    {
        zed_style.ghost_element_disabled = Some(HexColor::new(element_disabled).with_alpha("33"));
    }
}

/// Surface and chrome defaults: disabled element, toolbar, transparent border,
/// and scrollbar track.
fn derive_surface_defaults(zed_style: &mut ZedThemeStyle) {
    // Set element_disabled from element_background if not set
    if zed_style.element_disabled.is_none() {
        zed_style.element_disabled = zed_style.element_background.clone();
    }

    // Set toolbar_background from tab_bar or editor background
    if zed_style.toolbar_background.is_none() {
        zed_style.toolbar_background = zed_style
            .tab_bar_background
            .clone()
            .or_else(|| zed_style.editor_background.clone());
    }

    // Transparent border is always semantically correct as fully transparent
    if zed_style.border_transparent.is_none() {
        zed_style.border_transparent = Some("#00000000".to_string());
    }

    // Scrollbar track matches editor background
    if zed_style.scrollbar_track_background.is_none() {
        zed_style.scrollbar_track_background = zed_style.editor_background.clone();
    }
}

/// Icon colors, derived from the corresponding text colors.
fn derive_icon_colors(zed_style: &mut ZedThemeStyle) {
    // Set icon colors based on text colors
    if let Some(text) = &zed_style.text
        && zed_style.icon.is_none()
    {
        zed_style.icon = Some(text.clone());
    }
    if let Some(text_muted) = &zed_style.text_muted
        && zed_style.icon_muted.is_none()
    {
        zed_style.icon_muted = Some(text_muted.clone());
    }
    if let Some(text_disabled) = &zed_style.text_disabled
        && zed_style.icon_disabled.is_none()
    {
        zed_style.icon_disabled = Some(text_disabled.clone());
    }
    if let Some(text_placeholder) = &zed_style.text_placeholder
        && zed_style.icon_placeholder.is_none()
    {
        zed_style.icon_placeholder = Some(text_placeholder.clone());
    }
}

/// Editor-area derivations: subheader/drop-target backgrounds, pane borders,
/// indent guides, wrap guides, invisibles, and document highlights.
fn derive_editor_guide_colors(zed_style: &mut ZedThemeStyle) {
    // Set editor-specific colors based on general background
    if let Some(bg) = &zed_style.background {
        if zed_style.editor_subheader_background.is_none() {
            zed_style.editor_subheader_background = Some(bg.clone());
        }
        if zed_style.drop_target_background.is_none() {
            zed_style.drop_target_background = Some(HexColor::new(bg).with_alpha("22"));
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
            zed_style.editor_indent_guide = Some(HexColor::new(border).with_alpha("33"));
        }
        if zed_style.editor_indent_guide_active.is_none() {
            zed_style.editor_indent_guide_active = Some(HexColor::new(border).with_alpha("66"));
        }
        if zed_style.panel_indent_guide.is_none() {
            zed_style.panel_indent_guide = Some(HexColor::new(border).with_alpha("33"));
        }
        if zed_style.panel_indent_guide_active.is_none() {
            zed_style.panel_indent_guide_active = Some(HexColor::new(border).with_alpha("66"));
        }
        if zed_style.panel_indent_guide_hover.is_none() {
            zed_style.panel_indent_guide_hover = Some(HexColor::new(border).with_alpha("55"));
        }
    }

    // Set editor wrap guides
    if let Some(line_number) = &zed_style.editor_line_number {
        if zed_style.editor_wrap_guide.is_none() {
            zed_style.editor_wrap_guide = Some(HexColor::new(line_number).with_alpha("44"));
        }
        if zed_style.editor_active_wrap_guide.is_none() {
            zed_style.editor_active_wrap_guide = Some(HexColor::new(line_number).with_alpha("77"));
        }
    }

    // Set invisible characters color
    if let Some(text_muted) = &zed_style.text_muted
        && zed_style.editor_invisible.is_none()
    {
        zed_style.editor_invisible = Some(HexColor::new(text_muted).with_alpha("33"));
    }

    // Set document highlight colors based on selection
    if let Some(selected) = &zed_style.element_selected {
        if zed_style.editor_document_highlight_read_background.is_none() {
            zed_style.editor_document_highlight_read_background = Some(HexColor::new(selected).with_alpha("33"));
        }
        if zed_style.editor_document_highlight_write_background.is_none() {
            zed_style.editor_document_highlight_write_background = Some(HexColor::new(selected).with_alpha("55"));
        }
    }
}

/// Sets semantic status colors for Git, diagnostics, etc.
///
/// A dispatcher over single-concern helpers. Diagnostics run first because the
/// git-status defaults fall back to the `success`/`warning`/`error` colors set
/// there.
fn set_semantic_status_colors(zed_style: &mut ZedThemeStyle) {
    set_diagnostic_status_defaults(zed_style);
    set_git_status_defaults(zed_style);
    set_special_state_defaults(zed_style);
}

/// Diagnostic color families (error/warning/success/info/hint), each with a
/// hardcoded fallback plus derived background and border.
fn set_diagnostic_status_defaults(zed_style: &mut ZedThemeStyle) {
    // Error colors (red family)
    if zed_style.error.is_none() {
        zed_style.error = Some("#FF6B6B".to_string());
    }
    if let Some(error) = &zed_style.error {
        if zed_style.error_background.is_none() {
            zed_style.error_background = Some(HexColor::new(error).with_alpha("22"));
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
            zed_style.warning_background = Some(HexColor::new(warning).with_alpha("22"));
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
            zed_style.success_background = Some(HexColor::new(success).with_alpha("22"));
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
            zed_style.info_background = Some(HexColor::new(info).with_alpha("22"));
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
            zed_style.hint_background = Some(HexColor::new(hint).with_alpha("22"));
        }
        if zed_style.hint_border.is_none() {
            zed_style.hint_border = Some(hint.clone());
        }
    }
}

/// Git status colors, defaulting to the diagnostic colors, then deriving
/// background and border for each.
fn set_git_status_defaults(zed_style: &mut ZedThemeStyle) {
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
                *bg_field = Some(HexColor::new(color_val).with_alpha("22"));
            }
            if border_field.is_none() {
                *border_field = Some(color_val.clone());
            }
        }
    }
}

/// Remaining special-state colors: predictive, unreachable, ignored, hidden.
fn set_special_state_defaults(zed_style: &mut ZedThemeStyle) {
    // Predictive and other special states
    if zed_style.predictive.is_none() {
        zed_style.predictive = Some("#8E8E93".to_string());
    }
    if let Some(predictive) = &zed_style.predictive {
        if zed_style.predictive_background.is_none() {
            zed_style.predictive_background = Some(HexColor::new(predictive).with_alpha("22"));
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
        zed_style.hidden = zed_style.ignored.clone().or_else(|| Some("#48484A".to_string()));
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
    let (font_style, font_weight) = parse_font_style(&rule.settings.font_style);
    let highlight_style = HighlightStyle {
        color: rule.settings.foreground.clone(),
        background_color: rule.settings.background.clone(),
        font_style,
        font_weight,
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

/// Parses a VSCode `fontStyle` set into Zed's `font_style` + `font_weight`.
///
/// VSCode `fontStyle` is a space-separated set of keywords (e.g. "bold italic
/// strikethrough"), or the empty string to unset. `italic`/`oblique` map to
/// `font_style` and `bold` to `font_weight`. `underline`/`strikethrough` have
/// no Zed equivalent and are dropped (logged at debug).
fn parse_font_style(font_style: &Option<String>) -> (Option<FontStyle>, Option<FontWeight>) {
    let mut style = None;
    let mut weight = None;
    if let Some(raw) = font_style.as_deref() {
        for token in raw.split_whitespace() {
            match token.to_ascii_lowercase().as_str() {
                "italic" => style = Some(FontStyle::Italic),
                "oblique" => style = Some(FontStyle::Oblique),
                "bold" => weight = Some(FontWeight::Number(700)),
                dropped @ ("underline" | "strikethrough") => {
                    log::debug!("Zed has no {dropped} support; dropping it from fontStyle \"{raw}\"");
                }
                other => log::debug!("Unrecognized fontStyle keyword \"{other}\" in \"{raw}\""),
            }
        }
    }
    (style, weight)
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
            selection: Some(HexColor::new(base_color).with_alpha("22")), // Add alpha
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

    #[test]
    fn transparent_colors_not_mapped() {
        assert!(is_transparent("#0000"));
        assert!(is_transparent("#00000000"));
        assert!(!is_transparent("#FF6B6B"));
        assert!(!is_transparent("#eb6f92"));
        assert!(!is_transparent("#000000"));
        assert!(!is_transparent("#000"));
    }

    #[test]
    fn derived_translucent_colors_keep_leading_hash() {
        // Regression: translucent colors are synthesized by appending a 2-digit
        // alpha suffix. The result must stay `#rrggbbaa` — a hash-less `rrggbbaa`
        // is silently rejected by Zed. (See HexColor.)
        let vscode_theme =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode theme");
        let zed: ZedTheme = (&vscode_theme).into();

        // A known derived translucent field must keep its leading '#'.
        let drop_target = zed
            .style
            .drop_target_background
            .as_deref()
            .expect("drop_target_background is derived from the editor background");
        assert!(
            drop_target.starts_with('#') && drop_target.len() == 9,
            "derived alpha color must be #rrggbbaa, got {drop_target:?}"
        );

        // No top-level color field may be a hash-less hex string.
        let style = serde_json::to_value(&zed.style).expect("style serializes");
        for (field, value) in style.as_object().expect("style is a JSON object") {
            if let Some(s) = value.as_str()
                && !s.starts_with('#')
                && (s.len() == 6 || s.len() == 8)
                && s.chars().all(|c| c.is_ascii_hexdigit())
            {
                panic!("field `{field}` has a hash-less hex color: {s:?}");
            }
        }
    }

    #[test]
    fn diagnostic_colors_from_vscode() {
        let vscode_theme =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode theme");

        let zed: ZedTheme = (&vscode_theme).into();

        // Rose Pine Moon has explicit diagnostic colors
        assert_eq!(zed.style.error.as_deref(), Some("#eb6f92"));
        assert_eq!(zed.style.warning.as_deref(), Some("#f6c177"));
        assert_eq!(zed.style.info.as_deref(), Some("#9ccfd8"));
        assert_eq!(zed.style.hint.as_deref(), Some("#908caa"));
    }

    #[test]
    fn git_colors_from_vscode() {
        let vscode_theme =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode theme");

        let zed: ZedTheme = (&vscode_theme).into();

        // Rose Pine Moon has explicit git decoration colors
        assert_eq!(zed.style.modified.as_deref(), Some("#ea9a97"));
        assert_eq!(zed.style.deleted.as_deref(), Some("#908caa"));
        assert_eq!(zed.style.created.as_deref(), Some("#f6c177"));
        assert_eq!(zed.style.conflict.as_deref(), Some("#eb6f92"));
        assert_eq!(zed.style.renamed.as_deref(), Some("#3e8fb0"));
        assert_eq!(zed.style.ignored.as_deref(), Some("#6e6a86"));
    }

    #[test]
    fn text_accent_and_placeholder() {
        let vscode_theme =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode theme");

        let zed: ZedTheme = (&vscode_theme).into();

        assert_eq!(zed.style.text_accent.as_deref(), Some("#c4a7e7"));
        assert!(zed.style.text_placeholder.is_some());
        assert!(zed.style.link_text_hover.is_some());
    }

    #[test]
    fn catppuccin_diagnostic_colors() {
        let vscode_theme = VsCodeTheme::read("fixtures/vscode/catppuccin.catppuccin-vsc-3.17.0/themes/mocha.json")
            .expect("Failed to load Catppuccin Mocha theme");

        let zed: ZedTheme = (&vscode_theme).into();

        assert_eq!(zed.style.error.as_deref(), Some("#f38ba8"));
        assert_eq!(zed.style.warning.as_deref(), Some("#fab387"));
        assert_eq!(zed.style.info.as_deref(), Some("#89b4fa"));
        assert!(zed.style.text_disabled.is_some());
    }

    #[test]
    fn font_style_empty_or_none_is_unset() {
        assert_eq!(parse_font_style(&None), (None, None));
        assert_eq!(parse_font_style(&Some(String::new())), (None, None));
    }

    #[test]
    fn font_style_single_keywords() {
        assert_eq!(
            parse_font_style(&Some("italic".to_string())),
            (Some(FontStyle::Italic), None)
        );
        assert_eq!(
            parse_font_style(&Some("oblique".to_string())),
            (Some(FontStyle::Oblique), None)
        );
        assert_eq!(
            parse_font_style(&Some("bold".to_string())),
            (None, Some(FontWeight::Number(700)))
        );
    }

    #[test]
    fn font_style_bold_italic_combination_is_order_independent() {
        let expected = (Some(FontStyle::Italic), Some(FontWeight::Number(700)));
        assert_eq!(parse_font_style(&Some("bold italic".to_string())), expected);
        assert_eq!(parse_font_style(&Some("italic bold".to_string())), expected);
    }

    #[test]
    fn font_style_drops_underline_and_strikethrough() {
        // Zed has no field for these decorations; the rest of the set still maps.
        assert_eq!(
            parse_font_style(&Some("italic strikethrough".to_string())),
            (Some(FontStyle::Italic), None)
        );
        assert_eq!(
            parse_font_style(&Some("bold strikethrough".to_string())),
            (None, Some(FontWeight::Number(700)))
        );
        assert_eq!(parse_font_style(&Some("underline".to_string())), (None, None));
    }

    #[test]
    fn font_style_tolerates_extra_whitespace() {
        assert_eq!(
            parse_font_style(&Some("  bold   italic  ".to_string())),
            (Some(FontStyle::Italic), Some(FontWeight::Number(700)))
        );
    }
}
