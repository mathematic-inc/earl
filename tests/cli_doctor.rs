use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn write_config(home: &std::path::Path, content: &str) {
    let config_path = home.join(".config/earl/config.toml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(config_path, content).unwrap();
}

fn write_template(cwd: &std::path::Path, name: &str, content: &str) {
    let path = cwd.join("templates").join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn doctor_succeeds_when_network_allowlist_is_missing() {
    let cwd = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let mut cmd = cargo_bin_cmd!("earl");
    cmd.current_dir(cwd.path())
        .env("HOME", home.path())
        .args(["doctor"]);

    let out = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(out).unwrap();
    assert!(stdout.contains("[OK] network_allowlist"));
    assert!(stdout.contains("outbound requests are allowed by default"));
    assert!(stdout.contains("Summary:"));
}

#[test]
fn doctor_succeeds_for_minimal_valid_setup() {
    let cwd = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    write_config(
        home.path(),
        r#"
[[network.allow]]
scheme = "https"
host = "api.example.com"
port = 443
path_prefix = "/"
"#,
    );
    write_template(
        cwd.path(),
        "demo.hcl",
        include_str!("fixtures/templates/valid_minimal.hcl"),
    );

    let mut cmd = cargo_bin_cmd!("earl");
    cmd.current_dir(cwd.path())
        .env("HOME", home.path())
        .args(["doctor"]);

    let out = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(out).unwrap();
    assert!(stdout.contains("[OK] network_allowlist"));
    assert!(stdout.contains("[OK] template_validation"));
    assert!(stdout.contains("Summary:"));
    assert!(stdout.contains("0 error"));
}

#[test]
fn doctor_json_output_includes_summary_and_checks() {
    let cwd = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    write_config(
        home.path(),
        r#"
[[network.allow]]
scheme = "https"
host = "api.example.com"
port = 443
path_prefix = "/"
"#,
    );
    write_template(
        cwd.path(),
        "demo.hcl",
        include_str!("fixtures/templates/valid_minimal.hcl"),
    );

    let mut cmd = cargo_bin_cmd!("earl");
    cmd.current_dir(cwd.path())
        .env("HOME", home.path())
        .args(["doctor", "--json"]);

    let out = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed["summary"]["error"], 0);
    assert!(parsed["summary"]["ok"].as_u64().unwrap() > 0);
    assert!(!parsed["checks"].as_array().unwrap().is_empty());
}
