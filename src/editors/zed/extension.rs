//! Read and write Zed theme extensions.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use glob::glob;
use serde::{Deserialize, Serialize};

use crate::editors::{Extension, ThemeFile};
use crate::vscode::VsCodeExtension;
use crate::{ThemeError, VsCodeTheme, ZedTheme, ZedThemeFamily};

static EXTENSION_DIR: &str = "Library/Application Support/Zed/extensions/installed";

/// Our representation of a Zed theme extension.
#[derive(Debug, Clone)]
pub struct ZedExtension {
    /// Where this extension resides on disk. Pub(crate) for testing.
    pub(crate) directory: PathBuf,
    /// A portion of the toml extension manifest.
    metadata: ZedManifest,
    /// The human name of the extension.
    name: String,
    /// All the theme families this extension provides.
    families: Vec<ZedThemeFamily>,
}

impl ZedExtension {
    pub fn find_from_name(name: &str, extdir: &str) -> Result<Box<Self>, ThemeError> {
        let barename = name.replace(".json", "");
        let extname = ZedExtension::normalize_name(name);
        let globby = format!("{}/**/{}", extdir, extname);

        // use globs to find a file named `name.json` somewhere in this as a subdir
        let mut matches = glob(globby.as_str())?;
        let Some(Ok(found)) = matches.find(|xs| xs.is_ok()) else {
            return Err(ThemeError::ThemeNotFound(name.to_owned()));
        };

        ZedExtension::read_from_path(&found, barename)
    }

    pub fn normalize_name(input: &str) -> String {
        format!("{}.json", slug::slugify(input.replace(".json", "")))
    }

    /// All theme families
    pub fn families(&self) -> &[ZedThemeFamily] {
        self.families.as_slice()
    }

    pub fn read_from_path(extpath: &PathBuf, barename: String) -> Result<Box<Self>, ThemeError> {
        if extpath.ends_with("extension.toml") {
            let contents = std::fs::read_to_string(extpath)?;
            let metadata: ZedManifest = toml::from_str(contents.as_str())?;
            let mut themepath = extpath.clone();
            themepath.pop();
            themepath.push("themes/stub.json");
            return ZedExtension::from_metadata(&themepath, metadata);
        }
        if let Some(metadata) = ZedExtension::extension_metadata(extpath) {
            // Cool. We can use the metadata to build our extension
            return ZedExtension::from_metadata(extpath, metadata);
        }

        let family = ZedThemeFamily::read(extpath)?;
        let id = slug::slugify(&barename);
        let themes = vec![format!(
            "./themes/{}",
            ZedExtension::normalize_name(family.name.as_str())
        )];

        let metadata = ZedManifest {
            id,
            name: family.name.clone(),
            version: "0.1.0".to_string(),
            description: "Constructed from a directory of color themes".to_string(),
            themes,
            ..Default::default()
        };

        let extension = Self {
            name: family.name.clone(),
            directory: extpath.clone(),
            families: vec![family],
            metadata,
        };

        Ok(Box::new(extension))
    }

    /// Input is a path to a theme file; we find a manifest in the parent dir if one exists.
    pub fn extension_metadata(fpath: &Path) -> Option<ZedManifest> {
        // parent dir must exist and be named "themes"
        let parent = fpath.parent()?;
        if !parent.is_dir() || !parent.ends_with("themes") {
            return None;
        }
        // hop up one more.
        let extdir = parent.parent()?;
        // this one must have a extension.toml file
        let mut pkg_path = PathBuf::from(extdir);
        pkg_path.push("extension.toml");
        let Ok(contents) = std::fs::read_to_string(&pkg_path) else {
            return None;
        };
        toml::from_str::<ZedManifest>(contents.as_str()).ok()
    }

