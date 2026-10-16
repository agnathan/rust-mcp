# Section 10.1: Dependency injection

`provide` stores a value by type. `tool_inject` extracts `Dep<T>` before the tool runs. The schema is only the argument struct.

```bash
cargo test -p ch10-1-fastmcp-core
cargo bench -p ch10-1-fastmcp-core --bench inject
cargo run -p ch10-1-fastmcp-core --example demo
```

The book section is [Chapter 10: Dependency injection](../../book/chapter-10-dependency-injection.md).
