//! Read and write Zed theme extensions.

use std::io::Write;
use std::path::{Path, PathBuf};

use glob::glob;
use serde::{Deserialize, Serialize};
use textdistance::{Algorithm, Prefix};

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
        let twiddle = home::home_dir().unwrap_or_default();
        format!("{}/{}/{}", twiddle.display(), extdir, name)
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

/// Figure out the best extension name and the best family groups, if possible,
/// from the names of incoming themes. We don't bother splitting things up if
/// there are only three or fewer, but we do try to determine a name.
fn group_families(theme_names: Vec<String>) -> (String, Vec<Vec<String>>) {
    // We have the advantage of knowing that we never have more than a handful of themes.
    // Uh. most of the time.

    let mut name_iter = theme_names.iter();
    let Some(first) = name_iter.next() else {
        // quite surprising really
        return ("Unknown".to_string(), vec![theme_names]);
    };

    if theme_names.len() <= 3 {
        return (first.clone(), vec![theme_names]);
    }

    let dist = Prefix::default();
    let prefix_len = name_iter.fold(first.len(), |acc, xs| {
        let distance = dist.for_str(first.as_str(), xs.as_str()).dist();
        std::cmp::min(acc, distance)
    });

    if prefix_len <= 3 {
        // too short to be useful
        return (first.clone(), vec![theme_names]);
    }

    // cool, we have a useful common stem.
    //
    todo!()
}

impl From<VsCodeExtension> for ZedExtension {
    fn from(extension: VsCodeExtension) -> Self {
        let vs_meta = extension.metadata();
        let id = vs_meta.name().to_owned();
        let name = vs_meta.display_name().to_owned();
        let author = vs_meta.publisher().to_owned();
        let description = vs_meta.description().to_owned();
        let repository = vs_meta.repository().to_owned();
        let authors = vec![vs_meta.publisher().to_owned()];

        let incoming = extension.themes();

        // TODO: group themes by name similarity into families
        // using group_families()
        // let names: Vec<String> = incoming.clone().iter().map(|xs| xs.name.clone()).collect();
        //let (extname, groups) = group_families(names);

        let themes: Vec<ZedTheme> = incoming.clone().into_iter().map(|xs| ZedTheme::from(&xs)).collect();
        eprintln!(
            "{} themes converted from {} vscode themes",
            themes.len(),
            incoming.len()
        );

        let family = ZedThemeFamily {
            schema: None,
            author,
            name: name.clone(),
            themes,
        };
        let family_list = vec![format!(
            "./themes/{}",
            ZedExtension::normalize_name(family.name.as_str())
        )];

        let metadata = ZedManifest {
            id,
            name: name.clone(),
            version: "0.1.0".to_string(),
            schema_version: 1,
            description,
            repository,
            authors,
            themes: family_list,
            ..Default::default()
        };
        let directory = ZedExtension::build_official_path(metadata.id.as_str(), EXTENSION_DIR);

        ZedExtension {
            directory: directory.into(),
            name,
            families: vec![family],
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
    fn wont_pass_in_ci() {
        let theme =
            ZedExtension::read("rose-pine-moon.json").expect("expected to read official rose pine moon extension");
        assert!(theme.name().contains("Rosé Pine"));
    }

    #[test]
    fn can_read_from_manifest() {}

    #[ignore = "not implemented yet"]
    #[test]
    fn grouping() {
        let theme_names = vec![
            "Rosé Pine".to_string(),
            "Rosé Pine (no italics)".to_string(),
            "Rosé Pine Moon".to_string(),
            "Rosé Pine Moon (no italics)".to_string(),
            "Rosé Pine Dawn".to_string(),
            "Rosé Pine Dawn (no italics)".to_string(),
        ];

        let (name, grouping) = group_families(theme_names);
        assert_eq!(
            name.as_str(),
            "Rosé Pine",
            "expected theme name to be detected and trimmed"
        );
        assert_eq!(grouping.len(), 3, "expected 3 theme groups");
    }
}
