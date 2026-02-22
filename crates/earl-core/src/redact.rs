use std::collections::BTreeSet;

use base64::Engine;

#[derive(Debug, Default, Clone)]
pub struct Redactor {
    secrets: Vec<String>,
}

impl Redactor {
    pub fn new<I: IntoIterator<Item = String>>(secrets: I) -> Self {
        let mut variants = BTreeSet::new();

        for secret in secrets {
            if secret.is_empty() {
                continue;
            }

            variants.insert(secret.clone());

            if secret.len() >= 6 {
                let bytes = secret.as_bytes();
                variants.insert(base64::engine::general_purpose::STANDARD.encode(bytes));
                variants.insert(base64::engine::general_purpose::URL_SAFE.encode(bytes));
                variants.insert(hex_encode(bytes, false));
                variants.insert(hex_encode(bytes, true));

                let url_encoded = url::form_urlencoded::byte_serialize(bytes).collect::<String>();
                if !url_encoded.is_empty() {
                    variants.insert(url_encoded);
                }
            }
        }

        let mut ordered: Vec<String> = variants.into_iter().collect();
        ordered.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));

        Self { secrets: ordered }
    }

    pub fn redact(&self, input: &str) -> String {
        let mut out = input.to_string();
        for secret in &self.secrets {
            out = out.replace(secret, "[REDACTED]");
        }
        out
    }

    pub fn redact_json(&self, value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Null => serde_json::Value::Null,
            serde_json::Value::Bool(v) => serde_json::Value::Bool(*v),
            serde_json::Value::Number(v) => serde_json::Value::Number(v.clone()),
            serde_json::Value::String(v) => serde_json::Value::String(self.redact(v)),
            serde_json::Value::Array(values) => {
                serde_json::Value::Array(values.iter().map(|v| self.redact_json(v)).collect())
            }
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), self.redact_json(v)))
                    .collect(),
            ),
        }
    }
}

fn hex_encode(bytes: &[u8], uppercase: bool) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        if uppercase {
            out.push_str(&format!("{:02X}", b));
        } else {
            out.push_str(&format!("{:02x}", b));
        }
    }
    out
}
