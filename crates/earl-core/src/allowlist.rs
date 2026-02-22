use anyhow::{Result, bail};
use url::Url;

use crate::schema::AllowRule;

pub fn ensure_url_allowed(url: &Url, allow_rules: &[AllowRule]) -> Result<()> {
    if allow_rules.is_empty() {
        return Ok(());
    }

    if allow_rules.iter().any(|rule| matches_rule(url, rule)) {
        return Ok(());
    }

    bail!(
        "url `{}` is not allowed by template allowlist policy",
        url.as_str()
    )
}

pub fn matches_rule(url: &Url, rule: &AllowRule) -> bool {
    let scheme_match = url.scheme().eq_ignore_ascii_case(&rule.scheme);
    let host_match = url
        .host_str()
        .map(|host| host.eq_ignore_ascii_case(&rule.host))
        .unwrap_or(false);
    let port_match = url
        .port_or_known_default()
        .map(|port| port == rule.port)
        .unwrap_or(false);
    let path_match = path_prefix_matches(url.path(), &rule.path_prefix);

    scheme_match && host_match && port_match && path_match
}

fn path_prefix_matches(path: &str, prefix: &str) -> bool {
    let normalized_path = normalize_path(path);
    let normalized_prefix = normalize_path(prefix);

    if normalized_prefix == "/" {
        return true;
    }

    normalized_path == normalized_prefix
        || normalized_path.starts_with(&format!("{normalized_prefix}/"))
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    let with_leading = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    if with_leading.len() > 1 {
        with_leading.trim_end_matches('/').to_string()
    } else {
        "/".to_string()
    }
}
