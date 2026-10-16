# Building FastMCP in Rust — Proposed Outline (v2)

Working title: *Learning Rust by Building FastMCP*. Reference: PrefectHQ/fastmcp `main` @ e2fcc41 (v4.1.0). Paths are relative to `fastmcp_slim/fastmcp/` unless they're prefixed. v1 is kept at `rust-fastmcp-outline-v1.md`. The feature research behind this revision is in `fastmcp-features.md`.

Rule: each Rust concept appears the first time a chapter needs it, never earlier.

**Protocol arc:** FastMCP 4 serves MCP `2026-07-28` and older ("handshake-era") clients from one deployment, negotiating the protocol per connection. The book teaches this in three steps:
- **Ch 1** hand-rolls the handshake-era `initialize` exchange (pinned to `2025-06-18`, as PR #1 already does).
- **Ch 2** adds version negotiation and the `2026-07-28` model on top of rmcp.
- **Ch 12** makes negotiation work per connection over HTTP.

| # | Chapter | Goal | FastMCP mirror | Rust concepts introduced | Candidate benchmark |
|---|---|---|---|---|---|
| 0 | Introduction | What MCP and FastMCP are, the toolchain and mdBook, how steps and tags work | `README.md`, `server/server.py` | cargo, crates, `cargo run/test`, workspace | none |
| 1 | Speaking JSON-RPC | Hand-roll JSON-RPC 2.0 plus handshake-era `initialize` and `ping`, with the protocol version pinned to `2025-06-18` | `server/low_level.py`, `server/mixins/mcp_operations.py`, `types.py` | structs, enums, serde, `Result`, `?`, `#[test]` | parse/serialize of 1 and 1,000 messages; `initialize` dispatch (already in PR #1) |
| 2 | Negotiating the protocol | Accept a range of protocol versions, choose one per client and branch behaviour between handshake-era and `2026-07-28`, then switch the transport to rmcp over stdio | `server/low_level.py`, `utilities/versions.py`, `server/mixins/transport.py`, `server/server.py` (`run`) | `async`/tokio, traits (`ServerHandler`), `match` on enums for each protocol era, `const` slices | negotiation cost; initialize→list round trip on stdio |
| 3 | The `FastMcp` builder and tools | `FastMcp::new("name").tool(...)`, a registry with typed arguments | `tools/{base,function_tool}.py`, `server/providers/local_provider/` | closures, `Box<dyn Fn>`, generics, `HashMap`, builder pattern | tool lookup and dispatch over 1k tools |
| 4 | Schemas from types | JSON Schema for parameters and structured output | `tools/function_parsing.py`, `utilities/json_schema.py` | derive macros, `schemars`, `Option` | schema generation per tool |
| 5 | A `#[tool]` macro | The equivalent of the Python decorator, first through rmcp's macros and then our own | `decorators.py` | proc macros (`syn`, `quote`) | compile-time vs runtime registration |
| 6 | Resources and templates | Static resources and `{param}` URI templates | `resources/{base,template,function_resource}.py` | lifetimes, `&str` vs `String`, pattern matching | template match over 10k resources |
| 7 | Prompts, completions, pagination | Prompt registry, argument completion, cursor-paginated `list_*` | `prompts/*.py`, `server/completions.py`, `utilities/pagination.py` | iterators, `impl Trait` | prompt render; paginated list of 10k items |
| 8 | Errors done right | Tool errors vs protocol errors | `exceptions.py`, `server/middleware/error_handling.py` | `thiserror`, `anyhow`, `From` | error path vs happy path |
| 9 | Context | Logging, progress, sampling, reading resources and request state from inside a tool | `server/context.py` | `Arc`, `Clone`, `Send + Sync` | progress-notification throughput |
| 10 | Dependency injection | `Depends`/`CallArgument`-style injection of the DB, request or token, kept out of the tool schema | `dependencies.py`, `server/dependencies.py` | extractor pattern (`FromContext` trait), generic impls over tuples, `PhantomData` | injected vs hand-passed argument overhead |
| 11 | Lifespan and state | Composable startup and shutdown with shared resources | `server/lifespan.py`, `utilities/lifespan.py` | `Arc<Mutex/RwLock>`, `Drop`, cancellation | contended shared-state access |
| 12 | Streamable HTTP, sessions, both protocol eras | axum server; per-connection negotiation; sessions and resumability on stateless connections; legacy SSE noted | `server/http.py`, `server/sessions.py`, `server/event_store.py`, `server/session_scoped_event_store.py` | axum, tower, `Router`, graceful shutdown | req/s at 1 and 64 clients; stdio vs HTTP; legacy vs `2026-07-28` per-request cost |
| 13 | Elicitation and interactive tools | Ask the user for structured input mid-call: handshake-era `elicit` and v4 multi-round interactive tools | `server/elicitation.py`, `client/elicitation.py` | state machines with enums, `oneshot` channels | round-trip latency per elicitation round |
| 14 | The client and client groups | `Client` with transport inference (stdio, HTTP, SSE, in-memory, `mcpServers` config); `ClientGroup` with one client per server, collision-checked `{server}_{tool}` names and routing | `client/client.py`, `client/transports/*.py`, `client/group.py`, `mcp_config.py` | trait objects vs generics, `tokio::process`, `JoinSet` | latency per transport; fan-out `list_tools` across N servers |
| 15 | Testing utilities | In-memory transport, test harness, snapshot tests | `client/transports/memory.py`, `utilities/tests.py` | `#[cfg(test)]`, `tokio::test`, `insta` | in-memory vs stdio test runtime |
| 16 | Middleware | A chain of hooks: logging, timing, rate limiting, caching, response limiting | `server/middleware/*.py` | `tower::Layer`, `Pin<Box<dyn Future>>` | overhead per layer at 0 and 5 layers |
| 17 | Providers and composition | `Provider` trait (local, aggregate, mounted server, proxy, filesystem discovery); mount and proxy built on top | `server/providers/{base,local_provider,aggregate,fastmcp_provider,proxy,filesystem*}.py` | trait design, `async_trait`/AFIT, modules and visibility, newtypes | proxy vs direct; aggregate list over 10 providers |
| 18 | Transforms and visibility | Namespace, tool transformation, tags/enable/disable, prompts-as-tools, resources-as-tools, `list_changed` | `server/transforms/{namespace,tool_transform,visibility,prompts_as_tools,resources_as_tools}.py`, `tools/tool_transform.py` | `Cow`, `HashSet`, function composition | filtered list over 10k tools |
| 19 | Tool search and code mode | Swap a large catalog for on-demand search, then let the model write a script that calls tools in a sandbox | `server/transforms/search/`, `server/transforms/catalog.py`, docs `servers/transforms/code-mode.mdx` | text indexing (BM25 by hand), embedding a scripting engine (`rhai` as the Rust-native stand-in for FastMCP's Python sandbox), resource limits | search latency over 10k tools; script-orchestrated N calls vs N model turns |
| 20 | Background tasks and extensions | `tool(task=true)` via the `io.modelcontextprotocol/tasks` extension (submit, poll, progress, result); a generic `add_extension` with negotiated capabilities | `fastmcp_tasks/`, `utilities/tasks.py`, `server/extensions.py`, `server/mixins/extensions.py` | `tokio::spawn`, task handles, `watch` channels, type-erased registries (`Any`) | submit-to-result overhead; 1k concurrent tasks |
| 21 | OpenAPI to MCP | Generate tools from an OpenAPI spec, call them through `reqwest` | `server/providers/openapi/`, `utilities/openapi/` | reqwest, larger serde models, cargo features | spec→tools for a large spec |
| 22 | Authentication | JWT/JWKS verification, then OAuth resource-server metadata (the proxies and the full server are summarised only) | `server/auth/{auth,jwt_issuer,middleware}.py`, `server/auth/providers/` | `jsonwebtoken`, axum extractors, secret handling | JWT verify per request |
| 23 | Production: authorization, versioning, storage, telemetry | Callable authorization checks; serving several component versions; pluggable storage (memory, file, Redis) for cache and OAuth state; OpenTelemetry tracing; tool fingerprinting | `utilities/authorization.py`, `server/middleware/authorization.py`, `server/transforms/version_filter.py`, `utilities/versions.py`, `server/caching.py`, docs `servers/storage-backends.mdx`, `telemetry.py`, `server/telemetry.py`, docs `servers/tool-fingerprinting.mdx` | `tracing` and `opentelemetry`, trait objects for backends, feature flags, hashing (`sha2`) | authorization-check overhead; storage backend get/set; tracing on vs off |
| 24 | Settings and CLI | Settings from environment and config, `fastmcp.json`, `run`/`dev`/`inspect`/`install` | `settings.py`, `cli/`, `utilities/mcp_server_config/` | `clap` derive, `figment`, `OnceLock` | cold start |
| 25 | Shipping it | Publish, docs.rs, release binaries, head-to-head against Python FastMCP | `pyproject.toml`, release workflows | `cargo publish`, semver, `cargo-dist` | Rust vs Python summary |

Appendix A: Apps (interactive UIs, `apps/`). Appendix B: OAuth proxy, OIDC proxy and the full OAuth server (`server/auth/oauth_proxy/`, `oidc_proxy.py`). Appendix C: icons and metadata.

## Proposed repo layout (unchanged from v1; mirrors rust-agent-harness and matches PR #1)

```
rust-mcp/
  Cargo.toml           # workspace, resolver 3
  book/                # book.toml, SUMMARY.md, chapter-NN-slug.md, book-outline.md, playground/, social/
  steps/chNN-M-slug/crates/{fastmcp-core,fastmcp-cli}/   # complete cumulative copies; package prefix chNN-M-
  fastmcp-rs/          # the crate the reader grows
  benchmarks/          # data.json, descriptions.json, index.html (gh-pages)
  docs/                # chapter-template, guardrails, performance, tags
  .github/workflows/   # ci.yml (fmt, clippy, per-step tests on Linux/macOS/Windows, mdBook), benchmarks.yml
```

## rmcp note

Use rmcp (3.5.1) for the protocol model types, the session machinery and the stdio/HTTP transports from Ch 2 onward; Ch 1 hand-rolls the wire format once, for learning. Before Ch 2 is written, confirm which protocol versions rmcp 3.5.1 negotiates, and whether it supports `2026-07-28` and the tasks extension. If it doesn't, Ch 2, 12 and 20 implement the missing parts themselves, which also makes good teaching material. Everything that makes FastMCP what it is (builder, macro, providers, transforms, DI, middleware, tasks/extensions plumbing, auth policy, production features, CLI) is implemented by the book.

## Impact on PR #1 (agnathan/rust-mcp#1)

The Chapter 1 code is compatible as it stands. It pins `PROTOCOL_VERSION = "2025-06-18"` and already says that negotiating versions is a later chapter, which matches the new Ch 2. What needs to change:
1. `book/book-outline.md` (and probably `book/SUMMARY.md` and the outline paragraph in `chapter-00-introduction.md`) still has the v1 chapter list. Replace it with this v2 outline.
2. Optional: one sentence in chapter 1 saying that `2025-06-18` is the handshake era and that FastMCP 4 also speaks `2026-07-28`, with a pointer to Ch 2.
3. Check `.github/chapters.json` and `docs/tags.md`, in case they list chapter numbers or titles beyond 1.
