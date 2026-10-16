# Section 5.1: A `#[tool]` macro

`#[tool]` leaves the function callable and emits a `ToolDecl`. `FastMcp::tool_decl` registers it. The schema is still built at registration.

```bash
cargo test -p ch05-1-fastmcp-core
cargo bench -p ch05-1-fastmcp-core --bench macro_reg
cargo run -p ch05-1-fastmcp-core --example demo
```

The book section is [Chapter 5: A `#[tool]` macro](../../book/chapter-05-a-tool-macro.md).
