# 19.1 Tool search and code mode

```bash
cargo test -p ch19-1-fastmcp-core
cargo bench -p ch19-1-fastmcp-core --bench search
cargo run -p ch19-1-fastmcp-core --example demo
```

BM25 ranks tool text. A rhai script calls `call_tool` and stops when it passes the limit. FastMCP uses a Python sandbox for the same job.
