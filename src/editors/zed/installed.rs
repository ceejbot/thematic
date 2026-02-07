//! Zed keeps a list of installed extensions. When we port a theme,
//! we need to add it to this list so it becomes available in Zed to
//! be used.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{Extension, ThemeFile, ZedExtension, ZedManifest};

/// This is the index.json file in the Zed extensions directory. It contains a
/// list of all the installed extensions Zed knows about, along with a directory
/// of extensions sorted by type. Newly-converted themes need to be added here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledExtensions {
    /// map of extension manifests
    extensions: HashMap<String, ExtensionWrapper>,
    /// map of color theme pointers
    themes: HashMap<String, ZedThemeIndex>,
    /// map of icon theme pointers
    icon_themes: HashMap<String, ZedThemeIndex>,
    /// We need to preserve entries for all other extension types, but we do not
    /// inspect them. Using flatten to capture languages, grammars,
    /// snippets, context_servers, slash_commands, language_servers, and any
    /// future extension types Zed adds.
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtensionWrapper {
    manifest: ZedManifest,
    dev: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ZedThemeIndex {
    /// The short slug name of the extension (id)
    extension: String,
    /// The extension path, in the form "themes/foo.json"
    path: String,
}

impl InstalledExtensions {
    pub fn add_extension(&mut self, extension: &ZedExtension) {
        let id = extension.manifest().id().to_string();

        // Do the manifest
        let manifest = extension.manifest().clone();
        let wrapper = ExtensionWrapper { manifest, dev: true };
        self.extensions.insert(id.clone(), wrapper);

        // Generate theme pointers from the extension's theme families
        let mut theme_ptrs: HashMap<String, ZedThemeIndex> = HashMap::new();
        for family in extension.families() {
            for theme in &family.themes {
                // Find the corresponding theme file path from the manifest
                if let Some(theme_path) = extension
                    .manifest()
                    .themes()
                    .iter()
                    .find(|path| path.contains(&slug::slugify(&family.name)))
                {
                    theme_ptrs.insert(
                        theme.name.clone(),
                        ZedThemeIndex {
                            extension: id.clone(),
                            path: theme_path.clone(),
                        },
                    );
                }
            }
        }
        self.themes.extend(theme_ptrs);

        // Generate icon theme pointers from the extension's icon themes
        let mut icon_theme_ptrs: HashMap<String, ZedThemeIndex> = HashMap::new();
        for icon_theme in extension.icon_themes() {
            // Find the corresponding icon theme file path from the manifest
            if let Some(icon_path) = extension
                .manifest()
                .icon_themes()
                .iter()
                .find(|path| path.contains(&slug::slugify(&icon_theme.name)))
            {
                icon_theme_ptrs.insert(
                    icon_theme.name.clone(),
                    ZedThemeIndex {
                        extension: id.clone(),
                        path: icon_path.clone(),
                    },
                );
            }
        }
        self.icon_themes.extend(icon_theme_ptrs);
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, crate::ThemeError> {
        let mut fpath = PathBuf::new();
        fpath.push(path);
        if fpath.ends_with("installed") {
            fpath.pop();
        }
        if fpath.ends_with("extensions") {
            fpath.push("index.json");
        }

        InstalledExtensions::read(fpath)
    }
}

impl ThemeFile for InstalledExtensions {
    type T = InstalledExtensions;

    fn read<P: AsRef<std::path::Path>>(path: P) -> Result<Self::T, crate::ThemeError> {
        let contents = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str::<InstalledExtensions>(contents.as_str())?)
    }

    fn write_to<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), crate::ThemeError> {
        let bytes = serde_json::to_vec_pretty(self)?;
        Ok(std::fs::write(path, bytes)?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, crate::ThemeError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_fixture() {
        let fixture =
            InstalledExtensions::new("fixtures/zed/index.json").expect("we expect to be able to read the fixture");
        assert_eq!(fixture.extensions.len(), 19);
    }

    #[test]
    fn can_roundtrip_fixture() {
        let fixture =
            InstalledExtensions::new("fixtures/zed/index.json").expect("we expect to be able to read the fixture");

        let temp_path = std::env::temp_dir().join("test_index.json");
        fixture.write_to(&temp_path).expect("should be able to write fixture");

        let reloaded = InstalledExtensions::read(&temp_path).expect("should be able to read written fixture");

        assert_eq!(fixture.extensions.len(), reloaded.extensions.len());
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_add_extension() {
        // Test that the InstalledExtensions struct can be created and basic operations
        // work
        let installed = InstalledExtensions {
            extensions: HashMap::new(),
            themes: HashMap::new(),
            icon_themes: HashMap::new(),
            other: HashMap::new(),
        };

        // Verify initial state
        assert_eq!(installed.extensions.len(), 0);
        assert_eq!(installed.themes.len(), 0);
        assert_eq!(installed.icon_themes.len(), 0);
    }
}
