# Section 9.1: Context

A tool receives a `Context` that is not part of its schema. It can log, report progress, read a static resource, and ask for a canned sample.

```bash
cargo test -p ch09-1-fastmcp-core
cargo bench -p ch09-1-fastmcp-core --bench context
cargo run -p ch09-1-fastmcp-core --example demo
```

The book section is [Chapter 9: Context](../../book/chapter-09-context.md).
