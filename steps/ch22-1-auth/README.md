# 22.1 Authentication

```bash
cargo test -p ch22-1-fastmcp-core
cargo bench -p ch22-1-fastmcp-core --bench auth
cargo run -p ch22-1-fastmcp-core --example demo
```

`verify` checks an HS256 token and the audience. Errors say `token rejected` and leave the secret out. `jwks_kid` reads a key id from a document. `ResourceMetadata` names the resource and its authorization servers.
