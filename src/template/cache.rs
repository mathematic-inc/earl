use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::template::catalog::TemplateCatalog;

pub const CACHE_VERSION: u32 = 1;

/// Serialized catalog cache file stored at ~/.cache/earl/catalog-1.bin.
#[derive(Serialize, Deserialize)]
pub struct CacheFile {
    pub version: u32,
    /// Sorted list of (absolute_path, mtime_unix_secs) for every .hcl file.
    pub fingerprint: Vec<(PathBuf, u64)>,
    pub catalog: TemplateCatalog,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn cache_file_roundtrips_bincode() {
        let original = CacheFile {
            version: CACHE_VERSION,
            fingerprint: vec![(PathBuf::from("/tmp/foo.hcl"), 1_700_000_000u64)],
            catalog: crate::template::catalog::TemplateCatalog::empty(),
        };
        let bytes = bincode::serialize(&original).expect("serialize");
        let decoded: CacheFile = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded.version, CACHE_VERSION);
        assert_eq!(decoded.fingerprint, original.fingerprint);
        assert_eq!(decoded.catalog.entries.len(), 0);
    }
}
