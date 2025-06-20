/// Library error type.
use thiserror::Error;

/// Error type for theme operations
#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Unrecognized theme in file")]
    UnknownThemeType,
    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),
}
