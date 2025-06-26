//! Icon file management for theme conversions
//!
//! This module handles the tracking and copying of icon files during theme conversion.
//! When converting between VSCode and Zed icon themes, we need to copy the actual
//! icon files (SVG, PNG, etc.) from the source to the destination.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ThemeError;

/// Manages icon files during theme conversion
#[derive(Debug, Clone)]
pub struct IconFileManager {
    /// Maps logical icon names to their source file paths
    icon_sources: HashMap<String, PathBuf>,
    /// Base directory where source icons are located
    source_base: PathBuf,
    /// Base directory where converted icons should be placed
    dest_base: PathBuf,
    /// Subdirectory within dest_base where icons should be copied (e.g., "icons")
    dest_subdir: String,
}

impl IconFileManager {
    /// Create a new IconFileManager
    pub fn new<P: AsRef<Path>>(source_base: P, dest_base: P, dest_subdir: &str) -> Self {
        Self {
            icon_sources: HashMap::new(),
            source_base: source_base.as_ref().to_path_buf(),
            dest_base: dest_base.as_ref().to_path_buf(),
            dest_subdir: dest_subdir.to_string(),
        }
    }

    /// Track an icon file for later copying
    /// * `logical_name` - The logical name used to reference this icon
    /// * `relative_path` - Path relative to the source_base
    pub fn track_icon<P: AsRef<Path>>(&mut self, logical_name: String, relative_path: P) {
        let full_source_path = self.source_base.join(relative_path.as_ref());
        self.icon_sources.insert(logical_name, full_source_path);
    }

    /// Track an icon file with an absolute source path
    /// * `logical_name` - The logical name used to reference this icon
    /// * `absolute_path` - Absolute path to the source icon file
    pub fn track_icon_absolute<P: AsRef<Path>>(&mut self, logical_name: String, absolute_path: P) {
        self.icon_sources
            .insert(logical_name, absolute_path.as_ref().to_path_buf());
    }

    /// Get the destination path for an icon file
    /// Accepts the filename for the icon (e.g., "file.svg").
    /// Returns the full destination path where the icon should be copied.
    pub fn get_dest_path(&self, filename: &str) -> PathBuf {
        self.dest_base.join(&self.dest_subdir).join(filename)
    }

    /// Get the relative path for an icon file (for use in theme JSON)
    /// Takes the filename for the icon (e.g., "file.svg")
    /// Returns the relative path to use in theme JSON files (e.g., "./icons/file.svg")
    pub fn get_relative_path(&self, filename: &str) -> String {
        format!("./{}/{}", self.dest_subdir, filename)
    }

    /// Copy all tracked icon files to their destination
    pub fn copy_icons(&self) -> Result<(), ThemeError> {
        // Create the destination directory if it doesn't exist
        let dest_dir = self.dest_base.join(&self.dest_subdir);
        if !dest_dir.exists() {
            fs::create_dir_all(&dest_dir)?;
        }

        // Copy each tracked icon
        for (logical_name, source_path) in &self.icon_sources {
            if !source_path.exists() {
                return Err(ThemeError::IconFileNotFound(
                    logical_name.clone(),
                    source_path.display().to_string(),
                ));
            }

            let filename = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| ThemeError::InvalidIconPath(source_path.display().to_string()))?;

            let dest_path = self.get_dest_path(filename);

            // Copy the file
            fs::copy(source_path, &dest_path)?;
        }

        Ok(())
    }

    /// Get all tracked icon logical names
    pub fn tracked_icons(&self) -> Vec<String> {
        self.icon_sources.keys().cloned().collect()
    }

    /// Check if an icon is tracked
    pub fn has_icon(&self, logical_name: &str) -> bool {
        self.icon_sources.contains_key(logical_name)
    }

    /// Get the source path for a tracked icon
    pub fn get_source_path(&self, logical_name: &str) -> Option<&PathBuf> {
        self.icon_sources.get(logical_name)
    }

    /// Extract filename from a path string (handles both forward and backward slashes)
    pub fn extract_filename(path: &str) -> Option<String> {
        // Handle both forward slashes and backslashes
        let path = path.replace('\\', "/");

        // Remove leading "./" if present
        let path = if let Some(stripped) = path.strip_prefix("./") {
            stripped
        } else {
            &path
        };

        // Extract the filename
        path.split('/').next_back().map(|s| s.to_string())
    }

    /// Create a mapping of icon paths to logical names for theme conversion
    /// This is useful when converting from one format to another
    pub fn create_path_to_name_mapping(&self) -> HashMap<String, String> {
        let mut mapping = HashMap::new();

        for (logical_name, source_path) in &self.icon_sources {
            if let Some(filename) = source_path.file_name().and_then(|name| name.to_str()) {
                let relative_path = self.get_relative_path(filename);
                mapping.insert(relative_path, logical_name.clone());
            }
        }

        mapping
    }
}

/// Utility functions for icon file operations
pub mod utils {
    use std::path::Path;

