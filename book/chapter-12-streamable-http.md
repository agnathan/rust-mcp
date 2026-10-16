# Chapter 12: Streamable HTTP, sessions, both protocol eras

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Stdio is one pipe on one machine. A client on another process needs a port. Streamable HTTP is that port: one URL, `POST` for a request, and a body that is either JSON or a server-sent event.

Two kinds of client share the port. A handshake client sends `initialize` and then keeps the `Mcp-Session-Id` the server hands back. A `2026-07-28` client sends `server/discover` and puts the protocol date on that request. The second call does not open a session.

## What you'll build

A listener on `127.0.0.1`, path `/mcp`, and four lines from a client that speaks both eras.

```text
initialize: 2025-06-18
session: present
list: handshake era: this session negotiated with initialize
discover: 2026-07-28
```

The snapshot is `steps/ch12-1-http`. It is a full copy of chapter 11, plus `http.rs`. The crates are `ch12-1-fastmcp-core` and `ch12-1-fastmcp-macros`.

```bash
cargo run -p ch12-1-fastmcp-core --example demo
```

## Goals

- Bind a loopback socket and serve Streamable HTTP at `/mcp`.
- Read `Mcp-Session-Id` from `initialize` and send it on `tools/list`.
- Call `server/discover` for `2026-07-28` and see that the reply has no session id.
- Read three timings: a full round trip, one initialize, and one discover.

## Roadmap

**12.1 The listener.** The problem is a server that only speaks stdio. `serve_http` binds the port and hands each connection to `rmcp`.

**12.2 A session, then a discover.** The problem is two clients on one URL. The handshake gets a session. Discover answers on its own.

**12.3 Build, run, and benchmark.** The tests, the demo above, and the three timings.

