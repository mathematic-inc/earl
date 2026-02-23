pub mod keychain;
pub mod metadata_index;
pub mod resolver;
pub mod store;

use anyhow::Result;
use secrecy::SecretString;
use std::path::PathBuf;

use crate::config;

use self::keychain::KeychainSecretStore;
use self::metadata_index::{load_index, save_index};
use self::resolver::SecretResolver;
use self::store::{SecretIndex, SecretMetadata, SecretStore};

pub struct SecretManager {
    store: Box<dyn SecretStore + Send + Sync>,
    resolvers: Vec<Box<dyn SecretResolver>>,
    index_path: std::path::PathBuf,
}

impl SecretManager {
    pub fn new() -> Self {
        Self {
            store: Box::new(KeychainSecretStore),
            resolvers: Vec::new(),
            index_path: config::secrets_index_path(),
        }
    }

    pub fn with_store_and_index(
        store: Box<dyn SecretStore + Send + Sync>,
        index_path: PathBuf,
    ) -> Self {
        Self {
            store,
            resolvers: Vec::new(),
            index_path,
        }
    }

    pub fn set(&self, key: &str, secret: SecretString) -> Result<()> {
        self.store.as_ref().set_secret(key, secret)?;
        let mut index = self.load_index()?;
        index.upsert(key);
        save_index(&self.index_path, &index)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<SecretMetadata>> {
        let secret_exists = self.store.as_ref().get_secret(key)?.is_some();
        if !secret_exists {
            return Ok(None);
        }
        let mut index = self.load_index()?;
        if index.get(key).is_none() {
            index.upsert(key);
            save_index(&self.index_path, &index)?;
        }
        Ok(index.get(key).cloned())
    }

    pub fn list(&self) -> Result<Vec<SecretMetadata>> {
        let index = self.load_index()?;
        let mut entries: Vec<_> = index.list().into_iter().cloned().collect();
        entries.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(entries)
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        let deleted = self.store.as_ref().delete_secret(key)?;
        let mut index = self.load_index()?;
        index.remove(key);
        save_index(&self.index_path, &index)?;
        Ok(deleted)
    }

    pub fn store(&self) -> &dyn SecretStore {
        self.store.as_ref()
    }

    pub fn resolvers(&self) -> &[Box<dyn SecretResolver>] {
        &self.resolvers
    }

    fn load_index(&self) -> Result<SecretIndex> {
        load_index(&self.index_path)
    }
}

impl Default for SecretManager {
    fn default() -> Self {
        Self::new()
    }
}
