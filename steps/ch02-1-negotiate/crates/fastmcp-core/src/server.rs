//! A small MCP server on top of `rmcp`, speaking stdio.
//!
//! Chapter 1 parsed the envelope by hand. This server lets `rmcp` read and
//! write those bytes. The version list and the era note are still ours.

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::model::{
    ErrorData, Implementation, InitializeRequestParams, InitializeResult, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value};

use crate::negotiate::{choose_version, era_note, era_of, SUPPORTED_VERSIONS};

/// The server chapter 2 runs. Tools arrive in chapter 3.
#[derive(Debug, Clone)]
pub struct FastMcp {
    name: String,
    version: String,
}

impl FastMcp {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".into(),
        }
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
}

fn empty_object_schema() -> Arc<Map<String, Value>> {
    let mut schema = Map::new();
    schema.insert("type".into(), Value::String("object".into()));
    schema.insert("properties".into(), Value::Object(Map::new()));
    Arc::new(schema)
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
        Ok(ListToolsResult::with_all_items(vec![status_tool(&version)]))
    }
}
