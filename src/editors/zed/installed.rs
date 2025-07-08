//! Zed keeps a list of installed extensions. When we port a theme,
//! we need to add it to this list so it becomes available in Zed to
//! be used.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{Extension, ThemeFile, ZedExtension, ZedManifest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledExtensions {
    #[serde(flatten)]
    value: HashMap<String, ZedManifest>,
}

impl InstalledExtensions {
    pub fn add_extension(&mut self, extension: &ZedExtension) {
        let id = extension.manifest().id().to_string();
        let manifest = extension.manifest().clone();
        self.value.insert(id, manifest);
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, crate::ThemeError> {
        let mut fpath = PathBuf::new();
        fpath.push(path);
        if fpath.ends_with("installed") {
            fpath.pop();
        }
        if fpath.ends_with("extensions") {
            fpath.push("index.json");
        }

        InstalledExtensions::read(fpath)
    }
}

impl ThemeFile for InstalledExtensions {
    type T = InstalledExtensions;

    fn read<P: AsRef<std::path::Path>>(path: P) -> Result<Self::T, crate::ThemeError> {
        let contents = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str::<InstalledExtensions>(contents.as_str())?)
    }

    fn write_to<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), crate::ThemeError> {
        let bytes = serde_json::to_vec_pretty(self)?;
        Ok(std::fs::write(path, bytes)?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::T, crate::ThemeError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
