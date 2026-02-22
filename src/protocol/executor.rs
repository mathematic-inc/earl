use std::future::Future;
use std::net::IpAddr;
use std::time::Duration;

use anyhow::{Context, Result};
use url::Url;

use super::extract::extract_result;
#[allow(unused_imports)]
use earl_core::ProtocolExecutor;
use earl_core::decode_response;

use crate::security::dns::resolve_and_validate_host;

#[allow(unused_imports)]
use super::builder::PreparedProtocolData;
use super::builder::PreparedRequest;

pub use earl_core::ExecutionResult;

pub async fn execute_prepared_request(prepared: &PreparedRequest) -> Result<ExecutionResult> {
    execute_prepared_request_with_host_validator(prepared, |url: Url| async move {
        let host = url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("request URL missing host"))?;
        resolve_and_validate_host(host).await
    })
    .await
}

pub async fn execute_prepared_request_with_host_validator<F, Fut>(
    prepared: &PreparedRequest,
    mut host_validator: F,
) -> Result<ExecutionResult>
where
    F: FnMut(Url) -> Fut + Send,
    Fut: Future<Output = Result<Vec<IpAddr>>> + Send,
{
    let attempts = prepared.transport.retry_max_attempts.max(1);
    let mut last_err: Option<anyhow::Error> = None;

    for attempt in 1..=attempts {
        #[allow(unreachable_patterns)]
        let outcome: Result<earl_core::RawExecutionResult> = match &prepared.protocol_data {
            #[cfg(feature = "http")]
            PreparedProtocolData::Http(http_data) => {
                earl_protocol_http::HttpExecutor {
                    host_validator: &mut host_validator,
                }
                .execute(http_data, &to_context(prepared))
                .await
            }
            #[cfg(feature = "graphql")]
            PreparedProtocolData::Graphql(http_data) => {
                earl_protocol_http::HttpExecutor {
                    host_validator: &mut host_validator,
                }
                .execute(http_data, &to_context(prepared))
                .await
            }
            #[cfg(feature = "grpc")]
            PreparedProtocolData::Grpc(grpc_data) => {
                earl_protocol_grpc::GrpcExecutor {
                    host_validator: &mut host_validator,
                }
                .execute(grpc_data, &to_context(prepared))
                .await
            }
            #[cfg(feature = "bash")]
            PreparedProtocolData::Bash(bash_data) => {
                earl_protocol_bash::BashExecutor
                    .execute(bash_data, &to_context(prepared))
                    .await
            }
            #[cfg(feature = "sql")]
            PreparedProtocolData::Sql(sql_data) => {
                earl_protocol_sql::SqlExecutor
                    .execute(sql_data, &to_context(prepared))
                    .await
            }
            _ => Err(anyhow::anyhow!(
                "unsupported protocol (feature not enabled)"
            )),
        };

        match outcome {
            Ok(raw) => {
                if prepared.transport.retry_on_status.contains(&raw.status) && attempt < attempts {
                    tokio::time::sleep(backoff(prepared.transport.retry_backoff, attempt)).await;
                    continue;
                }
                let decoded_body = decode_response(
                    prepared.result_template.decode,
                    raw.content_type.as_deref(),
                    &raw.body,
                )
                .context("failed decoding response")?;
                let extracted =
                    extract_result(prepared.result_template.extract.as_ref(), &decoded_body)?;
                return Ok(ExecutionResult {
                    status: raw.status,
                    url: raw.url,
                    result: extracted,
                    decoded: decoded_body.to_json_value(),
                });
            }
            Err(err) => {
                if attempt >= attempts {
                    return Err(err);
                }
                last_err = Some(err);
                tokio::time::sleep(backoff(prepared.transport.retry_backoff, attempt)).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("request failed without details")))
}

fn backoff(base: Duration, attempt: usize) -> Duration {
    let factor = attempt.max(1) as u32;
    base.saturating_mul(factor)
}

fn to_context(prepared: &PreparedRequest) -> earl_core::ExecutionContext {
    earl_core::ExecutionContext {
        key: prepared.key.clone(),
        mode: prepared.mode,
        allow_rules: prepared.allow_rules.clone(),
        transport: prepared.transport.clone(),
        result_template: prepared.result_template.clone(),
        args: prepared.args.clone(),
        redactor: prepared.redactor.clone(),
    }
}
