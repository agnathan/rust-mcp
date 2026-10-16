# Chapter 16: Middleware

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A tool call is one function today. Production wants a list of things that happen around it: write the name down, time it, refuse a caller who has spent the bucket, remember a reply, and cut a reply that grew past a limit. FastMCP puts those jobs in middleware. Each layer sees the call, then decides whether to run the next layer.

A ping is the same idea on a connection. The server sends a ping on a timer so a quiet client stays alive. The interval has to be a positive number of milliseconds, and one connection gets one task.

## What you'll build

One echo, five layers, and the default ping interval.

```text
layers: 5
body: hello
ping: 30000
```

The snapshot is `steps/ch16-1-middleware`. It is a full copy of chapter 15, plus `middleware.rs`. The crates are `ch16-1-fastmcp-core` and `ch16-1-fastmcp-macros`.

```bash
cargo run -p ch16-1-fastmcp-core --example demo
```

## Goals

- Wrap a tool call in a chain of hooks that run before the tool and after it.
- Stop a call when a token bucket is empty, and skip the tool when a cache already has the reply.
- Cut a reply that is longer than a byte limit.
- Reject a ping interval of zero, and start one ping task per connection.

## Roadmap

**16.1 The chain.** The problem is five jobs that all want to sit in front of the tool. `before` runs outer-first. `after` runs in reverse.

**16.2 A ping per connection.** The problem is a timer that starts twice, or an interval of zero. The schedule keeps a set of connection ids.

**16.3 Build, run, and benchmark.** The tests, the demo above, and a thousand calls with zero layers and with five.

| Section | Step | What it adds |
|---|---|---|
| 16.1–16.3 | `steps/ch16-1-middleware` | `ToolHook`, `PingMiddleware`, `FastMcp::invoke` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/middleware/middleware.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/middleware.py) `Middleware` (line 136) builds a handler chain. `tools/call` is one of the methods that gets a hook (line 175). [`logging.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/logging.py) `LoggingMiddleware` (line 148) records messages. [`timing.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/timing.py) `TimingMiddleware` (line 10) times a request in `on_request` (line 39). [`rate_limiting.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/rate_limiting.py) `TokenBucketRateLimiter` (line 25) spends tokens. `RateLimitingMiddleware` (line 95) defaults to 10 requests per second. [`caching.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/caching.py) `ResponseCachingMiddleware` (line 250) caches `tools/call`. [`response_limiting.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/response_limiting.py) `ResponseLimitingMiddleware` (line 22) defaults `max_size` to 1,000,000 (line 53) and rejects a non-positive size. [`ping.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/ping.py) `PingMiddleware` (line 12) defaults `interval_ms` to 30000 and raises when the interval is not positive (line 38). One connection id gets one task (line 60).
>
> **Adapted.** The chain is a `Vec` of `ToolHook`. `before` can stop the call or return a cached body. `after` can rewrite the text. The cache key is the tool name and the argument text. FastMCP also partitions the cache by access token. The rate limit here is one global bucket. A stopped call is `ToolError::Failed` with the message `rate limit exceeded`. FastMCP raises `RateLimitError`, an `MCPError` with code `-32000`.
>
> **Deferred.** Retry, error-handling middleware, and the authorization middleware. Authorization checks are chapter 23. This chapter does not start the async ping loop. It records that the task would start.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **W5.** `PingMiddleware::new(0)` is an error. The message is `interval_ms must be positive`. `note_connection` returns true the first time a connection id is seen, and false the second time. A different id starts its own task.
>
> **A hook that stops the call does not run the tool.** The bucket test allows one call and the second comes back as a failure the client can read.

[personal note: a cache key that forgot the arguments, and the reply that belonged to someone else]

## 16.1 The chain

`with_hook` pushes a layer. `invoke` walks `before` from the outside in. `Before::Next` keeps going. `Before::Stop` returns `ToolError::Failed` and the tool does not run. `Before::Cached` returns the stored text.

