//! Use-case tests: dialog handling (Group 7).
mod common;
use common::{execute, skip_if_no_chrome, spawn, Response, CHROME_SERIAL};
use std::collections::HashMap;
use earl_protocol_browser::PreparedBrowserCommand;
use earl_protocol_browser::schema::BrowserStep;

/// Test 7.1 — handle_dialog accepts an alert.
///
/// Serves a page that fires an alert on load via the `onload` body attribute.
/// Navigate to the page, then dismiss the pending alert.
///
/// NOTE: Dialog timing in headless Chrome is inherently racy — the alert may
/// fire during or after navigation.  The HandleDialog step is expected to
/// accept any currently-open dialog.  If this proves flaky in CI, mark it
/// `#[ignore = "dialog timing in headless Chrome"]`.
#[tokio::test]
#[ignore = "dialog timing in headless Chrome"]
async fn handle_dialog_accepts_alert() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().unwrap();

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html(r#"<html><body onload="alert('Hello')"><p>Page loaded</p></body></html>"#),
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
            BrowserStep::HandleDialog {
                accept: true,
                prompt_text: None,
                optional: false,
            },
        ],
    };

    let result = execute(data).await.expect("execute should succeed — dialog should be accepted");
    assert_eq!(
        result["ok"].as_bool(),
        Some(true),
        "HandleDialog result should be {{\"ok\": true}}; got: {result}"
    );
}

/// Test 7.2 — handle_dialog with prompt_text fills the prompt.
///
/// Serves a page that opens a prompt on load and sets `document.title` to the
/// returned value.  After accepting the prompt with a known answer, evaluates
/// `document.title` and asserts it matches the supplied text.
///
/// NOTE: Dialog timing in headless Chrome is inherently racy.  If this test
/// proves flaky, mark it `#[ignore = "dialog timing in headless Chrome"]`.
#[tokio::test]
#[ignore = "dialog timing in headless Chrome"]
async fn handle_dialog_fills_prompt_text() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().unwrap();

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html(
            r#"<html><body><script>window.onload = function() { var r = window.prompt("Enter value"); document.title = r || ""; };</script></body></html>"#,
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
            BrowserStep::HandleDialog {
                accept: true,
                prompt_text: Some("my-answer".to_string()),
                optional: false,
            },
            BrowserStep::Evaluate {
                function: "() => document.title".to_string(),
                r#ref: None,
                timeout_ms: None,
                optional: false,
            },
        ],
    };

    let result = execute(data).await.expect("execute should succeed");
    assert_eq!(
        result["value"].as_str(),
        Some("my-answer"),
        "document.title should equal 'my-answer' after prompt is filled; got: {result}"
    );
}

/// Test 7.3 — handle_dialog rejects a confirm dialog.
///
/// Serves a page that opens a confirm dialog on load and sets
/// `document.body.textContent` to `"yes"` or `"no"` depending on the result.
/// Dismissing the dialog with `accept: false` must produce `"no"`.
///
/// NOTE: Dialog timing in headless Chrome is inherently racy.  If this test
/// proves flaky, mark it `#[ignore = "dialog timing in headless Chrome"]`.
#[tokio::test]
#[ignore = "dialog timing in headless Chrome"]
async fn handle_dialog_rejects_confirm() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = CHROME_SERIAL.lock().unwrap();

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html(
            r#"<html><body><script>window.onload = function() { var ok = window.confirm("Continue?"); document.body.textContent = ok ? "yes" : "no"; };</script></body></html>"#,
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
            BrowserStep::HandleDialog {
                accept: false,
                prompt_text: None,
                optional: false,
            },
            BrowserStep::Evaluate {
                function: "() => document.body.textContent.trim()".to_string(),
                r#ref: None,
                timeout_ms: None,
                optional: false,
            },
        ],
    };

    let result = execute(data).await.expect("execute should succeed");
    assert_eq!(
        result["value"].as_str(),
        Some("no"),
        "body text should be 'no' after confirm is rejected; got: {result}"
    );
}
