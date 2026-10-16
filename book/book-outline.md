# Building FastMCP in Rust — Proposed Outline (draft)

Working title: *Learning Rust by Building FastMCP*. Reference: PrefectHQ/fastmcp (main, cloned 2026-10-15; package now lives in `fastmcp_slim/fastmcp/`, with `fastmcp_remote/` and `fastmcp_tasks/` as sibling packages). Paths below are relative to `fastmcp_slim/fastmcp/`.

Rule: each Rust concept appears the first time a chapter needs it, never earlier.

| # | Chapter | Goal | FastMCP mirror | Rust concepts introduced | Candidate benchmark |
|---|---|---|---|---|---|
| 0 | Introduction | What MCP is, what FastMCP does, toolchain + mdBook, how steps/tags work | `README.md`, `server/server.py` overview | cargo, crates, `cargo run/test`, workspace | none |
| 1 | Speaking JSON-RPC | Hand-parse/emit MCP JSON-RPC messages (initialize, ping) to understand the wire before using rmcp | `mcp_operations` concepts, `types.py` | structs, enums, `serde`/`serde_json`, `Result`, `?` | serde round-trip of 1k messages |
| 2 | First server over stdio with rmcp | Minimal `initialize`/`tools/list` server on stdio, test with MCP Inspector | `server/server.py` (`FastMCP.run`), `server/mixins/transport.py` | async/`tokio`, `#[tokio::main]`, traits (`ServerHandler`), ownership basics | initialize→list round-trip latency |
| 3 | The `FastMCP` builder and tools | `FastMcp::new("name").tool(...)` registry with typed args | `tools/base.py`, `tools/function_tool.py`, `server/providers/local_provider/` | closures, `Box<dyn Fn>`, generics, `HashMap`, builder pattern | tool lookup + dispatch over 1k tools |
| 4 | Schemas from types | Derive JSON Schema for tool params and structured output | `tools/function_parsing.py`, `utilities/json_schema.py` | derive macros, `schemars`, `Deserialize` bounds, `Option` | schema generation cost per tool |
| 5 | A `#[tool]` macro | Ergonomic decorator-equivalent (thin layer over rmcp macros, then our own) | `decorators.py`, `tool()` in `server/server.py` | attribute/proc macros (using, then writing a small one with `syn`/`quote`) | compile-time vs runtime registration |
| 6 | Resources and templates | Static resources + `{param}` URI templates | `resources/base.py`, `resources/template.py`, `resources/function_resource.py` | lifetimes (`&str` vs `String`), pattern matching, `regex` | URI template match over 10k resources |
| 7 | Prompts and completions | Prompt registry, arguments, completion | `prompts/base.py`, `prompts/function_prompt.py`, `server/completions.py` | iterators, `impl Trait`, `Vec` transformations | prompt render throughput |
| 8 | Errors done right | Tool errors vs protocol errors, `ToolError` mapping | `exceptions.py`, `server/middleware/error_handling.py` | `thiserror`, `anyhow`, `From` conversions | error-path overhead vs happy path |
| 9 | Context | Logging, progress, sampling, elicitation, request state from inside a tool | `server/context.py`, `server/elicitation.py`, `server/dependencies.py` | `Arc`, `Clone`, extractor pattern, `Send + Sync` | progress notification throughput |
| 10 | Lifespan and state | Startup/shutdown resources (DB pool) shared across requests | `server/lifespan.py`, `server/mixins/lifespan.py` | `Arc<Mutex/RwLock>`, `Drop`, async cancellation | contended shared-state access |
| 11 | Streamable HTTP | Serve over HTTP with sessions + resumability; mention legacy SSE | `server/http.py`, `server/sessions.py`, `server/event_store.py` | `axum`, `tower` services, `Router`, graceful shutdown | req/s at 1/64 concurrent clients, stdio vs HTTP |
| 12 | The client | `Client` with transport inference (stdio/HTTP/in-memory) | `client/client.py`, `client/transports/{stdio,http,sse,memory}.py` | enums of transports, trait objects vs generics, `tokio::process` | client call latency per transport |
| 13 | Testing utilities | In-memory transport + test harness, snapshot tests | `client/transports/memory.py`, `utilities/tests.py` | `#[cfg(test)]`, integration tests, `tokio::test`, `insta` | in-memory vs stdio test runtime |
| 14 | Middleware | on_request/on_call_tool chain: logging, timing, rate limit, caching | `server/middleware/{middleware,logging,timing,rate_limiting,caching}.py` | `tower::Layer`, async traits, `Pin<Box<dyn Future>>` (gently) | per-layer overhead with 0/5 layers |
| 15 | Composition: mount, namespace, proxy | Mount sub-servers with prefixes; proxy a remote server | `server/providers/{aggregate,fastmcp_provider,proxy}.py`, `server/transforms/namespace.py` | modules/visibility, newtypes, recursion with `Box` | proxy overhead vs direct |
| 16 | Transforms and visibility | Tool transforms, tags, enable/disable, list_changed | `tools/tool_transform.py`, `server/transforms/{visibility,tool_transform}.py` | `Cow`, smart filtering, `HashSet` | filtered list over 10k tools |
| 17 | OpenAPI to MCP | Generate tools from an OpenAPI spec, call via `reqwest` | `server/providers/openapi/`, `utilities/openapi/` | `reqwest`, larger serde models, feature flags | spec→tools generation for a large spec (e.g. Stripe) |
| 18 | Auth | Bearer/JWT verification, then OAuth resource-server metadata | `server/auth/{auth,jwt_issuer,middleware}.py`, `server/auth/providers/` | `jsonwebtoken`, axum extractors, secrets hygiene | JWT verify per request |
| 19 | Settings and CLI | Env/config settings and a `fastmcp run`-style CLI | `settings.py`, `cli/cli.py`, `cli/run.py`, `mcp_config.py` | `clap` derive, `figment`/`config`, `OnceLock` | cold start time |
| 20 | Shipping it | Publish crate, docs.rs, release binaries, perf comparison vs Python FastMCP | `pyproject.toml`, release workflows | `cargo publish`, semver, `cargo-dist` | head-to-head Rust vs Python benchmark summary |

