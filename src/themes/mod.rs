pub mod vsc_extension;
pub mod vscode;
pub mod zed;
pub mod zed_extension;

use std::path::{Path, PathBuf};

pub use vsc_extension::*;
pub use vscode::*;
pub use zed::*;
pub use zed_extension::*;

use crate::ThemeError;

pub trait Extension {
    type ThemeType;

    /// Attempt to find and read the theme from its name.
    fn read(name: &str) -> Result<Box<Self>, ThemeError>;
    /// Write out a minimum viable theme extension for this editor.
    fn write(&self) -> Result<(), ThemeError>;
    /// The directory this theme belongs in.
    fn directory(&self) -> &str;
    /// The official extensions path for this editor.
    fn extensions_path() -> String;
    /// The human name of this extension (as opposed to theme).
    fn name(&self) -> &str;
    /// Get all themes associated with this extension.
    fn themes(&self) -> &[Self::ThemeType];
    /// Construct the path this editor type expects to find this extension in, from the name only.
    fn official_path_for(name: &str, extdir: &str) -> String;
}

pub trait ThemeTr {
    type T;

    fn load<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError>;
}

#[derive(Debug, Clone)]
pub enum Theme {
    VSCode(VSCodeTheme),
    Zed(ZedThemeFamily),
    // Unknown,
}

impl Theme {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ThemeError> {
        let mut pathbuf = PathBuf::new();
        pathbuf.push(&path);
        pathbuf = ensure_json_extension(pathbuf);
        if !pathbuf.exists() {
            return Err(ThemeError::FileDoesNotExist(pathbuf.display().to_string()));
        }

        if let Ok(theme) = ZedThemeFamily::load(&pathbuf) {
            return Ok(Theme::Zed(theme));
        }
        if let Ok(theme) = VSCodeTheme::load(&pathbuf) {
            return Ok(Theme::VSCode(theme));
        }
        Err(ThemeError::UnknownThemeType)
    }
}

/// Ensure the path has a .json extension, adding it if not present
/// Returns the path to use, preferring the original if it exists
fn ensure_json_extension<P: AsRef<Path>>(input: P) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(&input);

    // If the original path exists, use it regardless of extension
    if path.exists() {
        return path;
    }

    // If it already has .json extension, return as-is
    if path.extension() == Some(std::ffi::OsStr::new("json")) {
        return path;
    }

    // Try adding .json extension
    let mut json_path = path.clone();
    if path.extension().is_none() {
        json_path.set_extension("json");
    } else {
        // If it has a different extension, append .json
        let mut os_string = path.into_os_string();
        os_string.push(".json");
        json_path = PathBuf::from(os_string);
    }

    json_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_generic() {
        let something = Theme::load("fixtures/vscode/catppuccin-latte.json").expect("we should recognize this one");
        match something {
            Theme::VSCode(theme) => {
                assert_eq!(theme.name.as_str(), "Catppuccin Latte");
            }
            Theme::Zed(_) => unreachable!(),
        }
    }

    #[test]
    fn test_ensure_json_extension_already_has_json() {
        let path = PathBuf::from("theme.json");
        let result = ensure_json_extension(path.clone());
        assert_eq!(result, path);
    }

    #[test]
    fn test_ensure_json_extension_no_extension() {
        let path = PathBuf::from("theme");
        let result = ensure_json_extension(path);
        assert_eq!(result, PathBuf::from("theme.json"));
    }

    #[test]
    fn test_ensure_json_extension_different_extension() {
        let path = PathBuf::from("theme.txt");
        let result = ensure_json_extension(path);
        assert_eq!(result, PathBuf::from("theme.txt.json"));
    }

    #[test]
    fn test_ensure_json_extension_with_path() {
        let path = PathBuf::from("path/to/theme");
        let result = ensure_json_extension(path);
        assert_eq!(result, PathBuf::from("path/to/theme.json"));
    }

    #[test]
    fn test_ensure_json_extension_complex_path() {
        let path = PathBuf::from("../themes/my-theme");
        let result = ensure_json_extension(path);
        assert_eq!(result, PathBuf::from("../themes/my-theme.json"));
    }
}
