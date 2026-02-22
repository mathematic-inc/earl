use serde_json::Value;
use thiserror::Error;

use crate::template::schema::{ParamSpec, ParamType};

use super::ast::CallExpression;

#[derive(Debug, Error)]
pub enum CliArgsError {
    #[error("invalid command format `{0}`: expected provider.command")]
    InvalidCommand(String),
    #[error("unknown parameter `--{0}`")]
    UnknownParam(String),
    #[error("missing value for parameter `--{0}`")]
    MissingValue(String),
    #[error("failed to parse value for `--{name}`: expected {expected}, got `{value}`")]
    InvalidValue {
        name: String,
        expected: &'static str,
        value: String,
    },
    #[error("unexpected bare argument `{0}` (use --param-name value)")]
    BareArgument(String),
}

pub fn parse_cli_args(
    command: &str,
    raw_args: &[String],
    params: &[ParamSpec],
) -> Result<CallExpression, CliArgsError> {
    let (provider, cmd) = command
        .split_once('.')
        .ok_or_else(|| CliArgsError::InvalidCommand(command.to_string()))?;

    if provider.is_empty() || cmd.is_empty() {
        return Err(CliArgsError::InvalidCommand(command.to_string()));
    }

    let mut named_args: Vec<(String, Value)> = Vec::new();
    let mut i = 0;

    while i < raw_args.len() {
        let token = &raw_args[i];

        let Some(param_name) = token.strip_prefix("--") else {
            return Err(CliArgsError::BareArgument(token.clone()));
        };

        if param_name.is_empty() {
            return Err(CliArgsError::BareArgument(token.clone()));
        }

        let spec = params
            .iter()
            .find(|p| p.name == param_name)
            .ok_or_else(|| CliArgsError::UnknownParam(param_name.to_string()))?;

        i += 1;

        let value = if spec.r#type == ParamType::Boolean {
            // Booleans: --flag (no value) means true, --flag true/false also supported
            if i < raw_args.len() && !raw_args[i].starts_with("--") {
                let raw = &raw_args[i];
                match raw.as_str() {
                    "true" => {
                        i += 1;
                        Value::Bool(true)
                    }
                    "false" => {
                        i += 1;
                        Value::Bool(false)
                    }
                    _ => {
                        // Next token isn't a bool literal — treat --flag as true
                        Value::Bool(true)
                    }
                }
            } else {
                Value::Bool(true)
            }
        } else {
            // Non-boolean: requires a value
            if i >= raw_args.len() {
                return Err(CliArgsError::MissingValue(param_name.to_string()));
            }
            let raw = &raw_args[i];
            i += 1;
            parse_typed_value(param_name, raw, spec.r#type)?
        };

        named_args.push((param_name.to_string(), value));
    }

    Ok(CallExpression {
        provider: provider.to_string(),
        command: cmd.to_string(),
        positional_args: vec![],
        named_args,
    })
}

fn parse_typed_value(name: &str, raw: &str, param_type: ParamType) -> Result<Value, CliArgsError> {
    match param_type {
        ParamType::String => Ok(Value::String(raw.to_string())),
        ParamType::Integer => {
            let n: i64 = raw.parse().map_err(|_| CliArgsError::InvalidValue {
                name: name.to_string(),
                expected: "integer",
                value: raw.to_string(),
            })?;
            Ok(Value::Number(n.into()))
        }
        ParamType::Number => {
            let n: f64 = raw.parse().map_err(|_| CliArgsError::InvalidValue {
                name: name.to_string(),
                expected: "number",
                value: raw.to_string(),
            })?;
            serde_json::Number::from_f64(n)
                .map(Value::Number)
                .ok_or_else(|| CliArgsError::InvalidValue {
                    name: name.to_string(),
                    expected: "number",
                    value: raw.to_string(),
                })
        }
        ParamType::Boolean => match raw {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(CliArgsError::InvalidValue {
                name: name.to_string(),
                expected: "boolean",
                value: raw.to_string(),
            }),
        },
        ParamType::Null => {
            if raw == "null" {
                Ok(Value::Null)
            } else {
                Err(CliArgsError::InvalidValue {
                    name: name.to_string(),
                    expected: "null",
                    value: raw.to_string(),
                })
            }
        }
        ParamType::Array | ParamType::Object => {
            serde_json::from_str(raw).map_err(|_| CliArgsError::InvalidValue {
                name: name.to_string(),
                expected: if param_type == ParamType::Array {
                    "JSON array"
                } else {
                    "JSON object"
                },
                value: raw.to_string(),
            })
        }
    }
}
