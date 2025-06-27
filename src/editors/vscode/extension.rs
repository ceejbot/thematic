//! Read and write VSCode theme extensions.
//! We don't try to be complete shiny publishable extensions, but just enough
//! that the editor can load and use the theme.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use glob::glob;

use super::{Contributions, Repository, ThemePointer, VsCodePackageJson, VsCodeTheme};
use crate::editors::{Extension, ThemeFile, ZedExtension};
use crate::{IconFileManager, IconThemePointer, ThemeError, VsCodeIconTheme, ZedThemeFamily, find_extension_name};

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
    icon_themes: Vec<VsCodeIconTheme>,
    manifest: VsCodePackageJson,
}

impl VsCodeExtension {
    // start cleanup here

    pub fn find_from_name(name: &str, extdir: &str) -> Result<Box<VsCodeExtension>, ThemeError> {
        // use globs to find a file named `name(-color-theme)?.json` somewhere in this as a subdir
        let barename = name.replace(".json", "");
        let globby = format!("{}/**/themes/{}*.json", extdir, name.replace(".json", ""));

        let mut matches = glob(globby.as_str())?;
        if let Some(Ok(found)) = matches.find(|xs| xs.is_ok()) {
            return Self::read_from_path(found, barename);
        };

        let extname_glob = format!("{}/*{}*/package.json", extdir, barename);
        let mut matches = glob(extname_glob.as_str())?;
        if let Some(Ok(found)) = matches.find(|xs| xs.is_ok()) {
            return Self::read_from_path(found, barename);
        };

        Err(ThemeError::ThemeNotFound(name.to_owned()))
    }

    pub fn read_from_path(extpath: PathBuf, barename: String) -> Result<Box<VsCodeExtension>, ThemeError> {
        if extpath.ends_with("package.json") {
            eprintln!("{} ends with package.json", extpath.display());
            let contents = std::fs::read_to_string(&extpath)?;

            let metadata: VsCodePackageJson = serde_json::from_str(contents.as_str())?;
            eprintln!("from_metadata() is next");
            return VsCodeExtension::from_metadata(extpath, metadata);
        }

        if let Some(metadata) = VsCodeExtension::extension_metadata(&extpath) {
            // Cool. We can use the metadata to build our extension
            return VsCodeExtension::from_metadata(extpath, metadata);
        }

        // TODO consider if we want to do this at all

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

        let theme_names: Vec<String> = themes.iter().map(|xs| xs.name.clone()).collect();
        let extension_name = if let Some(found) = find_extension_name(theme_names.as_slice()) {
            found
        } else {
            theme_names[0].clone()
        };

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
            icon_themes: Vec::new(),
        };

        let metadata = VsCodePackageJson {
            name: barename.to_owned(),
            display_name: extension_name,
            description: "Constructed from a directory full of theme files.".to_string(),
            publisher: "none".to_string(),
            contributes,
            repository: Repository { url: "".to_string() },
        };

