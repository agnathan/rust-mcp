# Section 8.1: Errors done right

`to_mcp_error` maps not-found, disabled, and validation to `invalid_params`. An `anyhow` error is `internal_error`. A tool's own `Err` stays a result with `is_error: true`.

```bash
cargo test -p ch08-1-fastmcp-core
cargo bench -p ch08-1-fastmcp-core --bench errors
cargo run -p ch08-1-fastmcp-core --example demo
```

The book section is [Chapter 8: Errors done right](../../book/chapter-08-errors-done-right.md).
