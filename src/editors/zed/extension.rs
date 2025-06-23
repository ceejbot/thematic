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
/// from the names of incoming themes using text distance algorithms.
///
/// This approach uses similarity scoring to group themes that are likely variants
/// of the same base theme (e.g., "tweed" and "tweed-contrast" should be grouped together).
fn group_families(theme_names: Vec<String>) -> (Option<String>, Vec<Vec<String>>) {
    let mut name_iter = theme_names.iter();
    let Some(first) = name_iter.next() else {
        return (None, vec![theme_names]);
    };

    if theme_names.len() <= 3 {
        return (None, vec![theme_names]);
    }

    // Find the longest common prefix for extension naming
    let common_prefix = theme_names.iter().fold(first.clone(), |acc, theme| {
        let mut prefix = String::new();
        for (a, b) in acc.chars().zip(theme.chars()) {
            if a == b {
                prefix.push(a);
            } else {
                break;
            }
        }
        prefix
    });

    // Determine extension name based on collection size and common patterns
    let extension_name = if common_prefix.trim().len() > 3 && theme_names.len() < 50 {
        Some(common_prefix.trim().to_string())
    } else if theme_names.len() < 20 {
        Some(first.clone())
    } else {
        None
    };

    // Use distance-based clustering to group similar theme names
    let groups = cluster_themes_by_distance(&theme_names);

    (extension_name, groups)
}

/// Cluster theme names using a hybrid approach that combines word-level and character-level similarity.
/// This handles both cases like "tweed" + "tweed-contrast" and "Rosé Pine" + "Rosé Pine (no italics)".
fn cluster_themes_by_distance(theme_names: &[String]) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut used = vec![false; theme_names.len()];

    for (i, theme) in theme_names.iter().enumerate() {
        if used[i] {
            continue;
        }

        let mut group = vec![theme.clone()];
        used[i] = true;

        // Find all themes similar to this one using hybrid similarity
        for (j, other_theme) in theme_names.iter().enumerate() {
            if i != j && !used[j] {
                if themes_should_be_grouped(theme, other_theme) {
                    group.push(other_theme.clone());
                    used[j] = true;
                }
            }
        }

        // Sort group by name for consistency
        group.sort();
        groups.push(group);
    }

    // Sort groups by the first theme name in each group
    groups.sort_by(|a, b| a[0].cmp(&b[0]));

    groups
}

/// Determine if two themes should be grouped together by comparing their suffixes
/// after removing the common prefix. This prevents grouping "Rose Pine Moon" with
/// "Rose Pine Dawn" while still grouping "Rose Pine" with "Rose Pine (no italics)".
fn themes_should_be_grouped(theme1: &str, theme2: &str) -> bool {
    // Extract base names by removing common suffixes/variants
    let base1 = extract_base_theme_name(theme1);
    let base2 = extract_base_theme_name(theme2);

    // If base names are identical after normalization, they should be grouped
    if base1 == base2 {
        return true;
    }

    // Find the longest common prefix
    let common_prefix = find_common_prefix(theme1, theme2);

    // If there's no meaningful common prefix, don't group
    if common_prefix.trim().len() < 3 {
        return false;
    }

    // Get the suffixes after removing the common prefix
    let suffix1 = theme1[common_prefix.len()..].trim();
    let suffix2 = theme2[common_prefix.len()..].trim();

    // Group if one suffix is empty (base theme) and the other is a variant indicator
    // or if both suffixes are recognized variant indicators
    is_theme_variant_pair(suffix1, suffix2)
}

/// Extract base theme name by removing common variant patterns
fn extract_base_theme_name(name: &str) -> String {
    let mut result = name.to_lowercase();

    // Remove parenthetical variants first
    result = result.replace(" (no italics)", "");
    result = result.replace("(no italics)", "");
    result = result.replace(" - no italics", "");
    result = result.replace(" no italics", "");

    // Remove suffix variants
    result = result.replace("-contrast", "");
    result = result.replace("-light", "");
    result = result.replace("-colorblind", "");
    result = result.replace(" contrast", "");
    result = result.replace(" light", "");
    result = result.replace(" colorblind", "");

    result.trim().to_string()
}

