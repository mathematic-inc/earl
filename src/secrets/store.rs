use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow, bail};
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use super::resolver::SecretResolver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretIndex {
    pub keys: BTreeMap<String, SecretMetadata>,
}

impl SecretIndex {
    pub fn upsert(&mut self, key: &str) {
        let now = Utc::now();
        self.keys
            .entry(key.to_string())
            .and_modify(|meta| {
                meta.updated_at = now;
            })
            .or_insert_with(|| SecretMetadata {
                key: key.to_string(),
                created_at: now,
                updated_at: now,
            });
    }

    pub fn remove(&mut self, key: &str) {
        self.keys.remove(key);
    }

    pub fn list(&self) -> Vec<&SecretMetadata> {
        self.keys.values().collect()
    }

    pub fn get(&self, key: &str) -> Option<&SecretMetadata> {
        self.keys.get(key)
    }
}

pub trait SecretStore {
    fn set_secret(&self, key: &str, secret: SecretString) -> Result<()>;
    fn get_secret(&self, key: &str) -> Result<Option<SecretString>>;
    fn delete_secret(&self, key: &str) -> Result<bool>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemorySecretStore {
    values: Arc<Mutex<BTreeMap<String, String>>>,
}

impl SecretStore for InMemorySecretStore {
    fn set_secret(&self, key: &str, secret: SecretString) -> Result<()> {
        let mut values = self
            .values
            .lock()
            .map_err(|_| anyhow!("in-memory secret store mutex poisoned"))?;
        values.insert(key.to_string(), secret.expose_secret().to_string());
        Ok(())
    }

    fn get_secret(&self, key: &str) -> Result<Option<SecretString>> {
        let values = self
            .values
            .lock()
            .map_err(|_| anyhow!("in-memory secret store mutex poisoned"))?;
        Ok(values.get(key).map(|v| SecretString::new(v.clone().into())))
    }

    fn delete_secret(&self, key: &str) -> Result<bool> {
        let mut values = self
            .values
            .lock()
            .map_err(|_| anyhow!("in-memory secret store mutex poisoned"))?;
        Ok(values.remove(key).is_some())
    }
}

pub fn require_secret(
    store: &dyn SecretStore,
    resolvers: &[Box<dyn SecretResolver>],
    key: &str,
) -> Result<String> {
    // URI-prefixed keys dispatch to the matching resolver
    if let Some((scheme, _)) = key.split_once("://") {
        for resolver in resolvers {
            if resolver.scheme() == scheme {
                return Ok(resolver.resolve(key)?.expose_secret().to_string());
            }
        }
        bail!("no secret resolver registered for scheme `{scheme}://`");
    }
    // Plain keys fall back to the keychain store
    let secret = store
        .get_secret(key)?
        .ok_or_else(|| anyhow!("missing required secret `{key}`"))?;
    Ok(secret.expose_secret().to_string())
}
