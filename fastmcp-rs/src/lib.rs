//! The crate you grow. Chapter 1's finished code is
//! `steps/ch01-1-jsonrpc`. This function stays `todo!()` until you replace it.
//! CI tests the step snapshot, not this crate.

#![allow(dead_code)]

/// Return the JSON object for a successful `ping` response with id `1`.
pub fn ping_response_json() -> String {
    todo!("chapter 1: return a JSON-RPC success response for ping")
}

/// The protocol version `initialize` echoes when the client asks for `2025-06-18`.
pub fn negotiated_version_label() -> &'static str {
    todo!("chapter 2: echo the handshake version the client requested")
}

/// The name of the tool chapter 3 registers on the builder.
pub fn registered_tool_name() -> &'static str {
    todo!("chapter 3: return the name passed to FastMcp::tool")
}
