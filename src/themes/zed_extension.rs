//! Read and write Zed theme extensions.

use glob::glob;

use super::VsCodeExtension;
use crate::themes::{Extension, ThemeFile, ensure_json_extension};
use crate::{ThemeError, VsCodeTheme, ZedTheme, ZedThemeFamily};

static EXTENSION_DIR: &str = "Library/Application Support/Zed/extensions/installed";

pub struct ZedExtension {
    directory: String,
    name: String,
    family: ZedThemeFamily,
}

impl ZedExtension {
    /// Ensure the name of a theme file stored in an extension is in the form
    /// zed expects "name.json".
    pub fn normalize_filename(name: &str) -> String {
        let extname = name.replace(".json", "");
        format!("{extname}.json")
    }

    fn read_from_name(name: &str, extdir: &str) -> Result<Box<Self>, ThemeError> {
        eprintln!("name = {name}; extdir = {extdir};");
        let extname = ensure_json_extension(name);
        let globby = format!("{}/**/{}", extdir, extname.display());
        eprintln!("{globby}");

        // use globs to find a file named `name.json` somewhere in this as a subdir

        let mut matches = glob(globby.as_str())?;
        let Some(Ok(found)) = matches.find(|xs| xs.is_ok()) else {
            return Err(ThemeError::ThemeNotFound(name.to_owned()));
        };
        let family = ZedThemeFamily::read(&found)?;

        let extension = Self {
            name: family.name.clone(),
            directory: found.to_string_lossy().to_string(),
            family,
        };

        Ok(Box::new(extension))
    }
}

impl Extension for ZedExtension {
    type ThemeType = ZedTheme;

    fn read(name: &str) -> Result<Box<Self>, ThemeError> {
        ZedExtension::read_from_name(name, ZedExtension::extensions_path().as_str())
    }

    fn write(&self) -> Result<(), ThemeError> {
        todo!()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn directory(&self) -> &str {
        self.directory.as_str()
    }

    fn extensions_path() -> String {
        let twiddle = home::home_dir().unwrap_or_default();
        format!("{}/{}", twiddle.display(), EXTENSION_DIR)
    }

    fn official_path_for(name: &str, extdir: &str) -> String {
        let twiddle = home::home_dir().unwrap_or_default();
        format!("{}/{}/{}", twiddle.display(), extdir, name)
    }

    fn themes(&self) -> &[Self::ThemeType] {
        self.family.themes.as_slice()
    }
}

impl From<&VsCodeExtension> for ZedExtension {
    fn from(value: &VsCodeExtension) -> Self {
        let directory = ZedExtension::official_path_for(value.name(), EXTENSION_DIR);
        let themes = value.themes().iter().map(ZedTheme::from).collect();
        let family = ZedThemeFamily {
            schema: None,
            author: "unknown".to_string(),
            name: value.name().to_owned(),
            themes,
        };

        ZedExtension {
            directory,
            name: value.name().to_owned(),
            family,
        }
    }
}

impl From<&VsCodeTheme> for ZedExtension {
    fn from(vscode_theme: &VsCodeTheme) -> Self {
        let zed: ZedTheme = ZedTheme::from(vscode_theme);
        let family = ZedThemeFamily {
            schema: Some("https://zed.dev/schema/themes/v0.2.0.json".to_string()),
            author: "".to_string(),
            name: vscode_theme.name.clone(),
            themes: vec![zed],
        };

        // well, if we have a filename, we should use it.
        let theme_filename = slug::slugify(family.name.as_str());
        let directory = ZedExtension::official_path_for(theme_filename.as_str(), EXTENSION_DIR);
        ZedExtension {
            name: vscode_theme.name.clone(),
            family,
            directory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_fixtures() {
        let fixtures = format!("{}/fixtures/zed", env!("CARGO_MANIFEST_DIR"));
        let theme = ZedExtension::read_from_name("catppuccin-latte", fixtures.as_str())
            .expect("expected to read catppuccin-latte fixture");
        assert_eq!(theme.name(), "Catppuccin");

        let theme = ZedExtension::read_from_name("rose-pine-moon.json", fixtures.as_str())
            .expect("expected to read rose pine moon fixture");
        assert_eq!(theme.name(), "Rosé Pine Moon");
    }

    #[test]
    fn wont_pass_in_ci() {
        let theme =
            ZedExtension::read("rose-pine-moon.json").expect("expected to official read rose pine moon extension");
        assert_eq!(theme.name(), "Rosé Pine Moon");
    }
}
