# Section 13.1: Elicitation

`Desk::ask` waits on a oneshot. `submit` accepts. A nested object in the schema is rejected.

```bash
cargo test -p ch13-1-fastmcp-core
cargo bench -p ch13-1-fastmcp-core --bench elicit
cargo run -p ch13-1-fastmcp-core --example demo
```

The book section is [Chapter 13: Elicitation and interactive tools](../../book/chapter-13-elicitation.md).
