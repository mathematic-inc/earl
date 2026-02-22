mod common;

use earl::search::index::build_documents;
use earl::template::loader::load_catalog_from_dirs;

#[test]
fn builds_corpus_from_template_fields() {
    let ws = common::temp_workspace();
    let hcl = r#"
version = 1
provider = "github"
categories = ["scm", "issues"]

command "search_issues" {
  title = "Search Issues"
  summary = "Search issues by text"
  description = <<-EOF
Finds issues by text query.

## Example
`earl call github.search_issues --query "bug"`
EOF
  categories = ["search"]

  annotations {
    mode = "read"
    secrets = []
  }

  param "query" {
    type = "string"
    required = true
  }

  operation {
    protocol = "http"
    method = "GET"
    url = "https://api.github.com/search/issues"
  }

  result {
    output = "Found {{ result.total_count }}"
  }
}
"#;

    common::write_template(&ws.local_templates, "github.hcl", hcl);
    let catalog = load_catalog_from_dirs(&ws.global_templates, &ws.local_templates).unwrap();

    let docs = build_documents(&catalog);
    assert_eq!(docs.len(), 1);
    let doc = &docs[0];

    assert_eq!(doc.key, "github.search_issues");
    assert_eq!(doc.mode, "read");
    assert!(doc.categories.contains(&"scm".to_string()));
    assert!(doc.categories.contains(&"search".to_string()));
    assert!(doc.text.contains("Search Issues"));
    assert!(doc.text.contains("Search issues by text"));
    assert!(doc.text.contains("Finds issues by text query."));
    assert!(doc.text.contains("## Example"));
    assert!(doc.text.contains("https://api.github.com/search/issues"));
    assert!(doc.text.contains("query:string"));
    assert!(doc.text.contains("Found {{ result.total_count }}"));
}
