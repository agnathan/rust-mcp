# 25.1 Shipping

```bash
cargo test -p ch25-1-fastmcp-core
cargo bench -p ch25-1-fastmcp-core --bench shipping
cargo run -p ch25-1-fastmcp-core --example demo
python3 python/call_add.py
```

`publish_plan` returns `cargo publish --dry-run` and refuses `0.0.0`. `release_archives` names three hosts. `call_add` returns 5. The Python script times FastMCP's `call_tool` with middleware off.
