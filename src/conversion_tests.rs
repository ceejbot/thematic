//! Comprehensive conversion tests using fixtures
//!
//! These tests verify that conversions between VSCode and Zed formats work
//! correctly by using real fixture data and comparing outputs.

use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;

use crate::editors::{Extension, ThemeFile, VsCodeExtension, ZedExtension};
use crate::vscode::TokenColors;
use crate::zed::{FontStyle, FontWeight};
use crate::{ThemeError, VsCodeTheme, ZedManifest, ZedTheme, ZedThemeFamily};

const FIXTURES_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// Helper to create a unique temporary directory
fn create_temp_dir(test_name: &str) -> Result<PathBuf, std::io::Error> {
    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("thematic_test_{}_{}", test_name, std::process::id()));
    fs::create_dir_all(&temp_path)?;
    Ok(temp_path)
}

/// Helper to clean up temporary directory
fn cleanup_temp_dir(path: &PathBuf) -> Result<(), std::io::Error> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Helper to get fixture path
fn fixture_path(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::from(FIXTURES_DIR);
    path.push("fixtures");
    path.push(relative_path);
    path
}

/// Read a VSCode theme fixture
fn read_vscode_fixture(path: &str) -> Result<VsCodeTheme, ThemeError> {
    let fixture_path = fixture_path(path);
    VsCodeTheme::read(fixture_path)
}

/// Read a Zed theme fixture
fn read_zed_fixture(path: &str) -> Result<ZedThemeFamily, ThemeError> {
    let fixture_path = fixture_path(path);
    ZedThemeFamily::read(fixture_path)
}

#[test]
fn vscode_to_zed_rose_pine_conversion() -> Result<(), ThemeError> {
    // Load VSCode Rose Pine Moon extension fixture
    let vscode_ext = VsCodeExtension::find_from_name("rose-pine", "./fixtures/vscode/")?;
    // Convert to Zed extension
    let zed_extension = ZedExtension::from(vscode_ext.clone());

    // Verify the extension has correct metadata
    assert!(!zed_extension.name().is_empty(), "Extension should have a name");
    assert!(
        !zed_extension.manifest().themes().is_empty(),
        "Extension should reference theme files"
    );

    // both extensions should have Rose Pine Moon in them
    let families = zed_extension.families();
    assert!(!families.is_empty());
    let _first = &families[0];
    // was going to do something with this

    let this_family = families.iter().find(|fam| {
        let found = fam.themes.iter().find(|theme| theme.name == "Rosé Pine Moon");
        found.is_some()
    });
    assert!(this_family.is_some(), "cannot find a family with Pine Moon");

    let zed_themes = zed_extension.themes();
    let maybe_zed_moon = zed_themes.iter().find(|theme| theme.name == "Rosé Pine Moon");
    assert!(maybe_zed_moon.is_some(), "Zed extension should include Rosé Pine Moon");
    let maybe_vsc_moon = vscode_ext
        .themes()
        .iter()
        .find(|theme| theme.name == "Rosé Pine Moon")
        .cloned();
    assert!(
        maybe_vsc_moon.is_some(),
        "VSCode extension should include Rosé Pine Moon"
    );

    #[allow(clippy::unwrap_used)]
    let zed_moon = maybe_zed_moon.unwrap();
    #[allow(clippy::unwrap_used)]
    let _vsc_moon = maybe_vsc_moon.unwrap();

    assert!(
        matches!(zed_moon.appearance, crate::zed::Appearance::Dark),
        "Rosé Pine Moon should be dark"
    );

    // Verify that essential colors are present
    assert!(
        zed_moon.style.background.is_some(),
        "Background style should be present"
    );

    Ok(())
}

#[test]
fn zed_rose_pine_manifest() {
    let pkg_path = "./fixtures/zed/rose-pine-theme/extension.toml";
    let contents = std::fs::read_to_string(pkg_path).expect("this should be a valid manifest path");
    assert!(!contents.is_empty());
    let manifest: ZedManifest =
        toml::from_str(contents.as_str()).expect("this is a valid manifest we should deserialize");
    assert_eq!(manifest.name(), "Rosé Pine");
}

