# Section 6.1: Resources and templates

Static resources live in a `HashMap` keyed by URI. `{param}` templates match one path segment. The name borrows the pattern and the value borrows the URI. The server copies both before it calls the template.

```bash
cargo test -p ch06-1-fastmcp-core
cargo bench -p ch06-1-fastmcp-core --bench resources
cargo run -p ch06-1-fastmcp-core --example demo
```

The book section is [Chapter 6: Resources and templates](../../book/chapter-06-resources-and-templates.md).
