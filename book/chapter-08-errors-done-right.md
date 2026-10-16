# Chapter 8: Errors done right

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A client has to tell two failures apart. The tool ran, and it said no. Or the call never became a tool result, because the name was missing, the arguments were wrong, or something inside the server broke. FastMCP draws that line in `to_mcp_error`. An error that already has a JSON-RPC code keeps it. Not found, disabled, and validation become `invalid_params`. Everything else is `internal_error`.

A tool's own `Err` is not in that function. The tool ran. The result comes back with `is_error: true`, and the client reads the message. This chapter builds both outcomes, with `thiserror` for the enum and `anyhow` for the bucket that is not one of the named cases.

## What you'll build

Four lines. A sum, a tool failure, and two protocol codes.

```text
ok: {"total":5}
tool error: nope
not found: -32602
other: -32603
```

`-32602` is `invalid_params`. `-32603` is `internal_error`. The string `nope` never becomes either of those codes, because a tool failure is a result.

The snapshot is `steps/ch08-1-errors`. It is a full copy of chapter 7, plus `app_error.rs`. The crates are `ch08-1-fastmcp-core` and `ch08-1-fastmcp-macros`.

```bash
cargo run -p ch08-1-fastmcp-core --example demo
```

## Goals

- Separate a tool result with `is_error` from a JSON-RPC error.
- Map not-found, disabled, and validation to `invalid_params`, and keep a code that was already chosen.
- Use `thiserror` for the enum and `anyhow` for the other bucket, with `From` so `?` works.
- Read the benchmark that runs a thousand successes next to a thousand misses.

## Roadmap

**8.1 The map.** The problem is one `Err` meaning two things. `AppError` names the cases. `to_mcp_error` picks the code.

**8.2 The tool path.** `tool_app` looks at the variant before it calls the map. `ToolFailed` stays a result.

**8.3 Build, run, and benchmark.** The tests, the demo above, and the two timings.

| Section | Step | What it adds |
|---|---|---|
| 8.1–8.3 | `steps/ch08-1-errors` | `AppError`, `to_mcp_error`, `tool_app` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`exceptions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/exceptions.py) `to_mcp_error` (near line 117) returns an `MCPError` unchanged. `NotFoundError` and `DisabledError` (near lines 70 and 74) and `ValidationError` (near line 46) become `INVALID_PARAMS` (-32602). Every other exception becomes `INTERNAL_ERROR`. [`server/middleware/error_handling.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/error_handling.py) `ErrorHandlingMiddleware` (near line 17) logs the exception and, when `transform_errors` is true (the default), turns it into an `MCPError` before re-raising.
>
> **Adapted.** This chapter's map is `to_mcp_error`, which is W4. The middleware's second table (`ValueError` to invalid params, a missing resource to `-32002`, a timeout to `-32000`) waits for the middleware chapter. `ToolFailed` is matched before `to_mcp_error`, so a tool's own failure stays `is_error`. `From<String>` and `From<anyhow::Error>` are how `?` lands in the enum.
>
> **Deferred.** The middleware log, the error counts, and `include_traceback` wait. Authorization errors wait for chapter 22.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **W4 is this map.** Not found, disabled, and validation are `invalid_params`. An error that is already a protocol error keeps its code. Anything else that escapes the tool is `internal_error`.
>
> **A tool failure is a result.** `is_error: true` means the client got a message. A JSON-RPC error means the call did not produce a tool result. The two are not interchangeable.

[personal note: a failure I first sent as the wrong code, and what the client did with it]

## 8.1 The map

`AppError` is the list of cases. `thiserror` writes `Display` and `Error` from the `#[error]` attributes. `Other` is `#[from] anyhow::Error`, so `Err(anyhow::anyhow!("boom"))?` becomes `AppError::Other` without a hand-written match. `From<String>` makes a bare string a `ToolFailed`, which is the chapter 3 closure's `Err("nope")` landing in the new type.

```rust
pub fn to_mcp_error(error: AppError) -> ErrorData {
    match error {
        AppError::Protocol { code, message } => ErrorData::new(code, message, None),
        AppError::NotFound(message)
        | AppError::Disabled(message)
        | AppError::Validation(message) => ErrorData::invalid_params(message, None),
        AppError::ToolFailed(message) => ErrorData::internal_error(message, None),
        AppError::Other(error) => ErrorData::internal_error(error.to_string(), None),
    }
}
```

`Protocol` is the "already an MCPError" arm. The code you chose is the code the client sees. `NotFound("gone")` is `-32602` and the message is `gone`. `anyhow!("boom")` is `-32603` and the message is `boom`.

