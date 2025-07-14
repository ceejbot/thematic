//! Read and write Zed theme extensions.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::editors::{Extension, ThemeFile};
use crate::vscode::VsCodeExtension;
use crate::{
    IconFileManager, InstalledExtensions, ThemeError, VsCodeTheme, ZedIconTheme, ZedIconThemeFamily, ZedManifest,
    ZedTheme, ZedThemeFamily,
};

/// Platform-specific Zed extension directory paths
fn get_zed_extension_dir() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Library/Application Support/Zed/extensions/installed"
    }
    #[cfg(target_os = "linux")]
    {
        ".config/zed/extensions/installed"
    }
    #[cfg(target_os = "windows")]
    {
        "AppData/Roaming/Zed/extensions/installed"
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        ".config/zed/extensions/installed" // Default to Linux-style for other Unix-like systems
    }
}

/// Our representation of a Zed theme extension.
#[derive(Debug, Clone)]
pub struct ZedExtension {
    /// Where this extension resides on disk. Pub(crate) for testing.
    pub(crate) directory: PathBuf,
    /// Source directory for copying assets (e.g., icon files) during conversion
    source_directory: Option<PathBuf>,
    /// A portion of the toml extension manifest.
    metadata: ZedManifest,
    /// The human name of the extension.
    name: String,
    /// All the theme families this extension provides.
    families: Vec<ZedThemeFamily>,
    /// All the icon themes this extension provides.
    icon_themes: Vec<ZedIconTheme>,
    /// All themes flat-mapped
    all_themes: Vec<ZedTheme>,
}

impl ZedExtension {
    /// Add the passed-in extension to Zed's list of installed extensions.
    pub fn add_installed(extension: &ZedExtension) -> Result<(), ThemeError> {
        let mut fpath = PathBuf::new();
        fpath.push(ZedExtension::extensions_path());
        fpath.pop();
        fpath.push("index.json");
        let mut installed = InstalledExtensions::new(&fpath)?;
        installed.add_extension(extension);
        log::debug!("Writing out new Zed extensions manifest.");
        installed.write_to(&fpath)?;

        Ok(())
    }

    pub fn find_from_name(name: &str, extdir: &str) -> Result<Box<Self>, ThemeError> {
        let barename = name.replace(".json", "");
        let slugged_name = slug::slugify(&barename);

        // First, try to find extension.toml files in directories that match the name

        let globby = format!("{extdir}/*/extension.toml");
        let matches = glob::glob(globby.as_str())?;

        for toml_path in matches.flatten() {
            if let Some(dir_name) = toml_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
                // Check if directory name contains our search term
                if dir_name.contains(&slugged_name) || dir_name.contains(&barename) {
                    return ZedExtension::read_from_path(&toml_path, barename);
                }
            }
        }

        // If no extension.toml found, fall back to looking for individual JSON theme files
        let extname = ZedExtension::normalize_name(name);
        let globby = format!("{extdir}/**/{extname}");

        let mut matches = glob::glob(globby.as_str())?;
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

    /// All icon themes
    pub fn icon_themes(&self) -> &[ZedIconTheme] {
        self.icon_themes.as_slice()
    }

