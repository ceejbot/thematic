//! The metadata for Zed themes is stored in the extension's
//! manifest, extension.toml. We read this file to get useful
//! information about the extension's name and author to populate
//! the VSCode equivalent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZedManifest {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) schema_version: usize,
    pub(crate) description: String,
    pub(crate) repository: String,
    pub(crate) authors: Vec<String>,
    pub(crate) themes: Vec<String>,
    pub(crate) icon_themes: Vec<String>,
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

    pub fn icon_themes(&self) -> &[String] {
        self.icon_themes.as_slice()
    }
}
