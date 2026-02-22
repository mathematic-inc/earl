use earl::output::human::render_human_output;
use earl::output::json::render_json_output;
use earl::protocol::executor::ExecutionResult;
use earl::template::schema::{ResultDecode, ResultTemplate};
use serde_json::{Map, json};

#[test]
fn renders_human_output_with_alias() {
    let template = ResultTemplate {
        decode: ResultDecode::Json,
        extract: None,
        output: "id={{ issue.id }} q={{ args.query }}".to_string(),
        result_alias: Some("issue".to_string()),
    };

    let mut args = Map::new();
    args.insert("query".to_string(), json!("hello"));
    let result = json!({"id": 123});

    let out = render_human_output(&template, &args, &result).unwrap();
    assert_eq!(out, "id=123 q=hello");
}

#[test]
fn renders_structured_json_output_shape() {
    let execution = ExecutionResult {
        status: 200,
        url: "https://api.example.com".to_string(),
        result: json!({"ok": true}),
        decoded: json!({"raw": "value"}),
    };

    let out = render_json_output(&execution);
    assert_eq!(out["status"], json!(200));
    assert_eq!(out["url"], json!("https://api.example.com"));
    assert_eq!(out["result"]["ok"], json!(true));
    assert_eq!(out["decoded"]["raw"], json!("value"));
}
