use anyhow::Result;
use serde_json::{Map, Value};

use crate::template::render::render_string_raw;
use crate::template::schema::ResultTemplate;

pub fn render_human_output(
    result_template: &ResultTemplate,
    args: &Map<String, Value>,
    result: &Value,
) -> Result<String> {
    let alias = result_template
        .result_alias
        .clone()
        .unwrap_or_else(|| "result".to_string());

    let mut context = Map::new();
    context.insert("args".to_string(), Value::Object(args.clone()));
    context.insert("result".to_string(), result.clone());
    context.insert(alias, result.clone());

    render_string_raw(&result_template.output, &Value::Object(context))
}
