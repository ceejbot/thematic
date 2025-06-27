pub mod vscode;
pub mod zed;

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
pub use vscode::*;
pub use zed::*;

use crate::ThemeError;

pub trait Extension {
    type ThemeType;
    type IconThemeType;
    type Manifest;

    /// Attempt to find and read the theme from its name.
    fn read(name: &str) -> Result<Box<Self>, ThemeError>;
    /// Write this theme extension to the default place its editor expects it.
    fn write(&self) -> Result<(), ThemeError>;
    /// Write the theme to a specific directory.
    fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError>;
    /// The official extensions path for this editor.
    fn extensions_path() -> String;
    /// The human name of this extension (as opposed to theme).
    fn name(&self) -> &str;
    /// Get the extension's metadata
    fn manifest(&self) -> &Self::Manifest;
    /// Get all themes associated with this extension.
    fn themes(&self) -> &[Self::ThemeType];
    /// Get all icon themes associated with this extension.
    fn icon_themes(&self) -> &[Self::IconThemeType];
    /// Where this extension is stored, or should be stored.
    fn official_path(&self) -> &PathBuf;
    /// Construct the path this editor type expects to find this extension in, from the name only.
    fn build_official_path(name: &str, extdir: &str) -> String;
}

pub trait ThemeFile {
    type T;

    fn read<P: AsRef<Path>>(path: P) -> Result<Self::T, ThemeError>;
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ThemeError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self::T, ThemeError>;
}

// text utilities

static PUNCT_PATT: LazyLock<Regex> =
    LazyLock::new(|| regex::Regex::new("[[:punct:]]").expect("this regex better be good"));

pub fn find_extension_name(theme_names: &[String]) -> Option<String> {
    // Find the longest common prefix for extension naming
    let mut name_iter = theme_names.iter();
    let first = name_iter.next()?;

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
    if common_prefix.trim().len() > 3 && theme_names.len() < 50 {
        Some(common_prefix.trim().to_string())
    } else if theme_names.len() < 20 {
        Some(first.clone())
    } else {
        None
    }
}

/// Figure out the best extension name and the best family groups, if possible,
/// from the names of incoming themes using simple prefix examination.
///
/// Returns a vector of tuples of the name of a family and the names of themes that belong
/// to that family. If it cannot determine a good name for the family, it returns None.
pub fn group_families(theme_names: Vec<String>) -> Vec<(Option<String>, Vec<String>)> {
    if theme_names.len() <= 3 {
        return vec![(None, theme_names)];
    }

    let groups = cluster_theme_names(&theme_names);

    let mut results = Vec::new();
    for group in groups {
        let extension_name = find_extension_name(group.as_slice());
        results.push((extension_name, group));
    }

    results
}

/// Cluster theme names using a hybrid approach that combines word-level and character-level similarity.
/// This handles both cases like "tweed" + "tweed-contrast" and "Rosé Pine" + "Rosé Pine (no italics)".
fn cluster_theme_names(theme_names: &[String]) -> Vec<Vec<String>> {
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
            if i != j && !used[j] && themes_should_be_grouped(theme, other_theme) {
                group.push(other_theme.clone());
                used[j] = true;
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
/// Unlike Claude, I do the simplest, stupidest thing in the world
/// by removing everything after the first non-word punctuation.
fn extract_base_theme_name(name: &str) -> String {
    let mut splits = PUNCT_PATT.splitn(name, 2);
    if let Some(first) = splits.next() {
        return first.trim().to_string();
    }

    name.trim().to_string()
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
    PUNCT_PATT.is_match(s.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;
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

        let family_name_pairs = group_families(theme_names.clone());
        assert_eq!(family_name_pairs.len(), 3, "expected 3 theme groups");

        println!("Rose Pine grouping result:");
        for (i, group) in family_name_pairs.iter().enumerate() {
            println!("  Group {}: {:#?} {:#?}", i, group.0, group.1);
        }

        let maybe_name = find_extension_name(theme_names.as_slice());
        let name = maybe_name.expect("we expected to identify a name");
        assert_eq!(
            name.as_str(),
            "Rosé Pine",
            "expected theme name to be detected and trimmed"
        );

        // Test all pairwise combinations to understand why they're being grouped
        for (i, theme1) in theme_names.iter().enumerate() {
            for (j, theme2) in theme_names.iter().enumerate() {
                if i < j {
                    let should_group = themes_should_be_grouped(theme1, theme2);
                    println!("  '{}' vs '{}': should_group = {}", theme1, theme2, should_group);
                }
            }
        }
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

        let family_name_pairs = group_families(theme_names);

        // Should have 3 groups: tweed variants, tickle variants, and completely-different
        assert_eq!(
            family_name_pairs.len(),
            3,
            "Expected 3 groups, got {}",
            family_name_pairs.len()
        );

        // Find the tweed group
        let found = family_name_pairs.iter().find(|(maybe_name, _families)| {
            if let Some(name) = maybe_name {
                name.contains(&"tweed".to_string())
            } else {
                false
            }
        });
        assert!(found.is_some(), "Should have a group containing 'tweed'");
        let (_maybe_name, tweed_group) = found.unwrap();
        assert!(tweed_group.contains(&"tweed-contrast".to_string()));
        assert!(tweed_group.contains(&"tweed-light".to_string()));

        // Find the tickle group
        let found = family_name_pairs.iter().find(|(maybe_name, _families)| {
            if let Some(name) = maybe_name {
                name.contains(&"tickle".to_string())
            } else {
                false
            }
        });
        assert!(found.is_some(), "Should have a group containing 'tickle'");
        let (_maybe_name, tickle_group) = found.unwrap();
        assert!(tickle_group.contains(&"tickle-contrast".to_string()));

        // The completely different theme should be in its own group
        let found = family_name_pairs.iter().find(|(maybe_name, _families)| {
            if let Some(name) = maybe_name {
                name.contains(&"completely-different".to_string())
            } else {
                false
            }
        });
        let (_maybe_name, and_now) = found.expect("Should have a group containing 'completely-different'");
        assert_eq!(and_now.len(), 1, "Should be in its own group");
    }

    #[test]
    fn no_ci_grouping_worst_case() {
        // There are 325+ themes in Rainglow, and they are mostly in light/dark pairs.
        // We want all of the pairs to be grouped into families.

        let found = VsCodeExtension::find_from_name("rainglow", VsCodeExtension::extensions_path().as_str())
            .expect("failed to find Rainglow");
        assert_eq!(found.name(), "Rainglow");

        // Get theme names before consuming the extension
        let theme_names: Vec<String> = found.themes().iter().map(|theme| theme.name.clone()).collect();
        assert!(theme_names.len() >= 325, "Expected at least 325 themes");

        let family_name_pairs = group_families(theme_names);
        assert_eq!(family_name_pairs.len(), 109);

        let (_maybe_first_name, first_family) = family_name_pairs
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

    #[test]
    fn extension_name_extraction() {
        let theme_names = vec![
            "Rosé Pine".to_string(),
            "Rosé Pine (no italics)".to_string(),
            "Rosé Pine Moon".to_string(),
            "Rosé Pine Moon (no italics)".to_string(),
            "Rosé Pine Dawn".to_string(),
            "Rosé Pine Dawn (no italics)".to_string(),
        ];
        let extname = find_extension_name(theme_names.as_slice()).expect("we should find a name for Rosé Pine");
        assert_eq!(extname, "Rosé Pine", "Rosé Pine really should be called that");
    }
}
