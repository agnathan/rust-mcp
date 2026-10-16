# Chapter 11: Lifespan and state

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A database pool should open before the first request and close after the last one. A cache should open second and close first, while the database is still there to flush into. FastMCP composes those lifetimes with `|`. The left side enters first. Shutdown runs in reverse.

This chapter builds that order with `Drop`. `enter` runs the open now and pushes a close hook. When the last owner goes away, the hooks run from last to first. The banner the tools read is an `Arc<RwLock<String>>`, so a clone of the server sees the same string.

## What you'll build

A ready banner, then the open lines, then the closes in reverse.

```text
state: ready
open: db
open: cache
close: cache
close: db
```

The snapshot is `steps/ch11-1-lifespan`. It is a full copy of chapter 10, plus `lifespan.rs`. The crates are `ch11-1-fastmcp-core` and `ch11-1-fastmcp-macros`.

```bash
cargo run -p ch11-1-fastmcp-core --example demo
```

## Goals

- Open two resources and close them in reverse on `Drop`.
- Share a banner with `Arc<RwLock<String>>` across clones of the server.
- Say why only the last clone runs the hooks.
- Read the benchmark: four threads reading that lock.

## Roadmap

**11.1 Enter, then leave in reverse.** The problem is a pool that closes too early. `Drop` is the close.

**11.2 One banner, many clones.** `RwLock` lets readers overlap. A writer waits.

**11.3 Build, run, and benchmark.** The tests, the demo above, and the timing.

| Section | Step | What it adds |
|---|---|---|
| 11.1–11.3 | `steps/ch11-1-lifespan` | `Lifespan`, reverse shutdown, `AppState` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/lifespan.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/lifespan.py) `Lifespan` (near line 61) wraps a startup function. `__or__` (near line 94) composes two of them. `ComposedLifespan.__call__` (near line 160) enters the left lifespan, then the right, and the `async with` exits in reverse. The yielded dicts are shallow-merged. `lifespan` (near line 176) is the decorator that builds a `Lifespan`.
>
> **Adapted.** This chapter's hooks are synchronous. `Drop` runs them, because Rust has no `async Drop` in the version this book uses. The merge is a shared `AppState`, an `Arc<RwLock<String>>`, rather than a dict of yielded values. Only the last `Lifespan` clone runs the hooks: `Arc::get_mut` succeeds when this clone is the only owner.
>
> **Deferred.** Async startup, yielding a dict from a generator, and cancelling an in-flight request when the process stops wait.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **Shutdown is reverse open order.** The cache closes while the database is still open. The opposite order would flush into a pool that is already gone.
>
> **A clone is not a second startup.** `FastMcp` is `Clone`. The banner is shared. The hooks run once, when the last lifespan owner drops.

[personal note: a pool I closed before the flush, and what the log showed]

## 11.1 Enter, then leave in reverse

`enter("db")` appends `open: db` to the log and pushes a closure that will append `close: db`. `enter("cache")` does the same for the cache. The closures sit in a `Vec`. `Drop` pops, so the cache closure runs first.

```rust
fn drop(&mut self) {
    let Some(hooks) = Arc::get_mut(&mut self.hooks) else {
        return;
    };
    let mut hooks = std::mem::take(hooks.get_mut().expect("hooks"));
    while let Some(hook) = hooks.pop() {
        hook();
    }
}
```

`Arc::get_mut` is `None` while another clone still exists. The demo keeps a handle to the log, which is a separate `Arc`, and drops the lifespan. The closes show up in the log after that drop. Holding a clone of the lifespan itself would keep the hooks alive, which is why the demo keeps the log and not the lifespan.

## 11.2 One banner, many clones

`AppState` is an `Arc<RwLock<String>>`. `set` takes the write lock. `get` takes a read lock. Several readers can hold the read lock together. A writer waits until they finish. `FastMcp::app_state` clones the `Arc`, so `server.clone()` sees `set` from the original.

The lock is `std::sync::RwLock`, not a tokio lock. The benchmark uses `std::thread`, so the critical section does not need to be async. A tokio lock is the right tool when the task must yield while it waits. This read does not yield.

## 11.3 Build, run, and benchmark

`tests/lifespan.rs` checks the four log lines and checks that a cloned server reads the banner the original set. Chapter 10's tests still pass in this copy. Sixty-six tests passed: the previous 64, plus these two.

```bash
cargo test -p ch11-1-fastmcp-core
cargo run -p ch11-1-fastmcp-core --example demo
cargo bench -p ch11-1-fastmcp-core --bench lifespan -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
state: ready
open: db
open: cache
close: cache
close: db
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
contended_reads         time:   [238.64 µs 243.07 µs 249.87 µs]
```

Local median from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `contended_reads` | 244.3 µs | 240.7 µs | 245.6 µs | < 2 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `contended_reads` | Spawning four threads, 1,000 read-lock acquisitions on each, joining the threads, and summing the lengths | Building the `RwLock` and the string `ready` | The banner `ready`. Four threads. 1,000 reads each. | No socket and no database. The string is already in the lock. | Well under 2 ms. About 244 µs here. Spawn and join are part of the number. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI median below is filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `contended_reads` | pending | pending | pending | < 2 ms |

The local table is the machine where the command transcript was captured. The chapter quotes the CI median once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `shutdown_order` stub. `tests/ch11.rs` expects `"cache,db"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch11-lifespan.rs`](playground/ch11-lifespan.rs) pushes two closes and pops them. The playground does not have this book's crate.

[Run the closes on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4047da64d1b70dd8063d479955dae46f).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
state: ready
open: db
open: cache
close: cache
close: db
```

## Recap

`enter` records an open and pushes a close. `Drop` pops the closes, so the cache closes before the database. The banner is an `Arc<RwLock<String>>`. Clones of the server share it. The hooks run once, on the last lifespan owner.

`cargo run -p ch11-1-fastmcp-core --example demo` prints `ready` and then the four lines. Locally, four threads reading the lock take 244.3 µs.

[personal note: the drop that ran earlier than I expected, and the clone that was still holding it]

## What's next

Chapter 12 leaves stdio. The same server listens on Streamable HTTP, keeps a session, and answers both the handshake-era client and a `2026-07-28` discover on one port.
