pub mod vscode;
pub mod zed;

use std::path::{Path, PathBuf};

pub use vscode::*;
pub use zed::*;

use crate::ThemeError;

pub trait Extension {
    type ThemeType;
    type Metadata;

    /// Attempt to find and read the theme from its name.
    fn read(name: &str) -> Result<Box<Self>, ThemeError>;
    /// Write out a minimum viable theme extension for this editor.
    fn write(&self) -> Result<(), ThemeError>;
    /// The official extensions path for this editor.
    fn extensions_path() -> String;
    /// The human name of this extension (as opposed to theme).
    fn name(&self) -> &str;
    /// Get the extension's metadata
    fn metadata(&self) -> &Self::Metadata;
    /// Get all themes associated with this extension.
    fn themes(self) -> Vec<Self::ThemeType>;
    /// Where this extension is stored, or should be stored.
    fn official_path(&self) -> &PathBuf;
    /// Construct the path this editor type expects to find this extension in, from the name only.
    fn build_official_path(name: &str, extdir: &str) -> String;
}

pub trait ThemeFile {
    type T;

    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError>;
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError>;
}
