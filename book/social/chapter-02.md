# Social posts: Chapter 2, "Negotiating the protocol"

## X

Chapter 2 of Learning Rust by Building FastMCP is up: Negotiating the protocol.

Chapter 1 pinned `initialize` to `2025-06-18`. This chapter keeps that handshake and adds the dates around it, including `2026-07-28`, which has no `initialize` at all. `rmcp` 3.5.1 already knows that revision. The new code is the choice: echo a handshake version we support, otherwise answer `2025-11-25`, and let `server/discover` still speak `2026-07-28`.

The server then runs on stdio through `rmcp`. `tools/list` returns one tool, `status`, whose description says which era the connection is in.

[personal note]

The Rust in this chapter is only what it needs: `async` and `#[tokio::main]`, the `ServerHandler` trait, `match` on an enum, and a `const` slice.

#rustlang #MCP

## LinkedIn

Chapter 2 of Learning Rust by Building FastMCP is out: Negotiating the protocol.

FastMCP 4 serves older handshake clients and `2026-07-28` from one server. `server/low_level.py` sends both `initialize` and `server/discover` through the same middleware, and `server/mixins/transport.py` is the stdio entry (`run`, `run_stdio_async`). `utilities/versions.py` is a different kind of version: component strings such as `"1.0"`, not protocol dates.

`rmcp` 3.5.1 already defines `2026-07-28` and the tasks methods. This chapter does not reimplement that revision and does not turn tasks on. It chooses a version from a fixed list and branches on handshake versus modern, then serves stdio.

The tests use an in-memory duplex pipe, the same async read/write transport as stdin and stdout. `initialize` for `2026-07-28` comes back as `2025-11-25`. A discover client still sees the modern era on `tools/list`.

[personal note]

Written for a reader who is new to Rust. Async, traits, `match`, and `const` show up here because the server needs them, not before.

#Rust #MCP #LearningInPublic #SoftwareEngineering
