//! A name-keyed registry of tool closures.
//!
//! Each closure has its own parameter type and return type. The map stores
//! `Arc<dyn Fn>`, plus the JSON Schema `schemars` built for those types.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::schema::json_schema_of;

/// What `#[tool]` emits. The struct names the tool. The function stays callable.
pub trait ToolDecl {
    /// The argument struct the macro built from the function's parameters.
    type Params: DeserializeOwned + JsonSchema + Send + 'static;
    /// The value the function returns. A `Result` return uses the `Ok` type.
    type Output: Serialize + JsonSchema;
    /// The tool name on the wire.
    fn name() -> &'static str;
    /// The description `tools/list` shows.
    fn description() -> &'static str;
    /// Run the function. The registry calls this after `Params` deserializes.
    fn invoke(args: Self::Params) -> Result<Self::Output, String>;
}

/// Why [`crate::FastMcp::dispatch_tool`] did not return a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// No tool is registered under that name.
    Unknown,
    /// The JSON arguments do not deserialize into the tool's parameter type.
    /// The closure did not run.
    InvalidParams(String),
    /// The closure ran and returned `Err`, or its `Ok` value did not serialize.
    Failed(String),
}

/// One registered tool. `P` and `R` are erased inside `call`.
#[derive(Clone)]
pub struct RegisteredTool {
    description: String,
    schema: Arc<Map<String, Value>>,
    output_schema: Arc<Map<String, Value>>,
    call: Arc<dyn Fn(Value) -> Result<Value, ToolError> + Send + Sync>,
}

impl RegisteredTool {
    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn schema(&self) -> Arc<Map<String, Value>> {
        Arc::clone(&self.schema)
    }

    pub fn output_schema(&self) -> Arc<Map<String, Value>> {
        Arc::clone(&self.output_schema)
    }

    pub fn call(&self, arguments: Value) -> Result<Value, ToolError> {
        (self.call)(arguments)
    }
}

/// JSON Schema for `status`, which takes no fields.
pub fn empty_object_schema() -> Arc<Map<String, Value>> {
    let mut schema = Map::new();
    schema.insert("type".into(), Value::String("object".into()));
    schema.insert("properties".into(), Value::Object(Map::new()));
    Arc::new(schema)
}

/// Wrap `call` so the registry can store it next to tools with other types.
pub fn register<P, R, F>(description: impl Into<String>, call: F) -> RegisteredTool
where
    P: DeserializeOwned + JsonSchema + Send + 'static,
    R: Serialize + JsonSchema,
    F: Fn(P) -> Result<R, String> + Send + Sync + 'static,
{
    RegisteredTool {
        description: description.into(),
        schema: json_schema_of::<P>(),
        output_schema: json_schema_of::<R>(),
        call: Arc::new(move |value| {
            let params = serde_json::from_value::<P>(value)
                .map_err(|err| ToolError::InvalidParams(err.to_string()))?;
            let returned = call(params).map_err(ToolError::Failed)?;
            serde_json::to_value(returned).map_err(|err| ToolError::Failed(err.to_string()))
        }),
    }
}
