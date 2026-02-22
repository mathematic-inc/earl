use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    pub provider: String,
    pub command: String,
    pub positional_args: Vec<Value>,
    pub named_args: Vec<(String, Value)>,
}

impl CallExpression {
    pub fn command_key(&self) -> String {
        format!("{}.{}", self.provider, self.command)
    }
}
