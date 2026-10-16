# Social posts: Chapter 24, "Settings and CLI"

## X

Chapter 24 of Learning Rust by Building FastMCP is up: settings and a command parser.

Defaults, a JSON file, then `FASTMCP_HOST`. `inspect` prints `status` and does not open a socket. `dev` turns reload on. `install` returns a JSON snippet.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 24 of Learning Rust by Building FastMCP is out: settings and the CLI.

FastMCP reads `FASTMCP_` settings, and the CLI has `run`, `inspect`, a `dev` group, and an install snippet. The Rust chapter merges defaults, a JSON file, and the environment with figment, then parses `run`, `dev`, `inspect`, and `install` with clap. Inspect lists tool names and does not bind. Dev is a reload flag. The file watcher and the listening socket stay deferred.

[personal note]

Written for a reader who is new to Rust. `OnceLock` is how the first successful load sticks for the rest of the process.

#Rust #MCP #LearningInPublic #SoftwareEngineering
