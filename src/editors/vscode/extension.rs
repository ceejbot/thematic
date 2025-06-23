//! Read and write VSCode theme extensions.
//! We don't try to be complete shiny publishable extensions, but just enough
//! that the editor can load and use the theme.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use glob::glob;

use super::{Contributions, Repository, ThemePointer, VsCodePackageJson, VsCodeTheme};
use crate::editors::{Extension, ThemeFile, ZedExtension};
use crate::{ThemeError, ZedThemeFamily};

static EXTENSION_DIR: &str = ".vscode/extensions";

static OFFICIAL_DIR: LazyLock<String> = LazyLock::new(|| {
    let twiddle = home::home_dir().unwrap_or_default();
    format!("{}/{}", twiddle.display(), EXTENSION_DIR)
});

#[derive(Debug, Clone)]
pub struct VsCodeExtension {
    pub(crate) directory: PathBuf,
    name: String,
    themes: Vec<VsCodeTheme>,
    metadata: VsCodePackageJson,
}

impl VsCodeExtension {
    // start cleanup here

    pub fn find_from_name(name: &str, extdir: &str) -> Result<Box<VsCodeExtension>, ThemeError> {
        // use globs to find a file named `name(-color-theme)?.json` somewhere in this as a subdir
        let barename = name.replace(".json", "");
        let globby = format!("{}/**/themes/{}*.json", extdir, name.replace(".json", ""));
        // eprintln!("{globby}");

        let mut matches = glob(globby.as_str())?;
        if let Some(Ok(found)) = matches.find(|xs| xs.is_ok()) {
            return Self::read_from_path(found, barename);
        };

        let extname_glob = format!("{}/*{}*/package.json", extdir, barename);
        eprintln!("{extname_glob}");
        let mut matches = glob(extname_glob.as_str())?;
        if let Some(Ok(found)) = matches.find(|xs| xs.is_ok()) {
            return Self::read_from_path(found, barename);
        };

        Err(ThemeError::ThemeNotFound(name.to_owned()))
    }

