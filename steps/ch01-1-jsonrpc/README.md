# Section 1.1: Speaking JSON-RPC

Hand-rolled JSON-RPC 2.0 requests, notifications, responses, and errors, plus MCP `initialize` and `ping`. No `rmcp` yet.

```bash
cargo test -p ch01-1-fastmcp-core
cargo bench -p ch01-1-fastmcp-core --bench jsonrpc
cargo run -p ch01-1-fastmcp-core --example demo
```

The book section is [Chapter 1: Speaking JSON-RPC](../../book/chapter-01-speaking-json-rpc.md).
