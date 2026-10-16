# 16.1 Middleware

```bash
cargo test -p ch16-1-fastmcp-core
cargo bench -p ch16-1-fastmcp-core --bench middleware
cargo run -p ch16-1-fastmcp-core --example demo
```

`with_hook` adds a layer. `invoke` runs `before` outer-first and `after` in reverse. `PingMiddleware` refuses a zero interval and starts one task per connection id.
