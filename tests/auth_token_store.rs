mod common;

use chrono::{Duration, Utc};
use earl::auth::token_store::{OAuthTokenStore, StoredOAuthToken};
use secrecy::SecretString;

fn token(expires_at: Option<chrono::DateTime<Utc>>) -> StoredOAuthToken {
    StoredOAuthToken {
        access_token: "access-1".to_string(),
        refresh_token: Some("refresh-1".to_string()),
        token_type: Some("Bearer".to_string()),
        expires_at,
        scopes: vec!["repo".to_string()],
    }
}

#[test]
fn token_store_save_load_delete_roundtrip() {
    let ws = common::temp_workspace();
    let secrets =
        common::in_memory_secret_manager(&ws.root.path().join("state/secrets-index.json"));
    let store = OAuthTokenStore::new(&secrets);

    store
        .save("github", &token(Some(Utc::now() + Duration::hours(1))))
        .unwrap();

    let loaded = store.load("github").unwrap().unwrap();
    assert_eq!(loaded.access_token, "access-1");
    assert_eq!(loaded.refresh_token.as_deref(), Some("refresh-1"));

    let deleted = store.delete("github").unwrap();
    assert!(deleted);
    assert!(store.load("github").unwrap().is_none());
}

#[test]
fn token_store_reports_corrupted_payload() {
    let ws = common::temp_workspace();
    let secrets =
        common::in_memory_secret_manager(&ws.root.path().join("state/secrets-index.json"));

    secrets
        .set(
            "oauth2.github.token",
            SecretString::new("not-json".to_string().into()),
        )
        .unwrap();

    let store = OAuthTokenStore::new(&secrets);
    let err = store.load("github").unwrap_err();
    assert!(
        err.to_string()
            .contains("failed decoding token payload for profile `github`")
    );
}

#[test]
fn token_expiry_uses_safety_window() {
    let expired = token(Some(Utc::now() - Duration::seconds(1)));
    assert!(expired.is_expired());

    let near_expiry = token(Some(Utc::now() + Duration::seconds(10)));
    assert!(near_expiry.is_expired());

    let valid = token(Some(Utc::now() + Duration::minutes(5)));
    assert!(!valid.is_expired());
}
