use earl_core::Redactor;
use serde_json::json;

#[test]
fn redacts_plaintext_and_overlapping_values() {
    let redactor = Redactor::new(vec!["token-abc".to_string(), "abc".to_string()]);
    let input = "Authorization: Bearer token-abc";
    let output = redactor.redact(input);
    assert!(!output.contains("token-abc"));
    assert!(output.contains("[REDACTED]"));
}

#[test]
fn redacts_nested_json_values() {
    let redactor = Redactor::new(vec!["super-secret".to_string()]);
    let payload = json!({
        "token": "super-secret",
        "nested": {
            "arr": ["ok", "super-secret"]
        }
    });

    let redacted = redactor.redact_json(&payload);
    assert_eq!(redacted["token"], json!("[REDACTED]"));
    assert_eq!(redacted["nested"]["arr"][1], json!("[REDACTED]"));
}

#[test]
fn redacts_common_encoded_secret_forms() {
    let redactor = Redactor::new(vec!["super-secret".to_string()]);
    let input = "b64=c3VwZXItc2VjcmV0 url=super-secret hex=73757065722d736563726574";
    let output = redactor.redact(input);

    assert!(!output.contains("c3VwZXItc2VjcmV0"));
    assert!(!output.contains("73757065722d736563726574"));
    assert!(output.contains("[REDACTED]"));
}
