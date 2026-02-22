mod common;

use earl::template::catalog::TemplateScope;
use earl::template::loader::load_catalog_from_dirs;

#[test]
fn local_overrides_global_for_same_command_key() {
    let ws = common::temp_workspace();

    let global_hcl = r#"
version = 1
provider = "github"
categories = ["global_cat"]

command "search_issues" {
  title = "Global Search"
  summary = "Global search command"
  description = "global version"
  categories = ["global_cmd"]

  annotations {
    mode = "read"
    secrets = []
  }

  operation {
    protocol = "http"
    method = "GET"
    url = "https://api.github.com/search/issues"
  }

  result {
    output = "global"
  }
}
"#;

    let local_hcl = r#"
version = 1
provider = "github"
categories = ["local_cat"]

command "search_issues" {
  title = "Local Search"
  summary = "Local search command"
  description = "local version"
  categories = ["local_cmd"]

  annotations {
    mode = "read"
    secrets = []
  }

  operation {
    protocol = "http"
    method = "GET"
    url = "https://api.github.com/search/issues"
  }

  result {
    output = "local"
  }
}
"#;

    common::write_template(&ws.global_templates, "github.hcl", global_hcl);
    common::write_template(&ws.local_templates, "github.hcl", local_hcl);

    let catalog = load_catalog_from_dirs(&ws.global_templates, &ws.local_templates).unwrap();
    let entry = catalog.get("github.search_issues").unwrap();

    assert_eq!(entry.title, "Local Search");
    assert_eq!(entry.summary, "Local search command");
    assert_eq!(entry.description, "local version");
    assert_eq!(entry.source.scope, TemplateScope::Local);
    assert!(entry.categories.contains(&"local_cat".to_string()));
    assert!(entry.categories.contains(&"local_cmd".to_string()));
}

#[test]
fn loads_multiple_commands_from_single_provider_file() {
    let ws = common::temp_workspace();

    let hcl = r#"
version = 1
provider = "github"
categories = ["scm"]

command "search_issues" {
  title = "Search Issues"
  summary = "Search issues command"
  description = "Search issues in repositories"

  annotations {
    mode = "read"
    secrets = []
  }

  operation {
    protocol = "http"
    method = "GET"
    url = "https://api.github.com/search/issues"
  }

  result {
    output = "ok"
  }
}

command "create_issue" {
  title = "Create Issue"
  summary = "Create issue command"
  description = "Create an issue in a repository"

  annotations {
    mode = "write"
    secrets = []
  }

  operation {
    protocol = "http"
    method = "POST"
    url = "https://api.github.com/repos/org/repo/issues"
  }

  result {
    output = "ok"
  }
}
"#;

    common::write_template(&ws.local_templates, "github.hcl", hcl);

    let catalog = load_catalog_from_dirs(&ws.global_templates, &ws.local_templates).unwrap();
    assert!(catalog.get("github.search_issues").is_some());
    assert!(catalog.get("github.create_issue").is_some());
}
