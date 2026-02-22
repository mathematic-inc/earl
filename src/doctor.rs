use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config::{self, Config, OAuthFlow, OAuthProfile};
use crate::secrets::SecretManager;
use crate::template::catalog::TemplateCatalog;
use crate::template::loader::{is_template_file, load_catalog_from_dirs, validate_all_from_dirs};
use crate::template::schema::AuthTemplate;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DoctorStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    pub id: String,
    pub status: DoctorStatus,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorSummary {
    pub ok: usize,
    pub warning: usize,
    pub error: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn add_ok(&mut self, id: &str, message: impl Into<String>) {
        self.checks.push(DoctorCheck {
            id: id.to_string(),
            status: DoctorStatus::Ok,
            message: message.into(),
            suggestion: None,
        });
    }

    pub fn add_warning(
        &mut self,
        id: &str,
        message: impl Into<String>,
        suggestion: Option<String>,
    ) {
        self.checks.push(DoctorCheck {
            id: id.to_string(),
            status: DoctorStatus::Warning,
            message: message.into(),
            suggestion,
        });
    }

    pub fn add_error(&mut self, id: &str, message: impl Into<String>, suggestion: Option<String>) {
        self.checks.push(DoctorCheck {
            id: id.to_string(),
            status: DoctorStatus::Error,
            message: message.into(),
            suggestion,
        });
    }

    pub fn summary(&self) -> DoctorSummary {
        let mut summary = DoctorSummary {
            ok: 0,
            warning: 0,
            error: 0,
        };
        for check in &self.checks {
            match check.status {
                DoctorStatus::Ok => summary.ok += 1,
                DoctorStatus::Warning => summary.warning += 1,
                DoctorStatus::Error => summary.error += 1,
            }
        }
        summary
    }

    pub fn has_errors(&self) -> bool {
        self.checks
            .iter()
            .any(|check| matches!(check.status, DoctorStatus::Error))
    }

    pub fn error_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|check| matches!(check.status, DoctorStatus::Error))
            .count()
    }
}

pub fn run_checks(cwd: &Path) -> DoctorReport {
    let mut report = DoctorReport::default();
    let config_path = config::config_file_path();

    let cfg = if !config_path.exists() {
        report.add_warning(
            "config_file",
            format!("Config file not found at {}", config_path.display()),
            Some(
                "Create ~/.config/earl/config.toml (for example by copying examples/config.toml)."
                    .to_string(),
            ),
        );
        Config::default()
    } else {
        match config::load_config() {
            Ok(cfg) => {
                report.add_ok(
                    "config_file",
                    format!("Loaded config from {}", config_path.display()),
                );
                cfg
            }
            Err(err) => {
                report.add_error(
                    "config_file",
                    format!("Failed to parse {}: {err:#}", config_path.display()),
                    Some("Fix config TOML syntax, then re-run `earl doctor`.".to_string()),
                );
                Config::default()
            }
        }
    };

    if cfg.network.allow.is_empty() {
        report.add_ok(
            "network_allowlist",
            "No [[network.allow]] entries configured; outbound requests are allowed by default.",
        );
    } else {
        report.add_ok(
            "network_allowlist",
            format!("Configured {} allowlist rule(s).", cfg.network.allow.len()),
        );
    }

    let global_dir = config::global_templates_dir();
    let local_dir = config::local_templates_dir(cwd);

    let mut discovery_failed = false;
    let global_files = match discover_template_files(&global_dir) {
        Ok(files) => files,
        Err(err) => {
            discovery_failed = true;
            report.add_error(
                "templates",
                format!(
                    "Failed listing global templates in {}: {err:#}",
                    global_dir.display()
                ),
                None,
            );
            Vec::new()
        }
    };

    let local_files = match discover_template_files(&local_dir) {
        Ok(files) => files,
        Err(err) => {
            discovery_failed = true;
            report.add_error(
                "templates",
                format!(
                    "Failed listing local templates in {}: {err:#}",
                    local_dir.display()
                ),
                None,
            );
            Vec::new()
        }
    };

    let template_count = global_files.len() + local_files.len();
    if !discovery_failed {
        if template_count == 0 {
            report.add_warning(
                "templates",
                format!(
                    "No template files found in {} or {}.",
                    local_dir.display(),
                    global_dir.display()
                ),
                Some(
                    "Add a .hcl template under ./templates or ~/.config/earl/templates."
                        .to_string(),
                ),
            );
        } else {
            report.add_ok(
                "templates",
                format!(
                    "Found {} template file(s) (local: {}, global: {}).",
                    template_count,
                    local_files.len(),
                    global_files.len()
                ),
            );
        }
    }

    if !discovery_failed && template_count > 0 {
        match validate_all_from_dirs(&global_dir, &local_dir) {
            Ok(files) => report.add_ok(
                "template_validation",
                format!("Validated {} template file(s).", files.len()),
            ),
            Err(err) => report.add_error(
                "template_validation",
                format!("Template validation failed: {err:#}"),
                Some("Run `earl templates validate` for details.".to_string()),
            ),
        }
    }

    let mut catalog = None;
    if !discovery_failed {
        match load_catalog_from_dirs(&global_dir, &local_dir) {
            Ok(found) => {
                if found.entries.is_empty() {
                    report.add_warning(
                        "commands",
                        "No commands available after loading templates.".to_string(),
                        Some(
                            "Ensure template files define at least one `command \"<name>\" { ... }` block."
                                .to_string(),
                        ),
                    );
                } else {
                    report.add_ok(
                        "commands",
                        format!("Loaded {} command(s).", found.entries.len()),
                    );
                }
                catalog = Some(found);
            }
            Err(err) => {
                report.add_error(
                    "commands",
                    format!("Failed loading template catalog: {err:#}"),
                    None,
                );
            }
        }
    }

    if let Some(catalog) = &catalog {
        check_required_secrets(catalog, &mut report);
        check_oauth_profiles(catalog, &cfg, &mut report);
    }

    check_remote_search(&cfg, &mut report);
    check_jwt(&cfg, &mut report);
    check_policies(&cfg, &mut report);

    #[cfg(feature = "bash")]
    check_bash(&mut report);

    report
}

