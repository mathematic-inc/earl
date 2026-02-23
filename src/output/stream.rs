use anyhow::Result;
use serde_json::{Map, Value};
use tokio::sync::mpsc;

use earl_core::schema::ResultTemplate;
use earl_core::{Redactor, StreamChunk, decode_response};

use crate::output::human::render_human_output;
use crate::protocol::extract::extract_result;

/// Consume streaming chunks and render each one to stdout.
///
/// Each [`StreamChunk`] is decoded, extracted, and printed independently,
/// giving the user real-time output as data arrives from the server.
///
/// When `json_mode` is true and `active_env` is `Some`, the `"environment"`
/// key is included in every emitted JSON line for parity with non-streaming
/// JSON output.
pub async fn render_streaming_output(
    mut rx: mpsc::Receiver<StreamChunk>,
    result_template: &ResultTemplate,
    args: &Map<String, Value>,
    redactor: &Redactor,
    json_mode: bool,
    active_env: Option<&str>,
) -> Result<()> {
    use std::io::Write;

    while let Some(chunk) = rx.recv().await {
        let decoded_body = match decode_response(
            result_template.decode,
            chunk.content_type.as_deref(),
            &chunk.data,
        ) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("skipping stream chunk: failed to decode: {e}");
                continue;
            }
        };
        let extracted = match extract_result(result_template.extract.as_ref(), &decoded_body) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("skipping stream chunk: failed to extract: {e}");
                continue;
            }
        };

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        if json_mode {
            let mut map = Map::from_iter([
                ("result".to_string(), extracted.clone()),
                ("decoded".to_string(), decoded_body.to_json_value()),
            ]);
            if let Some(env) = active_env {
                map.insert("environment".to_string(), Value::String(env.to_string()));
            }
            let redacted = redactor.redact_json(&Value::Object(map));
            writeln!(out, "{}", serde_json::to_string(&redacted)?)?;
        } else {
            let output = render_human_output(result_template, args, &extracted)?;
            writeln!(out, "{}", redactor.redact(&output))?;
        }
        out.flush()?;
    }

    Ok(())
}
