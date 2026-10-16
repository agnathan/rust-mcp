# Section 14.1: The client

`infer_transport` classifies a string. `McpClient::connect_memory` handshakes on a duplex. `ClientGroup` prefixes tool names.

```bash
cargo test -p ch14-1-fastmcp-core
cargo bench -p ch14-1-fastmcp-core --bench client
cargo run -p ch14-1-fastmcp-core --example demo
```

The book section is [Chapter 14: The client and client groups](../../book/chapter-14-client-and-groups.md).