    /// Check if a file is a valid icon file based on its extension
    pub fn is_icon_file<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();

        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(
                extension.to_lowercase().as_str(),
                "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico"
            )
        } else {
            false
        }
    }

    /// Get the file extension of a path
    pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Sanitize a filename for cross-platform compatibility
    pub fn sanitize_filename(filename: &str) -> String {
        // Replace invalid characters with underscores
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
        let mut sanitized = filename.to_string();

        for invalid_char in invalid_chars {
            sanitized = sanitized.replace(invalid_char, "_");
        }

        // Remove leading/trailing whitespace and dots
        sanitized
            .trim_matches(|c: char| c.is_whitespace() || c == '.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_dirs() -> (TempDir, TempDir) {
        let source_dir = TempDir::new().expect("Failed to create temp source dir");
        let dest_dir = TempDir::new().expect("Failed to create temp dest dir");

        // Create a test icon file
        let icon_content = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="blue"/></svg>"#;
        let icon_path = source_dir.path().join("test-icon.svg");
        fs::write(&icon_path, icon_content).expect("Failed to write test icon");

        (source_dir, dest_dir)
    }

    #[test]
    fn test_icon_manager_creation() {
        let (source_dir, dest_dir) = setup_test_dirs();

        let manager = IconFileManager::new(source_dir.path(), dest_dir.path(), "icons");

        assert_eq!(manager.source_base, source_dir.path());
        assert_eq!(manager.dest_base, dest_dir.path());
        assert_eq!(manager.dest_subdir, "icons");
    }

    #[test]
    fn test_track_and_copy_icon() {
        let (source_dir, dest_dir) = setup_test_dirs();

        let mut manager = IconFileManager::new(source_dir.path(), dest_dir.path(), "icons");

        // Track the test icon
        manager.track_icon("test".to_string(), "test-icon.svg");

        // Verify it's tracked
        assert!(manager.has_icon("test"));
        assert_eq!(manager.tracked_icons(), vec!["test"]);

        // Copy icons
        manager.copy_icons().expect("Failed to copy icons");

        // Verify the icon was copied
        let dest_path = dest_dir.path().join("icons/test-icon.svg");
        assert!(dest_path.exists());

        // Verify content is correct
        let content = fs::read_to_string(&dest_path).expect("Failed to read copied icon");
        assert!(content.contains("svg"));
        assert!(content.contains("blue"));
    }

    #[test]
    fn test_get_relative_path() {
        let (source_dir, dest_dir) = setup_test_dirs();

        let manager = IconFileManager::new(source_dir.path(), dest_dir.path(), "icons");

        let relative_path = manager.get_relative_path("file.svg");
        assert_eq!(relative_path, "./icons/file.svg");
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(
            IconFileManager::extract_filename("./icons/file.svg"),
            Some("file.svg".to_string())
        );
        assert_eq!(
            IconFileManager::extract_filename("icons/file.svg"),
            Some("file.svg".to_string())
        );
        assert_eq!(
            IconFileManager::extract_filename("file.svg"),
            Some("file.svg".to_string())
        );
        assert_eq!(
            IconFileManager::extract_filename("./folder/subfolder/icon.png"),
            Some("icon.png".to_string())
        );
        assert_eq!(
            IconFileManager::extract_filename("C:\\icons\\file.svg"),
            Some("file.svg".to_string())
        );
    }

    #[test]
    fn test_utils_is_icon_file() {
        assert!(utils::is_icon_file("test.svg"));
        assert!(utils::is_icon_file("test.png"));
        assert!(utils::is_icon_file("test.jpg"));
        assert!(utils::is_icon_file("test.jpeg"));
        assert!(utils::is_icon_file("test.gif"));
        assert!(utils::is_icon_file("test.webp"));
        assert!(utils::is_icon_file("test.ico"));
        assert!(utils::is_icon_file("TEST.SVG")); // Case insensitive

        assert!(!utils::is_icon_file("test.txt"));
        assert!(!utils::is_icon_file("test.json"));
        assert!(!utils::is_icon_file("test"));
    }

    #[test]
    fn test_utils_sanitize_filename() {
        assert_eq!(utils::sanitize_filename("normal-file.svg"), "normal-file.svg");
        assert_eq!(
            utils::sanitize_filename("file<with>bad:chars.svg"),
            "file_with_bad_chars.svg"
        );
        assert_eq!(utils::sanitize_filename("  .file.svg.  "), "file.svg");
        assert_eq!(
            utils::sanitize_filename("file/with\\slashes.svg"),
            "file_with_slashes.svg"
        );
    }

    #[test]
    fn test_path_to_name_mapping() {
        let (source_dir, dest_dir) = setup_test_dirs();

        let mut manager = IconFileManager::new(source_dir.path(), dest_dir.path(), "icons");

        manager.track_icon("test".to_string(), "test-icon.svg");

        let mapping = manager.create_path_to_name_mapping();
        assert_eq!(mapping.get("./icons/test-icon.svg"), Some(&"test".to_string()));
    }
}
