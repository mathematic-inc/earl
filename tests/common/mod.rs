#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use earl::secrets::SecretManager;
use earl::secrets::store::InMemorySecretStore;
use tempfile::{TempDir, tempdir};

pub struct TestWorkspace {
    pub root: TempDir,
    pub local_templates: PathBuf,
    pub global_templates: PathBuf,
}

pub fn temp_workspace() -> TestWorkspace {
    let root = tempdir().expect("failed creating temp workspace");
    let local_templates = root.path().join("templates");
    let global_templates = root.path().join("global_templates");

    fs::create_dir_all(&local_templates).expect("failed creating local templates");
    fs::create_dir_all(&global_templates).expect("failed creating global templates");

    TestWorkspace {
        root,
        local_templates,
        global_templates,
    }
}

pub fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed creating parent directories");
    }
    fs::write(path, content).expect("failed writing file");
}

pub fn write_template(dir: &Path, name: &str, hcl: &str) {
    write_file(&dir.join(name), hcl);
}

pub fn in_memory_secret_manager(index_path: &Path) -> SecretManager {
    SecretManager::with_store_and_index(
        Box::new(InMemorySecretStore::default()),
        index_path.to_path_buf(),
    )
}
