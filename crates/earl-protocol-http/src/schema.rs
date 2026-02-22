use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use earl_core::schema::{AuthTemplate, BodyTemplate, TransportTemplate};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpOperationTemplate {
    pub method: String,
    pub url: String,
    pub path: Option<String>,
    pub query: Option<BTreeMap<String, Value>>,
    pub headers: Option<BTreeMap<String, Value>>,
    pub cookies: Option<BTreeMap<String, Value>>,
    pub auth: Option<AuthTemplate>,
    pub body: Option<BodyTemplate>,
    pub transport: Option<TransportTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphqlOperationTemplate {
    #[serde(default)]
    pub method: String,
    pub url: String,
    pub path: Option<String>,
    pub query: Option<BTreeMap<String, Value>>,
    pub headers: Option<BTreeMap<String, Value>>,
    pub cookies: Option<BTreeMap<String, Value>>,
    pub auth: Option<AuthTemplate>,
    pub graphql: GraphqlTemplate,
    pub transport: Option<TransportTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphqlTemplate {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: Option<Value>,
}