fn check_required_secrets(catalog: &TemplateCatalog, report: &mut DoctorReport) {
    let mut required = BTreeSet::new();
    for entry in catalog.values() {
        for secret in &entry.template.annotations.secrets {
            required.insert(secret.clone());
        }
    }

    if required.is_empty() {
        report.add_ok("required_secrets", "Templates do not require any secrets.");
        return;
    }

    let manager = SecretManager::new();
    let mut missing = Vec::new();

    for key in &required {
        match manager.store().get_secret(key) {
            Ok(Some(_)) => {}
            Ok(None) => missing.push(key.clone()),
            Err(err) => {
                report.add_warning(
                    "required_secrets",
                    format!("Could not verify keychain secrets: {err:#}"),
                    Some(
                        "Ensure keychain access is available, then run `earl doctor` again."
                            .to_string(),
                    ),
                );
                return;
            }
        }
    }

    if missing.is_empty() {
        report.add_ok(
            "required_secrets",
            format!("All {} required secret(s) are present.", required.len()),
        );
    } else {
        report.add_warning(
            "required_secrets",
            format!(
                "Missing {} required secret(s): {}",
                missing.len(),
                summarize_items(&missing, 5)
            ),
            Some("Set missing values with `earl secrets set <key>`.".to_string()),
        );
    }
}

fn check_oauth_profiles(catalog: &TemplateCatalog, cfg: &Config, report: &mut DoctorReport) {
    let mut referenced = BTreeSet::new();
    for entry in catalog.values() {
        if let Some(AuthTemplate::OAuth2Profile { profile }) = entry.template.operation.auth() {
            referenced.insert(profile.clone());
        }
    }

    if referenced.is_empty() {
        report.add_ok(
            "oauth_profiles",
            "Templates do not reference any oauth2_profile auth settings.",
        );
        return;
    }

    let mut missing = Vec::new();
    let mut invalid = Vec::new();

    for profile_name in &referenced {
        let Some(profile) = cfg.auth.profiles.get(profile_name) else {
            missing.push(profile_name.clone());
            continue;
        };

        let issues = validate_profile(profile);
        if !issues.is_empty() {
            invalid.push(format!("{} ({})", profile_name, issues.join("; ")));
        }
    }

    if missing.is_empty() && invalid.is_empty() {
        report.add_ok(
            "oauth_profiles",
            format!(
                "All {} referenced OAuth profile(s) are configured.",
                referenced.len()
            ),
        );
        return;
    }

    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!(
            "missing profiles: {}",
            summarize_items(&missing, 5)
        ));
    }
    if !invalid.is_empty() {
        parts.push(format!(
            "invalid profiles: {}",
            summarize_items(&invalid, 3)
        ));
    }

    report.add_error(
        "oauth_profiles",
        parts.join("; "),
        Some(
            "Add or fix entries under [auth.profiles.<name>] in ~/.config/earl/config.toml."
                .to_string(),
        ),
    );
}

