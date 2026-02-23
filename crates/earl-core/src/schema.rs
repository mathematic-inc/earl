use std::collections::BTreeMap;

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::with::AsJson;

#[derive(
    Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[serde(deny_unknown_fields)]
pub struct AllowRule {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path_prefix: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
pub enum CommandMode {
    #[serde(rename = "read")]
    Read,
    #[default]
    #[serde(rename = "write")]
    Write,
}

impl CommandMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandMode::Read => "read",
            CommandMode::Write => "write",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(deny_unknown_fields)]
pub struct ParamSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: ParamType,
    #[serde(default)]
    pub required: bool,
    #[rkyv(with = AsJson)]
    pub default: Option<Value>,
    pub description: Option<String>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Integer,
    Number,
    Boolean,
    Null,
    Array,
    Object,
}

impl std::fmt::Display for ParamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            ParamType::String => "string",
            ParamType::Integer => "integer",
            ParamType::Number => "number",
            ParamType::Boolean => "boolean",
            ParamType::Null => "null",
            ParamType::Array => "array",
            ParamType::Object => "object",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthTemplate {
    None,
    ApiKey {
        location: ApiKeyLocation,
        name: String,
        secret: String,
    },
    Bearer {
        secret: String,
    },
    Basic {
        username: String,
        password_secret: String,
    },
    OAuth2Profile {
        profile: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyLocation {
    Header,
    Query,
    Cookie,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BodyTemplate {
    None,
    Json {
        #[rkyv(with = AsJson)]
        value: Value,
    },
    FormUrlencoded {
        #[rkyv(with = AsJson)]
        fields: BTreeMap<String, Value>,
    },
    Multipart {
        parts: Vec<MultipartPartTemplate>,
    },
    RawText {
        value: String,
        content_type: Option<String>,
    },
    RawBytesBase64 {
        value: String,
        content_type: Option<String>,
    },
    FileStream {
        path: String,
        content_type: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(deny_unknown_fields)]
pub struct MultipartPartTemplate {
    pub name: String,
    pub value: Option<String>,
    pub bytes_base64: Option<String>,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportTemplate {
    pub timeout_ms: Option<u64>,
    pub max_response_bytes: Option<u64>,
    pub redirects: Option<RedirectTemplate>,
    pub retry: Option<RetryTemplate>,
    pub compression: Option<bool>,
    pub tls: Option<TlsTemplate>,
    pub proxy_profile: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(deny_unknown_fields)]
pub struct RedirectTemplate {
    #[serde(default = "default_follow_redirects")]
    pub follow: bool,
    #[serde(default = "default_redirect_hops")]
    pub max_hops: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryTemplate {
    #[serde(default)]
    pub max_attempts: usize,
    #[serde(default = "default_backoff_ms")]
    pub backoff_ms: u64,
    #[serde(default)]
    pub retry_on_status: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsTemplate {
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultTemplate {
    #[serde(default)]
    pub decode: ResultDecode,
    pub extract: Option<ResultExtract>,
    #[serde(default = "default_result_output")]
    pub output: String,
    pub result_alias: Option<String>,
}

impl Default for ResultTemplate {
    fn default() -> Self {
        Self {
            decode: ResultDecode::default(),
            extract: None,
            output: default_result_output(),
            result_alias: None,
        }
    }
}

fn default_result_output() -> String {
    "{{ result }}".to_string()
}

#[derive(
    Debug, Clone, Copy, Default, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ResultDecode {
    #[default]
    Auto,
    Json,
    Text,
    Html,
    Xml,
    Binary,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(untagged)]
pub enum ResultExtract {
    JsonPointer { json_pointer: String },
    Regex { regex: String },
    XPath { xpath: String },
    CssSelector { css_selector: String },
}

pub fn default_follow_redirects() -> bool {
    true
}

pub fn default_redirect_hops() -> usize {
    5
}

pub fn default_backoff_ms() -> u64 {
    250
}
