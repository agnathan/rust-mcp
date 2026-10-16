# 20.1 Background tasks and extensions

```bash
cargo test -p ch20-1-fastmcp-core
cargo bench -p ch20-1-fastmcp-core --bench tasks
cargo run -p ch20-1-fastmcp-core --example demo
```

`ExtensionRegistry` records an identifier. `TaskBoard` refuses to start until `io.modelcontextprotocol/tasks` is present. `submit` returns an id. `poll` and `wait` read the result.
