use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

use super::schema::{CommandMode, CommandTemplate};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateCatalog {
    pub entries: BTreeMap<String, TemplateCatalogEntry>,
}

impl TemplateCatalog {
    pub fn empty() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&TemplateCatalogEntry> {
        self.entries.get(key)
    }

    pub fn upsert(&mut self, key: String, entry: TemplateCatalogEntry) {
        self.entries.insert(key, entry);
    }

    pub fn values(&self) -> impl Iterator<Item = &TemplateCatalogEntry> {
        self.entries.values()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateCatalogEntry {
    pub key: String,
    pub provider: String,
    pub command: String,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub categories: Vec<String>,
    pub mode: CommandMode,
    pub source: TemplateSource,
    pub template: CommandTemplate,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct TemplateSource {
    pub path: PathBuf,
    pub scope: TemplateScope,
}

#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TemplateScope {
    Local,
    Global,
}
