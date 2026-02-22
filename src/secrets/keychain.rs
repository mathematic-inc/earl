use anyhow::{Result, anyhow};
use keyring::{Entry, Error as KeyringError};
use secrecy::{ExposeSecret, SecretString};

use super::store::SecretStore;

const SERVICE_NAME: &str = "earl";

#[derive(Debug, Default)]
pub struct KeychainSecretStore;

impl SecretStore for KeychainSecretStore {
    fn set_secret(&self, key: &str, secret: SecretString) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|err| anyhow!("failed creating keyring entry for `{key}`: {err}"))?;
        entry
            .set_password(secret.expose_secret())
            .map_err(|err| anyhow!("failed storing secret `{key}` in keychain: {err}"))?;
        Ok(())
    }

    fn get_secret(&self, key: &str) -> Result<Option<SecretString>> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|err| anyhow!("failed creating keyring entry for `{key}`: {err}"))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(SecretString::new(value.into()))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(err) => Err(anyhow!(
                "failed reading secret `{key}` from keychain: {err}"
            )),
        }
    }

    fn delete_secret(&self, key: &str) -> Result<bool> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|err| anyhow!("failed creating keyring entry for `{key}`: {err}"))?;
        match entry.delete_credential() {
            Ok(_) => Ok(true),
            Err(KeyringError::NoEntry) => Ok(false),
            Err(err) => Err(anyhow!(
                "failed deleting secret `{key}` from keychain: {err}"
            )),
        }
    }
}
