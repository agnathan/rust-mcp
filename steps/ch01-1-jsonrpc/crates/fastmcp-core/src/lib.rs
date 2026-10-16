//! JSON-RPC 2.0 messages and the two MCP methods this chapter speaks:
//! `initialize` and `ping`. Hand-rolled with serde. No `rmcp` yet.

pub mod error;
pub mod jsonrpc;
pub mod mcp;

pub use error::{ErrorCode, ProtocolError};
pub use jsonrpc::{
    parse_message, ErrorObject, JsonRpcVersion, Message, Notification, Request, RequestId, Response,
};
pub use mcp::{
    dispatch, reply, Implementation, InitializeParams, InitializeResult, PROTOCOL_VERSION,
};
