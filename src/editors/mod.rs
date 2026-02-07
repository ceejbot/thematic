pub mod vscode;
pub mod zed;

use std::path::{Path, PathBuf};

pub use vscode::*;
pub use zed::*;

use crate::ThemeError;

pub trait Extension: Sized {
    type ThemeType;
    type IconThemeType;
    type Manifest;

    /// Search for extensions matching the input pattern.
    fn search(pattern: &str) -> Result<Vec<Self>, ThemeError>;
    /// Read the theme from a path to its manifest.
    fn read<P: AsRef<Path>>(path: P) -> Result<Self, ThemeError>;
    /// Write this theme extension to the default place its editor expects it.
    fn write(&self) -> Result<(), ThemeError>;
    /// Write the theme to a specific directory.
    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError>;
    /// The official extensions path for this editor.
    fn extensions_path() -> String;
    /// The human name of this extension (as opposed to theme).
    fn name(&self) -> &str;
    /// Get the extension's manifest file, with its metadata
    fn manifest(&self) -> &Self::Manifest;
    /// Get all themes associated with this extension.
    fn themes(&self) -> &[Self::ThemeType];
    /// Get all icon themes associated with this extension.
    fn icon_themes(&self) -> &[Self::IconThemeType];
    /// Where this extension is stored, or should be stored.
    fn official_path(&self) -> &PathBuf;
    /// Construct the path this editor type expects to find this extension in,
    /// from the name only.
    fn build_official_path(name: &str, extdir: &str) -> String;
}

pub trait ThemeFile {
    type T: ThemeFile;

    /// Read this item from its normal location.
    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError>;
    /// Write this item to the given location.
    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError>;
    /// Deserialize this item from the input bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError>;
}

pub(crate) fn globdir(glob: &str, directory: &str) -> Vec<PathBuf> {
    use fast_glob::glob_match;
    use walkdir::WalkDir;

    WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let epath = entry.path();
            let ostrich = epath.as_os_str().as_encoded_bytes();
            if glob_match(glob, ostrich) {
                Some(epath.into())
            } else {
                None
            }
        })
        .collect()
}
