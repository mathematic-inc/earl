use serde::{Deserialize, Serialize};
use serde_json::Value;

use earl_core::schema::TransportTemplate;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SqlOperationTemplate {
    pub sql: SqlQueryTemplate,
    pub transport: Option<TransportTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SqlQueryTemplate {
    pub connection_secret: String,
    pub query: String,
    pub params: Option<Vec<Value>>,
    pub sandbox: Option<SqlSandboxTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SqlSandboxTemplate {
    pub read_only: Option<bool>,
    pub max_rows: Option<u64>,
    pub max_time_ms: Option<u64>,
}
