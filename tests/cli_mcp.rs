use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn top_level_help_lists_mcp_command() {
    let mut cmd = cargo_bin_cmd!("earl");
    cmd.arg("--help");

    let out = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(out).unwrap();
    assert!(stdout.contains("mcp"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("web"));
    assert!(stdout.contains("completion"));
}

#[test]
fn mcp_help_shows_transport_and_flags() {
    let mut cmd = cargo_bin_cmd!("earl");
    cmd.args(["mcp", "--help"]);

    let out = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(out).unwrap();

    assert!(stdout.contains("stdio"));
    assert!(stdout.contains("http"));
    assert!(stdout.contains("--listen"));
    assert!(stdout.contains("--mode"));
    assert!(stdout.contains("discovery"));
    assert!(stdout.contains("--yes"));
}

#[test]
fn completion_generates_bash_script() {
    let mut cmd = cargo_bin_cmd!("earl");
    cmd.args(["completion", "bash"]);

    let out = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(out).unwrap();

    assert!(stdout.contains("_earl"));
    assert!(stdout.contains("complete -F"));
}
