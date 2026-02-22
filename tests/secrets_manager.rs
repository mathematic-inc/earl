mod common;

use secrecy::SecretString;

#[test]
fn set_get_list_delete_with_injected_store() {
    let ws = common::temp_workspace();
    let index_path = ws.root.path().join("state/secrets-index.json");
    let manager = common::in_memory_secret_manager(&index_path);

    manager
        .set(
            "github.token",
            SecretString::new("token-1".to_string().into()),
        )
        .unwrap();

    let meta = manager.get("github.token").unwrap().unwrap();
    assert_eq!(meta.key, "github.token");

    let list = manager.list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].key, "github.token");

    let deleted = manager.delete("github.token").unwrap();
    assert!(deleted);
    assert!(manager.get("github.token").unwrap().is_none());
}

#[test]
fn repeated_set_updates_metadata_timestamp() {
    let ws = common::temp_workspace();
    let index_path = ws.root.path().join("state/secrets-index.json");
    let manager = common::in_memory_secret_manager(&index_path);

    manager
        .set(
            "service.token",
            SecretString::new("first".to_string().into()),
        )
        .unwrap();
    let first = manager.get("service.token").unwrap().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(2));

    manager
        .set(
            "service.token",
            SecretString::new("second".to_string().into()),
        )
        .unwrap();
    let second = manager.get("service.token").unwrap().unwrap();

    assert_eq!(first.key, second.key);
    assert_eq!(first.created_at, second.created_at);
    assert!(second.updated_at >= first.updated_at);
}
