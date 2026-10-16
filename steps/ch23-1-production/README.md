# 23.1 Production

```bash
cargo test -p ch23-1-fastmcp-core
cargo bench -p ch23-1-fastmcp-core --bench production
cargo run -p ch23-1-fastmcp-core --example demo
```

`allow_call` can refuse a name. `highest_versions` keeps the newest component version of each name. `MemoryStore` and `FileStore` implement `Store`. `traced_call` opens a span. `fingerprint` is SHA-256 of the name and the schema.