    pub fn read_from_path(extpath: PathBuf, barename: String) -> Result<Box<VsCodeExtension>, ThemeError> {
        if extpath.ends_with("package.json") {
            let contents = std::fs::read_to_string(&extpath)?;
            let metadata: VsCodePackageJson = serde_json::from_str(contents.as_str())?;
            return VsCodeExtension::from_metadata(extpath, metadata);
        }

        if let Some(metadata) = VsCodeExtension::extension_metadata(&extpath) {
            // Cool. We can use the metadata to build our extension
            return VsCodeExtension::from_metadata(extpath, metadata);
        }

        log::debug!("Falling back to reading loose color theme json files.");
        // read all .json files in this directory and build a list of the ones that are valid themes
        let mut themes = if let Some(parent) = Path::new(&extpath).parent() {
            std::fs::read_dir(parent)?
                .filter_map(|xs| {
                    if let Ok(e) = xs {
                        let fpath = e.path();
                        let pathstring = fpath.to_string_lossy();
                        if pathstring.ends_with(".json")
                            && pathstring.contains(barename.as_str())
                            && !pathstring.contains("icon-theme")
                        {
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

        if themes.is_empty() {
            // We found no themes. There is no point.
            return Err(ThemeError::NoThemesFound(extpath.to_string_lossy().to_string()));
        }
        themes.sort_by(|left, right| left.name.cmp(&right.name));

        let mut theme_pointers: Vec<ThemePointer> = themes
            .iter()
            .map(|xs| {
                let ui_theme = if xs.is_dark_theme() { "vs-dark" } else { "vs" };
                ThemePointer {
                    label: slug::slugify(&xs.name),
                    ui_theme: ui_theme.to_string(),
                    path: format!("./themes/{}", xs.filename),
                }
            })
            .collect();
        theme_pointers.sort_by(|left, right| left.label.cmp(&right.label));
        let contributes = Contributions {
            themes: theme_pointers,
            // icon_themes: Vec::new(),
        };

        let metadata = VsCodePackageJson {
            name: barename.to_owned(),
            display_name: themes[0].name.clone(), // we know this exists
            description: "Constructed from a directory full of theme files.".to_string(),
            publisher: "none".to_string(),
            contributes,
            repository: Repository { url: "".to_string() },
        };

        let extension = VsCodeExtension {
            name: barename.to_owned(),
            directory: extpath,
            themes,
            metadata,
        };

        Ok(Box::new(extension))
    }

    pub fn from_metadata(found: PathBuf, metadata: VsCodePackageJson) -> Result<Box<VsCodeExtension>, ThemeError> {
        let mut extpath = found.clone();
        if !extpath.is_dir() {
            extpath.pop();
        }
        if extpath.ends_with("themes") {
            extpath.pop();
        }

        let themes: Vec<_> = metadata
            .contributes
            .themes
            .iter()
            .filter_map(|xs| {
                let mut themepath = extpath.clone();
                themepath.push(&xs.path);
                VsCodeTheme::read(&themepath).ok()
            })
            .collect();

        let extension = VsCodeExtension {
            name: metadata.display_name.clone(),
            directory: found,
            themes,
            metadata,
        };

        Ok(Box::new(extension))
    }

    /// Ensure the name of a theme file stored in an extension is in the form
    /// vscode expects "name.json".
    pub fn normalize_filename(name: &str) -> String {
        let extname = name.replace(".json", "").replace("-color-theme", "");
        format!("{extname}.json")
    }

    pub fn new(theme_name: &str, filename: &str, themes: Vec<VsCodeTheme>) -> Self {
        let dir = VsCodeExtension::build_official_path(filename, EXTENSION_DIR);
        let mut directory = PathBuf::new();
        directory.push(dir);
        let name = theme_name.to_owned();

        let theme_pointers = themes
            .iter()
            .map(|t| {
                let ui_theme = if t.is_dark_theme() { "vs-dark" } else { "vs" };
                ThemePointer {
                    label: t.filename.replace(".json", ""),
                    ui_theme: ui_theme.to_string(),
                    path: t.filename.clone(),
                }
            })
            .collect();
        let contributes = Contributions {
            themes: theme_pointers,
            // icon_themes: Vec::new(),
        };

        let metadata = VsCodePackageJson {
            name: slug::slugify(&name),
            display_name: name.clone(),
            description: "constructed extension".to_string(),
            publisher: "".to_string(),
            contributes,
            repository: Repository { url: "".to_string() },
        };

        Self {
            directory,
            name,
            themes,
            metadata,
        }
    }

    /// Input is a path to a theme file; we decide if it's part of an extension
    pub fn extension_metadata(fpath: &Path) -> Option<VsCodePackageJson> {
        // parent dir must exist and be named "themes"
        let parent = fpath.parent()?;
        if !parent.is_dir() || !parent.ends_with("themes") {
            return None;
        }
        // hop up one more.
        let extdir = parent.parent()?;
        // this one must have a package.json file
        let mut pkg_path = PathBuf::from(extdir);
        pkg_path.push("package.json");
        let Ok(contents) = std::fs::read_to_string(&pkg_path) else {
            return None;
        };
        serde_json::from_str::<VsCodePackageJson>(contents.as_str()).ok()
    }
}

impl Extension for VsCodeExtension {
    type ThemeType = VsCodeTheme;
    type Metadata = VsCodePackageJson;

    fn extensions_path() -> String {
        OFFICIAL_DIR.clone()
    }

    fn read(name: &str) -> Result<Box<VsCodeExtension>, ThemeError> {
        VsCodeExtension::find_from_name(name, VsCodeExtension::extensions_path().as_str())
    }

    fn write(&self) -> Result<(), ThemeError> {
        let mut workdir = PathBuf::from(&self.directory);
        workdir.push("themes");
        std::fs::create_dir_all(&workdir)?;

        for theme in self.themes.as_slice() {
            let slugged = slug::slugify(&theme.name);
            let themefile = format!("{slugged}.json");
            let mut filename = workdir.clone();
            filename.push(themefile);
            theme.write(filename)?;
        }

        workdir.pop();
        let package_bytes = serde_json::to_vec_pretty(self.metadata())?;
        workdir.push("package.json");
        let mut fp = std::fs::File::create(&workdir)?;
        fp.write_all(package_bytes.as_slice())?;
        log::info!("Wrote VSCode extension '{}' to {}", self.name, self.directory.display());
        Ok(())
    }

    fn build_official_path(name: &str, extdir: &str) -> String {
        let extname = VsCodeExtension::normalize_filename(name);
        let subdir = extname.replace("-color-theme.json", "");
        format!("{extdir}/{subdir}/{extname}")
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn official_path(&self) -> &PathBuf {
        &self.directory
    }

    fn metadata(&self) -> &VsCodePackageJson {
        &self.metadata
    }

    /// Consumes themes; use when converting.
    fn themes(self) -> Vec<VsCodeTheme> {
        self.themes
    }
}

impl From<ZedExtension> for VsCodeExtension {
    fn from(input: ZedExtension) -> Self {
        let name = input.name().to_owned();
        let display_name = input.metadata().name().to_owned();
        let description = input.metadata().description().to_owned();
        let publisher = input.metadata().authors().join(", ");
        let repository = input.metadata().repository().to_owned();
        let directory =
            VsCodeExtension::build_official_path(input.metadata().id(), VsCodeExtension::extensions_path().as_str());
        let families = input.families().to_owned();

        let themes = input.themes();
        let theme_pointers = themes
            .iter()
            .map(|theme| {
                let ui_theme = match theme.appearance {
                    crate::Appearance::Light => "vs".to_string(),
                    crate::Appearance::Dark => "vs-dark".to_string(),
                };

                let slugged = slug::slugify(&theme.name);
                let filename = format!("./themes/{slugged}.json");
                ThemePointer {
                    label: theme.name.clone(),
                    ui_theme,
                    path: filename,
                }
            })
            .collect();
        let contributes = Contributions {
            // icon_themes: Vec::new(),
            themes: theme_pointers,
        };

        let themes: Vec<VsCodeTheme> = families
            .iter()
            .map(|fam| {
                eprintln!("family has {} themes", fam.themes.len());
                let themelist: Vec<VsCodeTheme> = fam.into();
                eprintln!("converted them to {} themes", themelist.len());
                themelist
            })
            .flatten()
            .collect();
        eprintln!("---- {} themes", themes.len());

        let metadata = VsCodePackageJson {
            name: name.clone(),
            display_name,
            description,
            publisher,
            contributes,
            repository: Repository { url: repository },
        };

        VsCodeExtension {
            directory: directory.into(),
            name,
            themes,
            metadata,
        }
    }
}

impl From<ZedThemeFamily> for VsCodeExtension {
    fn from(family: ZedThemeFamily) -> Self {
        let themes: Vec<VsCodeTheme> = family.themes.into_iter().map(|xs| (&xs).into()).collect();
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
            "input.json",
            VsCodeExtension::normalize_filename("input-color-theme.json")
        );
        assert_eq!("input.json", VsCodeExtension::normalize_filename("input.json"));
        assert_eq!("input.json", VsCodeExtension::normalize_filename("input-color-theme"));
        assert_eq!("input.json", VsCodeExtension::normalize_filename("input"));
    }

    #[test]
    fn reading_by_name_works() {
        let fixtures = format!("{}/fixtures/vscode", env!("CARGO_MANIFEST_DIR"));
        let found = VsCodeExtension::find_from_name("rose-pine-moon", fixtures.as_str())
            .expect("failed to find Rosé Pine Moon");
        assert!(found.name.contains("Rosé Pine"));
        assert_eq!(found.themes.len(), 6);
    }

    #[test]
    fn reading_by_manifest_path_works() {
        let vscode_ext = VsCodeExtension::read_from_path(
            "fixtures/vscode/mvllow.rose-pine-2.14.0/package.json".into(),
            "unused".to_string(),
        )
        .expect("we expect to find the rose pine text fixture");
        assert!(!vscode_ext.themes.is_empty(), "we should have found some themes");
        assert_eq!(vscode_ext.themes.len(), 6, "we expected 6 themes");
    }

    #[test]
    fn wont_pass_in_ci_find() {
        let found = VsCodeExtension::find_from_name("bluloco-light", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Bluloco Light");
        assert_eq!(found.name, "Bluloco Light Theme");
        assert_eq!(found.themes.len(), 2);
    }

    #[test]
    fn find_by_extname_not_theme() {
        // there are many cases where the extension has a name that is not one of its theme names
        let found = VsCodeExtension::find_from_name("rainglow", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Rainglow");
        assert_eq!(found.name, "Rainglow");
        assert!(
            found.themes.len() >= 325,
            "Expected at least 325 themes, found {}",
            found.themes.len()
        );
    }

    #[test]
    fn wont_pass_in_ci_convert() {
        let found = VsCodeExtension::find_from_name("rainglow", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Rainglow");
        assert_eq!(found.name, "Rainglow");
        let theme_count = found.themes.len();
        assert!(
            theme_count >= 325,
            "Expected at least 325 themes, found {}",
            theme_count
        );

        let converted = ZedExtension::from((*found).clone());
        assert_eq!(converted.name(), found.name);
        let family = converted.families().first().expect("we have at least one theme family");
        assert_eq!(family.themes.len(), 3, "we expected grouping to work");
    }
}
