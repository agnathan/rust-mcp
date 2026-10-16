//! Chapter 2's negotiated `rmcp` server, plus a `HashMap` of tools.
//!
//! `FastMcp::tool` is the builder. The closure's parameter type is erased
//! into `Arc<dyn Fn>` so one map can hold every tool.

pub mod error;
pub mod jsonrpc;
pub mod mcp;
pub mod negotiate;
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
pub use server::FastMcp;
pub use tools::ToolError;
