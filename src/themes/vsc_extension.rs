//! Read and write VSCode theme extensions.
//! We don't try to be complete shiny publishable extensions, but just enough
//! that the editor can load and use the theme.

use std::path::{Path, PathBuf};

use glob::glob;
use serde::{Deserialize, Serialize};

use crate::themes::{Extension, ThemeFile, ZedExtension};
use crate::{ThemeError, VsCodeTheme, ZedThemeFamily};

static EXTENSION_DIR: &str = ".vscode/extensions";

#[derive(Debug, Clone)]
pub struct VsCodeExtension {
    directory: String,
    name: String,
    themes: Vec<VsCodeTheme>,
    metadata: Option<VSCMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VSCMetadata {
    name: String,
    display_name: String,
    description: String,
    publisher: String,
    contributes: Vec<ThemePointer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemePointer {
    label: String,
    path: PathBuf,
}

impl VsCodeExtension {
    fn read_from_name(name: &str, extdir: &str) -> Result<Box<VsCodeExtension>, ThemeError> {
        // use globs to find a file named `name-color-theme.json` somewhere in this as a subdir
        let barename = name.replace(".json", "");
        let globby = format!("{}/**/{}*.json", extdir, name.replace(".json", ""));

        let mut matches = glob(globby.as_str())?;
        let Some(Ok(found)) = matches.find(|xs| xs.is_ok()) else {
            return Err(ThemeError::ThemeNotFound(name.to_owned()));
        };
        // read all .json files in this directory and build a list of the ones that are valid themes
        let themes = if let Some(parent) = Path::new(&found).parent() {
            std::fs::read_dir(parent)?
                .filter_map(|xs| {
                    if let Ok(e) = xs {
                        let fpath = e.path();
                        let pathstring = fpath.to_string_lossy();
                        if pathstring.ends_with(".json") && pathstring.contains(barename.as_str()) {
                            Some(e.path())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .filter_map(|fpath| VsCodeTheme::read(fpath).ok())
                .collect()
        } else {
            Vec::new()
        };

        let extension = VsCodeExtension {
            name: name.to_owned(),
            directory: found.to_string_lossy().to_string(),
            themes,
        };

        Ok(Box::new(extension))
    }

    /// Ensure the name of a theme file stored in an extension is in the form
    /// vscode expects "name-color-theme.json".
    pub fn normalize_filename(name: &str) -> String {
        let extname = name.replace(".json", "").replace("-color-theme", "");
        format!("{extname}-color-theme.json")
    }

    pub fn new(theme_name: &str, filename: &str, themes: Vec<VsCodeTheme>) -> Self {
        let directory = VsCodeExtension::official_path_for(filename, EXTENSION_DIR);
        let name = theme_name.to_owned();
        Self {
            directory,
            name,
            themes,
        }
    }

    /// Input is a path to a theme file; we decide if it's part of an extension
    pub fn dir_is_extension(fpath: &PathBuf) -> bool {
        // parent dir must exist and be named "themes"
        let Some(parent) = fpath.parent() else {
            return false;
        };
        if !parent.is_dir() || !parent.ends_with("themes") {
            return false;
        }
        // hop up one more.

        true
    }
}

impl Extension for VsCodeExtension {
    type ThemeType = VsCodeTheme;

    fn extensions_path() -> String {
        let twiddle = home::home_dir().unwrap_or_default();
        format!("{}/{}", twiddle.display(), EXTENSION_DIR)
    }

    fn read(name: &str) -> Result<Box<VsCodeExtension>, ThemeError> {
        VsCodeExtension::read_from_name(name, VsCodeExtension::extensions_path().as_str())
    }

    fn write(&self) -> Result<(), ThemeError> {
        let themedir = PathBuf::from(format!("{}/themes", self.directory));
        mkdirp::mkdirp(&themedir)?;
        for theme in self.themes() {
            let mut filename = themedir.clone();
            filename.push(theme.name.as_str());
            theme.write(filename)?;
        }
        // write anything else required for the MVP extension
        Ok(())
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn directory(&self) -> &str {
        self.directory.as_str()
    }

    fn official_path_for(name: &str, extdir: &str) -> String {
        let extname = VsCodeExtension::normalize_filename(name);
        let subdir = extname.replace("-color-theme.json", "");
        format!("{extdir}/{subdir}/{extname}")
    }

    fn themes(&self) -> &[VsCodeTheme] {
        self.themes.as_slice()
    }
}

impl From<&ZedExtension> for VsCodeExtension {
    fn from(input: &ZedExtension) -> Self {
        let directory = VsCodeExtension::official_path_for(input.name(), VsCodeExtension::extensions_path().as_str());
        let themes = input.themes().iter().map(VsCodeTheme::from).collect();

        VsCodeExtension {
            directory,
            name: input.name().to_owned(),
            themes,
        }
    }
}

impl From<&ZedThemeFamily> for VsCodeExtension {
    fn from(family: &ZedThemeFamily) -> Self {
        let themes: Vec<VsCodeTheme> = family.themes.iter().map(|xs| xs.into()).collect();
        // well, if we have a filename, we should use it.
        let theme_filename = slug::slugify(family.name.as_str());
        VsCodeExtension::new(family.name.as_str(), theme_filename.as_str(), themes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizing_names() {
        assert_eq!(
            "input-color-theme.json",
            VsCodeExtension::normalize_filename("input-color-theme.json")
        );
        assert_eq!(
            "input-color-theme.json",
            VsCodeExtension::normalize_filename("input.json")
        );
        assert_eq!(
            "input-color-theme.json",
            VsCodeExtension::normalize_filename("input-color-theme")
        );
        assert_eq!("input-color-theme.json", VsCodeExtension::normalize_filename("input"));
    }

    #[test]
    fn reading_by_name_works() {
        let fixtures = format!("{}/fixtures/vscode", env!("CARGO_MANIFEST_DIR"));
        let found = VsCodeExtension::read_from_name("rose-pine-moon", fixtures.as_str())
            .expect("failed to find Rosé Pine Moon");
        assert_eq!(found.name, "rose-pine-moon");
        assert_eq!(found.themes.len(), 1);
    }

    #[test]
    fn wont_work_in_ci() {
        let found = VsCodeExtension::read_from_name("bluloco-light", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Bluloco Light");
        assert_eq!(found.name, "bluloco-light");
        assert_eq!(found.themes.len(), 2);
    }
}
