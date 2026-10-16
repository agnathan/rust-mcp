# Social posts: Chapter 19, "Tool search and code mode"

## X

Chapter 19 of Learning Rust by Building FastMCP is up: find a tool, then call it from a script.

BM25 ranks 10,000 tool descriptions. A rhai script calls `call_tool` twice and adds the numbers. A third call past the limit is an error.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 19 of Learning Rust by Building FastMCP is out: tool search and code mode.

FastMCP hides a large catalog behind a BM25 search tool named `search_tools`, then lets one sandboxed Python block call the tools it found. The call limit defaults to 50. The Rust chapter is a hand-rolled BM25 index and a rhai script with the same cap. Monty stays in the FastMCP box as the Python sandbox this chapter does not ship.

[personal note]

Written for a reader who is new to Rust. A `HashMap` of term frequencies is enough to rank a catalog.

#Rust #MCP #LearningInPublic #SoftwareEngineering