    pub fn read_from_path(extpath: &PathBuf, barename: String) -> Result<Box<Self>, ThemeError> {
        if extpath.ends_with("extension.toml") {
            let contents = std::fs::read_to_string(extpath)?;
            let metadata: ZedManifest = toml::from_str(contents.as_str())?;
            let themepath = extpath.clone();
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

        // Discover icon themes in the extension directory
        let icon_themes_dir = extpath.parent().unwrap_or(extpath).join("icon_themes");
        let mut icon_themes_paths = Vec::new();
        let mut discovered_icon_themes = Vec::new();

        if icon_themes_dir.exists() {
            // Look for icon theme JSON files
            let icon_theme_files = std::fs::read_dir(&icon_themes_dir)
                .map_err(ThemeError::IoError)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("json"))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();

            for entry in icon_theme_files {
                let icon_theme_path = entry.path();
                if let Ok(icon_theme_family) = ZedIconThemeFamily::read(&icon_theme_path) {
                    // Add to manifest paths
                    let relative_path = format!(
                        "./icon_themes/{}",
                        icon_theme_path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("icon-theme.json")
                    );

                    icon_themes_paths.push(relative_path);
                    discovered_icon_themes.extend(icon_theme_family.themes);
                }
            }
        }

        let all_themes = family.themes.clone();

        let metadata = ZedManifest {
            id,
            name: family.name.clone(),
            version: "0.1.0".to_string(),
            description: "Constructed from a directory of color themes".to_string(),
            themes,
            icon_themes: icon_themes_paths,
            ..Default::default()
        };

        let extension = Self {
            name: family.name.clone(),
            directory: extpath.clone(),
            source_directory: None,
            families: vec![family],
            icon_themes: discovered_icon_themes,
            all_themes,
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

    pub fn from_metadata<P: AsRef<Path>>(found: P, metadata: ZedManifest) -> Result<Box<ZedExtension>, ThemeError> {
        let mut extpath = PathBuf::new();
        extpath.push(found);
        if !extpath.is_dir() {
            extpath.pop();
        }
        if extpath.ends_with("themes") || extpath.ends_with("icon_themes") {
            extpath.pop();
        }
        let families: Vec<ZedThemeFamily> = metadata
            .themes()
            .iter()
            .filter_map(|xs| {
                let mut family_file = extpath.clone();
                family_file.push(xs);
                ZedThemeFamily::read(&family_file).ok()
            })
            .collect();
        // lazy clone
        let all_themes = families.iter().flat_map(|family| family.themes.clone()).collect();

        let icon_themes: Vec<ZedIconTheme> = metadata
            .icon_themes
            .iter()
            .filter_map(|xs| {
                let mut itheme = extpath.clone();
                itheme.push(xs);
                ZedIconThemeFamily::read(&itheme).ok()
            })
            .flat_map(|family| family.themes)
            .collect();

        let extension = ZedExtension {
            name: metadata.name.clone(),
            directory: extpath,
            source_directory: None,
            metadata,
            families,
            icon_themes,
            all_themes,
        };
        Ok(Box::new(extension))
    }

    pub fn make_manifest_glob(pattern: &str) -> String {
        let safer = slug::slugify(pattern);
        format!("**/*{safer}*/extension.toml")
    }

    pub fn make_themefile_glob(pattern: &str) -> String {
        let safer = slug::slugify(pattern);
        format!("**/*{safer}*.json")
    }
}

impl Extension for ZedExtension {
    type ThemeType = ZedTheme;
    type IconThemeType = ZedIconTheme;
    type Manifest = ZedManifest;

    fn read<P: AsRef<Path>>(extpath: P) -> Result<Box<Self>, ThemeError> {
        let contents = std::fs::read_to_string(&extpath)?;
        let metadata: ZedManifest = toml::from_str(contents.as_str())?;
        ZedExtension::from_metadata(extpath, metadata)
    }

    fn write(&self) -> Result<(), ThemeError> {
        self.write_to(self.official_path())
    }

    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError> {
        let mut workdir = PathBuf::new();
        workdir.push(&path);
        workdir.push("themes");
        std::fs::create_dir_all(&workdir)?;

        for family in self.families.as_slice() {
            let mut filename = workdir.clone();
            let themefile = ZedExtension::normalize_name(&family.name);
            filename.push(themefile);
            family.write_to(filename)?;
        }

        // Write icon themes if any exist
        if !self.icon_themes.is_empty() {
            workdir.pop();
            workdir.push("icon_themes");
            std::fs::create_dir_all(&workdir)?;

            for icon_theme in self.icon_themes.as_slice() {
                let mut filename = workdir.clone();
                let icon_theme_file = ZedExtension::normalize_name(&icon_theme.name);
                filename.push(icon_theme_file);

                // Copy icon files if they exist in the source directory
                let source_dir = self.source_directory.as_ref().unwrap_or(&self.directory);
                let dest_dir = path.as_ref().to_path_buf();

                // Create an IconFileManager for this icon theme
                let mut icon_manager = IconFileManager::new(source_dir, &dest_dir, "icons");

                // Track all icons from this theme with source base path
                icon_theme.track_icons_with_base(&mut icon_manager, Some(source_dir))?;

                // Create a ZedIconThemeFamily to wrap the individual theme
                let author = self
                    .metadata
                    .authors()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());

                let theme_to_write = if !icon_manager.tracked_icons().is_empty() {
                    // Try to copy icons - if it fails, we'll still write the theme JSON
                    if let Err(e) = icon_manager.copy_icons() {
                        log::warn!("Failed to copy icon files for theme '{}': {}", icon_theme.name, e);
                        // Use the theme as-is without updated paths
                        icon_theme.clone()
                    } else {
                        // Create a mapping of old paths to new paths
                        let mut path_mapping = HashMap::new();
                        for icon_path in icon_theme.get_icon_paths() {
                            if let Some(filename_str) = IconFileManager::extract_filename(&icon_path) {
                                let new_path = icon_manager.get_relative_path(&filename_str);
                                path_mapping.insert(icon_path, new_path);
                            }
                        }

                        // Update the icon theme with new paths
                        let mut updated_theme = icon_theme.clone();
                        updated_theme.update_icon_paths(&path_mapping);

                        log::debug!(
                            "Copied {} icon files for theme '{}'",
                            icon_manager.tracked_icons().len(),
                            icon_theme.name
                        );

                        updated_theme
                    }
                } else {
                    // No icons to copy, use the theme as-is
                    icon_theme.clone()
                };

                // Create a ZedIconThemeFamily wrapper
                let icon_family = crate::editors::zed::icon_theme::ZedIconThemeFamily {
                    schema: Some("https://zed.dev/schema/icon_themes/v0.2.0.json".to_string()),
                    name: icon_theme.name.clone(),
                    author,
                    themes: vec![theme_to_write],
                    source_path: None,
                };

                // Write the icon theme family
                icon_family.write(filename)?;
            }
        }

        workdir.pop();
        let tomlstr = toml::to_string_pretty(self.manifest())?;
        workdir.push("extension.toml");
        let mut fp = std::fs::File::create(&workdir)?;
        fp.write_all(tomlstr.as_bytes())?;
        log::info!("Wrote Zed extension '{}' to {}", self.name, self.directory.display());
        Ok(())
    }

    fn extensions_path() -> String {
        let twiddle = home::home_dir().unwrap_or_default();
        format!("{}/{}", twiddle.display(), get_zed_extension_dir())
    }

    fn build_official_path(name: &str, extdir: &str) -> String {
        format!("{extdir}/{name}")
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    /// All themes as slice
    fn themes(&self) -> &[Self::ThemeType] {
        self.all_themes.as_slice()
    }

    /// Icon themes as slice
    fn icon_themes(&self) -> &[ZedIconTheme] {
        self.icon_themes.as_slice()
    }

    /// Where this extension should live, or was read from
    fn official_path(&self) -> &PathBuf {
        &self.directory
    }

    /// The editor's manifest for this theme.
    fn manifest(&self) -> &Self::Manifest {
        &self.metadata
    }

    fn search(pattern: &str) -> Result<Vec<Box<Self>>, ThemeError> {
        let extensionfile_glob = ZedExtension::make_manifest_glob(pattern);
        let matches = crate::globdir(extensionfile_glob.as_str(), ZedExtension::extensions_path().as_str());
        let pile: Vec<_> = matches
            .iter()
            .filter_map(|xs| ZedExtension::read(xs).ok())
            .filter(|xs| !xs.manifest().icon_themes().is_empty() || !xs.manifest().themes().is_empty())
            .collect();

        if !pile.is_empty() {
            return Ok(pile);
        }

        // If no extension.toml found, fall back to looking for individual JSON theme files
        let themefile_glob = ZedExtension::make_themefile_glob(pattern);
        let matches = crate::globdir(themefile_glob.as_str(), ZedExtension::extensions_path().as_str());
        let pile: Vec<_> = matches
            .iter()
            .filter_map(|xs| {
                eprintln!("{themefile_glob}");

                let mut xpath = xs.clone();
                xpath.pop();
                xpath.pop();
                xpath.push("extension.toml");
                eprintln!("{xpath:#?}");
                match ZedExtension::read(&xpath) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        eprintln!("{e:#?}");
                        None
                    }
                }
            })
            .collect();

        Ok(pile)
    }
}

