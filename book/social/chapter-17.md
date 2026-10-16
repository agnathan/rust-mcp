# Social posts: Chapter 17, "Providers and composition"

## X

Chapter 17 of Learning Rust by Building FastMCP is up: tools that come from more than one place.

A local list, an aggregate, a mount that turns `status` into `api_status`, and a proxy that forwards the ping and keeps the upstream instructions.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 17 of Learning Rust by Building FastMCP is out: providers and composition.

FastMCP asks a provider for tools. An aggregate joins several. A namespace prefix keeps two `status` tools apart. A proxy forwards ping and copies instructions when it has none of its own. The Rust chapter is a `Provider` trait, a mount, an in-process proxy, and a directory scan of `*.tool` files.

[personal note]

Written for a reader who is new to Rust. `Option` is how a provider says “not mine,” so the next one can try.

#Rust #MCP #LearningInPublic #SoftwareEngineering
