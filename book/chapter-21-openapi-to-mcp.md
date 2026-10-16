# Chapter 21: OpenAPI to MCP

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

An HTTP API already has a description. OpenAPI lists the paths, the methods, and an `operationId` for each one. FastMCP reads that document and turns each operation into a tool. Calling the tool sends the HTTP request.

This chapter parses a small slice of that document: `paths`, the method, and `operationId`. A caller interface performs the request. The tests use a stand-in that records the URL. A reqwest client is there for a live server, and the tests do not send with it.

## What you'll build

One tool name, its method, and the body the caller returned.

```text
tool: get_widgets
method: GET
body: []
```

The snapshot is `steps/ch21-1-openapi`. It is a full copy of chapter 20, plus `openapi.rs`. The crates are `ch21-1-fastmcp-core` and `ch21-1-fastmcp-macros`.

```bash
cargo run -p ch21-1-fastmcp-core --example demo
```

## Goals

- Read `paths` and keep `operationId` as the tool name.
- Slug a name from the method and the path when `operationId` is missing.
- Fill `{param}` from the arguments and call an `HttpCaller`.
- Build a reqwest client without sending a request in the tests.

## Roadmap

**21.1 The document becomes tools.** The problem is a spec that is a map of paths, not a list of tools. The walk is the conversion.

**21.2 The call.** The problem is a tool that has to become a URL. The caller is the seam, so the test does not need a live server.

**21.3 Build, run, and benchmark.** The tests, the demo above, and a document with 1,000 operations.

| Section | Step | What it adds |
|---|---|---|
| 21.1–21.3 | `steps/ch21-1-openapi` | `tools_from_spec`, `call_operation`, `MapCaller`, `ReqwestCaller` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/providers/openapi/provider.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/openapi/provider.py) `OpenAPIProvider` (line 61) parses a spec during construction. The client argument is an `httpx2.AsyncClient` (line 84). When `operationId` is set, the component name is that id, or a mapped name (line 218). [`components.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/openapi/components.py) `OpenAPITool` (line 172) performs the HTTP call. Its `task_config` is `forbidden` (line 175). [`routing.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/openapi/routing.py) `MCPType` (line 42) chooses tool, resource, template, or exclude. `RouteMap` (line 59) matches methods and a path pattern.
>
> **Adapted.** The parser keeps GET, POST, PUT, PATCH, and DELETE. A missing `operationId` becomes `{method}_{slug}` with non-letters turned into underscores, so `GET /widgets` is `get_widgets`. FastMCP can also split an `operationId` on `__` and keep the first piece (line 222). The base URL is `servers[0].url`. `MapCaller` records the method and the URL. `ReqwestCaller` builds a `reqwest` client with rustls. The demo calls `MapCaller`, which returns `[]`.
>
> **Deferred.** Route maps, output-schema validation, and a default client created from the server URL with a 30 second timeout. The tests never send a reqwest request. OpenAPI tools in FastMCP cannot run as tasks (`task_config.mode="forbidden"`).

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **A spec with no `paths` object is an error.** The message contains `no paths`. A call names the operation, fills `{id}`, and the stand-in records `http://example.test/widgets/7`. The reqwest client is constructed and not asked to send.

[personal note: an operationId I renamed in the spec and forgot in the client]

## 21.1 The document becomes tools

```rust
let name = operation
    .get("operationId")
    .and_then(Value::as_str)
    .map(str::to_string)
    .unwrap_or_else(|| slug_name(method, path));
```

`tools_from_spec` returns the base URL and the operations. The demo spec has one GET and the id `get_widgets`.

## 21.2 The call

`call_operation` finds the name, replaces `{key}` with the argument, and skips a body for GET and DELETE. `HttpCaller::call` is the request. `MapCaller` stores the method and the URL and returns the body it was given.

`tests/openapi.rs` checks the id, the slug, the filled path, the reqwest builder, and a spec with no paths. Chapter 20's tests still pass in this copy. One hundred fifteen tests passed: the previous 110, plus these five.

## 21.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch21-1-fastmcp-core
cargo run -p ch21-1-fastmcp-core --example demo
cargo bench -p ch21-1-fastmcp-core --bench openapi -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
tool: get_widgets
method: GET
body: []
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
parse_1k                time:   [429.12 µs 438.06 µs 453.81 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `parse_1k` | 430.1 µs | 428.2 µs | 446.3 µs | < 2 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `parse_1k` | `serde_json` parse of the text, then `tools_from_spec` | Building the JSON text | 1,000 GET operations, ids `get_widget_0` through `get_widget_999` | No HTTP request | Well under 2 ms. About 430 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38963719818](https://github.com/agnathan/rust-mcp/actions/runs/38963719818) (2026-10-16T20:38:42Z). GitHub's `github.sha` for that pull request run is the merge commit `f64434c47ae61168900cf1ebeb5459423d091300`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `parse_1k` | 445.2 µs | 441.2 µs | 451.4 µs | < 2 ms |

On the Ubuntu runner, parsing 1,000 operations is 445.2 µs. That is the median this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows an `openapi_tool_name` stub. `tests/ch21.rs` expects `"get_widgets"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch21-openapi.rs`](playground/ch21-openapi.rs) finds `operationId` in a string and prints the demo lines. The playground does not have this book's crate. The crate parses JSON and can build a reqwest client.

[Run the scan on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4e143dcbe4d317651b4b4b42f0324bde).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
tool: get_widgets
method: GET
body: []
```

## Recap

`tools_from_spec` walks `paths` and uses `operationId`, or a slug when the id is missing. `call_operation` fills `{param}` and asks an `HttpCaller`. `MapCaller` records the request. `ReqwestCaller` builds a client for a live call the tests do not make.

`cargo run -p ch21-1-fastmcp-core --example demo` prints the three lines above. Locally, a 1,000-operation document becomes tools in 430.1 µs.

[personal note: a path parameter I left as `{id}` in the URL that went out]

## What's next

Chapter 22 checks a token before a tool runs. A shared secret verifies an HS256 JWT. The audience has to match. A small metadata document says which authorization server protects the resource.
