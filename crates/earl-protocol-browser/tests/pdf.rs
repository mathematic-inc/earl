//! Use-case tests: PDF save.
mod common;
use common::{Response, execute, skip_if_no_chrome, spawn};
use earl_protocol_browser::PreparedBrowserCommand;
use earl_protocol_browser::schema::BrowserStep;
use std::collections::HashMap;

fn unique_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", std::process::id(), count)
}

/// Test 8.1 — pdf_save writes a valid PDF to disk.
///
/// Serves an HTML invoice page, requests a PDF at an explicit temp path, and
/// verifies that the file exists and starts with the PDF magic bytes `%PDF`.
#[tokio::test]
async fn pdf_save_writes_valid_pdf_to_disk() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = common::chrome_lock().await;

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html("<html><body><h1>Invoice #001</h1><p>Total: $42.00</p></body></html>"),
    );
    let server = spawn(routes).await;

    let id = unique_id();
    let path = std::env::temp_dir().join(format!("earl-test-invoice-{id}.pdf"));
    let path_str = path.to_string_lossy().to_string();

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
            BrowserStep::PdfSave {
                path: Some(path_str.clone()),
                optional: false,
            },
        ],
    };

    let result = execute(data).await.expect("execute should succeed");

    assert_eq!(
        result["path"].as_str(),
        Some(path_str.as_str()),
        "result 'path' field should match the requested path; got: {}",
        result["path"]
    );
    assert!(
        std::path::Path::new(&path_str).exists(),
        "PDF file should exist on disk at {path_str}"
    );

    // Verify the PDF magic bytes: %PDF = [0x25, 0x50, 0x44, 0x46]
    let bytes = std::fs::read(&path_str).expect("should be able to read the PDF file");
    assert!(
        bytes.len() >= 4,
        "PDF file should contain at least 4 bytes; got {} bytes",
        bytes.len()
    );
    assert_eq!(
        &bytes[..4],
        &[0x25, 0x50, 0x44, 0x46],
        "expected PDF magic bytes (%PDF); got: {:?}",
        &bytes[..4]
    );

    std::fs::remove_file(&path_str).ok();
}

/// Test 8.2 — pdf_save with no path creates a temporary file.
///
/// Omits the `path` field so that the executor chooses a temp file location.
/// Verifies the returned path is non-empty, ends with `.pdf`, and exists on
/// disk.
#[tokio::test]
async fn pdf_save_no_path_creates_temp_file() {
    if skip_if_no_chrome() {
        return;
    }

    let _guard = common::chrome_lock().await;

    let mut routes = HashMap::new();
    routes.insert(
        "GET /".to_string(),
        Response::html("<html><body><p>temp PDF test</p></body></html>"),
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
            BrowserStep::PdfSave {
                path: None,
                optional: false,
            },
        ],
    };

    let result = execute(data).await.expect("execute should succeed");

    let returned_path = result["path"]
        .as_str()
        .expect("result should have a non-null 'path' field");

    assert!(
        !returned_path.is_empty(),
        "returned path should be a non-empty string"
    );
    assert!(
        returned_path.ends_with(".pdf"),
        "returned path should end with '.pdf'; got: {returned_path}"
    );
    assert!(
        std::path::Path::new(returned_path).exists(),
        "PDF file should exist on disk at {returned_path}"
    );

    std::fs::remove_file(returned_path).ok();
}