impl From<VsCodeExtension> for ZedExtension {
    fn from(extension: VsCodeExtension) -> Self {
        let vs_meta = extension.manifest();
        let id = vs_meta.name().to_owned();
        let display_name = vs_meta.display_name().to_owned();
        let author = vs_meta.publisher().to_owned();
        let description = vs_meta.description().to_owned();
        let repository = vs_meta.repository().to_owned();
        let authors = vec![vs_meta.publisher().to_owned()];

        let incoming = extension.themes();
        let all_themes = incoming.iter().map(ZedTheme::from).collect();

        // Now we do our first clever thing. We group themes by name similarity
        // into Zed theme families. We then convert by family.
        let mut theme_map: HashMap<String, VsCodeTheme> = HashMap::new();
        let theme_names: Vec<String> = incoming
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
            .filter_map(|(maybe_name, name_family)| {
                let themes: Vec<ZedTheme> = name_family
                    .iter()
                    .filter_map(|name| theme_map.remove(name).map(|xs| ZedTheme::from(&xs)))
                    .collect();

                // Skip empty theme families (happens with icon-only extensions)
                if themes.is_empty() {
                    return None;
                }

                let fam_name = if let Some(n) = maybe_name {
                    n.clone()
                } else {
                    themes[0].name.clone()
                };
                Some(ZedThemeFamily {
                    schema: None,
                    author: author.clone(),
                    name: fam_name,
                    themes,
                })
            })
            .collect();

        let family_pointers = families
            .iter()
            .map(|family| format!("./themes/{}", ZedExtension::normalize_name(family.name.as_str())))
            .collect();

        let icon_themes: Vec<_> = extension
            .icon_themes()
            .iter()
            .enumerate()
            .map(|(i, vscode_icon_theme)| {
                // Try to get the label from metadata, fallback to a default name
                let name = vs_meta
                    .icon_themes()
                    .get(i)
                    .map(|meta| meta.label.as_str())
                    .unwrap_or("Converted Icon Theme");
                ZedIconTheme::from_vscode_with_name(vscode_icon_theme, name)
            })
            .collect();
        let icon_theme_pointers = icon_themes
            .iter()
            .map(|itheme| format!("./icon_themes/{}", ZedExtension::normalize_name(itheme.name.as_str())))
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
            icon_themes: icon_theme_pointers,
            ..Default::default()
        };
        // Create proper Zed extension directory path
        let zed_extension_name = slug::slugify(&display_name);
        let zed_directory = PathBuf::from(ZedExtension::build_official_path(
            &zed_extension_name,
            &ZedExtension::extensions_path(),
        ));

