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
cargo test -p ch07-1-fastmcp-core
cargo run -p ch07-1-fastmcp-core --example demo
cargo bench -p ch07-1-fastmcp-core --bench prompts
cargo test -p ch08-1-fastmcp-core
cargo run -p ch08-1-fastmcp-core --example demo
cargo bench -p ch08-1-fastmcp-core --bench errors
cargo test -p ch09-1-fastmcp-core
cargo run -p ch09-1-fastmcp-core --example demo
cargo bench -p ch09-1-fastmcp-core --bench context
cargo test -p ch10-1-fastmcp-core
cargo run -p ch10-1-fastmcp-core --example demo
cargo bench -p ch10-1-fastmcp-core --bench inject
cargo test -p ch11-1-fastmcp-core
cargo run -p ch11-1-fastmcp-core --example demo
cargo bench -p ch11-1-fastmcp-core --bench lifespan
cargo test -p ch12-1-fastmcp-core
cargo run -p ch12-1-fastmcp-core --example demo
cargo bench -p ch12-1-fastmcp-core --bench http
cargo test -p ch13-1-fastmcp-core
cargo run -p ch13-1-fastmcp-core --example demo
cargo bench -p ch13-1-fastmcp-core --bench elicit
cargo test -p ch14-1-fastmcp-core
cargo run -p ch14-1-fastmcp-core --example demo
cargo bench -p ch14-1-fastmcp-core --bench client
cargo test -p ch15-1-fastmcp-core
cargo run -p ch15-1-fastmcp-core --example demo
cargo bench -p ch15-1-fastmcp-core --bench harness
cargo test -p ch16-1-fastmcp-core
cargo run -p ch16-1-fastmcp-core --example demo
cargo bench -p ch16-1-fastmcp-core --bench middleware
cargo test -p ch17-1-fastmcp-core
cargo run -p ch17-1-fastmcp-core --example demo
cargo bench -p ch17-1-fastmcp-core --bench providers
cargo test -p ch18-1-fastmcp-core
cargo run -p ch18-1-fastmcp-core --example demo
cargo bench -p ch18-1-fastmcp-core --bench transform
cargo test -p ch19-1-fastmcp-core
cargo run -p ch19-1-fastmcp-core --example demo
cargo bench -p ch19-1-fastmcp-core --bench search
cargo test -p ch20-1-fastmcp-core
cargo run -p ch20-1-fastmcp-core --example demo
cargo bench -p ch20-1-fastmcp-core --bench tasks
cargo test -p ch21-1-fastmcp-core
cargo run -p ch21-1-fastmcp-core --example demo
cargo bench -p ch21-1-fastmcp-core --bench openapi
cargo test -p ch22-1-fastmcp-core
cargo run -p ch22-1-fastmcp-core --example demo
cargo bench -p ch22-1-fastmcp-core --bench auth
```

| Step | Chapter | Adds |
|---|---|---|
| ch01-1-jsonrpc | 1 | JSON-RPC 2.0 requests, notifications, responses, and errors; MCP `initialize` and `ping`; parse and serialize benches for 1 and 1,000 messages; dispatch of `initialize` |
| ch02-1-negotiate | 2 | A const list of protocol versions, handshake versus `2026-07-28`, and an `rmcp` server on stdio; negotiation benches and one initialize-then-list round trip on an in-memory duplex |
| ch03-1-builder | 3 | `FastMcp::new(...).tool(...)`, a `HashMap` of closures with typed arguments, `tools/call`; lookup of one tool among 1,000 and dispatch of all 1,000 |
| ch04-1-schema | 4 | JSON Schema for a tool's argument struct and return struct via `schemars`; `Option` stays out of `required`; schema generation for 1 and 1,000 tools |
| ch05-1-tool-macro | 5 | `#[tool]` emits a `ToolDecl` and leaves the function callable; `tool_decl` registers it; one registration versus 1,000 direct calls |
| ch06-1-resources | 6 | Static resources in a `HashMap`, `{param}` URI templates with two lifetimes, `resources/read`; one lookup among 10,000 URIs and a scan of 10,000 templates |
| ch07-1-prompts | 7 | Prompts, prefix completion capped at 100, and a base64 offset cursor on every `list_*`; one render and one page from 10,000 items |
| ch08-1-errors | 8 | `AppError` and `to_mcp_error`: not-found is `invalid_params`, `anyhow` is `internal_error`, a tool `Err` stays `is_error`; 1,000 successes versus 1,000 misses |
| ch09-1-context | 9 | `Context` via `Arc` and `Mutex`: logs, progress, a static resource read, a canned sample; 1,000 progress notes in memory |
| ch10-1-depends | 10 | `provide` and `Dep<T>` with `PhantomData`; a tuple `Extract`; the schema lists `q` and omits the database and the token |
| ch11-1-lifespan | 11 | `Lifespan` opens in order and `Drop` closes in reverse; `Arc<RwLock<String>>` shares a banner; four threads read the lock |
| ch12-1-http | 12 | Streamable HTTP on `127.0.0.1`, `Mcp-Session-Id` after `initialize`, `server/discover` for `2026-07-28` with no session; one round trip, one initialize, one discover |
| ch13-1-elicit | 13 | `Desk` asks on a oneshot; `submit` accepts; a nested elicitation object is rejected; one in-process round |
| ch14-1-client | 14 | `infer_transport` classifies a string; an in-memory client pings; a group prefixes `{server}_{tool}` and fans out with `JoinSet` |
| ch15-1-testing | 15 | `Harness` lists and calls tools in process; the duplex lists the same names; `LevelGuard` restores a log level |
| ch16-1-middleware | 16 | `ToolHook` chain: log, time, token bucket, cache, response limit; `PingMiddleware` wants a positive interval and one task per connection |
| ch17-1-providers | 17 | `Provider`: local list, aggregate, `{namespace}_` mount, proxy ping and instructions, `*.tool` discovery |
| ch18-1-transforms | 18 | Catalog transforms: namespace `Cow`, argument rename, tag visibility, prompts and resources as tools, `list_changed` |
| ch19-1-search | 19 | BM25 search over tool text, and a rhai script whose `call_tool` stops at a limit |
| ch20-1-tasks | 20 | Extension registry and a task board: submit, poll, and result, gated on `io.modelcontextprotocol/tasks` |
| ch21-1-openapi | 21 | OpenAPI `paths` and `operationId` become tools; `HttpCaller` fills `{param}`; reqwest builds and does not send in tests |
| ch22-1-auth | 22 | HS256 verify with audience check, a JWKS `kid` read, and protected-resource metadata |

Your own copy is `fastmcp-rs/` at the repo root. Its chapter tests panic in `todo!()` until you implement them. The course plan is chapters 0–25 in [book/book-outline.md](../book/book-outline.md) and `.github/chapters.json`. The steps in the table above exist so far. After a chapter merges, tag `step-NN` on that merge commit. Do not push the tag from the chapter branch. See [docs/tags.md](../docs/tags.md).
