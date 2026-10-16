//! Chapter 1's hand-rolled JSON-RPC, plus chapter 2's protocol negotiation
//! and an `rmcp` server on stdio.
//!
//! `rmcp` 3.5.1 already defines `ProtocolVersion::V_2026_07_28`. This crate
//! does not reimplement that revision. It chooses a version from a const
//! slice and branches on [`ProtocolEra`].

pub mod error;
pub mod jsonrpc;
pub mod mcp;
pub mod negotiate;
pub mod server;

pub use error::{ErrorCode, ProtocolError};
pub use jsonrpc::{
    parse_message, ErrorObject, JsonRpcVersion, Message, Notification, Request, RequestId, Response,
};
pub use mcp::{
    dispatch, reply, Implementation, InitializeParams, InitializeResult, PROTOCOL_VERSION,
};
pub use negotiate::{choose_version, era_note, era_of, ProtocolEra, SUPPORTED_VERSIONS};
pub use server::FastMcp;