    pub fn from_metadata(found: &Path, metadata: ZedManifest) -> Result<Box<ZedExtension>, ThemeError> {
        let mut extpath = found.to_path_buf();
        if !extpath.is_dir() {
            extpath.pop();
        }
        if extpath.ends_with("themes") {
            extpath.pop();
        }
        let families: Vec<ZedThemeFamily> = metadata
            .themes
            .iter()
            .filter_map(|xs| {
                let mut family_file = extpath.clone();
                family_file.push(xs);
                ZedThemeFamily::read(&family_file).ok()
            })
            .collect();

        let extension = ZedExtension {
            name: metadata.name.clone(),
            directory: extpath,
            metadata,
            families,
        };
        Ok(Box::new(extension))
    }
}

impl Extension for ZedExtension {
    type ThemeType = ZedTheme;
    type Metadata = ZedManifest;

    fn read(name: &str) -> Result<Box<Self>, ThemeError> {
        ZedExtension::find_from_name(name, ZedExtension::extensions_path().as_str())
    }

    fn write(&self) -> Result<(), ThemeError> {
        let mut workdir = PathBuf::from(&self.directory);
        workdir.push("themes");
        std::fs::create_dir_all(&workdir)?;

        for family in self.families.as_slice() {
            let mut filename = workdir.clone();
            let themefile = ZedExtension::normalize_name(&family.name);
            filename.push(themefile);
            family.write(filename)?;
        }

        workdir.pop();
        let tomlstr = toml::to_string_pretty(self.metadata())?;
        workdir.push("extension.toml");
        let mut fp = std::fs::File::create(&workdir)?;
        fp.write_all(tomlstr.as_bytes())?;
        log::info!("Wrote Zed extension '{}' to {}", self.name, self.directory.display());
        Ok(())
    }

    fn extensions_path() -> String {
        let twiddle = home::home_dir().unwrap_or_default();
        format!("{}/{}", twiddle.display(), EXTENSION_DIR)
    }

