use anyhow::{Result, bail};
use serde_json::Value;

use earl_core::{ExecutionContext, ProtocolExecutor, RawExecutionResult};

use crate::PreparedSqlQuery;

/// Execute a single SQL query via sqlx and return the result.
pub async fn execute_sql_once(
    data: &PreparedSqlQuery,
    ctx: &ExecutionContext,
) -> Result<RawExecutionResult> {
    let result = crate::sandbox::execute_query(
        &data.connection_url,
        &data.query,
        &data.params,
        data.read_only,
        data.max_rows,
        ctx.transport.timeout,
    )
    .await;

    // Wrap errors through the redactor to prevent credential leakage.
    let rows = match result {
        Ok(rows) => rows,
        Err(err) => {
            let redacted_msg = ctx.redactor.redact(&format!("{err:#}"));
            bail!("SQL query failed: {redacted_msg}");
        }
    };

    let result_value = Value::Array(rows.into_iter().map(Value::Object).collect());
    let body = serde_json::to_vec(&result_value)?;

    Ok(RawExecutionResult {
        status: 0,
        url: "sql://query".into(),
        body,
        content_type: Some("application/json".to_string()),
    })
}

/// SQL protocol executor.
pub struct SqlExecutor;

impl ProtocolExecutor for SqlExecutor {
    type PreparedData = PreparedSqlQuery;

    async fn execute(
        &mut self,
        data: &PreparedSqlQuery,
        ctx: &ExecutionContext,
    ) -> anyhow::Result<RawExecutionResult> {
        execute_sql_once(data, ctx).await
    }
}
