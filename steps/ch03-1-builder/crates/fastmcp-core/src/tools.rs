//! A name-keyed registry of tool closures.
//!
//! Each closure has its own parameter type. The map cannot store those types
//! side by side, so registration wraps the closure in `Arc<dyn Fn>`.

use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

/// Why [`crate::FastMcp::dispatch_tool`] did not return a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// No tool is registered under that name.
    Unknown,
    /// The JSON arguments do not deserialize into the tool's parameter type.
    /// The closure did not run.
    InvalidParams(String),
    /// The closure ran and returned `Err`.
    Failed(String),
}

/// One registered tool. The parameter type `P` is erased inside `call`.
#[derive(Clone)]
pub struct RegisteredTool {
    description: String,
    schema: Arc<Map<String, Value>>,
    call: Arc<dyn Fn(Value) -> Result<Value, ToolError> + Send + Sync>,
}

impl RegisteredTool {
    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn schema(&self) -> Arc<Map<String, Value>> {
        Arc::clone(&self.schema)
    }

    pub fn call(&self, arguments: Value) -> Result<Value, ToolError> {
        (self.call)(arguments)
    }
}

/// JSON Schema for a tool whose arguments are not described yet.
///
/// Chapter 4 replaces this with a schema derived from the parameter type.
pub fn empty_object_schema() -> Arc<Map<String, Value>> {
    let mut schema = Map::new();
    schema.insert("type".into(), Value::String("object".into()));
    schema.insert("properties".into(), Value::Object(Map::new()));
    Arc::new(schema)
}

/// Wrap `call` so the registry can store it next to tools with other parameter types.
pub fn register<P, F>(description: impl Into<String>, call: F) -> RegisteredTool
where
    P: DeserializeOwned + Send + 'static,
    F: Fn(P) -> Result<Value, String> + Send + Sync + 'static,
{
    RegisteredTool {
        description: description.into(),
        schema: empty_object_schema(),
        call: Arc::new(move |value| {
            let params = serde_json::from_value::<P>(value)
                .map_err(|err| ToolError::InvalidParams(err.to_string()))?;
            call(params).map_err(ToolError::Failed)
        }),
    }
}