    fn build_official_path(name: &str, extdir: &str) -> String {
        format!("{}/{}", extdir, name)
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn themes(self) -> Vec<Self::ThemeType> {
        self.families.into_iter().flat_map(|family| family.themes).collect()
    }

    fn official_path(&self) -> &PathBuf {
        &self.directory
    }

    fn metadata(&self) -> &Self::Metadata {
        &self.metadata
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZedManifest {
    id: String,
    name: String,
    version: String,
    schema_version: usize,
    description: String,
    repository: String,
    authors: Vec<String>,
    themes: Vec<String>,
    icon_themes: Vec<String>,
}

impl Default for ZedManifest {
    fn default() -> Self {
        ZedManifest {
            id: String::default(),
            name: String::default(),
            version: String::default(),
            schema_version: 1,
            description: String::default(),
            repository: String::default(),
            authors: Vec::new(),
            themes: Vec::new(),
            icon_themes: Vec::new(),
        }
    }
}

impl ZedManifest {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    pub fn repository(&self) -> &str {
        self.repository.as_str()
    }

    pub fn authors(&self) -> &[String] {
        self.authors.as_slice()
    }

    pub fn themes(&self) -> &[String] {
        self.themes.as_slice()
    }
}

impl From<VsCodeExtension> for ZedExtension {
    fn from(extension: VsCodeExtension) -> Self {
        let vs_meta = extension.metadata();
        let id = vs_meta.name().to_owned();
        let display_name = vs_meta.display_name().to_owned();
        let author = vs_meta.publisher().to_owned();
        let description = vs_meta.description().to_owned();
        let repository = vs_meta.repository().to_owned();
        let authors = vec![vs_meta.publisher().to_owned()];

        let incoming = extension.themes();

        // Now we do our first clever thing. We group themes by name similarity
        // into Zed theme families. We then convert by family.
        let mut theme_map: HashMap<String, VsCodeTheme> = HashMap::new();
        let theme_names: Vec<String> = incoming
            .clone()
            .iter()
            .map(|xs| {
                let name = xs.name.clone();
                theme_map.insert(name.clone(), xs.clone());
                xs.name.clone()
            })
            .collect();
        let family_name_pairs = crate::group_families(theme_names);
        let families: Vec<ZedThemeFamily> = family_name_pairs
            .iter()
            .map(|(maybe_name, name_family)| {
                let themes: Vec<ZedTheme> = name_family
                    .iter()
                    .filter_map(|name| theme_map.remove(name).map(|xs| ZedTheme::from(&xs)))
                    .collect();
                let fam_name = if let Some(n) = maybe_name {
                    n.clone()
                } else {
                    themes.as_slice()[0].name.clone()
                };
                ZedThemeFamily {
                    schema: None,
                    author: author.clone(),
                    name: fam_name,
                    themes,
                }
            })
            .collect();

        let family_pointers = families
            .iter()
            .map(|family| format!("./themes/{}", ZedExtension::normalize_name(family.name.as_str())))
            .collect();

        let metadata = ZedManifest {
            id,
            name: display_name.clone(),
            version: "0.1.0".to_string(),
            schema_version: 1,
            description,
            repository,
            authors,
            themes: family_pointers,
            ..Default::default()
        };
        let directory = ZedExtension::build_official_path(metadata.id.as_str(), EXTENSION_DIR);

        ZedExtension {
            directory: directory.into(),
            name: display_name,
            families,
            metadata,
        }
    }
}

impl From<&VsCodeTheme> for ZedExtension {
    fn from(vscode_theme: &VsCodeTheme) -> Self {
        let zed: ZedTheme = ZedTheme::from(vscode_theme);
        let family = ZedThemeFamily {
            schema: Some("https://zed.dev/schema/themes/v0.2.0.json".to_string()),
            author: "Unknown".to_string(),
            name: vscode_theme.name.clone(),
            themes: vec![zed],
        };

        // well, if we have a filename, we should use it.
        let theme_filename = slug::slugify(family.name.as_str());
        let directory = ZedExtension::build_official_path(theme_filename.as_str(), EXTENSION_DIR);
        let themes = family
            .themes
            .iter()
            .map(|xs| {
                let slugname = slug::slugify(&xs.name);
                format!("./themes/{slugname}.json")
            })
            .collect();

        let metadata = ZedManifest {
            id: theme_filename,
            name: vscode_theme.name.clone(),
            version: "0.1.0".to_string(),
            schema_version: 1,
            description: format!("converted from VSCode theme {}", vscode_theme.name),
            repository: "".to_string(),
            authors: Vec::new(),
            themes,
            ..Default::default()
        };

        ZedExtension {
            name: vscode_theme.name.clone(),
            directory: directory.into(),
            metadata,
            families: vec![family],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_fixtures() {
        let themelist = vec![
            "fixtures/zed/catppuccin/themes/catppuccin-mauve.json",
            "fixtures/zed/catppuccin/themes/catppuccin-no-italics-mauve.json",
            "fixtures/zed/rose-pine-theme/themes/rose-pine.json",
            "fixtures/zed/rose-pine-theme/themes/rose-pine-dawn.json",
            "fixtures/zed/rose-pine-theme/themes/rose-pine-moon.json",
        ];

        for theme in themelist {
            let family = ZedThemeFamily::read(theme).expect("expected test fixture to be readable");
            assert!(!family.themes.is_empty());
        }
    }

    #[test]
    fn find_fixtures() {
        let fixtures = format!("{}/fixtures/zed", env!("CARGO_MANIFEST_DIR"));
        let theme = ZedExtension::find_from_name("catppuccin-mauve", fixtures.as_str())
            .expect("expected to read catppuccin-mauve fixture");
        assert_eq!(theme.name(), "Catppuccin");

        let theme = ZedExtension::find_from_name("rose-pine-moon.json", fixtures.as_str())
            .expect("expected to read rose pine moon fixture");
        assert!(theme.name().contains("Rosé Pine"));
    }

    #[test]
    fn no_ci_find_zed_extensions() {
        let theme =
            ZedExtension::read("rose-pine-moon.json").expect("expected to read official rose pine moon extension");
        assert!(theme.name().contains("Rosé Pine"));
    }
}
