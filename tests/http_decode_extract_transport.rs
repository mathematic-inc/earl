use std::collections::BTreeMap;

use earl::protocol::extract::extract_result;
use earl::protocol::transport::resolve_transport;
use earl::template::schema::{
    RedirectTemplate, ResultDecode, ResultExtract, RetryTemplate, TlsTemplate, TransportTemplate,
};
use earl_core::decode::{DecodedBody, decode_response};
use serde_json::json;

#[test]
fn decode_auto_json_and_text_modes() {
    let json_bytes = br#"{"ok":true}"#;
    let decoded =
        decode_response(ResultDecode::Auto, Some("application/json"), json_bytes).unwrap();
    match decoded {
        DecodedBody::Json(value) => assert_eq!(value["ok"], json!(true)),
        _ => panic!("expected JSON"),
    }

    let text_bytes = b"hello";
    let decoded = decode_response(ResultDecode::Auto, Some("text/plain"), text_bytes).unwrap();
    match decoded {
        DecodedBody::Text(value) => assert_eq!(value, "hello"),
        _ => panic!("expected text"),
    }
}

#[test]
fn decode_binary_mode_roundtrip_to_json_value() {
    let bytes = vec![1_u8, 2, 3, 4];
    let decoded = decode_response(
        ResultDecode::Binary,
        Some("application/octet-stream"),
        &bytes,
    )
    .unwrap();
    let as_json = decoded.to_json_value();
    assert!(as_json.as_str().unwrap().len() > 4);
}

#[test]
fn extract_json_pointer_regex_css_and_xpath() {
    let decoded_json = DecodedBody::Json(json!({"data": {"id": 42}}));
    let out = extract_result(
        Some(&ResultExtract::JsonPointer {
            json_pointer: "/data/id".to_string(),
        }),
        &decoded_json,
    )
    .unwrap();
    assert_eq!(out, json!(42));

    let decoded_text = DecodedBody::Text("id=abc-123".to_string());
    let out = extract_result(
        Some(&ResultExtract::Regex {
            regex: "id=([a-z0-9-]+)".to_string(),
        }),
        &decoded_text,
    )
    .unwrap();
    assert_eq!(out, json!("abc-123"));

    let decoded_html =
        DecodedBody::Html("<html><body><h1>Hello</h1><h1>World</h1></body></html>".to_string());
    let out = extract_result(
        Some(&ResultExtract::CssSelector {
            css_selector: "h1".to_string(),
        }),
        &decoded_html,
    )
    .unwrap();
    assert_eq!(out, json!(["Hello", "World"]));

    let decoded_xml = DecodedBody::Xml("<root><item>A</item><item>B</item></root>".to_string());
    let out = extract_result(
        Some(&ResultExtract::XPath {
            xpath: "//item/text()".to_string(),
        }),
        &decoded_xml,
    )
    .unwrap();
    assert_eq!(out, json!(["A", "B"]));
}

#[test]
fn extract_reports_failures() {
    let decoded_json = DecodedBody::Json(json!({"a": 1}));
    let err = extract_result(
        Some(&ResultExtract::JsonPointer {
            json_pointer: "/missing".to_string(),
        }),
        &decoded_json,
    )
    .unwrap_err();
    assert!(err.to_string().contains("did not match"));
}

#[test]
fn transport_defaults_and_overrides() {
    let defaults = resolve_transport(None, &BTreeMap::new()).unwrap();
    assert_eq!(defaults.retry_max_attempts, 1);
    assert_eq!(defaults.max_redirect_hops, 5);
    assert!(defaults.compression);
    assert_eq!(defaults.max_response_bytes, 8 * 1024 * 1024);

    let override_input = TransportTemplate {
        timeout_ms: Some(2_000),
        max_response_bytes: Some(16 * 1024),
        redirects: Some(RedirectTemplate {
            follow: false,
            max_hops: 2,
        }),
        retry: Some(RetryTemplate {
            max_attempts: 0,
            backoff_ms: 0,
            retry_on_status: vec![429, 500],
        }),
        compression: Some(true),
        tls: Some(TlsTemplate {
            min_version: Some("1.2".to_string()),
        }),
        proxy_profile: Some("corp".to_string()),
    };

    let proxy_profiles = BTreeMap::from([(
        "corp".to_string(),
        earl::config::ProxyProfile {
            url: "http://127.0.0.1:8888".to_string(),
        },
    )]);

    let resolved = resolve_transport(Some(&override_input), &proxy_profiles).unwrap();
    assert_eq!(resolved.retry_max_attempts, 1);
    assert_eq!(resolved.max_redirect_hops, 2);
    assert!(!resolved.follow_redirects);
    assert_eq!(resolved.retry_on_status, vec![429, 500]);
    assert_eq!(resolved.timeout.as_millis(), 2_000);
    assert!(resolved.compression);
    assert_eq!(resolved.max_response_bytes, 16 * 1024);
    assert_eq!(resolved.proxy_url.as_deref(), Some("http://127.0.0.1:8888"));
    assert_eq!(
        resolved.tls_min_version,
        Some(reqwest::tls::Version::TLS_1_2)
    );
}
