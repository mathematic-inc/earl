use std::net::IpAddr;
use std::str::FromStr;

use earl::security::ssrf::{ensure_safe_ip, is_blocked_ip};
use earl::template::schema::AllowRule;
use earl_core::allowlist::{ensure_url_allowed, matches_rule};
use url::Url;

fn rule() -> AllowRule {
    AllowRule {
        scheme: "https".to_string(),
        host: "api.github.com".to_string(),
        port: 443,
        path_prefix: "/search/issues".to_string(),
    }
}

#[test]
fn allowlist_matches_scheme_host_port_and_path_prefix() {
    let allowed = Url::parse("https://api.github.com/search/issues?q=abc").unwrap();
    let allowed_exact = Url::parse("https://api.github.com/search/issues").unwrap();
    let disallowed_scheme = Url::parse("http://api.github.com/search/issues").unwrap();
    let disallowed_host = Url::parse("https://example.com/search/issues").unwrap();
    let disallowed_port = Url::parse("https://api.github.com:8443/search/issues").unwrap();
    let disallowed_path = Url::parse("https://api.github.com/repos/owner/repo").unwrap();
    let disallowed_prefix_boundary =
        Url::parse("https://api.github.com/search/issues-archive").unwrap();

    assert!(matches_rule(&allowed, &rule()));
    assert!(matches_rule(&allowed_exact, &rule()));
    assert!(!matches_rule(&disallowed_scheme, &rule()));
    assert!(!matches_rule(&disallowed_host, &rule()));
    assert!(!matches_rule(&disallowed_port, &rule()));
    assert!(!matches_rule(&disallowed_path, &rule()));
    assert!(!matches_rule(&disallowed_prefix_boundary, &rule()));

    ensure_url_allowed(&allowed, &[rule()]).unwrap();
    assert!(ensure_url_allowed(&disallowed_path, &[rule()]).is_err());
    assert!(ensure_url_allowed(&disallowed_prefix_boundary, &[rule()]).is_err());
}

#[test]
fn empty_allowlist_allows_all_urls() {
    let url = Url::parse("https://example.com/anything").unwrap();
    ensure_url_allowed(&url, &[]).unwrap();
}

#[test]
fn ssrf_blocks_unsafe_ranges_and_allows_public_ip() {
    let blocked = [
        "127.0.0.1",
        "10.0.0.1",
        "169.254.169.254",
        "100.64.0.1",
        "198.18.0.1",
        "240.0.0.1",
        "0.0.0.0",
        "::1",
        "fe80::1",
        "fd00::1",
        "::ffff:10.0.0.1",
    ];

    for ip in blocked {
        let parsed = IpAddr::from_str(ip).unwrap();
        assert!(is_blocked_ip(parsed), "expected blocked IP: {ip}");
        assert!(ensure_safe_ip(parsed).is_err());
    }

    let public = IpAddr::from_str("8.8.8.8").unwrap();
    assert!(!is_blocked_ip(public));
    ensure_safe_ip(public).unwrap();
}