/// Find the longest common prefix between two strings
fn find_common_prefix(s1: &str, s2: &str) -> String {
    let mut prefix = String::new();
    for (c1, c2) in s1.chars().zip(s2.chars()) {
        if c1 == c2 {
            prefix.push(c1);
        } else {
            break;
        }
    }
    prefix
}

/// Determine if two suffixes represent a valid theme variant pair
fn is_theme_variant_pair(suffix1: &str, suffix2: &str) -> bool {
    let s1 = suffix1.to_lowercase();
    let s2 = suffix2.to_lowercase();

    // One is empty (base theme) and the other is a variant
    if (s1.is_empty() && is_variant_suffix(&s2)) || (s2.is_empty() && is_variant_suffix(&s1)) {
        return true;
    }

    // Both are variants of the same type (e.g., both are style variants)
    if is_variant_suffix(&s1) && is_variant_suffix(&s2) {
        // Check if they're the same category of variant
        let s1_category = get_variant_category(&s1);
        let s2_category = get_variant_category(&s2);
        return s1_category == s2_category;
    }

    false
}

/// Check if a suffix represents a theme variant
fn is_variant_suffix(suffix: &str) -> bool {
    let s = suffix.trim().to_lowercase();
    s.starts_with("(no italics)")
        || s.starts_with("- no italics")
        || s.starts_with("no italics")
        || s.starts_with("-contrast")
        || s.starts_with("-light")
        || s.starts_with("-colorblind")
        || s == "(no italics)"
        || s == "- no italics"
        || s == "no italics"
        || s == "-contrast"
        || s == "-light"
        || s == "-colorblind"
}

