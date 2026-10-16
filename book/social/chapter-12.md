# Social posts: Chapter 12, "Streamable HTTP, sessions, both protocol eras"

## X

Chapter 12 of Learning Rust by Building FastMCP is up: one port, two eras.

`initialize` comes back with `Mcp-Session-Id`. The next `tools/list` sends that id. `server/discover` for `2026-07-28` answers in JSON, with no session. Loopback, three timings: a round trip, one initialize, one discover.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 12 of Learning Rust by Building FastMCP is out: Streamable HTTP, sessions, and both protocol eras.

Stdio was one pipe. This chapter binds `127.0.0.1` and serves `/mcp` with axum and rmcp. A handshake client keeps a session. A `2026-07-28` discover stands on its own request. The older SSE app and the event-store replay stay for later.

[personal note]

Written for a reader who is new to Rust. The session id is a response header, and the next call has to carry it.

#Rust #MCP #LearningInPublic #SoftwareEngineering
