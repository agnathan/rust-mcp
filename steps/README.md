# Steps

Each `chNN-<slug>/` is the code at the end of that book section. Chapters are cumulative. A later step is a new copy, not a patch on the previous directory, so every chapter still builds on its own.

They are members of the root cargo workspace. Package names are prefixed (`ch01-1-fastmcp-core`) because cargo needs unique names. The library crate name stays `fastmcp_core`, which is what the tests import.

```bash
cargo test -p ch01-1-fastmcp-core
cargo run -p ch01-1-fastmcp-core --example demo
cargo bench -p ch01-1-fastmcp-core --bench jsonrpc
cargo test -p ch02-1-fastmcp-core
cargo run -p ch02-1-fastmcp-core --example demo
cargo bench -p ch02-1-fastmcp-core --bench negotiate
cargo test -p ch03-1-fastmcp-core
cargo run -p ch03-1-fastmcp-core --example demo
cargo bench -p ch03-1-fastmcp-core --bench tools
cargo test -p ch04-1-fastmcp-core
cargo run -p ch04-1-fastmcp-core --example demo
cargo bench -p ch04-1-fastmcp-core --bench schema
cargo test -p ch05-1-fastmcp-core
cargo run -p ch05-1-fastmcp-core --example demo
cargo bench -p ch05-1-fastmcp-core --bench macro_reg
cargo test -p ch06-1-fastmcp-core
cargo run -p ch06-1-fastmcp-core --example demo
cargo bench -p ch06-1-fastmcp-core --bench resources
```

| Step | Chapter | Adds |
|---|---|---|
| ch01-1-jsonrpc | 1 | JSON-RPC 2.0 requests, notifications, responses, and errors; MCP `initialize` and `ping`; parse and serialize benches for 1 and 1,000 messages; dispatch of `initialize` |
| ch02-1-negotiate | 2 | A const list of protocol versions, handshake versus `2026-07-28`, and an `rmcp` server on stdio; negotiation benches and one initialize-then-list round trip on an in-memory duplex |
| ch03-1-builder | 3 | `FastMcp::new(...).tool(...)`, a `HashMap` of closures with typed arguments, `tools/call`; lookup of one tool among 1,000 and dispatch of all 1,000 |
| ch04-1-schema | 4 | JSON Schema for a tool's argument struct and return struct via `schemars`; `Option` stays out of `required`; schema generation for 1 and 1,000 tools |
| ch05-1-tool-macro | 5 | `#[tool]` emits a `ToolDecl` and leaves the function callable; `tool_decl` registers it; one registration versus 1,000 direct calls |
| ch06-1-resources | 6 | Static resources in a `HashMap`, `{param}` URI templates with two lifetimes, `resources/read`; one lookup among 10,000 URIs and a scan of 10,000 templates |

Your own copy is `fastmcp-rs/` at the repo root. Its chapter tests panic in `todo!()` until you implement them. The course plan is chapters 0–25 in [book/book-outline.md](../book/book-outline.md) and `.github/chapters.json`. The steps in the table above exist so far. After a chapter merges, tag `step-NN` on that merge commit. Do not push the tag from the chapter branch. See [docs/tags.md](../docs/tags.md).
