use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::config::ProxyProfile;
use crate::template::schema::TransportTemplate;

pub use earl_core::transport::ResolvedTransport;

const DEFAULT_MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

pub fn resolve_transport(
    input: Option<&TransportTemplate>,
    proxy_profiles: &BTreeMap<String, ProxyProfile>,
) -> Result<ResolvedTransport> {
    let timeout = input
        .and_then(|t| t.timeout_ms)
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(30));

    let (follow_redirects, max_redirect_hops) = input
        .and_then(|t| t.redirects.as_ref())
        .map(|r| (r.follow, r.max_hops))
        .unwrap_or((true, 5));

    let (retry_max_attempts, retry_backoff, retry_on_status) = input
        .and_then(|t| t.retry.as_ref())
        .map(|r| {
            (
                if r.max_attempts == 0 {
                    1
                } else {
                    r.max_attempts
                },
                Duration::from_millis(r.backoff_ms.max(1)),
                r.retry_on_status.clone(),
            )
        })
        .unwrap_or((1, Duration::from_millis(250), Vec::new()));

    let compression = input.and_then(|t| t.compression).unwrap_or(true);
    let tls_min_version = parse_tls_min_version(input)?;
    let proxy_url = resolve_proxy_url(input, proxy_profiles)?;
    let max_response_bytes = input
        .and_then(|t| t.max_response_bytes)
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(DEFAULT_MAX_RESPONSE_BYTES)
        .clamp(1024, 128 * 1024 * 1024);

    Ok(ResolvedTransport {
        timeout,
        follow_redirects,
        max_redirect_hops,
        retry_max_attempts,
        retry_backoff,
        retry_on_status,
        compression,
        tls_min_version,
        proxy_url,
        max_response_bytes,
    })
}

fn parse_tls_min_version(
    input: Option<&TransportTemplate>,
) -> Result<Option<reqwest::tls::Version>> {
    let Some(raw) = input
        .and_then(|t| t.tls.as_ref())
        .and_then(|tls| tls.min_version.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        return Ok(None);
    };

    let version = match raw {
        "1.0" => reqwest::tls::Version::TLS_1_0,
        "1.1" => reqwest::tls::Version::TLS_1_1,
        "1.2" => reqwest::tls::Version::TLS_1_2,
        "1.3" => reqwest::tls::Version::TLS_1_3,
        _ => bail!("unsupported tls.min_version `{raw}`; expected one of 1.0, 1.1, 1.2, 1.3"),
    };

    Ok(Some(version))
}

fn resolve_proxy_url(
    input: Option<&TransportTemplate>,
    proxy_profiles: &BTreeMap<String, ProxyProfile>,
) -> Result<Option<String>> {
    let Some(profile_name) = input
        .and_then(|t| t.proxy_profile.as_deref())
        .map(str::trim)
        .filter(|name| !name.is_empty())
    else {
        return Ok(None);
    };

    let profile = proxy_profiles
        .get(profile_name)
        .with_context(|| format!("unknown transport proxy_profile `{profile_name}`"))?;

    let proxy_url = profile.url.trim();
    if proxy_url.is_empty() {
        bail!("proxy profile `{profile_name}` has empty url");
    }

    Ok(Some(proxy_url.to_string()))
}
