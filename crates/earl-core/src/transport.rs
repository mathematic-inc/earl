use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ResolvedTransport {
    pub timeout: Duration,
    pub follow_redirects: bool,
    pub max_redirect_hops: usize,
    pub retry_max_attempts: usize,
    pub retry_backoff: Duration,
    pub retry_on_status: Vec<u16>,
    pub compression: bool,
    pub tls_min_version: Option<reqwest::tls::Version>,
    pub proxy_url: Option<String>,
    pub max_response_bytes: usize,
}
