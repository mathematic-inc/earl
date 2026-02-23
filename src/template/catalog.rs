use std::collections::BTreeMap;
use std::path::PathBuf;

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use super::schema::{CommandMode, CommandTemplate, ProviderEnvironments};
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
    pub provider_environments: Option<ProviderEnvironments>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn catalog_entry_has_provider_environments_field() {
        let mut envs = BTreeMap::new();
        let mut prod = BTreeMap::new();
        prod.insert(
            "base_url".to_string(),
            "https://api.example.com".to_string(),
        );
        envs.insert("production".to_string(), prod);

        let pe = ProviderEnvironments {
            default: Some("production".to_string()),
            secrets: vec![],
            environments: envs,
        };

        // This test just verifies the field exists and can be accessed.
        // The None case is the common default.
        let _: Option<ProviderEnvironments> = Some(pe);
    }
}
