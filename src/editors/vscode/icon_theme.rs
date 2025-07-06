//! Structures and traits for VSCode icon themes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::editors::ThemeFile;
use crate::icon_files::IconFileManager;
use crate::zed::ZedIconThemeFamily;
use crate::{ThemeError, ZedIconTheme};

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

    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError> {
        Ok(serde_json::from_slice::<VsCodeIconTheme>(bytes)?)
    }
}

impl VsCodeIconTheme {
    /// Create an IconFileManager from this VSCode icon theme
    ///
    /// # Arguments
    /// * `dest_base` - Base directory where converted theme should be placed
    /// * `dest_subdir` - Subdirectory within dest_base for icons (e.g., "icons")
    pub fn create_icon_manager<P: AsRef<Path>>(
        &self,
        dest_base: P,
        dest_subdir: &str,
    ) -> Result<IconFileManager, ThemeError> {
        let source_base = self
            .source_path
            .as_ref()
            .and_then(|p| p.parent())
            .ok_or_else(|| ThemeError::IconProcessingError("No source path available".to_string()))?;

        let mut manager = IconFileManager::new(source_base, dest_base.as_ref(), dest_subdir);

        // Track icons from icon definitions
        if let Some(icon_definitions) = &self.icon_definitions {
            for (icon_key, icon_def) in icon_definitions {
                if let Some(icon_path) = &icon_def.icon_path {
                    if let Some(_filename) = IconFileManager::extract_filename(icon_path) {
                        manager.track_icon(icon_key.clone(), icon_path);
                    }
                }
            }
        }

        Ok(manager)
    }

    /// Get all icon paths referenced in this theme
    pub fn get_icon_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();

        if let Some(icon_definitions) = &self.icon_definitions {
            for icon_def in icon_definitions.values() {
                if let Some(icon_path) = &icon_def.icon_path {
                    paths.push(icon_path.clone());
                }
            }
        }

