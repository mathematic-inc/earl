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
    #[serde(default)]
    pub stream: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_operation_defaults_stream_false() {
        let json = r#"{"url":"https://example.com","grpc":{"service":"test.Svc","method":"Call"}}"#;
        let op: GrpcOperationTemplate = serde_json::from_str(json).unwrap();
        assert!(!op.stream);
    }

    #[test]
    fn grpc_operation_accepts_stream_true() {
        let json = r#"{"url":"https://example.com","stream":true,"grpc":{"service":"test.Svc","method":"Call"}}"#;
        let op: GrpcOperationTemplate = serde_json::from_str(json).unwrap();
        assert!(op.stream);
    }
}
