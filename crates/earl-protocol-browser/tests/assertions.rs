//! Use-case tests: multi-step assertion steps (Group 9).
mod common;
use common::{CHROME_SERIAL, Response, execute, skip_if_no_chrome, spawn};
use earl_protocol_browser::PreparedBrowserCommand;
use earl_protocol_browser::schema::BrowserStep;
use std::collections::HashMap;

/// Test 9.1 — verify_text_visible passes when text is present.
#[tokio::test]
async fn verify_text_visible_passes_when_text_present() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().await;

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
#[tokio::test]
async fn verify_text_visible_fails_when_text_absent() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().await;

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
#[tokio::test]
async fn verify_element_visible_passes_when_element_exists() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().await;

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