        ZedExtension {
            directory: zed_directory,
            source_directory: Some(extension.official_path().clone()),
            name: display_name,
            families,
            metadata,
            icon_themes,
            all_themes,
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
        let directory = ZedExtension::build_official_path(theme_filename.as_str(), &ZedExtension::extensions_path());
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
        let all_themes = family.themes.clone();

        ZedExtension {
            name: vscode_theme.name.clone(),
            directory: directory.into(),
            source_directory: None,
            metadata,
            families: vec![family],
            all_themes,
            icon_themes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_specific_paths() {
        // Test that we get reasonable platform-specific paths
        let extensions_path = ZedExtension::extensions_path();
        assert!(extensions_path.contains("extensions/installed"));

        // Verify the path contains platform-appropriate directory separators
        #[cfg(target_os = "macos")]
        assert!(extensions_path.contains("Library/Application Support/Zed"));

        #[cfg(target_os = "linux")]
        assert!(extensions_path.contains(".config/zed"));

        #[cfg(target_os = "windows")]
        assert!(extensions_path.contains("AppData/Roaming/Zed"));

        // Verify that the path is non-empty and absolute-like
        assert!(!extensions_path.is_empty());
    }

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
        let candidates = ZedExtension::search("rainglow").expect("expected to find exactly one rainglow extensions");
        assert_eq!(candidates.len(), 1, "expected exactly one");
        let theme = candidates
            .first()
            .expect("first() in a non-empty list should not be None");
        assert!(theme.name().contains("Rainglow"));

        let candidates =
            ZedExtension::search("coffee").expect("expected to find possible extension containing a coffee theme");
        assert!(!candidates.is_empty(), "expected at least one");
        let theme = candidates
            .first()
            .expect("first() in a non-empty list should not be None");
        assert!(theme.name().contains("Rainglow"));
    }

    #[test]
    fn test_icon_file_copying_during_extension_write() {
        use std::collections::HashMap;
        use std::fs;

        use tempfile::TempDir;

        // Create temporary directories
        let source_dir = TempDir::new().expect("Failed to create temp source dir");
        let dest_dir = TempDir::new().expect("Failed to create temp dest dir");

        // Create source icon files
        let icons_dir = source_dir.path().join("icons");
        fs::create_dir_all(&icons_dir).expect("Failed to create icons directory");

        let file_icon_content = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect fill="blue"/></svg>"#;
        let folder_icon_content = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect fill="yellow"/></svg>"#;
        let folder_open_content = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect fill="green"/></svg>"#;

        fs::write(icons_dir.join("file.svg"), file_icon_content).expect("Failed to write file icon");
        fs::write(icons_dir.join("folder.svg"), folder_icon_content).expect("Failed to write folder icon");
        fs::write(icons_dir.join("folder-open.svg"), folder_open_content).expect("Failed to write folder-open icon");

        // Create Zed icon theme with file icons
        let mut file_icons = HashMap::new();
        file_icons.insert(
            "default".to_string(),
            crate::editors::zed::icon_theme::FileIcon {
                path: "./icons/file.svg".to_string(),
            },
        );

        let zed_icon_theme = ZedIconTheme {
            name: "Test Icon Theme".to_string(),
            appearance: "dark".to_string(),
            directory_icons: Some(crate::editors::zed::icon_theme::DirectoryIcons {
                collapsed: "./icons/folder.svg".to_string(),
                expanded: "./icons/folder-open.svg".to_string(),
            }),
            file_stems: None,
            file_suffixes: None,
            file_icons: Some(file_icons),
        };

        // Create Zed extension with icon theme
        let metadata = ZedManifest {
            id: "test-extension".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            schema_version: 1,
            description: "Test extension for icon copying".to_string(),
            repository: "".to_string(),
            authors: vec!["test".to_string()],
            themes: Vec::new(),
            icon_themes: vec!["./icon_themes/test-icon-theme.json".to_string()],
            ..Default::default()
        };

        let zed_extension = ZedExtension {
            directory: source_dir.path().to_path_buf(),
            source_directory: None,
            name: "Test Extension".to_string(),
            families: Vec::new(),
            icon_themes: vec![zed_icon_theme],
            all_themes: Vec::new(),
            metadata,
        };

        // Write the Zed extension
        zed_extension
            .write_to(dest_dir.path())
            .expect("Failed to write Zed extension");

        // Verify that icon files were copied
        let dest_icons_dir = dest_dir.path().join("icons");
        assert!(dest_icons_dir.exists(), "Icons directory should be created");
        assert!(dest_icons_dir.join("file.svg").exists(), "File icon should be copied");
        assert!(
            dest_icons_dir.join("folder.svg").exists(),
            "Folder icon should be copied"
        );
        assert!(
            dest_icons_dir.join("folder-open.svg").exists(),
            "Folder-open icon should be copied"
        );

        // Verify icon file contents
        let copied_file_content =
            fs::read_to_string(dest_icons_dir.join("file.svg")).expect("Failed to read copied file icon");
        assert_eq!(copied_file_content, file_icon_content, "File icon content should match");

        // Verify that the icon theme JSON was written
        let icon_theme_path = dest_dir.path().join("icon_themes").join("test-icon-theme.json");
        assert!(icon_theme_path.exists(), "Icon theme JSON should be written");

        // Verify that icon paths in the theme JSON were updated to point to ./icons/
        let theme_content = fs::read_to_string(&icon_theme_path).expect("Failed to read icon theme JSON");
        assert!(
            theme_content.contains("./icons/"),
            "Icon paths should be updated to ./icons/"
        );

        println!("✅ Icon file copying test passed successfully!");
    }
}
