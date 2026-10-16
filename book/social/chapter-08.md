# Social posts: Chapter 8, "Errors done right"

## X

Chapter 8 of Learning Rust by Building FastMCP is up: tool errors versus protocol errors.

A tool that returns Err is still a result the client can read (`is_error: true`). A missing component is `invalid_params` (-32602). An unexpected failure is `internal_error` (-32603). FastMCP's `to_mcp_error` is that map. The Rust version is a `thiserror` enum plus `anyhow` for the bucket that is not one of those cases.

The bench runs the happy path and the missing-name path a thousand times each. The miss is cheaper. It never enters the function.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 8 of Learning Rust by Building FastMCP is out: classifying errors.

FastMCP's `to_mcp_error` leaves an `MCPError` alone, maps not-found, disabled, and validation to `INVALID_PARAMS`, and sends everything else to `INTERNAL_ERROR`. The chapter writes that map with `thiserror` and `From`. `anyhow` is the "everything else" bucket.

A tool's own `Err` is a different outcome. The call succeeded as JSON-RPC. The tool reported a failure. Mixing those two up is how a client loses the message.

[personal note]

Written for a reader who is new to Rust. `thiserror` and `anyhow` show up here because the wire codes need a type.

#Rust #MCP #LearningInPublic #SoftwareEngineering
