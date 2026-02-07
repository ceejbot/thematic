//! Theme name grouping and family detection.
//!
//! Groups theme names into families using prefix analysis and variant
//! detection. For example, "Rosé Pine", "Rosé Pine Moon", and "Rosé Pine Dawn"
//! form three separate families, while "Rosé Pine" and "Rosé Pine (no italics)"
//! are grouped together.

use std::sync::LazyLock;

use regex::Regex;

static PUNCT_PATT: LazyLock<Regex> =
    LazyLock::new(|| regex::Regex::new("[[:punct:]]").expect("this regex better be good"));

/// Find the best extension name from a set of theme names using common prefix
/// analysis.
pub fn find_extension_name(theme_names: &[String]) -> Option<String> {
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

    if common_prefix.trim().len() > 3 && theme_names.len() < 50 {
        Some(common_prefix.trim().to_string())
    } else if theme_names.len() < 20 {
        Some(first.clone())
    } else {
        None
    }
}

/// Group theme names into families using prefix analysis.
///
/// Returns a vector of tuples of the name of a family and the names of themes
/// that belong to that family. If it cannot determine a good name for the
/// family, it returns None.
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

/// Cluster theme names using a hybrid approach that combines word-level and
/// character-level similarity. This handles both cases like "tweed" +
/// "tweed-contrast" and "Rosé Pine" + "Rosé Pine (no italics)".
fn cluster_theme_names(theme_names: &[String]) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut used = vec![false; theme_names.len()];

    for (i, theme) in theme_names.iter().enumerate() {
        if used[i] {
            continue;
        }

        let mut group = vec![theme.clone()];
        used[i] = true;

        for (j, other_theme) in theme_names.iter().enumerate() {
            if i != j && !used[j] && themes_should_be_grouped(theme, other_theme) {
                group.push(other_theme.clone());
                used[j] = true;
            }
        }

        group.sort();
        groups.push(group);
    }

    groups.sort_by(|a, b| a[0].cmp(&b[0]));
    groups
}

/// Determine if two themes should be grouped together by comparing their
/// suffixes after removing the common prefix.
fn themes_should_be_grouped(theme1: &str, theme2: &str) -> bool {
    let base1 = extract_base_theme_name(theme1);
    let base2 = extract_base_theme_name(theme2);

    if base1 == base2 {
        return true;
    }

    let common_prefix = find_common_prefix(theme1, theme2);

    if common_prefix.trim().len() < 3 {
        return false;
    }

    let suffix1 = theme1[common_prefix.len()..].trim();
    let suffix2 = theme2[common_prefix.len()..].trim();

    is_theme_variant_pair(suffix1, suffix2)
}

/// Extract base theme name by removing everything after the first non-word
/// punctuation.
fn extract_base_theme_name(name: &str) -> String {
    let mut splits = PUNCT_PATT.splitn(name, 2);
    if let Some(first) = splits.next() {
        return first.trim().to_string();
    }

    name.trim().to_string()
}

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

fn is_theme_variant_pair(suffix1: &str, suffix2: &str) -> bool {
    let s1 = suffix1.to_lowercase();
    let s2 = suffix2.to_lowercase();

    if (s1.is_empty() && is_variant_suffix(&s2)) || (s2.is_empty() && is_variant_suffix(&s1)) {
        return true;
    }

    if is_variant_suffix(&s1) && is_variant_suffix(&s2) {
        let s1_category = get_variant_category(&s1);
        let s2_category = get_variant_category(&s2);
        return s1_category == s2_category;
    }

    false
}

fn is_variant_suffix(suffix: &str) -> bool {
    let s = suffix.trim().to_lowercase();
    PUNCT_PATT.is_match(s.as_str())
}

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

        let family_name_pairs = group_families(theme_names.clone());
        assert_eq!(family_name_pairs.len(), 3, "expected 3 theme groups");

        let maybe_name = find_extension_name(theme_names.as_slice());
        let name = maybe_name.expect("we expected to identify a name");
        assert_eq!(
            name.as_str(),
            "Rosé Pine",
            "expected theme name to be detected and trimmed"
        );
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

        assert_eq!(
            family_name_pairs.len(),
            3,
            "Expected 3 groups, got {}",
            family_name_pairs.len()
        );

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