/// Get the category of a variant (style, brightness, accessibility, etc.)
fn get_variant_category(suffix: &str) -> &str {
    let s = suffix.trim().to_lowercase();
    if s.contains("italic") {
        "style"
    } else if s.contains("light") || s.contains("contrast") {
        "brightness"
    } else if s.contains("colorblind") {
        "accessibility"
    } else {
        "other"
    }
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
        let (_extname, groups) = group_families(theme_names);
        let families: Vec<ZedThemeFamily> = groups
            .iter()
            .map(|name_family| {
                let themes = name_family
                    .iter()
                    .filter_map(|name| theme_map.remove(name).map(|xs| ZedTheme::from(&xs)))
                    .collect();
                let family = ZedThemeFamily {
                    schema: None,
                    author: author.clone(),
                    name: name.clone(), // we should figure out family names
                    themes,
                };
                family
            })
            .collect();

        let family_pointers = families
            .iter()
            .map(|family| format!("./themes/{}", ZedExtension::normalize_name(family.name.as_str())))
            .collect();

        let metadata = ZedManifest {
            id,
            name: name.clone(),
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
            name,
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
    fn wont_pass_in_ci() {
        let theme =
            ZedExtension::read("rose-pine-moon.json").expect("expected to read official rose pine moon extension");
        assert!(theme.name().contains("Rosé Pine"));
    }

    #[test]
    fn test_grouping() {
        let theme_names = vec![
            "Rosé Pine".to_string(),
            "Rosé Pine (no italics)".to_string(),
            "Rosé Pine Moon".to_string(),
            "Rosé Pine Moon (no italics)".to_string(),
            "Rosé Pine Dawn".to_string(),
            "Rosé Pine Dawn (no italics)".to_string(),
        ];

        // Debug output to understand grouping behavior
        println!("Testing Rose Pine grouping with themes: {:?}", theme_names);

        // Test pairwise similarity for debugging
        for (i, theme1) in theme_names.clone().iter().enumerate() {
            for (j, theme2) in theme_names.clone().iter().enumerate() {
                if i < j {
                    let should_group = themes_should_be_grouped(theme1, theme2);
                    let base1 = extract_base_theme_name(theme1);
                    let base2 = extract_base_theme_name(theme2);
                    println!(
                        "  '{}' (base: '{}') vs '{}' (base: '{}'): should_group = {}",
                        theme1, base1, theme2, base2, should_group
                    );
                }
            }
        }

        let (maybe_name, grouping) = group_families(theme_names.clone());

        println!("Rose Pine grouping result:");
        for (i, group) in grouping.iter().enumerate() {
            println!("  Group {}: {:?}", i, group);
        }

        let name = maybe_name.expect("we expected to identify a name");
        assert_eq!(
            name.as_str(),
            "Rosé Pine",
            "expected theme name to be detected and trimmed"
        );
        assert_eq!(grouping.len(), 3, "expected 3 theme groups");

        // Debug output
        println!("Rose Pine grouping result:");
        for (i, group) in grouping.iter().enumerate() {
            println!("  Group {}: {:?}", i, group);
        }

        // Test all pairwise combinations to understand why they're being grouped
        for (i, theme1) in theme_names.iter().enumerate() {
            for (j, theme2) in theme_names.iter().enumerate() {
                if i < j {
                    let should_group = themes_should_be_grouped(theme1, theme2);
                    println!("  '{}' vs '{}': should_group = {}", theme1, theme2, should_group);
                }
            }
        }

        assert_eq!(grouping.len(), 3, "expected 3 theme groups");
    }

    #[test]
    fn distance_based_grouping() {
        let theme_names = vec![
            "tweed".to_string(),
            "tweed-contrast".to_string(),
            "tweed-light".to_string(),
            "tickle".to_string(),
            "tickle-contrast".to_string(),
            "completely-different".to_string(),
        ];

        let (_, groups) = group_families(theme_names);

        // Should have 3 groups: tweed variants, tickle variants, and completely-different
        assert_eq!(groups.len(), 3, "Expected 3 groups, got {}", groups.len());

        // Find the tweed group
        let tweed_group = groups.iter().find(|group| group.contains(&"tweed".to_string()));
        assert!(tweed_group.is_some(), "Should have a group containing 'tweed'");
        let tweed_group = tweed_group.unwrap();
        assert!(tweed_group.contains(&"tweed-contrast".to_string()));
        assert!(tweed_group.contains(&"tweed-light".to_string()));

        // Find the tickle group
        let tickle_group = groups.iter().find(|group| group.contains(&"tickle".to_string()));
        assert!(tickle_group.is_some(), "Should have a group containing 'tickle'");
        let tickle_group = tickle_group.unwrap();
        assert!(tickle_group.contains(&"tickle-contrast".to_string()));

        // The completely different theme should be in its own group
        let different_group = groups
            .iter()
            .find(|group| group.contains(&"completely-different".to_string()));
        assert!(
            different_group.is_some(),
            "Should have a group containing 'completely-different'"
        );
        assert_eq!(different_group.unwrap().len(), 1, "Should be in its own group");
    }

    #[test]
    fn grouping_worst_case() {
        // There are 325+ themes in Rainglow, and they are mostly in light/dark pairs.
        // We want all of the pairs to be grouped into families.

        let found = VsCodeExtension::find_from_name("rainglow", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Rainglow");
        assert_eq!(found.name(), "Rainglow");

        // Get theme names before consuming the extension
        let theme_names: Vec<String> = found.themes().iter().map(|theme| theme.name.clone()).collect();
        assert!(theme_names.len() >= 325, "Expected at least 325 themes");
        eprintln!("{theme_names:#?}");

        let (maybe_name, family_groups) = group_families(theme_names);
        eprintln!("{maybe_name:#?}");

        assert!(!family_groups.is_empty());
        assert!(family_groups.len() > 50, "how many DO we have?");
        eprintln!("groups: {}", family_groups.len());
        let first_family = family_groups
            .first()
            .expect("we said we had more than one, for pete's sake");
        assert!(!first_family.is_empty());
        assert!(
            first_family.len() >= 3,
            "Expected first family to have at least 3 themes (base + variants), got {}",
            first_family.len()
        );

        // TODO: Add more assertions about family groupings
    }
}
