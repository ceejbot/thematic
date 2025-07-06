//! Structures and traits for Zed icon themes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::ThemeError;
use crate::editors::ThemeFile;
use crate::icon_files::IconFileManager;
use crate::vscode::VsCodeIconTheme;

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

    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
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
        ThemeFile::write_to(self, destination)
    }

    pub fn themes(&self) -> &[ZedIconTheme] {
        self.themes.as_slice()
    }

    /// Create an IconFileManager from this Zed icon theme family
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

        // Track icons from all themes in the family
        for theme in &self.themes {
            theme.track_icons(&mut manager)?;
        }

        Ok(manager)
    }

    /// Get all icon paths referenced in this theme family
    pub fn get_icon_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for theme in &self.themes {
            paths.extend(theme.get_icon_paths());
        }
        paths
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
    pub directory_icons: Option<DirectoryIcons>,
    /// Mapping of specific file names (stems) to icon types
    pub file_stems: Option<HashMap<String, String>>,
    /// Mapping of file extensions to icon types
    pub file_suffixes: Option<HashMap<String, String>>,
    /// Definition of icon types and their associated files
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
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, ThemeError> {
        let content = std::fs::read_to_string(&path)?;
        let theme: Self = serde_json::from_str(&content)?;

        // Discover and validate referenced icon files
        let base_dir = path.as_ref().parent().unwrap_or_else(|| Path::new("."));
        let mut missing_icons = Vec::new();
        let mut found_icons = Vec::new();

        // Check directory icons
        if let Some(dir_icons) = &theme.directory_icons {
            for icon_path in [&dir_icons.collapsed, &dir_icons.expanded] {
                let full_path = base_dir.join(icon_path);
                if full_path.exists() {
                    found_icons.push(icon_path.clone());
                } else {
                    missing_icons.push(icon_path.clone());
                }
            }
        }

        // Check file icons
        if let Some(file_icons) = &theme.file_icons {
            for file_icon in file_icons.values() {
                let full_path = base_dir.join(&file_icon.path);
                if full_path.exists() {
                    found_icons.push(file_icon.path.clone());
                } else {
                    missing_icons.push(file_icon.path.clone());
                }
            }
        }

        // Log findings
        if !found_icons.is_empty() {
            log::debug!("Found {} icon files for theme '{}'", found_icons.len(), theme.name);
        }
        if !missing_icons.is_empty() {
            log::warn!(
                "Missing {} icon files for theme '{}': {:?}",
                missing_icons.len(),
                theme.name,
                missing_icons
            );
        }

        Ok(theme)
    }

    pub fn write<P: AsRef<Path>>(&self, destination: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(destination, content)?;
        Ok(())
    }

    /// Track icons from this theme in the provided IconFileManager
    pub fn track_icons(&self, manager: &mut IconFileManager) -> Result<(), ThemeError> {
        // Track directory icons with simple names for backward compatibility
        if let Some(dir_icons) = &self.directory_icons {
            if let Some(_filename) = IconFileManager::extract_filename(&dir_icons.collapsed) {
                manager.track_icon("directory_collapsed".to_string(), &dir_icons.collapsed);
            }
            if let Some(_filename) = IconFileManager::extract_filename(&dir_icons.expanded) {
                manager.track_icon("directory_expanded".to_string(), &dir_icons.expanded);
            }
        }

        // Track file icons with simple names
        if let Some(file_icons) = &self.file_icons {
            for (icon_type, file_icon) in file_icons {
                if let Some(_filename) = IconFileManager::extract_filename(&file_icon.path) {
                    manager.track_icon(icon_type.clone(), &file_icon.path);
                }
            }
        }

        Ok(())
    }

    /// Track icons from this theme with an optional source base path
    /// This is useful during conversion when icon paths need to be resolved relative to a source directory
    pub fn track_icons_with_base(
        &self,
        manager: &mut IconFileManager,
        source_base: Option<&std::path::Path>,
    ) -> Result<(), ThemeError> {
        // Track directory icons
        if let Some(dir_icons) = &self.directory_icons {
            if let Some(filename) = IconFileManager::extract_filename(&dir_icons.collapsed) {
                let logical_name = format!("directory_collapsed_{filename}");
                if let Some(base) = source_base {
                    let full_path = base.join(&dir_icons.collapsed);
                    if full_path.exists() {
                        manager.track_icon_absolute(logical_name, full_path);
                    }
                } else {
                    manager.track_icon(logical_name, &dir_icons.collapsed);
                }
            }
            if let Some(filename) = IconFileManager::extract_filename(&dir_icons.expanded) {
                let logical_name = format!("directory_expanded_{filename}");
                if let Some(base) = source_base {
                    let full_path = base.join(&dir_icons.expanded);
                    if full_path.exists() {
                        manager.track_icon_absolute(logical_name, full_path);
                    }
                } else {
                    manager.track_icon(logical_name, &dir_icons.expanded);
                }
            }
        }

        // Track file icons
        if let Some(file_icons) = &self.file_icons {
            for (icon_type, file_icon) in file_icons {
                if let Some(filename) = IconFileManager::extract_filename(&file_icon.path) {
                    let logical_name = format!("{icon_type}_{filename}");
                    if let Some(base) = source_base {
                        let full_path = base.join(&file_icon.path);
                        if full_path.exists() {
                            manager.track_icon_absolute(logical_name, full_path);
                        }
                    } else {
                        manager.track_icon(logical_name, &file_icon.path);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all icon paths referenced in this theme
    pub fn get_icon_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();

        // Add directory icons
        if let Some(dir_icons) = &self.directory_icons {
            paths.push(dir_icons.collapsed.clone());
            paths.push(dir_icons.expanded.clone());
        }

        // Add file icons
        if let Some(file_icons) = &self.file_icons {
            for file_icon in file_icons.values() {
                paths.push(file_icon.path.clone());
            }
        }

        paths
    }

    /// Update icon paths to use new base directory
    ///
    /// This is useful after copying icons to a new location
    pub fn update_icon_paths(&mut self, path_mapping: &HashMap<String, String>) {
        // Update directory icons
        if let Some(dir_icons) = &mut self.directory_icons {
            if let Some(new_path) = path_mapping.get(&dir_icons.collapsed) {
                dir_icons.collapsed = new_path.clone();
            }
            if let Some(new_path) = path_mapping.get(&dir_icons.expanded) {
                dir_icons.expanded = new_path.clone();
            }
        }

        // Update file icons
        if let Some(file_icons) = &mut self.file_icons {
            for file_icon in file_icons.values_mut() {
                if let Some(new_path) = path_mapping.get(&file_icon.path) {
                    file_icon.path = new_path.clone();
                }
            }
        }
    }
}

impl ThemeFile for ZedIconTheme {
    type T = ZedIconTheme;

    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError> {
        let content = std::fs::read_to_string(&path)?;
        let theme: ZedIconTheme = serde_json::from_str(&content)?;
        Ok(theme)
    }

    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError> {
        Ok(serde_json::from_slice::<ZedIconTheme>(bytes)?)
    }
}

impl From<VsCodeIconTheme> for ZedIconTheme {
    fn from(vscode_theme: VsCodeIconTheme) -> Self {
        ZedIconTheme::from(&vscode_theme)
    }
}

impl From<&VsCodeIconTheme> for ZedIconTheme {
    fn from(vscode_theme: &VsCodeIconTheme) -> Self {
        ZedIconTheme::from_vscode_with_name(vscode_theme, "Converted Icon Theme")
    }
}

impl ZedIconTheme {
    /// Create a ZedIconTheme from a VsCodeIconTheme with a custom name
    pub fn from_vscode_with_name(vscode_theme: &VsCodeIconTheme, name: &str) -> Self {
        let name = name.to_string();

        // Determine appearance based on common dark theme indicators
        let appearance = "dark".to_string(); // Default to dark, could be made smarter

        // Convert directory icons
        let directory_icons = if let Some(icon_definitions) = &vscode_theme.icon_definitions {
            let folder_key = vscode_theme.folder.as_deref().unwrap_or("_folder");
            let folder_expanded_key = vscode_theme.folder_expanded.as_deref().unwrap_or("_folder-open");

            let collapsed_path = icon_definitions
                .get(folder_key)
                .and_then(|def| def.icon_path.as_ref())
                .cloned()
                .unwrap_or_else(|| "./icons/folder.svg".to_string());

            let expanded_path = icon_definitions
                .get(folder_expanded_key)
                .and_then(|def| def.icon_path.as_ref())
                .cloned()
                .unwrap_or_else(|| "./icons/folder-open.svg".to_string());

            Some(DirectoryIcons {
                collapsed: collapsed_path,
                expanded: expanded_path,
            })
        } else {
            None
        };

        // Convert file extensions to file suffixes
        let file_suffixes = vscode_theme.file_extensions.clone();

        // Convert file names to file stems
        let file_stems = vscode_theme.file_names.clone();

        // Convert icon definitions to file icons
        let file_icons = if let Some(icon_definitions) = &vscode_theme.icon_definitions {
            let mut file_icons_map = HashMap::new();

            for (icon_key, icon_def) in icon_definitions {
                if let Some(icon_path) = &icon_def.icon_path {
                    // Skip directory icons as they're handled separately
                    let folder_key = vscode_theme.folder.as_deref().unwrap_or("_folder");
                    let folder_expanded_key = vscode_theme.folder_expanded.as_deref().unwrap_or("_folder-open");

                    if icon_key != folder_key && icon_key != folder_expanded_key {
                        // Convert icon key to a more generic name (remove leading underscore if present)
                        let logical_name = if let Some(stripped) = icon_key.strip_prefix('_') {
                            stripped.to_string()
                        } else {
                            icon_key.clone()
                        };

                        file_icons_map.insert(
                            logical_name,
                            FileIcon {
                                path: icon_path.clone(),
                            },
                        );
                    }
                }
            }

            if file_icons_map.is_empty() {
                None
            } else {
                Some(file_icons_map)
            }
        } else {
            None
        };

        ZedIconTheme {
            name,
            appearance,
            directory_icons,
            file_stems,
            file_suffixes,
            file_icons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Extension, ZedExtension};

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
        fs::write(icons_dir.join("folder-open.svg"), test_svg).expect("Failed to write test icon");

        // Create test file icons
        let mut file_icons = HashMap::new();
        file_icons.insert(
            "default".to_string(),
            FileIcon {
                path: "./icons/file.svg".to_string(),
            },
        );

        // Create test theme
        let theme = ZedIconTheme {
            name: "Test Theme".to_string(),
            appearance: "dark".to_string(),
            directory_icons: Some(DirectoryIcons {
                collapsed: "./icons/folder.svg".to_string(),
                expanded: "./icons/folder-open.svg".to_string(),
            }),
            file_stems: None,
            file_suffixes: None,
            file_icons: Some(file_icons),
        };

        // Create theme family
        let theme_family = ZedIconThemeFamily {
            schema: None,
            name: "Test Icon Theme".to_string(),
            author: "Test Author".to_string(),
            themes: vec![theme],
            source_path: Some(source_dir.path().join("theme.json")),
        };

        // Create icon manager
        let manager = theme_family
            .create_icon_manager(dest_dir.path(), "icons")
            .expect("Should create icon manager");

        // Verify the manager has tracked the icons
        assert!(manager.has_icon("directory_collapsed"));
        assert!(manager.has_icon("directory_expanded"));
        assert!(manager.has_icon("default"));
        assert_eq!(manager.tracked_icons().len(), 3);

        // Test copying icons
        manager.copy_icons().expect("Should copy icons successfully");

        // Verify icons were copied
        assert!(dest_dir.path().join("icons/file.svg").exists());
        assert!(dest_dir.path().join("icons/folder.svg").exists());
        assert!(dest_dir.path().join("icons/folder-open.svg").exists());
    }

    #[test]
    fn can_get_icon_paths() {
        let mut file_icons = HashMap::new();
        file_icons.insert(
            "default".to_string(),
            FileIcon {
                path: "./icons/file.svg".to_string(),
            },
        );

        let theme = ZedIconTheme {
            name: "Test Theme".to_string(),
            appearance: "dark".to_string(),
            directory_icons: Some(DirectoryIcons {
                collapsed: "./icons/folder.svg".to_string(),
                expanded: "./icons/folder-open.svg".to_string(),
            }),
            file_stems: None,
            file_suffixes: None,
            file_icons: Some(file_icons),
        };

        let paths = theme.get_icon_paths();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"./icons/file.svg".to_string()));
        assert!(paths.contains(&"./icons/folder.svg".to_string()));
        assert!(paths.contains(&"./icons/folder-open.svg".to_string()));
    }

    #[test]
    fn test_vscode_to_zed_conversion() {
        use std::collections::HashMap;

        use crate::vscode::{IconDefinition, VsCodeIconTheme};

        // Create a VSCode icon theme
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
        icon_definitions.insert(
            "_folder-open".to_string(),
            IconDefinition {
                icon_path: Some("./icons/folder-open.svg".to_string()),
                font_color: None,
                font_size: None,
                font_character: None,
                font_id: None,
            },
        );

        let mut file_extensions = HashMap::new();
        file_extensions.insert("js".to_string(), "_file".to_string());
        file_extensions.insert("ts".to_string(), "_file".to_string());

        let mut file_names = HashMap::new();
        file_names.insert("README.md".to_string(), "_file".to_string());

        let vscode_theme = VsCodeIconTheme {
            icon_definitions: Some(icon_definitions),
            file: Some("_file".to_string()),
            folder: Some("_folder".to_string()),
            folder_expanded: Some("_folder-open".to_string()),
            root_folder: None,
            root_folder_expanded: None,
            file_names: Some(file_names),
            file_extensions: Some(file_extensions),
            folder_names: None,
            folder_names_expanded: None,
            language_ids: None,
            hides_explorer_arrows: None,
            show_language_mode_icons: None,
            source_path: None,
        };

        // Convert to Zed theme
        let zed_theme = ZedIconTheme::from(&vscode_theme);

        // Verify conversion
        assert_eq!(zed_theme.name, "Converted Icon Theme");
        assert_eq!(zed_theme.appearance, "dark");

        // Check directory icons
        let dir_icons = zed_theme.directory_icons.expect("Should have directory icons");
        assert_eq!(dir_icons.collapsed, "./icons/folder.svg");
        assert_eq!(dir_icons.expanded, "./icons/folder-open.svg");

        // Check file suffixes
        let file_suffixes = zed_theme.file_suffixes.expect("Should have file suffixes");
        assert_eq!(file_suffixes.get("js"), Some(&"_file".to_string()));
        assert_eq!(file_suffixes.get("ts"), Some(&"_file".to_string()));

        // Check file stems
        let file_stems = zed_theme.file_stems.expect("Should have file stems");
        assert_eq!(file_stems.get("README.md"), Some(&"_file".to_string()));

        // Check file icons
        let file_icons = zed_theme.file_icons.expect("Should have file icons");
        assert!(file_icons.contains_key("file"));
        assert_eq!(file_icons.get("file").unwrap().path, "./icons/file.svg");
    }
}
