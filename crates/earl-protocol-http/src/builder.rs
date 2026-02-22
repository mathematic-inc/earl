use std::collections::BTreeMap;
use std::fs;

use anyhow::{Context, Result, bail};
use base64::Engine;
use serde_json::{Map, Value};
use url::Url;

use crate::PreparedHttpData;
use crate::schema::{GraphqlOperationTemplate, GraphqlTemplate, HttpOperationTemplate};
use earl_core::render::{TemplateRenderer, render_key_value_map};
use earl_core::schema::MultipartPartTemplate;
use earl_core::{PreparedBody, PreparedMultipartPart};

/// Build a complete `PreparedHttpData` from an HTTP operation template.
///
/// Auth is **not** applied here — the caller applies it afterward.
pub fn build_http_request(
    http: &HttpOperationTemplate,
    context: &Value,
    renderer: &dyn TemplateRenderer,
    command_key: &str,
) -> Result<PreparedHttpData> {
    let (url, query, headers, cookies) = render_http_primitives(
        &http.url,
        http.path.as_ref(),
        &http.query,
        &http.headers,
        &http.cookies,
        context,
        renderer,
        command_key,
    )?;

    let body = render_http_body(http, context, renderer, command_key)?;

    Ok(PreparedHttpData {
        method: parse_http_method(&http.method, None)?,
        url,
        query,
        headers,
        cookies,
        body,
    })
}

/// Build a complete `PreparedHttpData` from a GraphQL operation template.
///
/// Auth is **not** applied here — the caller applies it afterward.
pub fn build_graphql_request(
    graphql: &GraphqlOperationTemplate,
    context: &Value,
    renderer: &dyn TemplateRenderer,
    command_key: &str,
) -> Result<PreparedHttpData> {
    let (url, query, mut headers, cookies) = render_http_primitives(
        &graphql.url,
        graphql.path.as_ref(),
        &graphql.query,
        &graphql.headers,
        &graphql.cookies,
        context,
        renderer,
        command_key,
    )?;

    ensure_header_default(&mut headers, "Accept", "application/json");
    ensure_header_default(&mut headers, "Content-Type", "application/json");

    let body = PreparedBody::Json(render_graphql_body(&graphql.graphql, context, renderer)?);

    Ok(PreparedHttpData {
        method: parse_http_method(&graphql.method, Some("POST"))?,
        url,
        query,
        headers,
        cookies,
        body,
    })
}

pub fn parse_http_method(method: &str, fallback: Option<&str>) -> Result<reqwest::Method> {
    let raw = method.trim();
    let method = if raw.is_empty() {
        fallback.unwrap_or("")
    } else {
        raw
    };

    method
        .parse::<reqwest::Method>()
        .with_context(|| format!("invalid HTTP method `{method}`"))
}

pub fn ensure_header_default(headers: &mut Vec<(String, String)>, name: &str, value: &str) {
    if headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(name)) {
        return;
    }
    headers.push((name.to_string(), value.to_string()));
}

// ── Private helpers ──────────────────────────────────────────

type RenderedHttpPrimitives = (
    Url,
    Vec<(String, String)>,
    Vec<(String, String)>,
    Vec<(String, String)>,
);

#[expect(clippy::too_many_arguments)]
fn render_http_primitives(
    url_template: &str,
    path_template: Option<&String>,
    query_template: &Option<BTreeMap<String, Value>>,
    headers_template: &Option<BTreeMap<String, Value>>,
    cookies_template: &Option<BTreeMap<String, Value>>,
    context: &Value,
    renderer: &dyn TemplateRenderer,
    command_key: &str,
) -> Result<RenderedHttpPrimitives> {
    let url_text = renderer.render_str(url_template, context)?;
    let mut url = Url::parse(&url_text).with_context(|| {
        format!("template `{command_key}` rendered invalid operation URL `{url_text}`")
    })?;

    if let Some(path_template) = path_template {
        let rendered_path = renderer.render_str(path_template, context)?;
        if rendered_path.starts_with('/') {
            url.set_path(&rendered_path);
        } else {
            let current = url.path().trim_end_matches('/');
            let next = format!("{current}/{rendered_path}");
            url.set_path(&next);
        }
    }

    let query = render_key_value_map(query_template.as_ref(), context, renderer)?;
    let headers = render_key_value_map(headers_template.as_ref(), context, renderer)?;
    let cookies = render_key_value_map(cookies_template.as_ref(), context, renderer)?;

    Ok((url, query, headers, cookies))
}