fn validate_profile(profile: &OAuthProfile) -> Vec<String> {
    let mut issues = Vec::new();

    if profile.client_id.trim().is_empty() {
        issues.push("client_id is empty".to_string());
    }

    let has_issuer = profile
        .issuer
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

    if missing_text(&profile.token_url) && !has_issuer {
        issues.push("missing token_url (or issuer for discovery)".to_string());
    }

    match profile.flow {
        OAuthFlow::AuthCodePkce => {
            if missing_text(&profile.authorization_url) && !has_issuer {
                issues.push("missing authorization_url (or issuer for discovery)".to_string());
            }
        }
        OAuthFlow::DeviceCode => {
            if missing_text(&profile.device_authorization_url) && !has_issuer {
                issues
                    .push("missing device_authorization_url (or issuer for discovery)".to_string());
            }
        }
        OAuthFlow::ClientCredentials => {}
    }

    issues
}

fn check_remote_search(cfg: &Config, report: &mut DoctorReport) {
    let remote = &cfg.search.remote;
    if !remote.enabled {
        report.add_ok("remote_search", "Remote semantic search is disabled.");
        return;
    }

    let mut issues = Vec::new();
    if missing_text(&remote.base_url) {
        issues.push("base_url is not set".to_string());
    }
    if missing_text(&remote.api_key_secret) {
        issues.push("api_key_secret is not set".to_string());
    }

    if !issues.is_empty() {
        report.add_warning(
            "remote_search",
            format!("Remote semantic search enabled but {}", issues.join(", ")),
            Some("Set [search.remote].base_url and [search.remote].api_key_secret.".to_string()),
        );
        return;
    }

    let manager = SecretManager::new();
    let secret_key = remote.api_key_secret.as_ref().expect("checked above");
    match manager.store().get_secret(secret_key) {
        Ok(Some(_)) => report.add_ok(
            "remote_search",
            format!(
                "Remote semantic search is configured (base_url: {}).",
                remote.base_url.as_deref().unwrap_or_default()
            ),
        ),
        Ok(None) => report.add_warning(
            "remote_search",
            format!("Remote search API key secret `{secret_key}` is missing."),
            Some(format!("Set it with `earl secrets set {secret_key}`.")),
        ),
        Err(err) => report.add_warning(
            "remote_search",
            format!("Could not verify remote search API key secret: {err:#}"),
            Some("Ensure keychain access is available, then run `earl doctor` again.".to_string()),
        ),
    }
}

fn check_jwt(cfg: &Config, report: &mut DoctorReport) {
    let Some(jwt) = &cfg.auth.jwt else {
        report.add_warning(
            "jwt_config",
            "No [auth.jwt] configured. MCP HTTP transport will require --allow-unauthenticated.",
            Some(
                "Add [auth.jwt] to ~/.config/earl/config.toml to enable JWT authentication."
                    .to_string(),
            ),
        );
        return;
    };

    let has_discovery = jwt.oidc_discovery_url.is_some();
    let has_manual = jwt.issuer.is_some() || jwt.jwks_uri.is_some();
    if has_discovery && has_manual {
        report.add_error(
            "jwt_config",
            "Both oidc_discovery_url and issuer/jwks_uri are set. These are mutually exclusive.",
            Some("Remove either oidc_discovery_url or issuer/jwks_uri.".to_string()),
        );
        return;
    }
    if !has_discovery && !has_manual {
        report.add_error(
            "jwt_config",
            "Neither oidc_discovery_url nor issuer/jwks_uri are configured.",
            Some("Set oidc_discovery_url (recommended) or both issuer and jwks_uri.".to_string()),
        );
        return;
    }

    if jwt.audience.trim().is_empty() {
        report.add_error(
            "jwt_audience",
            "JWT audience is empty.",
            Some("Set audience to a non-empty value in [auth.jwt].".to_string()),
        );
    } else {
        report.add_ok("jwt_audience", format!("JWT audience: {}", jwt.audience));
    }

    if jwt.algorithms.is_empty() {
        report.add_warning(
            "jwt_algorithms",
            "No JWT algorithms configured.",
            Some("Add algorithms (e.g., [\"RS256\"]) to [auth.jwt].".to_string()),
        );
    } else if jwt.algorithms.iter().all(|a| a.to_uppercase() == "HS256") {
        report.add_warning(
            "jwt_algorithms",
            "Only HS256 is configured. Symmetric HMAC is unusual for external IdP tokens.",
            Some("Most IdPs use RS256. Verify your IdP's signing algorithm.".to_string()),
        );
    } else {
        report.add_ok(
            "jwt_algorithms",
            format!("JWT algorithms: {}", jwt.algorithms.join(", ")),
        );
    }

    if jwt.clock_skew_seconds > 300 {
        report.add_warning(
            "jwt_config",
            format!(
                "clock_skew_seconds is {} (max effective: 300).",
                jwt.clock_skew_seconds
            ),
            Some("Values above 300 are capped. Consider reducing to 30-60.".to_string()),
        );
    }

    if let Some(issuer) = &jwt.issuer {
        if !issuer.starts_with("https://") {
            report.add_warning(
                "jwt_issuer",
                format!("JWT issuer does not use HTTPS: {issuer}"),
                Some("Use an HTTPS URL for the issuer in production.".to_string()),
            );
        } else {
            report.add_ok("jwt_issuer", format!("JWT issuer: {issuer}"));
        }
    }

    if let Some(jwks_uri) = &jwt.jwks_uri {
        report.add_ok("jwt_jwks", format!("JWT JWKS URI: {jwks_uri}"));
    } else if let Some(discovery_url) = &jwt.oidc_discovery_url {
        report.add_ok(
            "jwt_config",
            format!("JWT using OIDC discovery: {discovery_url}"),
        );
    }
}

