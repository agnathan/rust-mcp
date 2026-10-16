# Chapter 20: Background tasks and extensions

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Some tool calls outlive the response that accepted them. The client gets an id, polls, and reads the result when the work finishes. FastMCP will not start that work unless the server has registered the tasks extension. The identifier is `io.modelcontextprotocol/tasks`.

An extension is a named capability the server advertises. This chapter keeps those names in a registry and starts a task only when the tasks name is there.

## What you'll build

One id, a finished status, and the text the task returned.

```text
id: 1
status: done
result: ok
```

The snapshot is `steps/ch20-1-tasks`. It is a full copy of chapter 19, plus `tasks.rs`. The crates are `ch20-1-fastmcp-core` and `ch20-1-fastmcp-macros`.

```bash
cargo run -p ch20-1-fastmcp-core --example demo
```

## Goals

- Register an extension by its reverse-DNS identifier.
- Refuse a task board when `io.modelcontextprotocol/tasks` is absent.
- `submit` a closure, return an id, and report `working` until the closure finishes.
- `wait` until the result is stored, then return that text.

## Roadmap

**20.1 The extension id.** The problem is a task feature that turns on by accident. The board checks the registry first.

**20.2 Submit, poll, result.** The problem is a call that returns before the work is done. The id is how the client comes back.

**20.3 Build, run, and benchmark.** The tests, the demo above, one task from submit to result, and one thousand tasks at once.

| Section | Step | What it adds |
|---|---|---|
| 20.1–20.3 | `steps/ch20-1-tasks` | `ExtensionRegistry`, `TaskBoard`, `TASKS_EXTENSION_ID` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`utilities/tasks.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/tasks.py) `TASKS_EXTENSION_ID` (line 21) is `io.modelcontextprotocol/tasks`. A tool with `task=True` requires that extension. `TaskMode` (line 14) is `forbidden`, `optional`, or `required`. `DEFAULT_POLL_INTERVAL_MS` is 5 seconds (line 23) and `DEFAULT_TTL_MS` is 60,000 (line 25). [`server/extensions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/extensions.py) `ServerExtension` (line 160) is the SEP-2133 base. `identifier` is reverse-DNS and is advertised under `ServerCapabilities.extensions`. `auto_register` defaults to `False` (line 180). [`server/mixins/extensions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/mixins/extensions.py) `ExtensionsMixin` (line 25) owns registration. `_required_extensions` (line 28) is what a server contributes when it is used as a provider.
>
> **Adapted.** The registry is a `HashMap` from identifier to `Arc<dyn Any + Send + Sync>`. `TaskBoard::submit` uses `tokio::spawn`. `poll` returns `working` or `done`. FastMCP speaks the tasks protocol through the `fastmcp-tasks` package and keeps core free of that import. The poll interval and the TTL are not enforced here.
>
> **Deferred.** The wire methods for submit, poll, and result, Docket routing, the 60 second TTL, and the 5 second poll hint. The `rmcp` tasks capability stays off. A proxy still does not run tasks. Chapter 17 recorded that FastMCP sets `task_config.mode="forbidden"` on proxied components.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **No extension, no board.** `TaskBoard::new` on an empty registry returns `tasks extension is not registered`. The test checks that string. A submitted task stays `working` until the closure is allowed to finish, then `poll` is `done` and `result` is the text.

[personal note: a task I polled once, saw working, and treated that as failure]

## 20.1 The extension id

```rust
if !registry.contains(TASKS_EXTENSION_ID) {
    return Err("tasks extension is not registered".into());
}
```

`insert` stores the identifier and a value. The demo stores `()`. The id string is the one FastMCP publishes.

## 20.2 Submit, poll, result

`submit` inserts `Working` under a new id, then spawns the closure. The first id is 1. `poll` reads the map. `wait` yields until `Done` is stored and returns the string.

`tests/tasks.rs` refuses an empty registry, sees `working` while a channel is closed, then sees `ok` after the send. Chapter 19's tests still pass in this copy. One hundred ten tests passed: the previous 108, plus these two.

## 20.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch20-1-fastmcp-core
cargo run -p ch20-1-fastmcp-core --example demo
cargo bench -p ch20-1-fastmcp-core --bench tasks -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
id: 1
status: done
result: ok
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
submit_to_result        time:   [4.1810 µs 4.2939 µs 4.4922 µs]
concurrent_1k           time:   [2.3667 ms 2.4588 ms 2.5718 ms]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `submit_to_result` | 4.219 µs | 4.057 µs | 4.250 µs | < 50 µs |
| `concurrent_1k` | 2.423 ms | 2.334 ms | 2.591 ms | < 20 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `submit_to_result` | `submit` of one closure, then `wait` until the result is stored | Building the runtime and the board | A closure that returns `ok` | No socket, no TTL, no poll interval | Well under 50 µs. About 4.2 µs here. |
| `concurrent_1k` | 1,000 submits, then a wait for each id | Building the runtime and the board | The same instant closure, one thousand times | The waits run after all submits, so the tasks overlap. No socket | Well under 20 ms. About 2.4 ms here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38963181333](https://github.com/agnathan/rust-mcp/actions/runs/38963181333) (2026-10-16T20:34:08Z). GitHub's `github.sha` for that pull request run is the merge commit `27e6d2cf41bf37af489b23a3239ae63df6d0ba6b`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `submit_to_result` | 11.27 µs | 10.18 µs | 12.27 µs | < 50 µs |
| `concurrent_1k` | 957.7 µs | 915.1 µs | 1.016 ms | < 20 ms |

On the Ubuntu runner, one task from submit to result is 11.27 µs, and a thousand overlapping tasks are 957.7 µs. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows a `tasks_extension_id` stub. `tests/ch20.rs` expects `"io.modelcontextprotocol/tasks"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch20-tasks.rs`](playground/ch20-tasks.rs) spawns one thread and prints the demo lines. The playground does not have this book's crate. The crate uses `tokio::spawn` and checks the extension id first.

[Run the thread on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ad768651a42df4bd9d50fc3bdb231e10).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
id: 1
status: done
result: ok
```

## Recap

`ExtensionRegistry` stores an identifier. `TaskBoard` opens only when `io.modelcontextprotocol/tasks` is present. `submit` returns an id and runs the closure on the runtime. `poll` is `working` or `done`. `wait` returns the text.

`cargo run -p ch20-1-fastmcp-core --example demo` prints the three lines above. Locally, one submit-to-result takes 4.219 µs, and one thousand overlapping tasks take 2.423 ms.

[personal note: the extension id I mistyped, and the board that refused to open]

## What's next

Chapter 21 reads an OpenAPI document and turns each operation into a tool. Calling the tool builds an HTTP request. The tests use a stand-in client so the chapter does not need a live server.
