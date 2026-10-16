//! Chapter 22's token check, plus production seams around a call.
//!
//! `Option` is a field the client may leave out. `schemars` leaves that
//! field out of `required`.

pub mod app_error;
pub mod auth;
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
pub mod openapi;
pub mod pagination;
pub mod production;
pub mod prompts;
pub mod provider;
pub mod resources;
pub mod schema;
pub mod search;
pub mod server;
pub mod tasks;
pub mod testing;
pub mod tools;
pub mod transform;

pub use app_error::{to_mcp_error, AppError};
pub use auth::{jwks_kid, sign, verify, ResourceMetadata, AUTH_ALGORITHM};
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
pub use openapi::{
    call_operation, tools_from_spec, widgets_spec, ApiTools, HttpCaller, MapCaller, ReqwestCaller,
};
pub use pagination::{decode_cursor, encode_cursor, paginate_sequence};
pub use production::{
    allow_call, fingerprint, highest_versions, traced_call, FileStore, MemoryStore, Store,
    Versioned, FINGERPRINT_ALGO,
};
pub use provider::{
    discover_tools, AggregateProvider, LocalProvider, MountedProvider, Provider, ProxyProvider,
};
pub use resources::match_template;
pub use schema::json_schema_of;
pub use search::{execute, tokenize, turn_calls, Bm25, SEARCH_TOOL_NAME};
pub use server::FastMcp;
pub use tasks::{ExtensionRegistry, TaskBoard, TASKS_EXTENSION_ID};
pub use testing::{override_log_level, Harness, LevelGuard};
pub use tools::{ToolDecl, ToolError};
pub use transform::{namespace_name, namespace_uri, rename_argument, Catalog};
