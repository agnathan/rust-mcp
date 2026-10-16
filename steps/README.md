# Steps

Each `chNN-<slug>/` is the code at the end of that book section. Chapters are cumulative. A later step is a new copy, not a patch on the previous directory, so every chapter still builds on its own.

They are members of the root cargo workspace. Package names are prefixed (`ch01-1-fastmcp-core`) because cargo needs unique names. The library crate name stays `fastmcp_core`, which is what the tests import.

```bash
cargo test -p ch01-1-fastmcp-core
cargo run -p ch01-1-fastmcp-core --example demo
cargo bench -p ch01-1-fastmcp-core --bench jsonrpc
```

| Step | Chapter | Adds |
|---|---|---|
| ch01-1-jsonrpc | 1 | JSON-RPC 2.0 requests, notifications, responses, and errors; MCP `initialize` and `ping`; parse and serialize benches for 1 and 1,000 messages; dispatch of `initialize` |

Your own copy is `fastmcp-rs/` at the repo root. Its chapter 1 test panics in `todo!()` until you implement it. The course plan is chapters 0–25 in [book/book-outline.md](../book/book-outline.md) and `.github/chapters.json`. Only the step in the table above exists so far. After chapter 1 merges, tag `step-01` on that merge commit. Do not push the tag from the chapter branch. See [docs/tags.md](../docs/tags.md).