#[test]
fn zed_to_vscode_rose_pine_conversion() {
    // Load VSCode Rose Pine Moon extension fixture
    let zed_extension =
        ZedExtension::find_from_name("rose-pine", "./fixtures/zed").expect("rose pine fixture should be findable");
    // Convert to VSCode extension
    let vscode_extension = VsCodeExtension::from(zed_extension.clone());

    // Verify the extension has correct metadata
    assert!(!vscode_extension.name().is_empty(), "Extension should have a name");
    assert_eq!(
        zed_extension.name(),
        vscode_extension.name(),
        "names should be identical"
    );
    assert_eq!(
        zed_extension.manifest().themes().len(),
        vscode_extension.manifest().themes().len(),
        "vscode version should have the same number of theme variants"
    );
    assert_eq!(
        vscode_extension.manifest().themes().len(),
        3,
        "we expected three theme variants"
    );

    // Get the themes from the extension
    let themes = vscode_extension.themes();
    assert!(!themes.is_empty(), "Extension should have at least one theme");
    assert_eq!(themes.len(), 3, "in fact we expected 3");

    // both extensions should have Rose Pine Moon in them
    let zed_themes = zed_extension.themes();
    let maybe_zed_moon = zed_themes.iter().find(|theme| theme.name.contains("Pine Moon"));
    assert!(maybe_zed_moon.is_some(), "Zed extension should include Rosé Pine Moon");
    let maybe_vsc_moon = themes.iter().find(|theme| theme.name.contains("Pine Moon")).cloned();
    assert!(
        maybe_vsc_moon.is_some(),
        "VSCode extension should include Rosé Pine Moon"
    );

    #[allow(clippy::unwrap_used)]
    let zed_moon = maybe_zed_moon.unwrap();
    #[allow(clippy::unwrap_used)]
    let vsc_moon = maybe_vsc_moon.unwrap();

    assert_eq!(vsc_moon.name, zed_moon.name, "Theme name should be preserved");
    assert!(
        matches!(zed_moon.appearance, crate::zed::Appearance::Dark),
        "Rosé Pine Moon should be dark"
    );
    assert!(vsc_moon.is_dark_theme(), "Rose Pine Moon should be dark");

    // Verify that colors were converted
    assert!(vsc_moon.colors.is_some(), "VSCode theme should have colors");
    if let Some(colors) = &vsc_moon.colors {
        assert!(!colors.is_empty(), "Colors map should not be empty");
        assert!(
            colors.contains_key("editor.background"),
            "Should have editor background color"
        );
    }
}

