# Social posts: Chapter 3, "The FastMcp builder and tools"

## X

Chapter 3 of Learning Rust by Building FastMCP is up: the builder and tools.

`FastMcp::new("fastmcp-rs").tool("add", "...", |args: Add| ...)`. The closure goes in a `HashMap`. `tools/call` deserializes the JSON into `Add` and runs it. `2 + 3` comes back as `5`.

FastMCP stores callables in `LocalProvider._components`, a dict keyed by component. `FastMCP.tool` delegates to `add_tool`, and `FunctionTool.from_function` holds the function. Rust closures each have their own type, so the map stores `Arc<dyn Fn>`.

An unknown name is `invalid_params`, not a panic. A closure that returns `Err` is a tool result the client can read.

[personal note]

The Rust in this chapter is only what it needs: closures, generics, `Arc<dyn Fn>`, `HashMap`, and a builder that returns `Self`.

#rustlang #MCP

## LinkedIn

Chapter 3 of Learning Rust by Building FastMCP is out: the `FastMcp` builder and tools.

FastMCP's `FastMCP.tool` (`server/server.py`) hands the callable to the local provider. That provider keeps a `dict` of components (`local_provider.py`) and wraps a function with `FunctionTool.from_function` (`tools/function_tool.py`).

The Rust step is the same shape with the types made visible:

- `FastMcp::new("fastmcp-rs").tool(...)` chains because `tool` takes the server by value and returns it
- the argument struct is deserialized before the closure runs
- unknown names and bad JSON are protocol errors
- the closure's own `Err` is `is_error: true` on a normal tool result
- lookup of one tool among 1,000, and a dispatch of all 1,000, are the benches

`status` from chapter 2 stays on `tools/list`, beside the tools you register. JSON Schema for the arguments is chapter 4.

[personal note]

Written for a reader who is new to Rust. Closures, generics, trait objects, and `HashMap` show up here because the registry needs them.

#Rust #MCP #LearningInPublic #SoftwareEngineering
