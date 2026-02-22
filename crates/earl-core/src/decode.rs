use anyhow::Result;
use base64::Engine;
use serde_json::Value;

use crate::schema::ResultDecode;

#[derive(Debug, Clone)]
pub enum DecodedBody {
    Json(Value),
    Text(String),
    Html(String),
    Xml(String),
    Binary(Vec<u8>),
}

impl DecodedBody {
    pub fn as_json(&self) -> Option<&Value> {
        match self {
            DecodedBody::Json(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            DecodedBody::Text(v) | DecodedBody::Html(v) | DecodedBody::Xml(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_json_value(&self) -> Value {
        match self {
            DecodedBody::Json(v) => v.clone(),
            DecodedBody::Text(v) => Value::String(v.clone()),
            DecodedBody::Html(v) => Value::String(v.clone()),
            DecodedBody::Xml(v) => Value::String(v.clone()),
            DecodedBody::Binary(v) => {
                Value::String(base64::engine::general_purpose::STANDARD.encode(v))
            }
        }
    }
}

pub fn decode_response(
    mode: ResultDecode,
    content_type: Option<&str>,
    bytes: &[u8],
) -> Result<DecodedBody> {
    let inferred_mode = match mode {
        ResultDecode::Auto => infer_mode(content_type, bytes),
        explicit => explicit,
    };

    let decoded = match inferred_mode {
        ResultDecode::Auto | ResultDecode::Text => {
            DecodedBody::Text(String::from_utf8_lossy(bytes).to_string())
        }
        ResultDecode::Json => {
            let value: Value = serde_json::from_slice(bytes)?;
            DecodedBody::Json(value)
        }
        ResultDecode::Html => DecodedBody::Html(String::from_utf8_lossy(bytes).to_string()),
        ResultDecode::Xml => DecodedBody::Xml(String::from_utf8_lossy(bytes).to_string()),
        ResultDecode::Binary => DecodedBody::Binary(bytes.to_vec()),
    };

    Ok(decoded)
}

fn infer_mode(content_type: Option<&str>, body: &[u8]) -> ResultDecode {
    if let Some(ct) = content_type {
        let lower = ct.to_ascii_lowercase();
        if lower.contains("application/json") || lower.ends_with("+json") {
            return ResultDecode::Json;
        }
        if lower.contains("text/html") {
            return ResultDecode::Html;
        }
        if lower.contains("xml") {
            return ResultDecode::Xml;
        }
        if lower.starts_with("text/") {
            return ResultDecode::Text;
        }
    }

    if serde_json::from_slice::<Value>(body).is_ok() {
        ResultDecode::Json
    } else {
        ResultDecode::Text
    }
}
