# A.1 Apps

```bash
cargo test -p apa-1-apps
cargo bench -p apa-1-apps --bench apps
cargo run -p apa-1-apps --example demo
```

`McpApp` lists model tools and calls a backend tool by `<hash>_<name>`. The hash is SHA-256 of the app name, a NUL, and the tool name, first 12 hex characters.
