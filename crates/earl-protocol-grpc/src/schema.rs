use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use earl_core::schema::{AuthTemplate, TransportTemplate};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcOperationTemplate {
    pub url: String,
    pub headers: Option<BTreeMap<String, Value>>,
    pub auth: Option<AuthTemplate>,
    pub grpc: GrpcTemplate,
    pub transport: Option<TransportTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrpcTemplate {
    pub service: String,
    pub method: String,
    pub body: Option<Value>,
    pub descriptor_set_file: Option<String>,
}