`ToolFailed` is in the match so the function is total. Callers that want the tool-result shape should take that variant out first. The next section does.

## 8.2 The tool path

`tool` still takes `Result<R, String>`. That `Err` is `ToolError::Failed`, and `call_tool` turns it into `CallToolResult::error`. `tool_app` takes `Result<R, AppError>`. `register_app` matches before it builds a wire error:

```rust
let returned = match call(params) {
    Ok(value) => value,
    Err(AppError::ToolFailed(message)) => return Err(ToolError::Failed(message)),
    Err(error) => return Err(ToolError::Wire(to_mcp_error(error))),
};
```

`call_tool` returns `ToolError::Wire` as `Err(ErrorData)`. The client library surfaces that as a protocol error. `Failed` is still `Ok` with `is_error: true`. Bad JSON is still `InvalidParams`, and the closure does not run. An unknown name is still `invalid_params` with `unknown tool`.

`From` is the trait `?` uses. A function that returns `Result<Sum, AppError>` can write `some_anyhow_call()?` and the `From` impl fills in `Other`.

## 8.3 Build, run, and benchmark

`tests/errors.rs` calls `fail`, `gone`, and `boom` over the duplex pipe. `fail` is a result with `is_error`. `gone` is `invalid_params` and the message is `gone`. `boom` is `internal_error` and the message is `boom`. The unit tests check the same map without a session. Chapter 7's tests still pass in this copy. Sixty-two tests passed: the previous 56, plus these six.

```bash
cargo test -p ch08-1-fastmcp-core
cargo run -p ch08-1-fastmcp-core --example demo
cargo bench -p ch08-1-fastmcp-core --bench errors -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
ok: {"total":5}
tool error: nope
not found: -32602
other: -32603
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
dispatch_ok_1000        time:   [126.11 µs 128.44 µs 130.15 µs]
dispatch_missing_1000   time:   [64.095 µs 69.395 µs 75.521 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `dispatch_ok_1000` | 128.4 µs | 125.4 µs | 131.5 µs | < 1 ms |
| `dispatch_missing_1000` | 64.08 µs | 63.63 µs | 66.04 µs | < 1 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `dispatch_ok_1000` | 1,000 `dispatch_tool("add", {left:2, right:3})` calls: clone the JSON, deserialize `Add`, run the closure, serialize `Sum` | Building the server | One registered `add`. The same argument object, cloned each call. | No socket | Well under 1 ms. About 128 µs here. |
| `dispatch_missing_1000` | 1,000 `dispatch_tool("missing", ...)` calls: clone the JSON, miss the `HashMap` | Deserialization and the closure, which do not run. Building the server. | The name `missing` on a server that only has `add` | No socket | Well under 1 ms, and below the happy path. About 64 µs here. The clone of the arguments is in both timers. The gap is the work the miss skips. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38865647838](https://github.com/agnathan/rust-mcp/actions/runs/38865647838) (2026-10-16T07:11:53Z). GitHub's `github.sha` for that pull request run is the merge commit `58d3f3f63b65a58c4d92c9953ff0a23bbd8e7508`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `dispatch_ok_1000` | 172.7 µs | 172.4 µs | 173.7 µs | < 1 ms |
| `dispatch_missing_1000` | 75.41 µs | 75.33 µs | 77.68 µs | < 1 ms |

On the Ubuntu runner, a thousand successful dispatches take 172.7 µs and a thousand misses take 75.41 µs. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows a `not_found_code` stub. `tests/ch08.rs` expects `-32602`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch08-errors.rs`](playground/ch08-errors.rs) is the classification without `thiserror` or `rmcp`. The playground does not have this book's crate.

[Run the classification on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=0cab6346941c023e63fe966e7c551a5c).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
ok: {"total":5}
tool error: nope
not found: -32602
other: -32603
```

## Recap

`AppError` names the cases. `to_mcp_error` keeps a chosen code, maps not-found, disabled, and validation to `-32602`, and sends `anyhow` to `-32603`. `tool_app` pulls `ToolFailed` out first, so that `Err` is a tool result with `is_error: true`.

`cargo run -p ch08-1-fastmcp-core --example demo` prints the sum, `nope`, `-32602`, and `-32603`. Locally, a thousand successful dispatches take 128.4 µs and a thousand misses take 64.08 µs. On the Ubuntu run linked above, a thousand successful dispatches take 172.7 µs and a thousand misses take 75.41 µs.

[personal note: the `?` that started working once `From` was in place]

## What's next

Chapter 9 gives a tool a `Context`: a place to log, to report progress, and to read a resource, without those values showing up in the tool's schema.
