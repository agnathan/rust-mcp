# Social posts: Chapter 5, "A #[tool] macro"

## X

Chapter 5 of Learning Rust by Building FastMCP is up: a `#[tool]` macro.

FastMCP's `@mcp.tool` records a function at import time. The Rust version is a procedural macro. It runs in the compiler, leaves `add` callable, and emits `AddTool` so `tool_decl` can register it.

The schema is still built when you register. The bench shows that: about 3 µs to register, and the thousand direct calls are a different, smaller number. The macro itself is not in the timer. It already ran.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 5 of Learning Rust by Building FastMCP is out: the decorator, as a procedural macro.

FastMCP attaches metadata to the function in `decorators.py` and registers it from `LocalProvider.tool`. The Rust chapter reads the same idea with `syn` and `quote`. `#[tool]` on `fn add` keeps the function, builds an argument struct from the parameters, and implements `ToolDecl`. The doc comment becomes the description.

`rmcp` has its own `#[tool]` that fills a `ToolRouter`. This chapter's macro feeds `FastMcp::tool` instead, because that is the registry the book is growing.

The return type has to be public. It shows up in the trait, and Rust treats that as part of the module's public interface.

[personal note]

Written for a reader who is new to Rust. Procedural macros show up here because the decorator needs them.

#Rust #MCP #LearningInPublic #SoftwareEngineering
