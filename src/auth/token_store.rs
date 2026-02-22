use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::secrets::SecretManager;

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredOAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

impl std::fmt::Debug for StoredOAuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredOAuthToken")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("token_type", &self.token_type)
            .field("expires_at", &self.expires_at)
            .field("scopes", &self.scopes)
            .finish()
    }
}

impl StoredOAuthToken {
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|expires| expires <= Utc::now() + chrono::Duration::seconds(30))
            .unwrap_or(false)
    }
}

pub struct OAuthTokenStore<'a> {
    secrets: &'a SecretManager,
}

impl<'a> OAuthTokenStore<'a> {
    pub fn new(secrets: &'a SecretManager) -> Self {
        Self { secrets }
    }

    pub fn load(&self, profile: &str) -> Result<Option<StoredOAuthToken>> {
        let key = key_for_profile(profile);
        let Some(secret) = self.secrets.store().get_secret(&key)? else {
            return Ok(None);
        };

        let token = serde_json::from_str::<StoredOAuthToken>(secret.expose_secret())
            .with_context(|| format!("failed decoding token payload for profile `{profile}`"))?;
        Ok(Some(token))
    }

    pub fn save(&self, profile: &str, token: &StoredOAuthToken) -> Result<()> {
        let key = key_for_profile(profile);
        let json = serde_json::to_string(token)?;
        self.secrets.set(&key, SecretString::new(json.into()))?;
        Ok(())
    }

    pub fn delete(&self, profile: &str) -> Result<bool> {
        let key = key_for_profile(profile);
        self.secrets.delete(&key)
    }
}

fn key_for_profile(profile: &str) -> String {
    format!("oauth2.{profile}.token")
}
