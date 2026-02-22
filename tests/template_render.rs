use earl::template::render::{render_json_value, render_string_raw};
use serde_json::json;

#[test]
fn renders_pure_expression_with_typed_value() {
    let context = json!({"args": {"count": 42}});
    let rendered = render_json_value(&json!("{{ args.count }}"), &context).unwrap();
    assert_eq!(rendered, json!(42));
}

#[test]
fn renders_mixed_text_as_string() {
    let context = json!({"args": {"name": "world"}});
    let rendered = render_json_value(&json!("hello {{ args.name }}"), &context).unwrap();
    assert_eq!(rendered, json!("hello world"));
}

#[test]
fn fails_on_undefined_variable_due_to_strict_mode() {
    let context = json!({"args": {}});
    let err = render_string_raw("{{ args.missing }}", &context).unwrap_err();
    assert!(err.to_string().contains("template render failed"));
}

#[test]
fn renders_object_keys_and_values() {
    let context = json!({"args": {"key": "x-id", "value": "123"}});
    let value = json!({"{{ args.key }}": "{{ args.value }}"});
    let rendered = render_json_value(&value, &context).unwrap();
    assert_eq!(rendered, json!({"x-id": 123}));
}
