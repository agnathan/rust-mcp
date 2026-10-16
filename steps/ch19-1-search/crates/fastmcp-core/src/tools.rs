//! A name-keyed registry of tool closures.
//!
//! Each closure has its own parameter type and return type. The map stores
//! `Arc<dyn Fn>`, plus the JSON Schema `schemars` built for those types.

use std::sync::Arc;

use rmcp::model::ErrorData;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::app_error::{to_mcp_error, AppError};
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
#[derive(Debug, Clone, PartialEq)]
pub enum ToolError {
    /// No tool is registered under that name.
    Unknown,
    /// The JSON arguments do not deserialize into the tool's parameter type.
    /// The closure did not run.
    InvalidParams(String),
    /// The closure ran and returned `Err`, or its `Ok` value did not serialize.
    Failed(String),
    /// The failure is a JSON-RPC error, already classified by `to_mcp_error`.
    Wire(ErrorData),
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

/// Like [`register`], for a closure that returns [`AppError`].
///
/// `ToolFailed` stays a tool failure. Every other variant becomes a wire error.
pub fn register_app<P, R, F>(description: impl Into<String>, call: F) -> RegisteredTool
where
    P: DeserializeOwned + JsonSchema + Send + 'static,
    R: Serialize + JsonSchema,
    F: Fn(P) -> Result<R, AppError> + Send + Sync + 'static,
{
    RegisteredTool {
        description: description.into(),
        schema: json_schema_of::<P>(),
        output_schema: json_schema_of::<R>(),
        call: Arc::new(move |value| {
            let params = serde_json::from_value::<P>(value)
                .map_err(|err| ToolError::InvalidParams(err.to_string()))?;
            let returned = match call(params) {
                Ok(value) => value,
                Err(AppError::ToolFailed(message)) => return Err(ToolError::Failed(message)),
                Err(error) => return Err(ToolError::Wire(to_mcp_error(error))),
            };
            serde_json::to_value(returned).map_err(|err| ToolError::Failed(err.to_string()))
        }),
    }
}

/// Like [`register`], with a [`crate::Context`] the schema does not mention.
pub fn register_ctx<P, R, F>(
    description: impl Into<String>,
    make_context: impl Fn() -> crate::Context + Send + Sync + 'static,
    call: F,
) -> RegisteredTool
where
    P: DeserializeOwned + JsonSchema + Send + 'static,
    R: Serialize + JsonSchema,
    F: Fn(crate::Context, P) -> Result<R, String> + Send + Sync + 'static,
{
    RegisteredTool {
        description: description.into(),
        schema: json_schema_of::<P>(),
        output_schema: json_schema_of::<R>(),
        call: Arc::new(move |value| {
            let params = serde_json::from_value::<P>(value)
                .map_err(|err| ToolError::InvalidParams(err.to_string()))?;
            let returned = call(make_context(), params).map_err(ToolError::Failed)?;
            serde_json::to_value(returned).map_err(|err| ToolError::Failed(err.to_string()))
        }),
    }
}

/// Like [`register_ctx`], with dependencies extracted before the call.
pub fn register_inject<P, R, E, F>(
    description: impl Into<String>,
    deps: crate::inject::InjectCtx,
    call: F,
) -> RegisteredTool
where
    P: DeserializeOwned + JsonSchema + Send + 'static,
    R: Serialize + JsonSchema,
    E: crate::inject::Extract + 'static,
    F: Fn(E, P) -> Result<R, String> + Send + Sync + 'static,
{
    RegisteredTool {
        description: description.into(),
        schema: json_schema_of::<P>(),
        output_schema: json_schema_of::<R>(),
        call: Arc::new(move |value| {
            let params = serde_json::from_value::<P>(value)
                .map_err(|err| ToolError::InvalidParams(err.to_string()))?;
            let extracted = E::extract(&deps).map_err(ToolError::Failed)?;
            let returned = call(extracted, params).map_err(ToolError::Failed)?;
            serde_json::to_value(returned).map_err(|err| ToolError::Failed(err.to_string()))
        }),
    }
}
