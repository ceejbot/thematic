//! Structures and traits for Zed icon themes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::editors::ThemeFile;
use crate::{ThemeError, vscode::VsCodeIconTheme};

/// The schema for Zed icon themes is here:
/// "$schema": "https://zed.dev/schema/icon_themes/v0.2.0.json
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZedIconThemeFamily {
    /// JSON schema reference
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    /// Human-readable name of the icon theme family
    pub name: String,
    /// Author information
    pub author: String,
    /// List of individual themes in this family
    pub themes: Vec<ZedIconTheme>,

    /// For tracking source paths during conversion (not serialized)
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

impl ThemeFile for ZedIconThemeFamily {
    type T = ZedIconThemeFamily;

    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError> {
        let content = std::fs::read_to_string(&path)?;
        let mut theme_family: ZedIconThemeFamily = serde_json::from_str(&content)?;
        theme_family.source_path = Some(path.as_ref().to_path_buf());
        Ok(theme_family)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError> {
        Ok(serde_json::from_slice::<ZedIconThemeFamily>(bytes)?)
    }
}

impl ZedIconThemeFamily {
    pub fn write<P: AsRef<Path>>(&self, destination: P) -> Result<(), ThemeError> {
        ThemeFile::write(self, destination)
    }
}

/// A single icon theme within a theme family
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZedIconTheme {
    /// Name of this specific theme
    pub name: String,
    /// Theme appearance: "light" or "dark"
    pub appearance: String,
    /// Directory/folder icon configuration
    #[serde(rename = "directory_icons")]
    pub directory_icons: Option<DirectoryIcons>,
    /// Mapping of specific file names (stems) to icon types
    #[serde(rename = "file_stems")]
    pub file_stems: Option<HashMap<String, String>>,
    /// Mapping of file extensions to icon types
    #[serde(rename = "file_suffixes")]
    pub file_suffixes: Option<HashMap<String, String>>,
    /// Definition of icon types and their associated files
    #[serde(rename = "file_icons")]
    pub file_icons: Option<HashMap<String, FileIcon>>,
}

/// Directory icon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryIcons {
    /// Icon for collapsed/closed directories
    pub collapsed: String,
    /// Icon for expanded/open directories
    pub expanded: String,
}

/// File icon definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIcon {
    /// Path to the icon file
    pub path: String,
}

impl ZedIconTheme {
    pub fn write<P: AsRef<Path>>(&self, destination: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(destination, content)?;
        Ok(())
    }
}

impl From<VsCodeIconTheme> for ZedIconTheme {
    fn from(_value: VsCodeIconTheme) -> Self {
        // TODO fill in conversion
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Extension, ZedExtension};

    use super::*;

    #[test]
    fn can_read_rose_pine_icons() {
        let data = std::fs::read_to_string("fixtures/zed/serendipity/icon_themes/serendipity-icon-theme.json")
            .expect("We expect to be able to read a test fixture");
        let icon_theme: ZedIconThemeFamily =
            serde_json::from_str(data.as_str()).expect("We expect to be able to parse the icon theme json file.");
        assert_eq!(icon_theme.name, "Serendipity Icons");
        assert_eq!(icon_theme.author, "Nguyen Dang Vinh <meocoder@gmail.com>");
        assert_eq!(icon_theme.themes.len(), 1);
        // Validate theme family metadata
        assert_eq!(icon_theme.name, "Serendipity Icons");
        assert_eq!(icon_theme.author, "Nguyen Dang Vinh <meocoder@gmail.com>");
        assert_eq!(icon_theme.themes.len(), 1);

        // Validate the individual theme
        let theme = &icon_theme.themes[0];
        assert_eq!(theme.name, "Serendipity Icons");
        assert_eq!(theme.appearance, "dark");

        // Validate directory icons
        let directory_icons = theme
            .directory_icons
            .as_ref()
            .expect("Theme should have directory icons");
        assert_eq!(directory_icons.collapsed, "./icons/folder.svg");
        assert_eq!(directory_icons.expanded, "./icons/folder-open.svg");

        // Validate file icons
        let file_icons = theme.file_icons.as_ref().expect("Theme should have file icons");
        assert!(file_icons.contains_key("special"));
        assert!(file_icons.contains_key("code"));
        assert!(file_icons.contains_key("default"));
        assert_eq!(file_icons.get("special").unwrap().path, "./icons/file-special.svg");

        // Validate file stems
        let file_stems = theme.file_stems.as_ref().expect("Theme should have file stems");
        assert!(file_stems.contains_key("README"));
        assert!(file_stems.contains_key("package.json"));
        assert_eq!(file_stems.get("README").unwrap(), "special");

        // Validate file suffixes
        let file_suffixes = theme.file_suffixes.as_ref().expect("Theme should have file suffixes");
        assert!(file_suffixes.contains_key("js"));
        assert!(file_suffixes.contains_key("rs"));
        assert_eq!(file_suffixes.get("js").unwrap(), "code");
    }

