use std::fs;

use earl::secrets::metadata_index::{load_index, save_index};
use earl::secrets::store::{InMemorySecretStore, SecretIndex, SecretStore, require_secret};
use secrecy::SecretString;
use tempfile::tempdir;

#[test]
fn secret_index_upsert_remove_get_list() {
    let mut index = SecretIndex::default();
    index.upsert("github.token");
    index.upsert("github.token");
    index.upsert("search.api_key");

    assert!(index.get("github.token").is_some());
    assert_eq!(index.list().len(), 2);

    index.remove("github.token");
    assert!(index.get("github.token").is_none());
    assert_eq!(index.list().len(), 1);
}

#[test]
fn metadata_index_load_save_and_corruption_handling() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state/secrets-index.json");

    let mut index = SecretIndex::default();
    index.upsert("github.token");
    save_index(&path, &index).unwrap();

    let loaded = load_index(&path).unwrap();
    assert!(loaded.get("github.token").is_some());

    fs::write(&path, "{not-json").unwrap();
    assert!(load_index(&path).is_err());
}

#[test]
fn require_secret_works_with_in_memory_store() {
    let store = InMemorySecretStore::default();
    store
        .set_secret(
            "github.token",
            SecretString::new("secret-value".to_string().into()),
        )
        .unwrap();

    let value = require_secret(&store, "github.token").unwrap();
    assert_eq!(value, "secret-value");

    let err = require_secret(&store, "missing").unwrap_err();
    assert!(
        err.to_string()
            .contains("missing required secret `missing`")
    );
}
