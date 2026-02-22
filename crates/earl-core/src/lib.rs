pub mod allowlist;
pub mod decode;
pub mod redact;
pub mod render;
pub mod schema;
pub mod transport;

use std::future::Future;

use serde_json::{Map, Value};

pub use allowlist::ensure_url_allowed;
pub use decode::{DecodedBody, decode_response};
pub use redact::Redactor;
pub use render::TemplateRenderer;
pub use schema::{AllowRule, CommandMode, ResultDecode, ResultExtract, ResultTemplate};
pub use transport::ResolvedTransport;

/// Body data prepared for HTTP-like protocol execution.
#[derive(Debug, Clone)]
pub enum PreparedBody {
    Empty,
    Json(Value),
    Form(Vec<(String, String)>),
    Multipart(Vec<PreparedMultipartPart>),
    RawBytes {
        bytes: Vec<u8>,
        content_type: Option<String>,
    },
}

/// A single part in a multipart body.
#[derive(Debug, Clone)]
pub struct PreparedMultipartPart {
    pub name: String,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
    pub filename: Option<String>,
}

/// Unified result type returned by all protocol executors.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub status: u16,
    pub url: String,
    pub result: Value,
    pub decoded: Value,
}

/// Raw protocol output before decode/extract post-processing.
#[derive(Debug, Clone)]
pub struct RawExecutionResult {
    pub status: u16,
    pub url: String,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
}

/// Shared execution context passed to all protocol executors alongside
/// their protocol-specific prepared data.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub key: String,
    pub mode: CommandMode,
    pub allow_rules: Vec<AllowRule>,
    pub transport: ResolvedTransport,
    pub result_template: ResultTemplate,
    pub args: Map<String, Value>,
    pub redactor: Redactor,
}

/// Contract implemented by all protocol executors.
///
/// Each protocol crate provides an executor struct that implements this trait.
/// The associated `PreparedData` type links the executor to its matching
/// prepared data produced by the builder.
pub trait ProtocolExecutor {
    /// Protocol-specific prepared data (e.g. `PreparedHttpData`, `PreparedBashScript`).
    type PreparedData: Clone + std::fmt::Debug + Send + Sync;

    /// Execute a single protocol request and return the raw result.
    fn execute(
        &mut self,
        data: &Self::PreparedData,
        context: &ExecutionContext,
    ) -> impl Future<Output = anyhow::Result<RawExecutionResult>> + Send;
}