fn check_policies(cfg: &Config, report: &mut DoctorReport) {
    if cfg.policy.is_empty() {
        if cfg.auth.jwt.is_some() {
            report.add_warning(
                "policies",
                "JWT authentication is configured but no [[policy]] rules are defined. All tool access will be denied (default deny).",
                Some("Add [[policy]] rules to config.toml to grant access.".to_string()),
            );
        }
        return;
    }

    let mut has_issues = false;
    for (i, rule) in cfg.policy.iter().enumerate() {
        if rule.subjects.is_empty() {
            has_issues = true;
            report.add_warning(
                "policies",
                format!(
                    "Policy rule {} has empty subjects list (matches nothing).",
                    i + 1
                ),
                None,
            );
        }
        if rule.tools.is_empty() {
            has_issues = true;
            report.add_warning(
                "policies",
                format!(
                    "Policy rule {} has empty tools list (matches nothing).",
                    i + 1
                ),
                None,
            );
        }
    }

    if !has_issues {
        report.add_ok(
            "policies",
            format!("Loaded {} policy rule(s).", cfg.policy.len()),
        );
    }
}

#[cfg(feature = "bash")]
fn check_bash(report: &mut DoctorReport) {
    if earl_protocol_bash::sandbox::sandbox_available() {
        let tool = earl_protocol_bash::sandbox::sandbox_tool_name();
        if cfg!(target_os = "macos") {
            report.add_warning(
                "bash",
                format!("Bash sandbox uses {} (deprecated on macOS).", tool),
                Some(
                    "Consider installing bubblewrap via Homebrew for stronger isolation."
                        .to_string(),
                ),
            );
        } else {
            report.add_ok("bash", format!("Bash sandbox tool available: {}", tool));
        }
    } else {
        report.add_error(
            "bash",
            format!(
                "Bash sandbox tool not found ({}).",
                earl_protocol_bash::sandbox::sandbox_tool_name()
            ),
            Some("Install bubblewrap (Linux) or verify sandbox-exec (macOS).".to_string()),
        );
    }
}

fn discover_template_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_discovered_template_files(dir, &mut files)?;

    files.sort();
    Ok(files)
}

fn collect_discovered_template_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed listing template directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed inspecting template path {}", path.display()))?;
        if file_type.is_dir() {
            collect_discovered_template_files(&path, files)?;
            continue;
        }
        if file_type.is_file() && is_template_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn missing_text(value: &Option<String>) -> bool {
    value
        .as_deref()
        .map(|text| text.trim().is_empty())
        .unwrap_or(true)
}

fn summarize_items(items: &[String], limit: usize) -> String {
    let mut parts = Vec::new();
    for item in items.iter().take(limit) {
        parts.push(item.clone());
    }

    if items.len() > limit {
        parts.push(format!("... +{} more", items.len() - limit));
    }

    parts.join(", ")
}
