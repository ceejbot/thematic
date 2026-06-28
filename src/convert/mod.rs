pub mod code_to_zed;
pub(crate) mod color;
pub mod zed_to_code;

#[cfg(test)]
mod tests {
    use crate::editors::ThemeFile;
    use crate::editors::vscode::{ThemeType, TokenColors, TokenScope};
    use crate::editors::zed::Appearance;
    use crate::{VsCodeTheme, ZedTheme, ZedThemeFamily};

    #[test]
    fn comprehensive_theme_conversion_test() {
        // Test conversion with Rose Pine Moon (dark theme)
        let vscode_rose_pine =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode Rose Pine Moon theme");

        let zed_rose_pine: ZedTheme = (&vscode_rose_pine).into();
        let vscode_converted_back: VsCodeTheme = (&zed_rose_pine).into();

        // Verify basic properties are preserved
        assert_eq!(zed_rose_pine.name, "Rosé Pine Moon");
        assert!(matches!(zed_rose_pine.appearance, Appearance::Dark));
        assert_eq!(vscode_converted_back.name, "Rosé Pine Moon");
        assert_eq!(vscode_converted_back.theme_type, Some(ThemeType::Dark));

        // Test with Catppuccin Latte (light theme)
        let vscode_latte = VsCodeTheme::read("fixtures/vscode/catppuccin.catppuccin-vsc-3.17.0/themes/latte.json")
            .expect("Failed to load VSCode Catppuccin Latte theme");

        let zed_latte: ZedTheme = (&vscode_latte).into();
        let vscode_latte_converted_back: VsCodeTheme = (&zed_latte).into();

        // Verify basic properties are preserved
        assert_eq!(zed_latte.name, "Catppuccin Latte");
        assert!(matches!(zed_latte.appearance, Appearance::Light));
        assert_eq!(vscode_latte_converted_back.name, "Catppuccin Latte");
        assert_eq!(vscode_latte_converted_back.theme_type, Some(ThemeType::Light));

        // Verify essential colors are mapped
        assert!(zed_rose_pine.style.editor_background.is_some());
        assert!(zed_rose_pine.style.editor_foreground.is_some());
        assert!(zed_rose_pine.style.syntax.is_some());
        assert!(zed_rose_pine.style.players.is_some());

        // Test Zed to VSCode conversion
        let zed_family = ZedThemeFamily::read("fixtures/zed/rose-pine-theme/themes/rose-pine-moon.json")
            .expect("Failed to load Zed Rose Pine Moon theme");

        let original_zed_theme = &zed_family.themes[0];
        let converted_vscode: VsCodeTheme = original_zed_theme.into();

        assert_eq!(converted_vscode.name, "Rosé Pine Moon");
        assert!(converted_vscode.colors.is_some());
        assert!(converted_vscode.token_colors.is_some());

        // Verify some key colors are present
        let colors = converted_vscode.colors.as_ref().unwrap();
        assert!(colors.contains_key("editor.background"));
        assert!(colors.contains_key("editor.foreground"));

        // Verify token colors are mapped
        if let Some(TokenColors::Rules(rules)) = &converted_vscode.token_colors {
            assert!(!rules.is_empty());
            // Check that we have some basic syntax highlighting
            let has_comment = rules.iter().any(|rule| {
                if let Some(TokenScope::Single(scope)) = &rule.scope {
                    scope == "comment"
                } else if let Some(TokenScope::Multiple(scopes)) = &rule.scope {
                    scopes.contains(&"comment".to_string())
                } else {
                    false
                }
            });
            assert!(has_comment, "Should have comment syntax highlighting");
        }
    }

    #[test]
    fn color_mapping_accuracy() {
        // Load both VSCode and Zed versions of the same theme
        let vscode_theme =
            VsCodeTheme::read("fixtures/vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
                .expect("Failed to load VSCode theme");
        let zed_family = ZedThemeFamily::read("fixtures/zed/rose-pine-theme/themes/rose-pine-moon.json")
            .expect("Failed to load Zed theme");
        let thematic = &zed_family.themes[0];

        // Convert VSCode to Zed
        let converted_zed: ZedTheme = (&vscode_theme).into();

        // Check that key colors match between original Zed and converted Zed
        if let (Some(orig_bg), Some(conv_bg)) = (
            &thematic.style.editor_background,
            &converted_zed.style.editor_background,
        ) {
            assert_eq!(orig_bg, conv_bg, "Editor background should match");
        }

        if let (Some(orig_fg), Some(conv_fg)) = (
            &thematic.style.editor_foreground,
            &converted_zed.style.editor_foreground,
        ) {
            assert_eq!(orig_fg, conv_fg, "Editor foreground should match");
        }
    }
}
