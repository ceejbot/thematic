//! Serialization, deserialization, and field accessors for vscode's theme
//! package files, in many variations.

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VsCodePackageJson {
    pub(crate) name: String,
    #[serde(rename = "displayName")]
    pub(crate) display_name: String,
    pub(crate) description: String,
    pub(crate) publisher: String,
    pub(crate) contributes: Contributions,
    #[serde(deserialize_with = "deserialize_repository", default)]
    pub(crate) repository: Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Repository {
    pub(crate) url: String,
}

/// Some extensions have a bare string here.
fn deserialize_repository<'de, D>(deserializer: D) -> Result<Repository, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let value: Option<Value> = Option::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(Repository::default());
    };

    if let Ok(repo) = Repository::deserialize(&value) {
        return Ok(repo);
    }
    if let Ok(url) = String::deserialize(&value) {
        return Ok(Repository { url });
    }

    Err(Error::custom("unknown format for repository field"))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contributions {
    // icon_themes: Vec<IconThemePointer>,
    pub(crate) themes: Vec<ThemePointer>,
}

impl VsCodePackageJson {
    pub fn new(name: &str, display_name: &str, description: &str, publisher: &str, themes: Vec<String>) -> Self {
        let regular_themes = themes.iter().map(|_xs| todo!()).collect();
        let contributes = Contributions {
            themes: regular_themes,
            // icon_themes: Vec::new(),
        };
        Self {
            name: name.to_owned(),
            display_name: display_name.to_owned(),
            description: description.to_owned(),
            publisher: publisher.to_owned(),
            contributes,
            repository: Repository { url: String::default() },
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn display_name(&self) -> &str {
        self.display_name.as_str()
    }
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
    pub fn publisher(&self) -> &str {
        self.publisher.as_str()
    }
    pub fn repository(&self) -> &str {
        self.repository.url.as_str()
    }
    pub fn themes(&self) -> &[ThemePointer] {
        self.contributes.themes.as_slice()
    }
    /* pub fn icon_themes(&self) -> &[IconThemePointer] {
        self.contributes.icon_themes.as_slice()
    } */
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemePointer {
    /// The slug name for this theme variation.
    pub(crate) label: String,
    /// regular or dark flavored
    pub(crate) ui_theme: String,
    /// The relative path to the file where the theme data is. Eg., ./themes/label.json
    pub(crate) path: String,
}

/// Unused at the moment, but at some point we'll start converting icon themes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IconThemePointer {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) path: String,
}
