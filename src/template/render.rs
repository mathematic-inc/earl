use std::sync::LazyLock;

use anyhow::{Context, Result};
use minijinja::{Environment, UndefinedBehavior, Value as JinjaValue};
use serde_json::{Map, Value};

static JINJA_ENV: LazyLock<Environment<'static>> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env
});

pub fn render_json_value(value: &Value, context: &Value) -> Result<Value> {
    match value {
        Value::Null => Ok(Value::Null),
        Value::Bool(v) => Ok(Value::Bool(*v)),
        Value::Number(v) => Ok(Value::Number(v.clone())),
        Value::String(v) => render_string_value(v, context),
        Value::Array(values) => {
            let mut out = Vec::with_capacity(values.len());
            for item in values {
                out.push(render_json_value(item, context)?);
            }
            Ok(Value::Array(out))
        }
        Value::Object(obj) => {
            let mut out = Map::new();
            for (k, v) in obj {
                let rendered_key = render_string_raw(k, context)?;
                out.insert(rendered_key, render_json_value(v, context)?);
            }
            Ok(Value::Object(out))
        }
    }
}

pub fn render_string_raw(input: &str, context: &Value) -> Result<String> {
    let ctx = JinjaValue::from_serialize(context);
    JINJA_ENV
        .render_str(input, ctx)
        .with_context(|| format!("template render failed for string `{input}`"))
}

fn render_string_value(input: &str, context: &Value) -> Result<Value> {
    let rendered = render_string_raw(input, context)?;
    if is_pure_expression(input)
        && let Ok(v) = serde_json::from_str::<Value>(&rendered)
    {
        return Ok(v);
    }
    Ok(Value::String(rendered))
}

fn is_pure_expression(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.starts_with("{{") && trimmed.ends_with("}}")
}
