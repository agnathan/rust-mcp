# Chapter 15: Testing utilities

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

The client from chapter 14 can talk to a server. A test wants the same conversation without a spare process and without a port. FastMCP's in-memory transport keeps the client and the server in one runtime. A second helper swaps a setting for the length of a test and puts the old value back.

This chapter builds that pair. A harness holds the server and calls it directly. The duplex client from chapter 14 is still there, and a test can ask both for the tool list. A snapshot compares the list to a string checked into the crate.

## What you'll build

The tool list, twice. Once from the harness. Once through the duplex.

```text
memory: status
duplex: status
```

The snapshot is `steps/ch15-1-testing`. It is a full copy of chapter 14, plus `testing.rs`. The crates are `ch15-1-fastmcp-core` and `ch15-1-fastmcp-macros`.

```bash
cargo run -p ch15-1-fastmcp-core --example demo
```

## Goals

- Call a tool from a test with no socket and no child process.
- See the same tool name come back through the in-memory duplex.
- Put a setting back when a guard drops.
- Keep a one-line snapshot of the tool list next to the test.

## Roadmap

**15.1 The harness.** The problem is a test that starts a process to ask one question. The harness holds the server.

**15.2 A setting that returns.** The problem is a test that changes a log level and leaves it changed. `Drop` puts the old value back.

**15.3 Build, run, and benchmark.** The tests, the demo above, and the two list timings.

| Section | Step | What it adds |
|---|---|---|
| 15.1–15.3 | `steps/ch15-1-testing` | `Harness`, `LevelGuard`, an inline snapshot |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`client/transports/memory.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/transports/memory.py) `FastMCPTransport` (line 37) connects a client to a server in the same process. The docstring says this is for unit tests. `connect_session` (line 58) opens memory streams and runs the server beside the client. [`utilities/tests.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/tests.py) `temporary_settings` (line 36) overrides settings for a `with` block and restores them on the way out.
>
> **Adapted.** `Harness` calls `dispatch_tool` on the value it holds. It does not open the memory streams. The duplex path is chapter 14's `McpClient::connect_memory`, which is the stand-in for a stdio session. The snapshot is an `insta` inline string. FastMCP's tests compare values in pytest. `LevelGuard` restores the log level in `Drop`.
>
> **Deferred.** `run_server_in_process` (line 87) starts a server in another process and returns a URL. The ASGI helpers in that file wait. This chapter never spawns a process.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **The duplex still speaks the protocol.** Ping, initialize, and discover stay on `McpClient`. The harness is a shortcut for a tool call. It does not invent a second reply shape.
>
> **A setting comes back.** The guard stores the previous level and writes it back in `Drop`. A test that panics still drops the guard.

[personal note: a test that left DEBUG on, and the log line that buried the failure]

## 15.1 The harness

`Harness` owns a `FastMcp`. `tool_names` asks the server for the list it would advertise. `status` is first, then any tool you registered.

```rust
pub fn snapshot(&self) -> String {
    self.tool_names().join("\n")
}
```

`snapshot` is that list with a newline between names. An empty server still has `status`, so the string is the one word `status`. `call` forwards to `dispatch_tool`. The arguments are a JSON value. The tool runs in this thread.

`tests/harness.rs` builds a harness, checks the snapshot, connects a duplex client to a second server, and checks that both lists are `status`. A third test registers `add` and calls it with `left` 2 and `right` 3. The JSON field `total` is 5. Chapter 14's tests still pass in this copy. Eighty-eight tests passed: the previous 84, plus one unit test and these three.

The inline snapshot is the string after `@`. When the list changes, the test fails and prints the text it saw. The expected text lives in the test, so a reviewer sees the change in the diff.

## 15.2 A setting that returns

`override_log_level` stores the current level and writes the new one. The returned `LevelGuard` writes the old level back in `Drop`.

```rust
impl Drop for LevelGuard {
    fn drop(&mut self) {
        let mut slot = LOG_LEVEL.lock().expect("log level");
        *slot = std::mem::take(&mut self.previous);
    }
}
```

`Drop` runs when the guard goes out of scope, including when the test panics. The unit test sets `DEBUG` inside a block and reads `INFO` after the block. The level is a process-wide `Mutex`. Two tests that both override it can race. This crate's tests take the lock for the whole change.

## 15.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch15-1-fastmcp-core
cargo run -p ch15-1-fastmcp-core --example demo
cargo bench -p ch15-1-fastmcp-core --bench harness -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
memory: status
duplex: status
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
memory_list             time:   [24.064 ns 24.164 ns 24.342 ns]
duplex_list             time:   [52.997 µs 53.878 µs 54.419 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `memory_list` | 24.20 ns | 24.10 ns | 24.40 ns | < 1 µs |
| `duplex_list` | 53.27 µs | 52.09 µs | 54.19 µs | < 200 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `memory_list` | `tool_names`: collect the map keys, sort, and put `status` first | Building the `Harness` and the `FastMcp` | A server named `widgets` with no extra tools | No socket and no duplex | Well under 1 µs. About 24 ns here. |
| `duplex_list` | One `tools/list` on a connected memory client, and collecting the names | `connect_memory` and the handshake | The same server name. The only tool is `status`. | No child process. The duplex is already up. | Well under 200 µs. About 53 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `memory_list` | pending | pending | pending | < 1 µs |
| `duplex_list` | pending | pending | pending | < 200 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `snapshot_tool_name` stub. `tests/ch15.rs` expects `"status"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch15-testing.rs`](playground/ch15-testing.rs) prints the two list lines and restores a level in `Drop`. The playground does not have this book's crate, and it does not run `insta`.

[Run the guard on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=bd61fdb8b6ff32586857a9e79c718eab).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
memory: status
duplex: status
level: DEBUG
level: INFO
```

## Recap

`Harness` lists tools and calls one without a socket. The duplex lists the same name through the pipe chapter 14 opened. `LevelGuard` writes the old log level back in `Drop`. The snapshot string is `status` until you register something else.

`cargo run -p ch15-1-fastmcp-core --example demo` prints the two lines above. Locally, the in-process list is 24.20 ns and the duplex list is 53.27 µs.

[personal note: the snapshot that failed because status had moved off the first line]

## What's next

Chapter 16 puts a chain of hooks around a tool call. One hook logs, one times, one refuses a caller who has used up a bucket of tokens, one caches a reply, and one cuts a reply that grew too long. A ping hook keeps a connection alive, and it refuses an interval that is not positive.
