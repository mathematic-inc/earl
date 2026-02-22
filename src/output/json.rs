use serde_json::{Map, Value};

use crate::protocol::executor::ExecutionResult;

pub fn render_json_output(execution: &ExecutionResult) -> Value {
    Value::Object(Map::from_iter([
        (
            "status".to_string(),
            Value::Number(serde_json::Number::from(execution.status)),
        ),
        ("url".to_string(), Value::String(execution.url.clone())),
        ("result".to_string(), execution.result.clone()),
        ("decoded".to_string(), execution.decoded.clone()),
    ]))
}
