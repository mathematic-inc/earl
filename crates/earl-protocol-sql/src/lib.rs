pub mod builder;
pub mod executor;
pub mod sandbox;
pub mod schema;

pub use executor::SqlExecutor;
pub use schema::{SqlOperationTemplate, SqlQueryTemplate, SqlSandboxTemplate};

/// Prepared SQL query data, ready for execution.
#[derive(Debug, Clone)]
pub struct PreparedSqlQuery {
    pub connection_url: String,
    pub query: String,
    pub params: Vec<serde_json::Value>,
    pub read_only: bool,
    pub max_rows: usize,
}
