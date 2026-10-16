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

/// The `Option` field chapter 4 leaves out of the schema's `required` array.
pub fn optional_argument_name() -> &'static str {
    todo!("chapter 4: return the name of the Option field")
}

/// The struct `#[tool]` emits for `fn add`.
pub fn macro_tool_struct_name() -> &'static str {
    todo!("chapter 5: return the ToolDecl struct name for fn add")
}

/// The exact URI of the static readme resource in chapter 6.
pub fn static_resource_uri() -> &'static str {
    todo!("chapter 6: return the URI passed to FastMcp::resource for the readme")
}

/// The prompt name chapter 7 renders as `Hello, Ada`.
pub fn sample_prompt_name() -> &'static str {
    todo!("chapter 7: return the name passed to FastMcp::prompt for the greeting")
}

/// The JSON-RPC code `to_mcp_error` uses for a missing component.
pub fn not_found_code() -> i32 {
    todo!("chapter 8: return the invalid_params code")
}

/// The string `Context::sample` returns in chapter 9.
pub fn canned_sample() -> &'static str {
    todo!("chapter 9: return the fixed sample string")
}

/// The only field chapter 10 puts in the tool schema.
pub fn injected_query_field() -> &'static str {
    todo!("chapter 10: return the argument field the client sends")
}

/// Close order for chapter 11: cache, then db.
pub fn shutdown_order() -> &'static str {
    todo!("chapter 11: return the close order, cache then db")
}

/// The path chapter 12 serves.
pub fn streamable_http_path() -> &'static str {
    todo!("chapter 12: return the path the listener serves")
}

/// The phase after a person accepts.
pub fn elicit_phase() -> &'static str {
    todo!("chapter 13: return the phase after submit")
}

/// The public name of the status tool on the widgets server.
pub fn grouped_tool_name() -> &'static str {
    todo!("chapter 14: return server name, underscore, tool name")
}

/// The first name in a harness snapshot of an empty server.
pub fn snapshot_tool_name() -> &'static str {
    todo!("chapter 15: return the built-in tool name")
}

/// The default ping interval, in milliseconds.
pub fn ping_interval_ms() -> &'static str {
    todo!("chapter 16: return the default ping interval")
}