    #[test]
    fn no_ci_can_read_large_theme() {
        const THEME_PATH: &str = "colored-zed-icons-theme/icon_themes/colored-zed-icons-theme.json";
        let extpath = ZedExtension::build_official_path(THEME_PATH, ZedExtension::extensions_path().as_str());
        let data = std::fs::read_to_string(extpath).expect("large theme text fixture should exist");
        let icon_theme: ZedIconThemeFamily =
            serde_json::from_str(data.as_str()).expect("We expect to be able to parse the icon theme json file.");
        assert_eq!(icon_theme.name, "Colored Zed Icons Theme");
        assert_eq!(icon_theme.author, "TheRedXD");
        assert_eq!(icon_theme.themes.len(), 2);
        // Validate theme family metadata
        assert_eq!(icon_theme.name, "Colored Zed Icons Theme");
        assert_eq!(icon_theme.author, "TheRedXD");
        assert_eq!(icon_theme.themes.len(), 2);

        // Validate first theme
        let first_theme = &icon_theme.themes[0];
        assert!(!first_theme.name.is_empty());
        assert!(!first_theme.appearance.is_empty());

        // Validate second theme
        let second_theme = &icon_theme.themes[1];
        assert!(!second_theme.name.is_empty());
        assert!(!second_theme.appearance.is_empty());

        // Ensure themes have different appearances or names
        assert!(
            first_theme.name != second_theme.name || first_theme.appearance != second_theme.appearance,
            "Themes should be distinct"
        );
    }

    #[test]
    fn can_write_zed_icon_theme() {
        // Create test file icons
        let mut file_icons = HashMap::new();
        file_icons.insert(
            "default".to_string(),
            FileIcon {
                path: "./icons/file.svg".to_string(),
            },
        );
        file_icons.insert(
            "special".to_string(),
            FileIcon {
                path: "./icons/file-special.svg".to_string(),
            },
        );

        // Create test file suffixes
        let mut file_suffixes = HashMap::new();
        file_suffixes.insert("rs".to_string(), "default".to_string());
        file_suffixes.insert("js".to_string(), "default".to_string());

        // Create test file stems
        let mut file_stems = HashMap::new();
        file_stems.insert("README".to_string(), "special".to_string());

        // Create a test theme
        let theme = ZedIconTheme {
            name: "Test Theme".to_string(),
            appearance: "dark".to_string(),
            directory_icons: Some(DirectoryIcons {
                collapsed: "./icons/folder.svg".to_string(),
                expanded: "./icons/folder-open.svg".to_string(),
            }),
            file_stems: Some(file_stems),
            file_suffixes: Some(file_suffixes),
            file_icons: Some(file_icons),
        };

        // Create theme family
        let theme_family = ZedIconThemeFamily {
            schema: Some("https://zed.dev/schema/icon_themes/v0.2.0.json".to_string()),
            name: "Test Icon Theme".to_string(),
            author: "Test Author".to_string(),
            themes: vec![theme],
            source_path: None,
        };

        // Test serialization
        let json = serde_json::to_string_pretty(&theme_family).expect("Should be able to serialize ZedIconThemeFamily");
        assert!(json.contains("Test Icon Theme"));
        assert!(json.contains("Test Author"));
        assert!(json.contains("directory_icons"));
        assert!(json.contains("file_icons"));

        // Test deserialization
        let parsed: ZedIconThemeFamily =
            serde_json::from_str(&json).expect("Should be able to deserialize ZedIconThemeFamily");
        assert_eq!(parsed.name, "Test Icon Theme");
        assert_eq!(parsed.author, "Test Author");
        assert_eq!(parsed.themes.len(), 1);
        assert_eq!(parsed.themes[0].name, "Test Theme");
        assert_eq!(parsed.themes[0].appearance, "dark");
    }
}
