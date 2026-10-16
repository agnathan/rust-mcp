# Section 2.1: Negotiating the protocol

Chapter 1's JSON-RPC types, plus a const list of protocol versions, a handshake-versus-modern branch, and an `rmcp` server on stdio.

```bash
cargo test -p ch02-1-fastmcp-core
cargo bench -p ch02-1-fastmcp-core --bench negotiate
cargo run -p ch02-1-fastmcp-core --example demo
```

The book section is [Chapter 2: Negotiating the protocol](../../book/chapter-02-negotiating-the-protocol.md).
