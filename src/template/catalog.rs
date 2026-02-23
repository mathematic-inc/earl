use std::collections::BTreeMap;
use std::path::PathBuf;

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use super::schema::{CommandMode, CommandTemplate};
use earl_core::with::AsPath;

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct TemplateSource {
    #[rkyv(with = AsPath)]
    pub path: PathBuf,
    pub scope: TemplateScope,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
pub enum TemplateScope {
    Local,
    Global,
}
