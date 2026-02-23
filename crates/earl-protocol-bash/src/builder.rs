use anyhow::{Result, bail};
use serde_json::Value;

use crate::schema::BashOperationTemplate;
use crate::{PreparedBashScript, ResolvedBashSandbox};
use earl_core::render::{TemplateRenderer, render_key_value_map};

/// Global sandbox limits passed from the main crate's SandboxConfig.
#[derive(Debug, Clone, Default)]
pub struct GlobalBashLimits {
    pub allow_network: bool,
    pub max_time_ms: Option<u64>,
    pub max_output_bytes: Option<u64>,
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_time_ms: Option<u64>,
}

/// Build a complete `PreparedBashScript` from a Bash operation template.
pub fn build_bash_request(
    bash_op: &BashOperationTemplate,
    context: &Value,
    renderer: &dyn TemplateRenderer,
    global_limits: &GlobalBashLimits,
) -> Result<PreparedBashScript> {
    let script = renderer.render_str(&bash_op.bash.script, context)?;
    if script.trim().is_empty() {
        bail!("operation.bash.script rendered empty");
    }

    let env = render_key_value_map(bash_op.bash.env.as_ref(), context, renderer)?;

    let cwd = bash_op
        .bash
        .cwd
        .as_ref()
        .map(|value| renderer.render_str(value, context))
        .transpose()?
        .filter(|value| !value.trim().is_empty());

    // Extract per-template sandbox config with safe defaults, then apply
    // global limits (most-restrictive-wins).
    let template_sandbox = &bash_op.bash.sandbox;

    let network = template_sandbox
        .as_ref()
        .and_then(|s| s.network)
        .unwrap_or(false)
        && global_limits.allow_network;

    let writable_paths = template_sandbox
        .as_ref()
        .and_then(|s| s.writable_paths.clone())
        .unwrap_or_default();

    let max_time_ms = most_restrictive_option(
        template_sandbox.as_ref().and_then(|s| s.max_time_ms),
        global_limits.max_time_ms,
    );

    let max_output_bytes = most_restrictive_option(
        template_sandbox.as_ref().and_then(|s| s.max_output_bytes),
        global_limits.max_output_bytes,
    )
    .map(|v| v as usize);

    let max_memory_bytes = most_restrictive_option(
        template_sandbox.as_ref().and_then(|s| s.max_memory_bytes),
        global_limits.max_memory_bytes,
    )
    .map(|v| v as usize);

    let max_cpu_time_ms = most_restrictive_option(
        template_sandbox.as_ref().and_then(|s| s.max_cpu_time_ms),
        global_limits.max_cpu_time_ms,
    );

    Ok(PreparedBashScript {
        script,
        env,
        cwd,
        stdin: None,
        sandbox: ResolvedBashSandbox {
            network,
            writable_paths,
            max_time_ms,
            max_output_bytes,
            max_memory_bytes,
            max_cpu_time_ms,
        },
    })
}

/// Return the smaller of two optional limits (most-restrictive-wins).
fn most_restrictive_option(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}
