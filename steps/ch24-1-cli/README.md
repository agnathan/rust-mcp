# 24.1 Settings and CLI

```bash
cargo test -p ch24-1-fastmcp-core
cargo bench -p ch24-1-fastmcp-core --bench settings
cargo run -p ch24-1-fastmcp-core --example demo
```

Settings come from defaults, an optional JSON file, and `FASTMCP_` environment variables. `inspect` lists tool names and does not bind a port. `dev` sets reload. `install` returns a JSON snippet.
