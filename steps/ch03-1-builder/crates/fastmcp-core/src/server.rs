//! A small MCP server on top of `rmcp`, speaking stdio.
//!
//! Chapter 2 chose a protocol version. This server also keeps a `HashMap` of
//! tools and calls the one the client names.

use std::borrow::Cow;
use std::collections::HashMap;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData,
    Implementation, InitializeRequestParams, InitializeResult, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::negotiate::{choose_version, era_note, era_of, SUPPORTED_VERSIONS};
use crate::tools::{empty_object_schema, register, RegisteredTool, ToolError};

/// The server this chapter runs.
///
/// `tool` takes `self` and returns `Self`, so registrations chain.
#[derive(Clone)]
pub struct FastMcp {
    name: String,
    version: String,
    tools: HashMap<String, RegisteredTool>,
}

impl FastMcp {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".into(),
            tools: HashMap::new(),
        }
    }

    /// Register `call` under `name`.
    ///
    /// `P` is the argument struct. A second call with the same name replaces
    /// the first. The closure runs only after `P` deserializes.
    pub fn tool<P, F>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        call: F,
    ) -> Self
    where
        P: DeserializeOwned + Send + 'static,
        F: Fn(P) -> Result<Value, String> + Send + Sync + 'static,
    {
        self.tools
            .insert(name.into(), register::<P, F>(description, call));
        self
    }

    /// `true` when `name` was registered with [`Self::tool`].
    ///
    /// `status` is not in the map. [`Self::call_tool`] answers it on its own.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Look up `name` and run it.
    ///
    /// # Errors
    ///
    /// [`ToolError::Unknown`] if the name is missing, [`ToolError::InvalidParams`]
    /// if `arguments` is not a `P`, [`ToolError::Failed`] if the closure returns `Err`.
    pub fn dispatch_tool(&self, name: &str, arguments: Value) -> Result<Value, ToolError> {
        let tool = self.tools.get(name).ok_or(ToolError::Unknown)?;
        tool.call(arguments)
    }

    /// Serve one connection on the process's stdin and stdout.
    ///
    /// # Errors
    ///
    /// Returns the transport or handshake error `rmcp` reports. A client that
    /// closes stdin ends the function.
    pub async fn run_stdio(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let running = self.serve(rmcp::transport::stdio()).await?;
        running.waiting().await?;
        Ok(())
    }

    fn info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(self.name.clone(), self.version.clone()))
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
    }

    fn listed_tools(&self, version: &ProtocolVersion) -> Vec<Tool> {
        let mut names: Vec<&String> = self.tools.keys().collect();
        names.sort();
        let mut listed = Vec::with_capacity(names.len() + 1);
        listed.push(status_tool(version));
        for name in names {
            let registered = &self.tools[name];
            listed.push(Tool::new(
                name.clone(),
                registered.description().to_string(),
                registered.schema(),
            ));
        }
        listed
    }
}

fn status_tool(version: &ProtocolVersion) -> Tool {
    Tool::new("status", era_note(era_of(version)), empty_object_schema())
}

impl ServerHandler for FastMcp {
    fn get_info(&self) -> ServerConfig {
        self.info()
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(SUPPORTED_VERSIONS)
    }

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        context.peer.set_peer_info(request.clone());
        let version = choose_version(&request.protocol_version, SUPPORTED_VERSIONS)?;
        let mut info = self.info();
        info.protocol_version = version.clone();
        info.instructions = Some(era_note(era_of(&version)).to_string());
        Ok(info)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let version = context
            .protocol_version()
            .unwrap_or(ProtocolVersion::V_2025_06_18);
        Ok(ListToolsResult::with_all_items(self.listed_tools(&version)))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if request.name == "status" {
            let version = context
                .protocol_version()
                .unwrap_or(ProtocolVersion::V_2025_06_18);
            let note = era_note(era_of(&version));
            return Ok(CallToolResult::success(vec![ContentBlock::text(note)]).into());
        }
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        match self.dispatch_tool(request.name.as_ref(), arguments) {
            Ok(value) => Ok(CallToolResult::structured(value).into()),
            Err(ToolError::Unknown) => Err(ErrorData::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            )),
            Err(ToolError::InvalidParams(message)) => Err(ErrorData::invalid_params(message, None)),
            Err(ToolError::Failed(message)) => {
                Ok(CallToolResult::error(vec![ContentBlock::text(message)]).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Add {
        left: i64,
        right: i64,
    }

    fn add_server() -> FastMcp {
        FastMcp::new("fastmcp-rs").tool("add", "add two integers", |args: Add| {
            Ok(Value::from(args.left + args.right))
        })
    }

    #[test]
    fn calls_a_registered_tool() {
        let server = add_server();
        let value = server
            .dispatch_tool("add", serde_json::json!({"left": 2, "right": 3}))
            .expect("add");
        assert_eq!(value, Value::from(5));
        assert!(server.has_tool("add"));
        assert!(!server.has_tool("missing"));
    }

    #[test]
    fn unknown_tool_is_not_a_panic() {
        let error = add_server()
            .dispatch_tool("nope", Value::Object(serde_json::Map::new()))
            .expect_err("missing");
        assert_eq!(error, ToolError::Unknown);
    }

    #[test]
    fn missing_field_does_not_run_the_tool() {
        let error = add_server()
            .dispatch_tool("add", serde_json::json!({"left": 1}))
            .expect_err("right is required");
        assert!(matches!(error, ToolError::InvalidParams(_)));
    }

    #[test]
    fn tool_err_is_a_failure_not_a_panic() {
        let server = FastMcp::new("fastmcp-rs")
            .tool("nope", "always fails", |_: Add| Err("nope".to_string()));
        let error = server
            .dispatch_tool("nope", serde_json::json!({"left": 1, "right": 2}))
            .expect_err("tool err");
        assert_eq!(error, ToolError::Failed("nope".to_string()));
    }

    #[test]
    fn a_second_registration_replaces_the_name() {
        let server = FastMcp::new("fastmcp-rs")
            .tool("add", "first", |args: Add| Ok(Value::from(args.left)))
            .tool("add", "second", |args: Add| Ok(Value::from(args.right)));
        let value = server
            .dispatch_tool("add", serde_json::json!({"left": 2, "right": 9}))
            .expect("replaced");
        assert_eq!(value, Value::from(9));
        assert_eq!(
            server.tools.get("add").expect("add").description(),
            "second"
        );
    }
}
