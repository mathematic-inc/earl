use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use std::time::Duration;

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

/// Check whether an error message suggests a transient failure worth retrying.
///
/// Note: AWS SDK has its own internal retry logic (3 attempts by default).
/// Earl's retry here is intentionally coarse-grained and covers all providers
/// uniformly, so AWS secrets may see up to 9 total attempts (3 Earl x 3 SDK).
fn is_transient(err: &anyhow::Error) -> bool {
    // Use alternate Display (`{:#}`) to inspect the full error chain,
    // not just the outermost .with_context() wrapper.
    let msg = format!("{:#}", err).to_lowercase();
    msg.contains("timeout")
        || msg.contains("timed out")
        || msg.contains("connection reset")
        || msg.contains("connection refused")
        || msg.contains("broken pipe")
        || msg.contains("http 500")
        || msg.contains("http 502")
        || msg.contains("http 503")
        || msg.contains("http 504")
        || msg.contains("http 429")
        // vaultrs and other libraries may format as "status code 503" etc.
        || msg.contains("status code 500")
        || msg.contains("status code 502")
        || msg.contains("status code 503")
        || msg.contains("status code 504")
        || msg.contains("status code 429")
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
                let max_attempts: u32 = 3;
                let backoff_base = Duration::from_millis(500);
                for attempt in 1..=max_attempts {
                    match resolver.resolve(key) {
                        Ok(secret) => return Ok(secret.expose_secret().to_string()),
                        Err(err) if attempt < max_attempts && is_transient(&err) => {
                            std::thread::sleep(backoff_base * attempt);
                            continue;
                        }
                        Err(err) => return Err(err),
                    }
                }
                unreachable!("retry loop should have returned");
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
