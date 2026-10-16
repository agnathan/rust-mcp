# Section 7.1: Prompts, completions, and pagination

A prompt renders a message from an argument map. Completion filters candidates by prefix and stops at 100. Every `list_*` returns a page. The cursor is URL-safe base64 of `{"o": offset}`.

```bash
cargo test -p ch07-1-fastmcp-core
cargo bench -p ch07-1-fastmcp-core --bench prompts
cargo run -p ch07-1-fastmcp-core --example demo
```

The book section is [Chapter 7: Prompts, completions, and pagination](../../book/chapter-07-prompts-completions-pagination.md).
