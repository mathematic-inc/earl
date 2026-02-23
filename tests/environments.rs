use std::path::Path;

use earl::template::environments::{resolve_active_env, select_for_env};
use earl::template::parser::parse_template_hcl;
use earl::template::validator::validate_template_file;

fn load_fixture() -> earl::template::schema::TemplateFile {
    let content = include_str!("fixtures/environments_test.hcl");
    parse_template_hcl(content, Path::new("tests/fixtures")).unwrap()
}

#[test]
fn fixture_parses_and_validates() {
    let file = load_fixture();
    validate_template_file(&file).unwrap();
}

#[test]
fn fixture_has_two_environments() {
    let file = load_fixture();
    let envs = file.environments.as_ref().unwrap();
    assert!(envs.environments.contains_key("production"));
    assert!(envs.environments.contains_key("staging"));
    assert_eq!(envs.default.as_deref(), Some("production"));
}

#[test]
fn fixture_environment_vars_accessible() {
    let file = load_fixture();
    let envs = file.environments.as_ref().unwrap();
    assert_eq!(
        envs.environments["production"]["base_url"],
        "https://prod.example.com"
    );
    assert_eq!(
        envs.environments["staging"]["base_url"],
        "https://staging.example.com"
    );
}

#[test]
#[cfg(feature = "http")]
fn select_uses_default_operation_when_no_env() {
    let file = load_fixture();
    let cmd = file.commands.get("override_in_staging").unwrap();
    let (op, _) = select_for_env(cmd, None);
    // Default operation is HTTP GET
    assert!(matches!(
        op,
        earl::template::schema::OperationTemplate::Http(_)
    ));
}

#[test]
#[cfg(feature = "bash")]
fn select_uses_override_for_matching_env() {
    let file = load_fixture();
    let cmd = file.commands.get("override_in_staging").unwrap();
    let (op, _) = select_for_env(cmd, Some("staging"));
    assert!(matches!(
        op,
        earl::template::schema::OperationTemplate::Bash(_)
    ));
    // The staging override's script is "echo staging_override"
    assert!(op.bash_script().unwrap().contains("staging_override"));
}

#[test]
#[cfg(feature = "http")]
fn select_falls_back_to_default_for_unrecognized_env() {
    let file = load_fixture();
    let cmd = file.commands.get("override_in_staging").unwrap();
    // "development" has no override — should fall back to default (HTTP GET)
    let (op, _) = select_for_env(cmd, Some("development"));
    assert!(matches!(
        op,
        earl::template::schema::OperationTemplate::Http(_)
    ));
}

#[test]
fn resolve_active_env_priority() {
    assert_eq!(
        resolve_active_env(Some("cli"), Some("config"), Some("template")),
        Some("cli")
    );
    assert_eq!(
        resolve_active_env(None, Some("config"), Some("template")),
        Some("config")
    );
    assert_eq!(
        resolve_active_env(None, None, Some("template")),
        Some("template")
    );
    assert_eq!(resolve_active_env(None::<&str>, None, None), None);
}

#[test]
#[cfg(feature = "bash")]
fn command_with_no_overrides_always_uses_default() {
    let file = load_fixture();
    let cmd = file.commands.get("echo_env").unwrap();
    let (op_none, _) = select_for_env(cmd, None);
    let (op_prod, _) = select_for_env(cmd, Some("production"));
    let (op_stg, _) = select_for_env(cmd, Some("staging"));
    // echo_env has no per-command overrides, so all three should return the same operation
    assert_eq!(
        op_none.bash_script().unwrap(),
        op_prod.bash_script().unwrap()
    );
    assert_eq!(
        op_none.bash_script().unwrap(),
        op_stg.bash_script().unwrap()
    );
}

// ── ProviderEnvironments struct construction ──────────────────────────────

#[test]
fn provider_environments_struct_constructed_correctly() {
    // Verify ProviderEnvironments fields are accessible as expected.
    // Actual redaction behaviour (that resolve_vars tracks values) is
    // covered by the builder unit test `resolve_vars_resolves_and_tracks_values`.
    use earl::template::schema::ProviderEnvironments;
    use std::collections::BTreeMap;

    let mut staging_vars: BTreeMap<String, String> = BTreeMap::new();
    staging_vars.insert("token".to_string(), "super_secret_value".to_string());

    let pe = ProviderEnvironments {
        default: None,
        secrets: vec![],
        environments: BTreeMap::from([("staging".to_string(), staging_vars)]),
    };

    assert!(pe.environments.contains_key("staging"));
    assert_eq!(pe.environments["staging"]["token"], "super_secret_value");
}