#[test]
fn round_trip_vscode_to_zed_to_vscode() {
    // Load original VSCode theme
    let original_vscode = read_vscode_fixture("vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
        .expect("vscode rose pine fixture should be findable");

    // Convert to Zed
    let zed_extension = ZedExtension::from(&original_vscode);
    let zed_themes = zed_extension.themes();

    // Convert back to VSCode
    let round_trip_vscode = VsCodeTheme::from(&zed_themes[0]);

    // Compare key properties that should be preserved
    assert_eq!(
        original_vscode.name, round_trip_vscode.name,
        "Theme name should survive round trip"
    );
    assert_eq!(
        original_vscode.is_dark_theme(),
        round_trip_vscode.is_dark_theme(),
        "Theme darkness should be preserved"
    );

    // Check that we have some colors preserved
    if let (Some(orig_colors), Some(rt_colors)) = (&original_vscode.colors, &round_trip_vscode.colors) {
        // At least some basic colors should be preserved
        assert!(!rt_colors.is_empty(), "Round trip theme should have colors");

        // Check specific colors that should definitely be preserved
        for key in ["editor.background", "editor.foreground"] {
            if orig_colors.contains_key(key) {
                assert!(
                    rt_colors.contains_key(key),
                    "Key color '{key}' should be preserved in round trip"
                );
            }
        }
    }
}

#[test]
fn round_trip_zed_to_vscode_to_zed() {
    // Load original Zed theme
    let original_zed_family = read_zed_fixture("zed/rose-pine-theme/themes/rose-pine-moon.json")
        .expect("zed rose pine fixture should be findable");
    let original_zed = &original_zed_family.themes[0];

    // Convert to VSCode
    let vscode_theme = VsCodeTheme::from(original_zed);

    // Convert back to Zed
    let round_trip_zed = ZedTheme::from(&vscode_theme);

    // Compare key properties
    assert_eq!(
        original_zed.name, round_trip_zed.name,
        "Theme name should survive round trip"
    );
    assert_eq!(
        original_zed.appearance, round_trip_zed.appearance,
        "Theme appearance should survive round trip"
    );

    // Check that essential style components exist
    if original_zed.style.background.is_some() {
        assert!(
            round_trip_zed.style.background.is_some(),
            "Background styles should be preserved"
        );
    }
}

#[test]
fn font_style_combination_round_trips() {
    // A VSCode `fontStyle` set ("bold italic") must split into Zed's separate
    // `font_style` + `font_weight`, then reassemble on the way back.
    let json = r##"{
        "name": "Font Style Test",
        "type": "dark",
        "tokenColors": [
            { "scope": "comment", "settings": { "foreground": "#abcdef", "fontStyle": "bold italic" } }
        ]
    }"##;
    let vscode: VsCodeTheme = serde_json::from_str(json).expect("minimal theme json should parse");

    // Forward: "bold italic" -> font_style: italic + font_weight: 700 on the Zed
    // "comment" key.
    let zed = ZedTheme::from(&vscode);
    let comment = zed
        .style
        .syntax
        .as_ref()
        .and_then(|syntax| syntax.get("comment"))
        .expect("comment syntax entry should exist");
    assert_eq!(comment.font_style, Some(FontStyle::Italic));
    assert_eq!(comment.font_weight, Some(FontWeight::Number(700)));

    // Reverse: italic + bold weight reassemble to the VSCode keyword set "italic
    // bold". The reverse mapping tags each rule with the Zed key as its `name`.
    let back = VsCodeTheme::from(&zed);
    let rules = match back.token_colors {
        Some(TokenColors::Rules(rules)) => rules,
        other => panic!("expected token color rules, got {other:?}"),
    };
    let comment_rule = rules
        .iter()
        .find(|rule| rule.name.as_deref() == Some("comment"))
        .expect("a round-tripped comment rule should exist");
    assert_eq!(comment_rule.settings.font_style.as_deref(), Some("italic bold"));
}