        let extension = VsCodeExtension {
            name: barename.to_owned(),
            directory: extpath,
            icon_themes: Vec::new(), // TODO
            themes,
            manifest: metadata,
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

        let icon_themes: Vec<_> = metadata
            .contributes
            .icon_themes
            .iter()
            .filter_map(|xs| {
                let mut themepath = extpath.clone();
                themepath.push(&xs.path);
                VsCodeIconTheme::read(&themepath).ok()
            })
            .collect();

        let extension = VsCodeExtension {
            name: metadata.display_name.clone(),
            directory: found,
            themes,
            icon_themes,
            manifest: metadata,
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
            icon_themes: Vec::new(),
        };

        let metadata = VsCodePackageJson {
            name: slug::slugify(&name),
            display_name: name.clone(),
            description: "constructed extension".to_string(),
            publisher: "n/a".to_string(),
            contributes,
            repository: Repository { url: "n/a".to_string() },
        };

        Self {
            directory,
            name,
            themes,
            manifest: metadata,
            icon_themes: Vec::new(),
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
    type IconThemeType = VsCodeIconTheme;
    type Manifest = VsCodePackageJson;

    fn extensions_path() -> String {
        OFFICIAL_DIR.clone()
    }

    fn read(name: &str) -> Result<Box<VsCodeExtension>, ThemeError> {
        VsCodeExtension::find_from_name(name, VsCodeExtension::extensions_path().as_str())
    }

    fn write(&self) -> Result<(), ThemeError> {
        self.write_to(self.official_path())
    }

    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let mut workdir = PathBuf::new();
        workdir.push(&path);
        workdir.push("themes");
        std::fs::create_dir_all(&workdir)?;

        for theme in self.themes.as_slice() {
            let slugged = slug::slugify(&theme.name);
            let themefile = format!("{slugged}.json");
            let mut filename = workdir.clone();
            filename.push(themefile);
            theme.write(filename)?;
        }

        // Write icon themes if any exist
        if !self.icon_themes.is_empty() {
            workdir.pop();
            workdir.push("icon_themes");
            std::fs::create_dir_all(&workdir)?;

            for (i, icon_theme) in self.icon_themes.iter().enumerate() {
                // Get the theme name from manifest, fallback to index-based name
                let theme_name = self
                    .manifest
                    .icon_themes()
                    .get(i)
                    .map(|ptr| ptr.label.as_str())
                    .unwrap_or("Icon Theme");

                let slugged = slug::slugify(theme_name);
                let icon_theme_file = format!("{slugged}.json");
                let mut filename = workdir.clone();
                filename.push(icon_theme_file);

                // Copy icon files if they exist in the source directory
                let source_dir = &self.directory;
                let dest_dir = path.as_ref().to_path_buf();

                // Create an IconFileManager for this icon theme
                let mut icon_manager = IconFileManager::new(source_dir, &dest_dir, "icons");

                // Track all icons from this theme with source base path
                if let Err(e) = icon_theme.track_icons_with_base(&mut icon_manager, Some(source_dir)) {
                    log::warn!("Failed to track icon files for theme '{}': {}", theme_name, e);
                }

                // Copy the icon files and update paths if icons were found
                if !icon_manager.tracked_icons().is_empty() {
                    // Try to copy icons - if it fails, we'll still write the theme JSON
                    if let Err(e) = icon_manager.copy_icons() {
                        log::warn!("Failed to copy icon files for theme '{}': {}", theme_name, e);
                        // Write the theme as-is without updated paths
                        icon_theme.write(filename)?;
                    } else {
                        // Create a mapping of old paths to new paths
                        let mut path_mapping = std::collections::HashMap::new();
                        for icon_path in icon_theme.get_icon_paths() {
                            if let Some(filename_str) = IconFileManager::extract_filename(&icon_path) {
                                let new_path = icon_manager.get_relative_path(&filename_str);
                                path_mapping.insert(icon_path, new_path);
                            }
                        }

                        // Update the icon theme with new paths and write it
                        let mut updated_theme = icon_theme.clone();
                        updated_theme.update_icon_paths(&path_mapping);
                        updated_theme.write(filename)?;

                        log::info!(
                            "Copied {} icon files for theme '{}'",
                            icon_manager.tracked_icons().len(),
                            theme_name
                        );
                    }
                } else {
                    // No icons to copy, write the theme as-is
                    icon_theme.write(filename)?;
                }
            }
        }

        workdir.pop();
        let package_bytes = serde_json::to_vec_pretty(self.manifest())?;
        workdir.push("package.json");
        let mut fp = std::fs::File::create(&workdir)?;
        fp.write_all(package_bytes.as_slice())?;
        log::info!("Wrote VSCode extension '{}' to {}", self.name, self.directory.display());
        Ok(())
    }

    fn build_official_path(name: &str, extdir: &str) -> String {
        format!("{}/{}", extdir, name)
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn official_path(&self) -> &PathBuf {
        &self.directory
    }

    fn manifest(&self) -> &VsCodePackageJson {
        &self.manifest
    }

    /// Themes as slice
    fn themes(&self) -> &[VsCodeTheme] {
        self.themes.as_slice()
    }

    /// Themes as slice
    fn icon_themes(&self) -> &[VsCodeIconTheme] {
        self.icon_themes.as_slice()
    }
}

impl From<ZedExtension> for VsCodeExtension {
    fn from(input: ZedExtension) -> Self {
        let name = input.name().to_owned();
        let display_name = input.manifest().name().to_owned();
        let description = input.manifest().description().to_owned();
        let publisher = input.manifest().authors().join(", ");
        let repository = input.manifest().repository().to_owned();
        let directory =
            VsCodeExtension::build_official_path(input.manifest().id(), VsCodeExtension::extensions_path().as_str());
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

        let theme_pairs: Vec<_> = input
            .icon_themes()
            .iter()
            .map(|xs| {
                // Generate a proper path for the icon theme based on its name
                let theme_filename = format!("{}.json", slug::slugify(&xs.name));
                let theme_path = format!("./icon_themes/{}", theme_filename);

                let pointer = IconThemePointer {
                    id: slug::slugify(xs.name.as_str()),
                    label: xs.name.clone(),
                    path: theme_path,
                };
                let vsi = VsCodeIconTheme::from(xs);
                (vsi, pointer)
            })
            .collect();
        let (icon_themes, icon_theme_pointers): (Vec<_>, Vec<_>) = theme_pairs.into_iter().unzip();

        let contributes = Contributions {
            icon_themes: icon_theme_pointers,
            themes: theme_pointers,
        };

        let themes: Vec<VsCodeTheme> = families
            .iter()
            .flat_map(|fam| {
                let themelist: Vec<VsCodeTheme> = fam.into();
                themelist
            })
            .collect();

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
            icon_themes,
            manifest: metadata,
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
    fn no_ci_find_vs_code_extensions() {
        let found = VsCodeExtension::find_from_name("bluloco-light", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Bluloco Light");
        assert_eq!(found.name, "Bluloco Light Theme");
        assert_eq!(found.themes.len(), 2);
    }

    #[test]
    fn no_ci_find_by_ext_name_not_theme() {
        // there are many cases where the extension has a name that is not one of its theme names
        let found = VsCodeExtension::find_from_name("rainglow", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Rainglow");
        assert_eq!(found.name, "Rainglow");
        assert!(
            found.themes.len() >= 325,
            "Expected at least 325 themes, found {}",
            found.themes.len()
        );

        let converted = ZedExtension::from((*found).clone());
        assert_eq!(converted.name(), found.name);
        let family = converted.families().first().expect("we have at least one theme family");
        assert_eq!(family.themes.len(), 3, "we expected grouping to work");
    }
}