| Section | Step | What it adds |
|---|---|---|
| 12.1–12.3 | `steps/ch12-1-http` | Loopback Streamable HTTP, sessions, discover |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/http.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/http.py) `create_streamable_http_app` (line 624) builds one ASGI app for the Streamable HTTP path. `stateless_http` defaults to `False` (line 631), so a handshake client keeps a session. `json_response` defaults to `False` (line 630). `session_idle_timeout` defaults to `"auto"` (line 638), and the docstring at line 663 says that means the MCP SDK default, 1800 seconds as of SDK 2.2. `FastMCPStreamableHTTPSessionManager` (line 44) is the session manager that app starts.
>
> **Adapted.** This chapter serves with `axum` and `rmcp`'s `StreamableHttpService`, on `127.0.0.1:0`. `json_response` is on, so a one-message modern reply is a JSON body. A handshake `initialize` is still an SSE body with `Mcp-Session-Id`, which is how `rmcp` creates that session. The in-memory session manager's idle keep-alive stays at its own default, 300 seconds. FastMCP's 1800-second `"auto"` is a different knob, and this chapter leaves it there.
>
> **Deferred.** [`create_sse_app`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/http.py) (line 469) is the older SSE transport, a separate app with its own path. This listener does not mount it. [`server/event_store.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/event_store.py) `EventStore` (line 52) and [`server/session_scoped_event_store.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/session_scoped_event_store.py) `SessionScopedEventStore` (line 19) replay events after a disconnect. Resumability waits. [`server/sessions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/sessions.py) `Session` (line 175) is a dict of user state under an authenticated principal. That bucket needs auth, which waits for chapter 22. Auth routes on the HTTP app wait with it.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **The session id comes back on initialize.** The next call sends `Mcp-Session-Id`. A list that omits it is rejected.
>
> **Discover stands alone.** The `2026-07-28` reply has no `Mcp-Session-Id`. The version rides on the request.
>
> **The bind is loopback.** `rmcp` allows `localhost`, `127.0.0.1`, and `::1` unless the host list is replaced. This chapter does not replace it.
>
> **W5 waits.** The server-to-client ping timer is chapter 16. **W6** is the client's `ping` and the discover session that has no `InitializeResult`. That is chapter 14.

[personal note: the first HTTP client I pointed at a stdio server, and the silence that followed]

## 12.1 The listener

`serve_http` takes the server by value, binds `127.0.0.1:0`, and returns the URL the kernel assigned. Port 0 means "pick a free port." The factory closes over the server and clones it for each session. `LocalSessionManager` keeps those sessions in memory.

```rust
let config = StreamableHttpServerConfig::default()
    .with_json_response(true)
    .with_legacy_session_mode(true)
    .with_sse_keep_alive(None)
    .with_sse_retry(None)
    .with_cancellation_token(cancel.clone());
let service = StreamableHttpService::new(
    move || Ok(server.clone()),
    Arc::new(LocalSessionManager::default()),
    config,
);
let router = axum::Router::new().nest_service("/mcp", service);
```

`axum::serve` runs on a spawned task. `HttpServer::shutdown` cancels that task. The rest of the socket code, including the small HTTP reader the demo uses, is in `steps/ch12-1-http/crates/fastmcp-core/src/http.rs`.

`Connection: close` on the request asks the server to finish the reply and drop the socket. The reader then reads until the socket ends. A half-close from the client, before that reply is written, can come back empty. The demo leaves the write side open and lets the server close.

## 12.2 A session, then a discover

The demo posts three times.

The first post is `initialize` with `protocolVersion` `2025-06-18` and the header `Mcp-Method: initialize`. The status is 200. The body is `text/event-stream`. The first non-empty `data:` line is the JSON-RPC result, and `protocolVersion` in that result is `2025-06-18`. The response header `Mcp-Session-Id` is the session.

The second post is `tools/list`. It sends that session id, `MCP-Protocol-Version: 2025-06-18`, and `Mcp-Method: tools/list`. The status tool's description is the handshake sentence from chapter 2.

The third post is `server/discover`. The body carries `_meta` with `io.modelcontextprotocol/protocolVersion` set to `2026-07-28`, plus the client name and capabilities. The headers are `MCP-Protocol-Version: 2026-07-28` and `Mcp-Method: server/discover`. The reply is `application/json`. `supportedVersions` includes `2026-07-28`. There is no session header.

A modern `tools/list` uses the same `_meta` and the same two headers, and no session id. The status tool's description is the `2026-07-28` sentence.

A request that names a method and omits `Mcp-Method` is rejected. On this `rmcp`, that rejection is JSON-RPC code `-32020` with the message `missing required Mcp-Method header`.

An `initialize` request takes the session path even when the body names `2026-07-28`. The handshake answer is still a version that has `initialize`. Chapter 2 already chose that version: the newest handshake date this server supports.

The bench deletes the session after a handshake sample. `DELETE` with the session id and `MCP-Protocol-Version` returns 202. The worker for that session stops, so a long bench does not keep thousands of sessions alive.

## 12.3 Build, run, and benchmark

`tests/http_server.rs` checks the session on initialize, the handshake description on `tools/list`, discover's JSON body and missing session id, and a modern `tools/list` whose description names `2026-07-28`. Chapter 11's tests still pass in this copy. Seventy-one tests passed: the previous 66, plus two unit tests for the chunked reader and the SSE line, plus these three.

```bash
cargo test -p ch12-1-fastmcp-core
cargo run -p ch12-1-fastmcp-core --example demo
cargo bench -p ch12-1-fastmcp-core --bench http -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
initialize: 2025-06-18
session: present
list: handshake era: this session negotiated with initialize
discover: 2026-07-28
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
http_roundtrip          time:   [290.24 µs 299.75 µs 306.07 µs]
initialize_once         time:   [167.00 µs 167.76 µs 168.91 µs]
discover_once           time:   [88.946 µs 90.026 µs 91.978 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `http_roundtrip` | 294.9 µs | 290.6 µs | 305.2 µs | < 2 ms |
| `initialize_once` | 167.2 µs | 166.4 µs | 169.8 µs | < 1 ms |
| `discover_once` | 91.22 µs | 89.26 µs | 94.15 µs | < 500 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `http_roundtrip` | A TCP connect, the `initialize` POST, the SSE read, the `tools/list` POST, that read, and the `DELETE` | Binding the listener and building `FastMcp` | `2025-06-18` initialize, one `tools/list` with the session id, one `DELETE` | Loopback only. No TLS and no second machine. | Well under 2 ms. About 295 µs here. |
| `initialize_once` | A TCP connect, the `initialize` POST, the SSE read, and the `DELETE` | `tools/list` and the bind | The same initialize body | The same loopback socket | Well under 1 ms. About 167 µs here. |
| `discover_once` | A TCP connect, one `server/discover` POST, and the JSON read | Session creation and `DELETE` | Discover for `2026-07-28` with `Mcp-Method` set | The same loopback socket | Well under 500 µs. About 91 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38870644362](https://github.com/agnathan/rust-mcp/actions/runs/38870644362) (2026-10-16T07:59:08Z). GitHub's `github.sha` for that pull request run is the merge commit `a14adcc4a62b988416c611c2635b0c4f757dc24a`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `http_roundtrip` | 346.6 µs | 336.0 µs | 350.8 µs | < 2 ms |
| `initialize_once` | 206.2 µs | 203.5 µs | 215.2 µs | < 1 ms |
| `discover_once` | 118.7 µs | 118.3 µs | 119.8 µs | < 500 µs |

On the Ubuntu runner, the round trip is 346.6 µs, one initialize is 206.2 µs, and one discover is 118.7 µs. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows a `streamable_http_path` stub. `tests/ch12.rs` expects `"/mcp"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch12-http.rs`](playground/ch12-http.rs) reads the first `data:` line of an SSE body. The playground has no socket, and it cannot depend on `axum` or `rmcp`, so the listener stays in the step crate.

[Run the SSE read on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e248d33dbe918d1fa605244d9aecb7b8).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
sse: {"protocolVersion":"2025-06-18"}
session header: Mcp-Session-Id
discover: 2026-07-28
```

## Recap

`serve_http` binds `127.0.0.1` and serves `/mcp`. A handshake `initialize` returns `Mcp-Session-Id` and an SSE body whose protocol date is `2025-06-18`. `tools/list` sends that id. `server/discover` for `2026-07-28` returns JSON and no session id.

`cargo run -p ch12-1-fastmcp-core --example demo` prints the four lines above. Locally, the round trip is 294.9 µs, one initialize is 167.2 µs, and one discover is 91.22 µs. On the Ubuntu run linked above, the round trip is 346.6 µs, one initialize is 206.2 µs, and one discover is 118.7 µs.

[personal note: the reply that came back empty because the client had already closed its write side]

## What's next

Chapter 13 asks the user a question in the middle of a tool call. The tool waits. The client sends a value, or it declines. The tool then finishes with that answer.
