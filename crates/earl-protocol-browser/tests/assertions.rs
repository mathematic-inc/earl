//! Use-case tests: multi-step assertion steps (Group 9).
//!
//! These tests define acceptance criteria for `VerifyTextVisible` and
//! `VerifyElementVisible` steps that are currently stubs in the executor.
//! Every test in this file is marked `#[ignore]` and must remain so until the
//! steps are fully implemented.
mod common;
use common::{execute, skip_if_no_chrome, spawn, Response, CHROME_SERIAL};
use std::collections::HashMap;
use earl_protocol_browser::PreparedBrowserCommand;
use earl_protocol_browser::schema::BrowserStep;

/// Test 9.1 — verify_text_visible passes when text is present.
///
/// Defines acceptance criteria for the VerifyTextVisible step.
/// TODO: remove #[ignore] when verify_text_visible is implemented.
#[tokio::test]
#[ignore = "VerifyTextVisible step is a stub — remove when implemented"]
async fn verify_text_visible_passes_when_text_present() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().unwrap();

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html("<html><body><p>Order confirmed</p></body></html>"),
    );
    let server = spawn(routes).await;

    let data = PreparedBrowserCommand {
        session_id: None,
        headless: true,
        timeout_ms: 30_000,
        on_failure_screenshot: false,
        steps: vec![
            BrowserStep::Navigate {
                url: server.url("/"),
                expected_status: None,
                timeout_ms: None,
                optional: false,
            },
            BrowserStep::VerifyTextVisible {
                text: "Order confirmed".to_string(),
                optional: false,
            },
        ],
    };

    execute(data).await.expect(
        "VerifyTextVisible should succeed when the text 'Order confirmed' is present in the DOM",
    );
}

/// Test 9.2 — verify_text_visible fails when text is absent.
///
/// Defines acceptance criteria for the VerifyTextVisible step.
/// TODO: remove #[ignore] when verify_text_visible is implemented.
#[tokio::test]
#[ignore = "VerifyTextVisible step is a stub — remove when implemented"]
async fn verify_text_visible_fails_when_text_absent() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().unwrap();

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html("<html><body><p>Nothing here</p></body></html>"),
    );
    let server = spawn(routes).await;

    let data = PreparedBrowserCommand {
        session_id: None,
        headless: true,
        timeout_ms: 30_000,
        on_failure_screenshot: false,
        steps: vec![
            BrowserStep::Navigate {
                url: server.url("/"),
                expected_status: None,
                timeout_ms: None,
                optional: false,
            },
            BrowserStep::VerifyTextVisible {
                text: "Order confirmed".to_string(),
                optional: false,
            },
        ],
    };

    let result = execute(data).await;
    assert!(
        result.is_err(),
        "VerifyTextVisible should fail with an error when 'Order confirmed' is not present; got Ok: {:?}",
        result.ok()
    );
}

/// Test 9.3 — verify_element_visible passes when element exists.
///
/// Defines acceptance criteria for the VerifyElementVisible step.
/// TODO: remove #[ignore] when verify_element_visible is implemented.
#[tokio::test]
#[ignore = "VerifyElementVisible step is a stub — remove when implemented"]
async fn verify_element_visible_passes_when_element_exists() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().unwrap();

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html(
            r#"<html><body><button role="button" aria-label="Submit">Submit</button></body></html>"#,
        ),
    );
    let server = spawn(routes).await;

    let data = PreparedBrowserCommand {
        session_id: None,
        headless: true,
        timeout_ms: 30_000,
        on_failure_screenshot: false,
        steps: vec![
            BrowserStep::Navigate {
                url: server.url("/"),
                expected_status: None,
                timeout_ms: None,
                optional: false,
            },
            BrowserStep::VerifyElementVisible {
                role: Some("button".to_string()),
                accessible_name: Some("Submit".to_string()),
                optional: false,
            },
        ],
    };

    execute(data).await.expect(
        "VerifyElementVisible should succeed when a button with aria-label 'Submit' is present",
    );
}
