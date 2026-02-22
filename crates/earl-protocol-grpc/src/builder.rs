use std::fs;

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};
use url::Url;

use crate::PreparedGrpcData;
use crate::schema::GrpcOperationTemplate;
use earl_core::PreparedBody;
use earl_core::render::{TemplateRenderer, render_key_value_map};

/// Build a complete `PreparedGrpcData` from a gRPC operation template.
///
/// Auth is **not** applied here — the caller applies it afterward.
pub fn build_grpc_request(
    grpc_op: &GrpcOperationTemplate,
    context: &Value,
    renderer: &dyn TemplateRenderer,
    command_key: &str,
) -> Result<PreparedGrpcData> {
    let url_text = renderer.render_str(&grpc_op.url, context)?;
    let url = Url::parse(&url_text).with_context(|| {
        format!("template `{command_key}` rendered invalid operation URL `{url_text}`")
    })?;

    let headers = render_key_value_map(grpc_op.headers.as_ref(), context, renderer)?;

    let grpc = &grpc_op.grpc;

    let service = renderer.render_str(&grpc.service, context)?;
    let service = service.trim().to_string();
    if service.is_empty() {
        bail!("operation.grpc.service rendered empty");
    }

    let method = renderer.render_str(&grpc.method, context)?;
    let method = method.trim().to_string();
    if method.is_empty() {
        bail!("operation.grpc.method rendered empty");
    }

    let descriptor_set = grpc
        .descriptor_set_file
        .as_ref()
        .map(|path| renderer.render_str(path, context))
        .transpose()?
        .map(|rendered_path| {
            fs::read(&rendered_path).with_context(|| {
                format!(
                    "failed reading operation.grpc.descriptor_set_file `{rendered_path}` for command {command_key}"
                )
            })
        })
        .transpose()?;

    let body = match &grpc.body {
        Some(body) => renderer.render_value(body, context)?,
        None => Value::Object(Map::new()),
    };

    Ok(PreparedGrpcData {
        url,
        headers,
        body: PreparedBody::Json(body),
        service,
        method,
        descriptor_set,
    })
}
