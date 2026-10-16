# Social posts: Chapter 1, "Speaking JSON-RPC"

## X

Chapter 1 of Learning Rust by Building FastMCP is up: Speaking JSON-RPC.

Before a server, the wire. One JSON-RPC request, one notification, one response. `initialize` and `ping`, hand-rolled with serde. No socket yet.

FastMCP (Prefect) does not parse this itself. `server/low_level.py` treats `initialize` as a request with an id, and `notifications/initialized` as a notification. `exceptions.py` maps failures onto JSON-RPC codes. This chapter builds that envelope in Rust so the next chapter can pick up `rmcp` without the bytes being a blur.

`ping` returns `{}`. An unknown method is a normal error response, code -32601, not a panic.

[personal note]

The Rust in this chapter is only what it needs: cargo, structs and enums, serde, `Result` and `?`, and `#[test]`.

Next: a stdio server.

#rustlang #MCP

## LinkedIn

Chapter 1 of Learning Rust by Building FastMCP is out: Speaking JSON-RPC.

The Model Context Protocol on the wire is JSON-RPC 2.0. FastMCP, the Python framework this book learns from, leaves that parsing to the MCP Python SDK. Its server still has to know the difference between a request and a notification: `initialize` carries an id (`server/low_level.py`), `notifications/initialized` does not, and tool errors become JSON-RPC codes (`exceptions.py`, `to_mcp_error`).

So the first Rust step is the envelope, with tests:

- a request, a notification, and a response
- `initialize` answered with protocol version, capabilities, and server info
- `ping` answered with an empty object
- benches for parsing and serializing 1 and 1,000 messages, and for dispatching `initialize`

The chapter is written for a reader who is new to Rust. Each language feature shows up in a short sidebar when the code needs it. There is no network in this step.

[personal note]

Next up: a stdio server and `tools/list`.

#Rust #MCP #LearningInPublic #SoftwareEngineering
