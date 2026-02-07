//! The metadata for Zed themes is stored in the extension's
//! manifest, extension.toml. We read this file to get useful
//! information about the extension's name and author to populate
//! the VSCode equivalent.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(default)]
    pub(crate) languages: Vec<String>,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) lib: Library,
    #[serde(default)]
    pub(crate) grammars: HashMap<String, Grammar>,
    #[serde(default)]
    pub(crate) language_servers: serde_json::Value,
    #[serde(default)]
    pub(crate) context_servers: serde_json::Value,
    #[serde(default)]
    pub(crate) slash_commands: serde_json::Value,
    #[serde(default)]
    pub(crate) indexed_docs_providers: serde_json::Value,
    #[serde(default)]
    pub(crate) snippets: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct Library {
    kind: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct Grammar {
    repository: String,
    rev: String,
    path: Option<String>,
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
            languages: Vec::new(),
            capabilities: Vec::new(),
            lib: Library {
                kind: None,
                version: None,
            },
            grammars: HashMap::new(),
            language_servers: serde_json::Value::Object(serde_json::Map::new()),
            context_servers: serde_json::Value::Object(serde_json::Map::new()),
            slash_commands: serde_json::Value::Object(serde_json::Map::new()),
            indexed_docs_providers: serde_json::Value::Object(serde_json::Map::new()),
            snippets: None,
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
