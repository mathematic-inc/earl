use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use earl_core::schema::TransportTemplate;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BashOperationTemplate {
    pub bash: BashScriptTemplate,
    #[serde(default)]
    pub stream: bool,
    pub transport: Option<TransportTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BashScriptTemplate {
    pub script: String,
    #[serde(default)]
    pub env: Option<BTreeMap<String, Value>>,
    pub cwd: Option<String>,
    pub sandbox: Option<BashSandboxTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BashSandboxTemplate {
    pub network: Option<bool>,
    pub writable_paths: Option<Vec<String>>,
    pub max_time_ms: Option<u64>,
    pub max_output_bytes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_operation_defaults_stream_false() {
        let json = r#"{"bash":{"script":"echo hello"}}"#;
        let op: BashOperationTemplate = serde_json::from_str(json).unwrap();
        assert!(!op.stream);
    }

    #[test]
    fn bash_operation_accepts_stream_true() {
        let json = r#"{"stream":true,"bash":{"script":"echo hello"}}"#;
        let op: BashOperationTemplate = serde_json::from_str(json).unwrap();
        assert!(op.stream);
    }
}
