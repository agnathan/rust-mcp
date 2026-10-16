# Social posts: Chapter 14, "The client and client groups"

## X

Chapter 14 of Learning Rust by Building FastMCP is up: the other end of the pipe.

A string becomes stdio, HTTP, SSE, memory, or config before anything opens. The demo connects in memory, pings, and lists two servers as `gadgets_status` and `widgets_status`. A duplicate public name is a collision.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 14 of Learning Rust by Building FastMCP is out: the client and client groups.

FastMCP infers a transport from a URL, a script path, a server object, or an mcpServers config. The Rust chapter classifies the string first. The connection it actually opens is an in-memory duplex, so the bench never spawns a process. A group lists every server on a JoinSet and prefixes tool names. Ping treats an empty result as success. A discover session has no initialize result.

[personal note]

Written for a reader who is new to Rust. JoinSet is how you wait on a pile of tasks that finish in their own order.

#Rust #MCP #LearningInPublic #SoftwareEngineering
