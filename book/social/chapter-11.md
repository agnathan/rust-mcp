# Social posts: Chapter 11, "Lifespan and state"

## X

Chapter 11 of Learning Rust by Building FastMCP is up: open, then close in reverse.

The database opens first. The cache opens second. `Drop` closes the cache, then the database. The banner the tools read is an `Arc<RwLock<String>>`, so a clone of the server sees the same string. Four threads reading that lock is the bench.

[personal note]

#rustlang #MCP

## LinkedIn

Chapter 11 of Learning Rust by Building FastMCP is out: lifespan and shared state.

FastMCP composes lifespans with `|`. The left side enters first, and the `async with` exits in reverse. The Rust chapter uses `Drop` for that exit, because the hooks are synchronous. `Arc::get_mut` runs them only when the last owner goes away, so a clone does not shut the pool down early.

[personal note]

Written for a reader who is new to Rust. `RwLock` shows up here because several tasks read one banner.

#Rust #MCP #LearningInPublic #SoftwareEngineering
