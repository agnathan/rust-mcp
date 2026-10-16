# Social posts: Chapter 20, "Background tasks and extensions"

## X

Chapter 20 of Learning Rust by Building FastMCP is up: a tool that finishes after the call returns.

The server registers `io.modelcontextprotocol/tasks`. `submit` returns an id. `poll` says working, then done. One thousand of those tasks finish in a few milliseconds.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 20 of Learning Rust by Building FastMCP is out: background tasks and extensions.

FastMCP keeps the tasks extension id in core and loads the protocol from the fastmcp-tasks package. A tool with task=True requires `io.modelcontextprotocol/tasks`. The Rust chapter is an extension registry and a `TaskBoard` on `tokio::spawn`. Poll and result stay in process. The wire methods, the TTL, and the poll interval stay in the deferred list.

[personal note]

Written for a reader who is new to Rust. `tokio::spawn` is how the closure keeps running after `submit` has returned an id.

#Rust #MCP #LearningInPublic #SoftwareEngineering
