use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use earl_core::{ExecutionContext, RawExecutionResult};

use crate::PreparedBashScript;
use crate::sandbox::{build_sandboxed_command, sandbox_available, sandbox_tool_name};

/// Execute a single bash script inside a sandbox and return the result.
pub async fn execute_bash_once(
    data: &PreparedBashScript,
    ctx: &ExecutionContext,
) -> Result<RawExecutionResult> {
    if !sandbox_available() {
        bail!(
            "bash sandbox tool ({}) is not available on this system; \
             install it or disable the bash feature",
            sandbox_tool_name()
        );
    }

    let mut command =
        build_sandboxed_command(&data.script, &data.env, data.cwd.as_deref(), &data.sandbox)?;

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if data.stdin.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }

    // SAFETY: setsid() creates a new session / process group so that we can
    // kill the entire group on timeout without leaking children.
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    let mut child = command
        .spawn()
        .context("failed spawning sandboxed bash command")?;

    let pid = child
        .id()
        .ok_or_else(|| anyhow::anyhow!("failed to obtain PID of spawned bash process"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed capturing bash stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed capturing bash stderr"))?;

    // Use sandbox output limit if set, otherwise fall back to transport limit.
    let max_bytes = data
        .sandbox
        .max_output_bytes
        .unwrap_or(ctx.transport.max_response_bytes);

    let stdout_reader =
        tokio::spawn(async move { read_stream_limited(stdout, max_bytes, "stdout").await });
    let stderr_reader =
        tokio::spawn(async move { read_stream_limited(stderr, max_bytes, "stderr").await });

    // Write stdin if present, then drop the handle so the child sees EOF.
    if let Some(input) = &data.stdin
        && let Some(mut stdin_handle) = child.stdin.take()
    {
        stdin_handle
            .write_all(input.as_bytes())
            .await
            .context("failed writing stdin to bash process")?;
    }

    // Use sandbox timeout if set, otherwise fall back to transport timeout.
    let timeout = data
        .sandbox
        .max_time_ms
        .map(Duration::from_millis)
        .unwrap_or(ctx.transport.timeout);

    let status = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(wait_result) => wait_result.context("failed waiting for bash process")?,
        Err(_) => {
            // Timeout: kill the entire process group.
            if let Ok(pgid) = i32::try_from(pid) {
                unsafe { libc::killpg(pgid, libc::SIGKILL) };
            }
            let _ = child.kill().await;
            let _ = child.wait().await;
            bail!("bash script timed out after {timeout:?}");
        }
    };

    let stdout_bytes = stdout_reader
        .await
        .context("failed joining stdout reader task")??;
    let stderr_bytes = stderr_reader
        .await
        .context("failed joining stderr reader task")??;

    let exit_code = status
        .code()
        .map(|c| c.clamp(0, u16::MAX as i32) as u16)
        .unwrap_or(1);

    let output_bytes = if stdout_bytes.is_empty() {
        &stderr_bytes
    } else {
        &stdout_bytes
    };

    Ok(RawExecutionResult {
        status: exit_code,
        url: "bash://script".into(),
        body: output_bytes.to_vec(),
        content_type: None,
    })
}

use earl_core::ProtocolExecutor;

/// Bash protocol executor.
pub struct BashExecutor;

impl ProtocolExecutor for BashExecutor {
    type PreparedData = PreparedBashScript;

    async fn execute(
        &mut self,
        data: &PreparedBashScript,
        ctx: &ExecutionContext,
    ) -> anyhow::Result<RawExecutionResult> {
        execute_bash_once(data, ctx).await
    }
}

async fn read_stream_limited<R>(mut reader: R, limit: usize, label: &str) -> Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut out = Vec::new();
    let mut buf = [0_u8; 8192];

    loop {
        let bytes_read = reader
            .read(&mut buf)
            .await
            .with_context(|| format!("failed reading bash {label}"))?;
        if bytes_read == 0 {
            break;
        }
        if out.len().saturating_add(bytes_read) > limit {
            bail!("bash {label} exceeded configured max_response_bytes ({limit} bytes)");
        }
        out.extend_from_slice(&buf[..bytes_read]);
    }

    Ok(out)
}
