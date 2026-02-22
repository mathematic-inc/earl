use anyhow::{Result, bail};
use serde_json::Value;

use crate::PreparedSqlQuery;
use crate::schema::SqlOperationTemplate;
use earl_core::TemplateRenderer;

/// Build a complete `PreparedSqlQuery` from a SQL operation template.
///
/// `connection_url` is passed in already resolved — secret resolution
/// stays in the main crate.
pub fn build_sql_request(
    sql_op: &SqlOperationTemplate,
    context: &Value,
    renderer: &dyn TemplateRenderer,
    connection_url: String,
) -> Result<PreparedSqlQuery> {
    let query = renderer.render_str(&sql_op.sql.query, context)?;
    if query.trim().is_empty() {
        bail!("operation.sql.query rendered empty");
    }

    let params = if let Some(param_templates) = &sql_op.sql.params {
        let mut rendered = Vec::with_capacity(param_templates.len());
        for p in param_templates {
            rendered.push(renderer.render_value(p, context)?);
        }
        rendered
    } else {
        Vec::new()
    };

    let read_only = sql_op
        .sql
        .sandbox
        .as_ref()
        .and_then(|s| s.read_only)
        .unwrap_or(true);
    let max_rows = sql_op
        .sql
        .sandbox
        .as_ref()
        .and_then(|s| s.max_rows)
        .unwrap_or(1000) as usize;

    Ok(PreparedSqlQuery {
        connection_url,
        query,
        params,
        read_only,
        max_rows,
    })
}
