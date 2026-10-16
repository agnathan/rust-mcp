//! Chapter 5's tool registry, plus static resources and `{param}` URI templates.
//!
//! `Option` is a field the client may leave out. `schemars` leaves that
//! field out of `required`.

pub mod error;
pub mod jsonrpc;
pub mod mcp;
pub mod negotiate;
pub mod resources;
pub mod schema;
pub mod server;
pub mod tools;

pub use error::{ErrorCode, ProtocolError};
pub use jsonrpc::{
    parse_message, ErrorObject, JsonRpcVersion, Message, Notification, Request, RequestId, Response,
};
pub use mcp::{
    dispatch, reply, Implementation, InitializeParams, InitializeResult, PROTOCOL_VERSION,
};
pub use negotiate::{choose_version, era_note, era_of, ProtocolEra, SUPPORTED_VERSIONS};
pub use resources::match_template;
pub use schema::json_schema_of;
pub use server::FastMcp;
pub use tools::{ToolDecl, ToolError};