fn render_http_body(
    http: &HttpOperationTemplate,
    context: &Value,
    renderer: &dyn TemplateRenderer,
    command_key: &str,
) -> Result<PreparedBody> {
    use earl_core::schema::BodyTemplate;

    match http.body.as_ref().unwrap_or(&BodyTemplate::None) {
        BodyTemplate::None => Ok(PreparedBody::Empty),
        BodyTemplate::Json { value } => {
            Ok(PreparedBody::Json(renderer.render_value(value, context)?))
        }
        BodyTemplate::FormUrlencoded { fields } => Ok(PreparedBody::Form(render_key_value_map(
            Some(fields),
            context,
            renderer,
        )?)),
        BodyTemplate::Multipart { parts } => Ok(PreparedBody::Multipart(render_multipart_parts(
            parts, context, renderer,
        )?)),
        BodyTemplate::RawText {
            value,
            content_type,
        } => {
            let rendered = renderer.render_str(value, context)?;
            Ok(PreparedBody::RawBytes {
                bytes: rendered.into_bytes(),
                content_type: content_type
                    .clone()
                    .or_else(|| Some("text/plain".to_string())),
            })
        }
        BodyTemplate::RawBytesBase64 {
            value,
            content_type,
        } => {
            let rendered = renderer.render_str(value, context)?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(rendered)
                .context("invalid base64 in raw_bytes_base64 body")?;
            Ok(PreparedBody::RawBytes {
                bytes,
                content_type: content_type.clone(),
            })
        }
        BodyTemplate::FileStream { path, content_type } => {
            let rendered_path = renderer.render_str(path, context)?;
            let bytes = fs::read(&rendered_path).with_context(|| {
                format!(
                    "failed reading file_stream body data from `{rendered_path}` for command {command_key}"
                )
            })?;
            Ok(PreparedBody::RawBytes {
                bytes,
                content_type: content_type.clone(),
            })
        }
    }
}

fn render_multipart_parts(
    parts: &[MultipartPartTemplate],
    context: &Value,
    renderer: &dyn TemplateRenderer,
) -> Result<Vec<PreparedMultipartPart>> {
    let mut out = Vec::new();
    for part in parts {
        let bytes = if let Some(value) = &part.value {
            renderer.render_str(value, context)?.into_bytes()
        } else if let Some(value) = &part.bytes_base64 {
            let rendered = renderer.render_str(value, context)?;
            base64::engine::general_purpose::STANDARD
                .decode(rendered)
                .context("invalid multipart bytes_base64")?
        } else if let Some(path) = &part.file_path {
            let rendered_path = renderer.render_str(path, context)?;
            fs::read(&rendered_path)
                .with_context(|| format!("failed reading multipart file `{rendered_path}`"))?
        } else {
            bail!(
                "multipart part `{}` does not contain any data source",
                part.name
            );
        };

        let filename = part
            .filename
            .as_ref()
            .map(|name| renderer.render_str(name, context))
            .transpose()?;

        out.push(PreparedMultipartPart {
            name: part.name.clone(),
            bytes,
            content_type: part.content_type.clone(),
            filename,
        });
    }
    Ok(out)
}

fn render_graphql_body(
    graphql: &GraphqlTemplate,
    context: &Value,
    renderer: &dyn TemplateRenderer,
) -> Result<Value> {
    let mut payload = Map::new();
    payload.insert(
        "query".to_string(),
        Value::String(renderer.render_str(&graphql.query, context)?),
    );

    if let Some(operation_name) = &graphql.operation_name {
        payload.insert(
            "operationName".to_string(),
            Value::String(renderer.render_str(operation_name, context)?),
        );
    }

    if let Some(variables) = &graphql.variables {
        payload.insert(
            "variables".to_string(),
            renderer.render_value(variables, context)?,
        );
    }

    Ok(Value::Object(payload))
}