        paths
    }

    /// Update icon paths to use new base directory
    ///
    /// This is useful after copying icons to a new location
    pub fn update_icon_paths(&mut self, path_mapping: &std::collections::HashMap<String, String>) {
        if let Some(icon_definitions) = &mut self.icon_definitions {
            for icon_def in icon_definitions.values_mut() {
                if let Some(icon_path) = &mut icon_def.icon_path {
                    if let Some(new_path) = path_mapping.get(icon_path) {
                        *icon_path = new_path.clone();
                    }
                }
            }
        }
    }

    /// Track icons from this theme in the provided IconFileManager
    pub fn track_icons(&self, manager: &mut crate::IconFileManager) -> Result<(), crate::ThemeError> {
        self.track_icons_with_base(manager, None)
    }

    /// Track icons from this theme with an optional source base path
    /// This is useful during conversion when icon paths need to be resolved relative to a source directory
    pub fn track_icons_with_base(
        &self,
        manager: &mut crate::IconFileManager,
        source_base: Option<&std::path::Path>,
    ) -> Result<(), crate::ThemeError> {
        if let Some(icon_definitions) = &self.icon_definitions {
            for (icon_key, icon_def) in icon_definitions {
                if let Some(icon_path) = &icon_def.icon_path {
                    if let Some(filename) = crate::IconFileManager::extract_filename(icon_path) {
                        let logical_name = format!("{icon_key}_{filename}");
                        if let Some(base) = source_base {
                            let full_path = base.join(icon_path);
                            if full_path.exists() {
                                manager.track_icon_absolute(logical_name, full_path);
                            }
                        } else {
                            manager.track_icon(logical_name, icon_path);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl From<&ZedIconThemeFamily> for Vec<VsCodeIconTheme> {
    fn from(zed_family: &ZedIconThemeFamily) -> Self {
        zed_family.themes().iter().map(VsCodeIconTheme::from).collect()
    }
}

impl From<&ZedIconTheme> for VsCodeIconTheme {
    fn from(zed_theme: &ZedIconTheme) -> Self {
        let mut icon_definitions = HashMap::new();
        let mut file_extensions = HashMap::new();
        let mut file_names = HashMap::new();

        // Convert file icons to icon definitions
        if let Some(file_icons) = &zed_theme.file_icons {
            for (icon_type, file_icon) in file_icons {
                // Create VSCode icon key (add underscore prefix)
                let icon_key = format!("_{icon_type}");

                icon_definitions.insert(
                    icon_key.clone(),
                    IconDefinition {
                        icon_path: Some(file_icon.path.clone()),
                        font_color: None,
                        font_size: None,
                        font_character: None,
                        font_id: None,
                    },
                );
            }
        }

        // Add directory icons to icon definitions
        let (folder_key, folder_expanded_key) = if let Some(dir_icons) = &zed_theme.directory_icons {
            let folder_key = "_folder".to_string();
            let folder_expanded_key = "_folder-open".to_string();

            icon_definitions.insert(
                folder_key.clone(),
                IconDefinition {
                    icon_path: Some(dir_icons.collapsed.clone()),
                    font_color: None,
                    font_size: None,
                    font_character: None,
                    font_id: None,
                },
            );

            icon_definitions.insert(
                folder_expanded_key.clone(),
                IconDefinition {
                    icon_path: Some(dir_icons.expanded.clone()),
                    font_color: None,
                    font_size: None,
                    font_character: None,
                    font_id: None,
                },
            );

            (Some(folder_key), Some(folder_expanded_key))
        } else {
            (None, None)
        };

        // Convert file suffixes to file extensions
        if let Some(suffixes) = &zed_theme.file_suffixes {
            for (extension, icon_type) in suffixes {
                let icon_key = format!("_{icon_type}");
                if icon_definitions.contains_key(&icon_key) {
                    file_extensions.insert(extension.clone(), icon_key);
                }
            }
        }

        // Convert file stems to file names
        if let Some(stems) = &zed_theme.file_stems {
            for (filename, icon_type) in stems {
                let icon_key = format!("_{icon_type}");
                if icon_definitions.contains_key(&icon_key) {
                    file_names.insert(filename.clone(), icon_key);
                }
            }
        }

        // Get default file icon
        let default_file_icon = icon_definitions
            .keys()
            .find(|k| k.contains("file") && !k.contains("folder"))
            .cloned()
            .or_else(|| icon_definitions.keys().next().cloned());

        VsCodeIconTheme {
            icon_definitions: if icon_definitions.is_empty() {
                None
            } else {
                Some(icon_definitions)
            },
            file: default_file_icon,
            folder: folder_key,
            folder_expanded: folder_expanded_key,
            root_folder: None,
            root_folder_expanded: None,
            file_names: if file_names.is_empty() { None } else { Some(file_names) },
            file_extensions: if file_extensions.is_empty() {
                None
            } else {
                Some(file_extensions)
            },
            folder_names: None,
            folder_names_expanded: None,
            language_ids: None,
            hides_explorer_arrows: None,
            show_language_mode_icons: None,
            source_path: None,
        }
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

    #[test]
    fn can_create_icon_manager() {
        use std::fs;

        use tempfile::TempDir;

        // Create temp directories
        let source_dir = TempDir::new().expect("Failed to create temp source dir");
        let dest_dir = TempDir::new().expect("Failed to create temp dest dir");

        // Create test icon files
        let icons_dir = source_dir.path().join("icons");
        fs::create_dir_all(&icons_dir).expect("Failed to create icons dir");

        let test_svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#;
        fs::write(icons_dir.join("file.svg"), test_svg).expect("Failed to write test icon");
        fs::write(icons_dir.join("folder.svg"), test_svg).expect("Failed to write test icon");

        // Create a test theme
        let mut icon_definitions = HashMap::new();
        icon_definitions.insert(
            "_file".to_string(),
            IconDefinition {
                icon_path: Some("./icons/file.svg".to_string()),
                font_color: None,
                font_size: None,
                font_character: None,
                font_id: None,
            },
        );
        icon_definitions.insert(
            "_folder".to_string(),
            IconDefinition {
                icon_path: Some("./icons/folder.svg".to_string()),
                font_color: None,
                font_size: None,
                font_character: None,
                font_id: None,
            },
        );

        let theme = VsCodeIconTheme {
            icon_definitions: Some(icon_definitions),
            file: Some("_file".to_string()),
            folder: Some("_folder".to_string()),
            folder_expanded: None,
            root_folder: None,
            root_folder_expanded: None,
            file_names: None,
            file_extensions: None,
            folder_names: None,
            folder_names_expanded: None,
            language_ids: None,
            hides_explorer_arrows: None,
            show_language_mode_icons: None,
            source_path: Some(source_dir.path().join("theme.json")),
        };

        // Create icon manager
        let manager = theme
            .create_icon_manager(dest_dir.path(), "icons")
            .expect("Should create icon manager");

        // Verify the manager has tracked the icons
        assert!(manager.has_icon("_file"));
        assert!(manager.has_icon("_folder"));
        assert_eq!(manager.tracked_icons().len(), 2);

        // Test copying icons
        manager.copy_icons().expect("Should copy icons successfully");

        // Verify icons were copied
        assert!(dest_dir.path().join("icons/file.svg").exists());
        assert!(dest_dir.path().join("icons/folder.svg").exists());
    }

    #[test]
    fn can_get_icon_paths() {
        let mut icon_definitions = HashMap::new();
        icon_definitions.insert(
            "_file".to_string(),
            IconDefinition {
                icon_path: Some("./icons/file.svg".to_string()),
                font_color: None,
                font_size: None,
                font_character: None,
                font_id: None,
            },
        );
        icon_definitions.insert(
            "_folder".to_string(),
            IconDefinition {
                icon_path: Some("./icons/folder.svg".to_string()),
                font_color: None,
                font_size: None,
                font_character: None,
                font_id: None,
            },
        );

        let theme = VsCodeIconTheme {
            icon_definitions: Some(icon_definitions),
            file: None,
            folder: None,
            folder_expanded: None,
            root_folder: None,
            root_folder_expanded: None,
            file_names: None,
            file_extensions: None,
            folder_names: None,
            folder_names_expanded: None,
            language_ids: None,
            hides_explorer_arrows: None,
            show_language_mode_icons: None,
            source_path: None,
        };

        let paths = theme.get_icon_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"./icons/file.svg".to_string()));
        assert!(paths.contains(&"./icons/folder.svg".to_string()));
    }

    #[test]
    fn test_zed_to_vscode_conversion() {
        use std::collections::HashMap;

        use crate::zed::{DirectoryIcons, FileIcon, ZedIconTheme};

        // Create test file icons
        let mut file_icons = HashMap::new();
        file_icons.insert(
            "code".to_string(),
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
        file_suffixes.insert("js".to_string(), "code".to_string());
        file_suffixes.insert("ts".to_string(), "code".to_string());

        // Create test file stems
        let mut file_stems = HashMap::new();
        file_stems.insert("README.md".to_string(), "special".to_string());

        // Create a Zed icon theme
        let zed_theme = ZedIconTheme {
            name: "Test Zed Theme".to_string(),
            appearance: "dark".to_string(),
            directory_icons: Some(DirectoryIcons {
                collapsed: "./icons/folder.svg".to_string(),
                expanded: "./icons/folder-open.svg".to_string(),
            }),
            file_stems: Some(file_stems),
            file_suffixes: Some(file_suffixes),
            file_icons: Some(file_icons),
        };

        // Convert to VSCode theme
        let vscode_theme = VsCodeIconTheme::from(&zed_theme);

        // Verify conversion
        let icon_definitions = vscode_theme.icon_definitions.expect("Should have icon definitions");

        // Check that file icons were converted
        assert!(icon_definitions.contains_key("_code"));
        assert!(icon_definitions.contains_key("_special"));
        assert_eq!(
            icon_definitions.get("_code").unwrap().icon_path,
            Some("./icons/file.svg".to_string())
        );
        assert_eq!(
            icon_definitions.get("_special").unwrap().icon_path,
            Some("./icons/file-special.svg".to_string())
        );

        // Check directory icons
        assert!(icon_definitions.contains_key("_folder"));
        assert!(icon_definitions.contains_key("_folder-open"));
        assert_eq!(
            icon_definitions.get("_folder").unwrap().icon_path,
            Some("./icons/folder.svg".to_string())
        );
        assert_eq!(
            icon_definitions.get("_folder-open").unwrap().icon_path,
            Some("./icons/folder-open.svg".to_string())
        );

        // Check folder mappings
        assert_eq!(vscode_theme.folder, Some("_folder".to_string()));
        assert_eq!(vscode_theme.folder_expanded, Some("_folder-open".to_string()));

        // Check file extensions
        let file_extensions = vscode_theme.file_extensions.expect("Should have file extensions");
        assert_eq!(file_extensions.get("js"), Some(&"_code".to_string()));
        assert_eq!(file_extensions.get("ts"), Some(&"_code".to_string()));

        // Check file names
        let file_names = vscode_theme.file_names.expect("Should have file names");
        assert_eq!(file_names.get("README.md"), Some(&"_special".to_string()));
    }

    #[test]
    fn test_zed_family_to_vscode_themes_conversion() {
        use std::collections::HashMap;

        use crate::zed::{DirectoryIcons, FileIcon, ZedIconTheme, ZedIconThemeFamily};

        // Create first theme
        let mut file_icons_1 = HashMap::new();
        file_icons_1.insert(
            "code".to_string(),
            FileIcon {
                path: "./icons/file.svg".to_string(),
            },
        );

        let mut file_suffixes_1 = HashMap::new();
        file_suffixes_1.insert("js".to_string(), "code".to_string());

        let theme_1 = ZedIconTheme {
            name: "Dark Theme".to_string(),
            appearance: "dark".to_string(),
            directory_icons: Some(DirectoryIcons {
                collapsed: "./icons/folder.svg".to_string(),
                expanded: "./icons/folder-open.svg".to_string(),
            }),
            file_stems: None,
            file_suffixes: Some(file_suffixes_1),
            file_icons: Some(file_icons_1),
        };

        // Create second theme
        let mut file_icons_2 = HashMap::new();
        file_icons_2.insert(
            "text".to_string(),
            FileIcon {
                path: "./icons/text.svg".to_string(),
            },
        );

        let mut file_suffixes_2 = HashMap::new();
        file_suffixes_2.insert("txt".to_string(), "text".to_string());

        let theme_2 = ZedIconTheme {
            name: "Light Theme".to_string(),
            appearance: "light".to_string(),
            directory_icons: Some(DirectoryIcons {
                collapsed: "./icons/folder-light.svg".to_string(),
                expanded: "./icons/folder-open-light.svg".to_string(),
            }),
            file_stems: None,
            file_suffixes: Some(file_suffixes_2),
            file_icons: Some(file_icons_2),
        };

        // Create theme family
        let zed_family = ZedIconThemeFamily {
            schema: Some("https://zed.dev/schema/icon_themes/v0.2.0.json".to_string()),
            name: "Test Family".to_string(),
            author: "Test Author".to_string(),
            themes: vec![theme_1, theme_2],
            source_path: None,
        };

        // Convert to VSCode themes
        let vscode_themes = Vec::<VsCodeIconTheme>::from(&zed_family);

        // Should have 2 themes
        assert_eq!(vscode_themes.len(), 2);

        // Check first theme
        let theme_1_vscode = &vscode_themes[0];
        let icon_defs_1 = theme_1_vscode.icon_definitions.as_ref().unwrap();
        assert!(icon_defs_1.contains_key("_code"));
        assert!(icon_defs_1.contains_key("_folder"));

        let file_exts_1 = theme_1_vscode.file_extensions.as_ref().unwrap();
        assert_eq!(file_exts_1.get("js"), Some(&"_code".to_string()));

        // Check second theme
        let theme_2_vscode = &vscode_themes[1];
        let icon_defs_2 = theme_2_vscode.icon_definitions.as_ref().unwrap();
        assert!(icon_defs_2.contains_key("_text"));
        assert!(icon_defs_2.contains_key("_folder"));

        let file_exts_2 = theme_2_vscode.file_extensions.as_ref().unwrap();
        assert_eq!(file_exts_2.get("txt"), Some(&"_text".to_string()));
    }

    #[test]
    fn test_conversion_with_file_copying() {
        use std::collections::HashMap;
        use std::fs;

        use tempfile::TempDir;

        use crate::zed::{DirectoryIcons, FileIcon, ZedIconTheme, ZedIconThemeFamily};

        // Create temp directories for source and destination
        let source_dir = TempDir::new().expect("Failed to create temp source dir");
        let dest_dir = TempDir::new().expect("Failed to create temp dest dir");

        // Create source icon files
        let icons_dir = source_dir.path().join("icons");
        fs::create_dir_all(&icons_dir).expect("Failed to create icons dir");

        let test_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="blue"/></svg>"#;
        fs::write(icons_dir.join("file.svg"), test_svg).expect("Failed to write file icon");
        fs::write(icons_dir.join("folder.svg"), test_svg).expect("Failed to write folder icon");
        fs::write(icons_dir.join("folder-open.svg"), test_svg).expect("Failed to write folder-open icon");

        // Create a Zed theme family
        let mut file_icons = HashMap::new();
        file_icons.insert(
            "code".to_string(),
            FileIcon {
                path: "./icons/file.svg".to_string(),
            },
        );

        let mut file_suffixes = HashMap::new();
        file_suffixes.insert("rs".to_string(), "code".to_string());
        file_suffixes.insert("js".to_string(), "code".to_string());

        let zed_theme = ZedIconTheme {
            name: "Test Theme".to_string(),
            appearance: "dark".to_string(),
            directory_icons: Some(DirectoryIcons {
                collapsed: "./icons/folder.svg".to_string(),
                expanded: "./icons/folder-open.svg".to_string(),
            }),
            file_stems: None,
            file_suffixes: Some(file_suffixes),
            file_icons: Some(file_icons),
        };

        let zed_family = ZedIconThemeFamily {
            schema: None,
            name: "Test Icon Family".to_string(),
            author: "Test Author".to_string(),
            themes: vec![zed_theme],
            source_path: Some(source_dir.path().join("theme.json")),
        };

        // Create icon manager from Zed theme
        let icon_manager = zed_family
            .create_icon_manager(dest_dir.path(), "icons")
            .expect("Should create icon manager");

        // Copy icons to destination
        icon_manager.copy_icons().expect("Should copy icons successfully");

        // Convert to VSCode themes
        let vscode_themes = Vec::<VsCodeIconTheme>::from(&zed_family);
        assert_eq!(vscode_themes.len(), 1);

        // Verify the conversion worked
        let vscode_theme = &vscode_themes[0];
        let icon_definitions = vscode_theme.icon_definitions.as_ref().unwrap();

        assert!(icon_definitions.contains_key("_code"));
        assert!(icon_definitions.contains_key("_folder"));
        assert!(icon_definitions.contains_key("_folder-open"));

        // Verify files were actually copied
        assert!(dest_dir.path().join("icons/file.svg").exists());
        assert!(dest_dir.path().join("icons/folder.svg").exists());
        assert!(dest_dir.path().join("icons/folder-open.svg").exists());

        // Verify file content is correct
        let copied_content =
            fs::read_to_string(dest_dir.path().join("icons/file.svg")).expect("Should read copied file");
        assert!(copied_content.contains("svg"));
        assert!(copied_content.contains("blue"));
    }

    #[test]
    fn test_fixture_conversion() {
        // Test conversion of actual fixture data
        let data = std::fs::read_to_string("fixtures/vscode/mvllow.rose-pine-2.14.0/icons/rose-pine-icon-theme.json")
            .expect("We expect to be able to read a test fixture");
        let vscode_theme: VsCodeIconTheme =
            serde_json::from_str(data.as_str()).expect("We expect to be able to parse the icon theme json file.");

        // Convert to Zed theme
        let zed_theme = ZedIconTheme::from(&vscode_theme);

        // Verify basic conversion worked
        assert_eq!(zed_theme.name, "Converted Icon Theme");
        assert_eq!(zed_theme.appearance, "dark");

        // Should have directory icons
        let dir_icons = zed_theme.directory_icons.as_ref().expect("Should have directory icons");
        assert_eq!(dir_icons.collapsed, "./folder.svg");
        assert_eq!(dir_icons.expanded, "./folder-open.svg");

        // Should have file icons
        let file_icons = zed_theme.file_icons.as_ref().expect("Should have file icons");
        assert!(file_icons.contains_key("file"));
        assert!(file_icons.contains_key("file-special"));

        // Should have file extensions
        let file_suffixes = zed_theme.file_suffixes.as_ref().expect("Should have file suffixes");
        assert!(file_suffixes.contains_key("css"));
        assert!(file_suffixes.contains_key("astro"));

        // Should have file names
        let file_stems = zed_theme.file_stems.as_ref().expect("Should have file stems");
        assert!(file_stems.contains_key("_pinecone-color-theme.json"));

        // Test round-trip conversion back to VSCode
        let round_trip_vscode = VsCodeIconTheme::from(&zed_theme);

        // Verify we still have icon definitions
        let icon_definitions = round_trip_vscode
            .icon_definitions
            .expect("Should have icon definitions");
        assert!(icon_definitions.contains_key("_file"));
        assert!(icon_definitions.contains_key("_file-special"));
        assert!(icon_definitions.contains_key("_folder"));
        assert!(icon_definitions.contains_key("_folder-open"));

        // Verify folder mappings
        assert_eq!(round_trip_vscode.folder, Some("_folder".to_string()));
        assert_eq!(round_trip_vscode.folder_expanded, Some("_folder-open".to_string()));
    }
}