Optional appendix: tasks (`fastmcp_tasks/`), apps/UI (`apps/`), telemetry (`telemetry.py`).

## Proposed repo layout (mirrors rust-agent-harness)

```
rust-fastmcp/
  Cargo.toml            # [workspace] resolver="3", every step crate is a member
  book/                 # mdBook: book.toml, SUMMARY.md, chapter-NN-slug.md, book-outline.md, social/chapter-NN.md
  steps/
    README.md           # table: step | chapter | adds
    ch01-1-jsonrpc/crates/fastmcp-core/
    ch02-stdio/crates/{fastmcp-core,fastmcp-cli}/
    ...                 # chNN-M-slug; each step is a full cumulative copy
  benchmarks/           # data.json, descriptions.json, index.html (gh-pages chart)
  fastmcp-rs/           # the "final" crate (like my-harness/)
  .github/workflows/    # ci.yml (fmt+clippy, per-step test matrix via list_steps.py), benchmarks.yml (Criterion -> gh-pages)
  CONTRIBUTING.md  LICENSE  README.md
```
Packages prefixed per step (`ch03-fastmcp-core`) for unique cargo names; library crate name stays `fastmcp_core`. Git tags `step-NN` at each chapter end.

## rmcp note

The official Rust SDK `rmcp` (modelcontextprotocol/rust-sdk) is at **3.5.1** (crates.io, updated 2026-10-05). It provides the protocol model types, the `ServerHandler`/`ClientHandler` service traits, stdio, child-process and Streamable HTTP transports (server via axum/tower, client via reqwest), `#[tool]`/`#[tool_router]`/`#[prompt]` macros, and OAuth client support. Decision: build on rmcp for the wire protocol, JSON-RPC session handling and transports (chapter 1 hand-rolls JSON-RPC once, for learning only). Implement ourselves the parts that make FastMCP FastMCP: the builder/registry, the decorator-style macro, resource templates, Context, middleware, mounting/proxy/transforms, OpenAPI generation, auth policy, settings/CLI and test utilities. This matches rust-agent-harness, which already uses rmcp in ch04, and keeps the book about FastMCP's design instead of re-implementing the spec.

## Notes recorded while writing

The table above is the approved outline. Sentences here are ideas a chapter read in FastMCP and left for later. FastMCP commit read for chapter 1: `e2fcc412001ff7414095c5dc282f94c70299a612` (2026-10-08). The outline's "cloned 2026-10-15" date is the outline's; this chapter opened the commit above.

- **Chapter 2.** `Client.initialize` and the `server/discover` era (`client/client.py`, `server/low_level.py`) are the real handshake. Chapter 1 only models the `initialize` JSON. The first stdio server is where a session starts.
- **Chapter 8.** `to_mcp_error` in `exceptions.py` maps `NotFoundError`, `DisabledError`, and `ValidationError` to `INVALID_PARAMS`, and leaves other failures as `INTERNAL_ERROR`. Chapter 1 has the JSON-RPC codes only.
- **Chapter 11.** Streamable HTTP sessions, resumability, and the event store are untouched here.
- **Chapter 12.** Transport inference and a client `ping` that checks `EmptyResult` (`client/client.py`) wait for the client chapter.
- **Chapter 14.** `PingMiddleware` (`server/middleware/ping.py`) keeps a connection alive with `interval_ms` defaulting to `30000` and one task per connection. Chapter 1 only answers an inbound `ping` with `{}`.
- **Chapter 15.** Proxy `initialize` forwarding of `instructions`, and proxy `ping` (`server/providers/proxy.py`), wait for composition.
