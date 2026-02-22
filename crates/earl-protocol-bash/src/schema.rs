use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use earl_core::schema::TransportTemplate;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BashOperationTemplate {
    pub bash: BashScriptTemplate,
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
