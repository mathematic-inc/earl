use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::secrets::store::SecretIndex;

pub fn load_index(path: &Path) -> Result<SecretIndex> {
    if !path.exists() {
        return Ok(SecretIndex::default());
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed reading secrets index {}", path.display()))?;
    let index = serde_json::from_str(&raw)
        .with_context(|| format!("invalid secrets index JSON {}", path.display()))?;
    Ok(index)
}

pub fn save_index(path: &Path, index: &SecretIndex) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating index directory {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(index)?;
    fs::write(path, &json)
        .with_context(|| format!("failed writing secrets index {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("failed setting permissions on {}", path.display()))?;
    }

    Ok(())
}
