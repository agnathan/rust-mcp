//! Chapter 17's providers, plus transforms over a tool catalog.
//!
//! `Option` is a field the client may leave out. `schemars` leaves that
//! field out of `required`.

pub mod app_error;
pub mod client;
pub mod context;
pub mod elicit;
pub mod error;
pub mod http;
pub mod inject;
pub mod jsonrpc;
pub mod lifespan;
pub mod mcp;
pub mod middleware;
pub mod negotiate;
pub mod pagination;
pub mod prompts;
pub mod provider;
pub mod resources;
pub mod schema;
pub mod server;
pub mod testing;
pub mod tools;
pub mod transform;

pub use app_error::{to_mcp_error, AppError};
pub use client::{infer_transport, ClientError, ClientGroup, McpClient, TransportKind};
pub use context::{Context, SharedNotes};
pub use elicit::{validate_elicitation_schema, Desk, ElicitError, Phase};
pub use error::{ErrorCode, ProtocolError};
pub use http::{delete_mcp, post_mcp, HttpReply, HttpServer};
pub use inject::{Dep, Extract, FromContext, InjectCtx};
pub use jsonrpc::{
    parse_message, ErrorObject, JsonRpcVersion, Message, Notification, Request, RequestId, Response,
};
pub use lifespan::{AppState, Lifespan};
pub use mcp::{
    dispatch, reply, Implementation, InitializeParams, InitializeResult, PROTOCOL_VERSION,
};
pub use middleware::{
    CallLog, PingMiddleware, RateLimit, ResponseCache, ResponseLimit, Timing, ToolHook,
};
pub use negotiate::{choose_version, era_note, era_of, ProtocolEra, SUPPORTED_VERSIONS};
pub use pagination::{decode_cursor, encode_cursor, paginate_sequence};
pub use provider::{
    discover_tools, AggregateProvider, LocalProvider, MountedProvider, Provider, ProxyProvider,
};
pub use resources::match_template;
pub use schema::json_schema_of;
pub use server::FastMcp;
pub use testing::{override_log_level, Harness, LevelGuard};
pub use tools::{ToolDecl, ToolError};
pub use transform::{namespace_name, namespace_uri, rename_argument, Catalog};
