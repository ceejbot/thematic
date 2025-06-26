//! Structures and traits for VSCode icon themes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::ThemeError;
use crate::editors::ThemeFile;
use crate::zed::ZedIconThemeFamily;

/// The structure of VSCode icon theme json files
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsCodeIconTheme {
    /// Icon definitions mapping icon keys to their properties
    #[serde(rename = "iconDefinitions")]
    pub icon_definitions: Option<HashMap<String, IconDefinition>>,
    /// Default file icon key
    pub file: Option<String>,
    /// Default folder icon key
    pub folder: Option<String>,
    /// Default expanded folder icon key
    #[serde(rename = "folderExpanded")]
    pub folder_expanded: Option<String>,
    /// Root folder icon key
    #[serde(rename = "rootFolder")]
    pub root_folder: Option<String>,
    /// Root expanded folder icon key
    #[serde(rename = "rootFolderExpanded")]
    pub root_folder_expanded: Option<String>,
    /// Mapping of specific file names to icon keys
    #[serde(rename = "fileNames")]
    pub file_names: Option<HashMap<String, String>>,
    /// Mapping of file extensions to icon keys
    #[serde(rename = "fileExtensions")]
    pub file_extensions: Option<HashMap<String, String>>,
    /// Mapping of folder names to icon keys
    #[serde(rename = "folderNames")]
    pub folder_names: Option<HashMap<String, String>>,
    /// Mapping of expanded folder names to icon keys
    #[serde(rename = "folderNamesExpanded")]
    pub folder_names_expanded: Option<HashMap<String, String>>,
    /// Language-specific icon mappings
    #[serde(rename = "languageIds")]
    pub language_ids: Option<HashMap<String, String>>,
    /// Whether to hide explorer arrows
    #[serde(rename = "hidesExplorerArrows")]
    pub hides_explorer_arrows: Option<bool>,
    /// Whether to show folder arrows
    #[serde(rename = "showLanguageModeIcons")]
    pub show_language_mode_icons: Option<bool>,

    /// For tracking source paths during conversion (not serialized)
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

/// Definition of an individual icon
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconDefinition {
    /// Path to the icon file
    #[serde(rename = "iconPath")]
    pub icon_path: Option<String>,
    /// Font color for font-based icons
    #[serde(rename = "fontColor")]
    pub font_color: Option<String>,
    /// Font size for font-based icons
    #[serde(rename = "fontSize")]
    pub font_size: Option<String>,
    /// Font character for font-based icons
    #[serde(rename = "fontCharacter")]
    pub font_character: Option<String>,
    /// Font ID for font-based icons
    #[serde(rename = "fontId")]
    pub font_id: Option<String>,
}

impl ThemeFile for VsCodeIconTheme {
    type T = VsCodeIconTheme;

    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError> {
        let content = fs::read_to_string(&path)?;
        let mut theme: VsCodeIconTheme = serde_json::from_str(&content)?;
        theme.source_path = Some(path.as_ref().to_path_buf());
        Ok(theme)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError> {
        Ok(serde_json::from_slice::<VsCodeIconTheme>(bytes)?)
    }
}

impl From<ZedIconThemeFamily> for Vec<VsCodeIconTheme> {
    fn from(_value: ZedIconThemeFamily) -> Self {
        // TODO fill in conversion
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_rose_pine_icons() {
        let data = std::fs::read_to_string("fixtures/vscode/mvllow.rose-pine-2.14.0/icons/rose-pine-icon-theme.json")
            .expect("We expect to be able to read a test fixture");
        let _icon_theme: VsCodeIconTheme =
            serde_json::from_str(data.as_str()).expect("We expect to be able to parse the icon theme json file.");

        // Validate icon definitions
        let icon_definitions = _icon_theme
            .icon_definitions
            .as_ref()
            .expect("Rose Pine theme should have icon definitions");
        assert!(icon_definitions.contains_key("_file"));
        assert!(icon_definitions.contains_key("_file-special"));
        assert!(icon_definitions.contains_key("_folder"));
        assert!(icon_definitions.contains_key("_folder-open"));

        // Validate default icons
        assert_eq!(_icon_theme.file.as_ref().unwrap(), "_file");
        assert_eq!(_icon_theme.folder.as_ref().unwrap(), "_folder");
        assert_eq!(_icon_theme.folder_expanded.as_ref().unwrap(), "_folder-open");

        // Validate file extensions mapping
        let file_extensions = _icon_theme
            .file_extensions
            .as_ref()
            .expect("Rose Pine theme should have file extensions");
        assert!(file_extensions.contains_key("css"));
        assert!(file_extensions.contains_key("astro"));

        // Validate file names mapping
        let file_names = _icon_theme
            .file_names
            .as_ref()
            .expect("Rose Pine theme should have file names");
        assert!(file_names.contains_key("_pinecone-color-theme.json"));
        assert_eq!(file_names.get("_pinecone-color-theme.json").unwrap(), "_file-special");

        // Validate explorer arrows setting
        assert_eq!(_icon_theme.hides_explorer_arrows, Some(true));
    }

    #[test]
    fn can_write_vscode_icon_theme() {
        // Create a test icon theme
        let mut icon_definitions = HashMap::new();
        icon_definitions.insert(
            "_file".to_string(),
            IconDefinition {
                icon_path: Some("./file.svg".to_string()),
                font_color: None,
                font_size: None,
                font_character: None,
                font_id: None,
            },
        );

        let theme = VsCodeIconTheme {
            icon_definitions: Some(icon_definitions),
            file: Some("_file".to_string()),
            folder: None,
            folder_expanded: None,
            root_folder: None,
            root_folder_expanded: None,
            file_names: None,
            file_extensions: None,
            folder_names: None,
            folder_names_expanded: None,
            language_ids: None,
            hides_explorer_arrows: Some(false),
            show_language_mode_icons: None,
            source_path: None,
        };

        // Test serialization
        let json = serde_json::to_string_pretty(&theme).expect("Should be able to serialize VsCodeIconTheme");
        assert!(json.contains("iconDefinitions"));
        assert!(json.contains("_file"));
        assert!(json.contains("./file.svg"));

        // Test deserialization
        let parsed: VsCodeIconTheme =
            serde_json::from_str(&json).expect("Should be able to deserialize VsCodeIconTheme");
        assert_eq!(parsed.file, Some("_file".to_string()));
        assert_eq!(parsed.hides_explorer_arrows, Some(false));
    }
}
