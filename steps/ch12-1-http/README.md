# Section 12.1: Streamable HTTP

`serve_http` binds `127.0.0.1:0` and serves `/mcp`. A handshake `initialize` returns `Mcp-Session-Id`. `server/discover` for `2026-07-28` does not.

```bash
cargo test -p ch12-1-fastmcp-core
cargo bench -p ch12-1-fastmcp-core --bench http
cargo run -p ch12-1-fastmcp-core --example demo
```

The book section is [Chapter 12: Streamable HTTP, sessions, both protocol eras](../../book/chapter-12-streamable-http.md).
