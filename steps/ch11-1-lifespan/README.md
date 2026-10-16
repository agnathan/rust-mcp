# Section 11.1: Lifespan and state

`enter` opens a resource and pushes a close hook. `Drop` runs the hooks in reverse. The banner is an `Arc<RwLock<String>>`.

```bash
cargo test -p ch11-1-fastmcp-core
cargo bench -p ch11-1-fastmcp-core --bench lifespan
cargo run -p ch11-1-fastmcp-core --example demo
```

The book section is [Chapter 11: Lifespan and state](../../book/chapter-11-lifespan-and-state.md).
