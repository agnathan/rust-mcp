//! Chapter 9's server, plus dependencies the schema does not list.
//!
//! `Option` is a field the client may leave out. `schemars` leaves that
//! field out of `required`.

pub mod app_error;
pub mod context;
pub mod error;
pub mod inject;
pub mod jsonrpc;
pub mod mcp;
pub mod negotiate;
pub mod pagination;
pub mod prompts;
pub mod resources;
pub mod schema;
pub mod server;
pub mod tools;

pub use app_error::{to_mcp_error, AppError};
pub use context::{Context, SharedNotes};
pub use error::{ErrorCode, ProtocolError};
pub use inject::{Dep, Extract, FromContext, InjectCtx};
pub use jsonrpc::{
    parse_message, ErrorObject, JsonRpcVersion, Message, Notification, Request, RequestId, Response,
};
pub use mcp::{
    dispatch, reply, Implementation, InitializeParams, InitializeResult, PROTOCOL_VERSION,
};
pub use negotiate::{choose_version, era_note, era_of, ProtocolEra, SUPPORTED_VERSIONS};
pub use pagination::{decode_cursor, encode_cursor, paginate_sequence};
pub use resources::match_template;
pub use schema::json_schema_of;
pub use server::FastMcp;
pub use tools::{ToolDecl, ToolError};
