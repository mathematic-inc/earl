use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn top_level_help_lists_web_command() {
    let mut cmd = cargo_bin_cmd!("earl");
    cmd.arg("--help");

    let out = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(out).unwrap();

    assert!(stdout.contains("web"));
}

#[test]
fn web_help_shows_expected_flags() {
    let mut cmd = cargo_bin_cmd!("earl");
    cmd.args(["web", "--help"]);

    let out = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(out).unwrap();

    assert!(stdout.contains("--listen"));
    assert!(stdout.contains("--no-open"));
}
