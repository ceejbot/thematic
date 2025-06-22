//! Read and write VSCode theme extensions.
//! We don't try to be complete shiny publishable extensions, but just enough
//! that the editor can load and use the theme.

use std::path::{Path, PathBuf};

use glob::glob;

use crate::themes::{Extension, ZedExtension};
use crate::{ThemeError, VSCodeTheme};

static EXTENSION_DIR: &str = ".vscode/extensions";

pub struct VsCodeExtension {
    directory: String,
    name: String,
    themes: Vec<VSCodeTheme>,
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
            std::fs::read_dir(&parent)?
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
                .filter_map(|fpath| VSCodeTheme::load(fpath).ok())
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
}

impl Extension for VsCodeExtension {
    type ThemeType = VSCodeTheme;

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
            theme.save(filename)?;
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

    fn themes(&self) -> &[VSCodeTheme] {
        self.themes.as_slice()
    }
}

impl From<&ZedExtension> for VsCodeExtension {
    fn from(input: &ZedExtension) -> Self {
        let directory = VsCodeExtension::official_path_for(input.name(), VsCodeExtension::extensions_path().as_str());
        let themes = input.themes().iter().map(VSCodeTheme::from).collect();

        VsCodeExtension {
            directory,
            name: input.name().to_owned(),
            themes,
        }
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