```rust
for hook in &self.hooks {
    match hook.before(name, &args_text) {
        Before::Next => {}
        Before::Cached(text) => return Ok(Value::String(text)),
        Before::Stop(message) => return Err(ToolError::Failed(message)),
    }
}
```

After `dispatch_tool`, `after` walks the same hooks in reverse. `ResponseLimit` truncates the text and appends `\n\n[Response truncated due to size limit]`. A limit of zero is rejected. `ResponseCache` stores the text it sees, so the next identical call hits in `before`.

`tests/middleware.rs` registers `echo`, stacks a log, a timer, a bucket of 10, a cache, and a limit of 1,000 bytes. Two calls with the same arguments run the tool once. The log holds the name twice, because the log sits outside the cache. A bucket of one token and a refill of zero stops the second call. A limit of 8 bytes cuts a long JSON reply. Chapter 15's tests still pass in this copy. Ninety-four tests passed: the previous 88, plus two unit tests and these four.

The protocol `call_tool` path uses `invoke` for a registered tool. With no hooks, the result is the value the tool returned.

## 16.2 A ping per connection

```rust
if interval_ms == 0 {
    return Err("interval_ms must be positive");
}
```

`note_connection` inserts the id into a `HashSet`. `HashSet::insert` returns true when the id is new. The second call for the same id returns false, so a second task is not started. This chapter does not spawn the loop that would send the pings. The schedule is the rule the loop has to obey.

## 16.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch16-1-fastmcp-core
cargo run -p ch16-1-fastmcp-core --example demo
cargo bench -p ch16-1-fastmcp-core --bench middleware -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
layers: 5
body: hello
ping: 30000
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
layers_0                time:   [193.34 µs 194.25 µs 195.15 µs]
layers_5                time:   [552.91 µs 564.19 µs 572.70 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `layers_0` | 194.0 µs | 193.1 µs | 195.4 µs | < 2 ms |
| `layers_5` | 557.2 µs | 551.6 µs | 564.3 µs | < 2 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `layers_0` | 1,000 `invoke` calls of `echo`, including argument clone and JSON | Building the server and the argument value | `{"text":"hi"}`, a thousand times | No hooks and no socket | Well under 2 ms. About 194 µs here. |
| `layers_5` | The same thousand calls through two logs, a timer, a token bucket, and a size limit | Building those hooks. The cache is not in this timer. | The same argument. The bucket holds 1e15 tokens and refills just as fast, so it never stops the loop. The limit is 1,000,000 bytes, so it does not cut the reply. | No socket | Well under 2 ms. About 557 µs here. Five layers cost more than zero. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `layers_0` | pending | pending | pending | < 2 ms |
| `layers_5` | pending | pending | pending | < 2 ms |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `ping_interval_ms` stub. `tests/ch16.rs` expects `"30000"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch16-middleware.rs`](playground/ch16-middleware.rs) prints the demo lines and cuts `hello world` to five bytes. The playground does not have this book's crate.

[Run the cut on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e5aa1f020327c718455922513f48b226).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
layers: 5
body: hello
ping: 30000
cut: hello
```

## Recap

`invoke` runs `before` hooks, the tool, then `after` hooks in reverse. A log records the name. A timer keeps the last duration. A token bucket can stop the call. A cache can skip the tool. A limit cuts the text. `PingMiddleware` wants a positive interval and one task per connection id.

`cargo run -p ch16-1-fastmcp-core --example demo` prints the three lines above. Locally, a thousand calls take 194.0 µs with no hooks and 557.2 µs with five.

[personal note: the interval I passed as zero, and the error that saved the loop from spinning]

## What's next

Chapter 17 splits the server into providers. A local provider holds tools you registered. An aggregate asks several providers. A mount prefixes names. A proxy forwards a call and a ping. A directory scan finds files that describe tools.
