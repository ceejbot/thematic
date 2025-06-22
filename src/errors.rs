/// Library error type.
use thiserror::Error;

/// Error type for theme operations
#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Glob error: {0}")]
    PatternError(#[from] glob::PatternError),
    #[error("Unrecognized theme in file")]
    UnknownThemeType,
    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
    #[error("No theme files found: {0}")]
    NoThemesFound(String),
    #[error("Toml serialization: {0}")]
    TomlSerError(#[from] toml::ser::Error),
    #[error("Toml deserialization: {0}")]
    TomlDeserError(#[from] toml::de::Error),
}
