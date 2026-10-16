# Social posts: Chapter 23, "Production"

## X

Chapter 23 of Learning Rust by Building FastMCP is up: the pieces around a tool call.

A check can refuse the call. Version `10` beats version `2`. Memory and a JSON file share a store trait. A tracing span is slower than no span, and a SHA-256 fingerprint names the tool plus its schema.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 23 of Learning Rust by Building FastMCP is out: production.

FastMCP runs authorization checks, filters component versions with PEP 440, and opens an OpenTelemetry span at the request seam. The Rust chapter is a boolean check, a numeric version compare that keeps the highest copy of each name, a `Store` trait with memory and file backends, a `tracing` span, and a SHA-256 fingerprint. Redis and an OTLP exporter stay deferred. Component versions are not the protocol dates from chapter 2.

[personal note]

Written for a reader who is new to Rust. A trait is how two stores share `get` and `set`.

#Rust #MCP #LearningInPublic #SoftwareEngineering
