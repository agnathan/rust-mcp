# 17.1 Providers and composition

```bash
cargo test -p ch17-1-fastmcp-core
cargo bench -p ch17-1-fastmcp-core --bench providers
cargo run -p ch17-1-fastmcp-core --example demo
```

A local provider holds tools. An aggregate asks several providers in order. A mount prefixes `{namespace}_`. A proxy forwards the call and the ping, and copies instructions when it has none. `discover_tools` reads `*.tool` files.