#[test]
fn fixture_consistency() {
    // Load both VSCode and Zed versions of Rose Pine Moon
    let vscode_theme = read_vscode_fixture("vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
        .expect("fixtures should work");
    let zed_family = read_zed_fixture("zed/rose-pine-theme/themes/rose-pine-moon.json").expect("fixtures should work");

    // Find the moon theme in the Zed family
    let zed_moon = zed_family
        .themes
        .iter()
        .find(|theme| theme.name.to_lowercase().contains("moon"))
        .expect("Should find moon theme in Zed fixture");

    // Both should be dark themes
    assert!(vscode_theme.is_dark_theme(), "VSCode Rose Pine Moon should be dark");
    assert!(
        matches!(zed_moon.appearance, crate::zed::Appearance::Dark),
        "Zed Rose Pine Moon should be dark"
    );

    // Names should be similar (allowing for formatting differences)
    let vscode_name_lower = vscode_theme.name.to_lowercase();
    let zed_name_lower = zed_moon.name.to_lowercase();
    assert!(
        (vscode_name_lower.contains("rose") || vscode_name_lower.contains("rosé"))
            && vscode_name_lower.contains("pine"),
        "VSCode theme name should contain 'rose/rosé pine'"
    );
    assert!(
        (zed_name_lower.contains("rose") || zed_name_lower.contains("rosé")) && zed_name_lower.contains("pine"),
        "Zed theme name should contain 'rose/rosé pine'"
    );
}

#[test]
fn extension_serialization() {
    // Load VSCode theme and convert to Zed
    let vscode_theme = read_vscode_fixture("vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
        .expect("fixtures should work");
    let zed_extension = ZedExtension::from(&vscode_theme);

    // Test that the extension has the right structure
    assert!(!zed_extension.name().is_empty(), "Extension should have a name");
    assert_eq!(
        zed_extension.manifest().themes().len(),
        1,
        "Should have exactly one theme reference"
    );

    // Test the theme file content
    let themes = zed_extension.themes();
    let zed_theme = &themes[0];

    // Serialize the theme to check it's valid JSON
    let theme_json = serde_json::to_string_pretty(zed_theme).expect("fixtures should work");
    assert!(!theme_json.is_empty(), "Theme should serialize to non-empty JSON");

    // Verify we can deserialize it back
    let _deserialized: ZedTheme = serde_json::from_str(&theme_json).expect("fixtures should work");
}

#[test]
fn vscode_to_zed_file_writing() -> Result<(), ThemeError> {
    let temp_dir = create_temp_dir("vscode_to_zed_write").expect("Failed to create temp directory");

    let original_vscode = VsCodeExtension::read_from_path(
        "fixtures/vscode/mvllow.rose-pine-2.14.0/package.json".into(),
        "unused".to_string(),
    )
    .expect("test fixtures should be readable");
    let _theme_count = original_vscode.manifest().themes().len();
    let zed_extension = ZedExtension::from(original_vscode.clone());

    // Debug: Check icon theme conversion
    eprintln!(
        "Original VSCode extension icon themes: {}",
        original_vscode.icon_themes().len()
    );
    for (i, icon_theme) in original_vscode.icon_themes().iter().enumerate() {
        eprintln!("  VSCode icon theme {}: source_path = {:?}", i, icon_theme.source_path);
    }

    eprintln!(
        "Converted Zed extension icon themes: {}",
        zed_extension.icon_themes().len()
    );
    for (i, icon_theme) in zed_extension.icon_themes().iter().enumerate() {
        eprintln!("  Zed icon theme {}: name = {}", i, icon_theme.name);
    }

    // Step 3: Check Zed theme briefly
    assert_eq!(
        zed_extension.name(),
        original_vscode.name(),
        "Zed theme name should match original"
    );
    assert_eq!(
        zed_extension.families().len(),
        3,
        "we expected 3 families for rose pine"
    );
    assert_eq!(
        zed_extension.manifest().icon_themes().len(),
        1,
        "we expected one icon theme"
    );

    let extension_name = format!("rose-pine-moon-zed-{}", std::process::id());
    let mut extension_dir = temp_dir.clone();
    extension_dir.push(&extension_name);
    fs::create_dir_all(&extension_dir).expect("Failed to create tmp extensions directory");

    // Write the zed extension out.
    eprintln!("Writing Zed extension to: {}", extension_dir.display());
    zed_extension
        .write_to(&extension_dir)
        .expect("we should be able to write the converted zed extension");

    // Check that the files we expect to see all exist.
    let mut filetest = extension_dir.clone();
    filetest.push("extension.toml");
    eprintln!("{}", filetest.display());
    assert!(std::fs::exists(&filetest)?, "expected extension.toml to exist");
    filetest.pop();
    filetest.push("themes/rose-pine.json");
    assert!(std::fs::exists(&filetest)?, "rose-pine.json family should exist");
    filetest.pop();
    filetest.push("rose-pine-dawn.json");
    assert!(std::fs::exists(&filetest)?, "rose-pine-dawn.json family should exist");
    filetest.pop();
    filetest.push("rose-pine-moon.json");
    assert!(std::fs::exists(&filetest)?, "rose-pine-moon.json family should exist");
    filetest.pop();
    filetest.pop();

    // Now we test that the icons got written out.
    filetest.push("icon_themes/rose-pine-icons.json");
    assert!(std::fs::exists(&filetest)?, "rose-pine-icons.json should exist");

    cleanup_temp_dir(&temp_dir).expect("Failed to cleanup temp directory");
    Ok(())
}

#[test]
fn zed_to_vscode_file_writing() {
    let temp_dir = create_temp_dir("zed_to_vscode").expect("Failed to create temp directory");

    // Load Zed theme and convert to VSCode
    let zed_family = read_zed_fixture("zed/rose-pine-theme/themes/rose-pine-moon.json").expect("fixtures should work");
    let vscode_extension = VsCodeExtension::from(zed_family);

    // Create extension directory
    let extension_name = format!("rose-pine-moon-vscode-{}", std::process::id());
    let mut extension_dir = temp_dir.clone();
    extension_dir.push(&extension_name);
    fs::create_dir_all(&extension_dir).expect("Failed to create extension directory");

    // Create themes directory
    let mut themes_dir = extension_dir.clone();
    themes_dir.push("themes");
    fs::create_dir_all(&themes_dir).expect("Failed to create themes directory");

    // Get metadata before consuming the extension
    let metadata = vscode_extension.manifest().clone();
    // Write theme files
    let themes = vscode_extension.themes();
    for theme in themes {
        let mut theme_file = themes_dir.clone();
        theme_file.push(format!("{}.json", slug::slugify(&theme.name)));
        let theme_json = serde_json::to_string_pretty(theme).expect("theme should be serializable");
        fs::write(&theme_file, theme_json).expect("Failed to write theme file");

        // Verify the file exists and can be read back
        assert!(theme_file.exists(), "Theme file should be created");
        let read_back_theme = VsCodeTheme::read(&theme_file).expect("should be able to read a theme file we wrote");
        assert_eq!(read_back_theme.name, theme.name, "Read back theme name should match");
    }

    // Write package.json
    let mut package_file = extension_dir.clone();
    package_file.push("package.json");
    let package_json = serde_json::to_string_pretty(&metadata).expect("manifest should be serializable");
    fs::write(&package_file, package_json).expect("Failed to write package.json");

    // Verify package.json exists
    assert!(package_file.exists(), "Package.json should be created");

    // Clean up
    cleanup_temp_dir(&temp_dir).expect("Failed to cleanup temp directory");
}

#[test]
fn full_round_trip_with_files() {
    let temp_dir = create_temp_dir("full_round_trip").expect("Failed to create temp directory");

    // Step 1: Load original VSCode theme
    let original_vscode = VsCodeExtension::read_from_path(
        "fixtures/vscode/mvllow.rose-pine-2.14.0/package.json".into(),
        "unused".to_string(),
    )
    .expect("test fixtures should be readable");
    let theme_count = original_vscode.manifest().themes().len();
    // Step 2: Convert to Zed and write to temp directory
    let zed_extension = ZedExtension::from(original_vscode.clone());

    // Step 3: Check Zed theme briefly
    assert_eq!(
        zed_extension.name(),
        original_vscode.name(),
        "Zed theme name should match original"
    );

    let extension_name = format!("rose-pine-moon-zed-{}", std::process::id());
    let mut extension_dir = temp_dir.clone();
    extension_dir.push(&extension_name);
    fs::create_dir_all(&extension_dir).expect("Failed to create tmp extensions directory");

    // Step 4: Convert back to VSCode and write to temp directory
    let mut round_trip_vscode = VsCodeExtension::from(zed_extension);
    assert_eq!(
        theme_count,
        round_trip_vscode.manifest().themes().len(),
        "roundtrip back from zed lost themes!"
    );
    eprintln!("extdir={}", extension_dir.display());
    round_trip_vscode.directory = extension_dir.clone();
    round_trip_vscode
        .write()
        .expect("should be able to write out VSCode extension");

    // Step 5: Read VSCode theme back from directory
    let mut read_dir = extension_dir.clone();
    read_dir.push("package.json");
    let read_vscode_theme = VsCodeExtension::read_from_path(read_dir, "unused".to_string())
        .expect("we should be able to read the package.json we just wrote");
    assert_eq!(
        read_vscode_theme.name(),
        original_vscode.name(),
        "Round trip VSCode theme name should match original"
    );
    assert_eq!(
        read_vscode_theme.manifest().themes().len(),
        original_vscode.manifest().themes().len(),
        "Theme metadata should be preserved through full round trip"
    );
    assert_eq!(
        theme_count,
        read_vscode_theme.themes().len(),
        "Should read the same number of themes that we had originally"
    );

    // Clean up
    cleanup_temp_dir(&temp_dir).expect("Failed to cleanup temp directory");
}

#[test]
fn metadata_preservation() {
    // Load VSCode theme
    let vscode_theme = read_vscode_fixture("vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
        .expect("fixtures should work");

    // Convert to Zed extension
    let zed_extension = ZedExtension::from(&vscode_theme);

    // Check that metadata is properly constructed
    let metadata = zed_extension.manifest();
    assert!(!metadata.name().is_empty(), "Extension should have a name");
    assert!(!metadata.id().is_empty(), "Extension should have an ID");
    assert!(!metadata.themes().is_empty(), "Extension should list its themes");

    // Convert back to VSCode
    let vscode_extension = VsCodeExtension::from(zed_extension);
    let vs_metadata = vscode_extension.manifest();

    assert!(!vs_metadata.name().is_empty(), "VSCode extension should have a name");
    assert!(
        !vs_metadata.display_name().is_empty(),
        "VSCode extension should have a display name"
    );
    assert!(
        !vs_metadata.themes().is_empty(),
        "VSCode extension should contribute themes"
    );
}

/// Helper to compare theme colors (allowing for small differences in color
/// conversion)
fn colors_approximately_equal(color1: &str, color2: &str) -> bool {
    // Remove # if present and compare
    let c1 = color1.trim_start_matches('#').to_lowercase();
    let c2 = color2.trim_start_matches('#').to_lowercase();
    c1 == c2
}

#[test]
fn specific_color_preservation() {
    // Load VSCode theme
    let vscode_theme = read_vscode_fixture("vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")
        .expect("fixtures should work");

    // Extract some key colors before conversion
    let original_bg = vscode_theme
        .colors
        .as_ref()
        .and_then(|colors| colors.get("editor.background"))
        .cloned();

    // Convert to Zed and back
    let zed_extension = ZedExtension::from(&vscode_theme);
    let zed_themes = zed_extension.themes();
    let round_trip_vscode = VsCodeTheme::from(&zed_themes[0]);

    // Check that the background color survived the round trip
    if let Some(original_bg) = original_bg {
        let round_trip_bg = round_trip_vscode
            .colors
            .as_ref()
            .and_then(|colors| colors.get("editor.background"));

        if let Some(rt_bg) = round_trip_bg {
            assert!(
                colors_approximately_equal(&original_bg, rt_bg),
                "Background color should be preserved: '{original_bg}' vs '{rt_bg}'"
            );
        }
    }
}

#[test]
fn theme_type_detection() -> Result<(), ThemeError> {
    // Load both light and dark fixtures if available
    let dark_theme = read_vscode_fixture("vscode/mvllow.rose-pine-2.14.0/themes/rose-pine-moon-color-theme.json")?;

    // Test dark theme detection
    assert!(dark_theme.is_dark_theme(), "Rose Pine Moon should be detected as dark");

    // Convert to Zed and verify appearance
    let zed_extension = ZedExtension::from(&dark_theme);
    let zed_themes = zed_extension.themes();
    let zed_theme = &zed_themes[0];

    assert!(
        matches!(zed_theme.appearance, crate::zed::Appearance::Dark),
        "Converted Zed theme should have dark appearance"
    );

    Ok(())
}

#[test]
fn no_ci_material_icon_theme() {
    // This theme does not have an associated color theme.
    let extpath = VsCodeExtension::build_official_path(
        "equinusocio.vsc-material-theme-icons-1.2.2",
        VsCodeExtension::extensions_path().as_str(),
    );
    let mut extdir = PathBuf::from(extpath);
    eprintln!("extdir = {}", extdir.display());
    extdir.push("package.json");
    let vsc_extension = VsCodeExtension::read_from_path(extdir, "unused".to_string())
        .expect("we expect to be able to read any extension");

    assert_eq!(vsc_extension.name(), "Material Theme Icons");
}
