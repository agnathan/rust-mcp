# B.1 OAuth

```bash
cargo test -p apb-1-oauth
cargo bench -p apb-1-oauth --bench oauth
cargo run -p apb-1-oauth --example demo
```

`MemoryOAuth` issues a single-use code with S256 PKCE. `OAuthProxy` keeps the client redirect and does not call upstream. `oidc_from_json` reads a discovery document.
