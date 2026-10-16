# 15.1 Testing utilities

```bash
cargo test -p ch15-1-fastmcp-core
cargo bench -p ch15-1-fastmcp-core --bench harness
cargo run -p ch15-1-fastmcp-core --example demo
```

`Harness` lists and calls tools in this process. `McpClient::connect_memory` lists them through the duplex that stdio tests use. `override_log_level` puts the level back when the guard drops.
