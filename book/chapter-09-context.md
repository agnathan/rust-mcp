# Chapter 9: Context

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A tool's arguments are what the client sends. The rest of what the tool needs is the server's: a log line, a progress mark, the body of a resource it already registered. FastMCP hands a tool a `Context` for that. The context is not a field in the JSON schema. The client never sends one.

This chapter adds that handle. It is `Clone` because the notes live in an `Arc`. A `Mutex` lets the tool push a line and lets the test read the line after the call returns. Both sides have to be `Send + Sync`, because the server moves the tool onto a task.

## What you'll build

A note that logs, reports progress, reads `file://readme`, and asks for a sample.

```text
note: "hi / hello / canned"
logs: 1
progress: 1
```

`canned` is a fixed string. This chapter does not call a model. The schema for `note` has `text` and does not have `ctx`.

The snapshot is `steps/ch09-1-context`. It is a full copy of chapter 8, plus `context.rs`. The crates are `ch09-1-fastmcp-core` and `ch09-1-fastmcp-macros`.

```bash
cargo run -p ch09-1-fastmcp-core --example demo
```

## Goals

- Pass a `Context` into a tool without putting it in the schema.
- Record a log line and a progress note in memory, shared through `Arc` and `Mutex`.
- Read a static resource from inside the tool.
- Say what `Send + Sync` is doing on that shared list.
- Read the benchmark: a thousand progress notes, and no socket.

## Roadmap

**9.1 The handle.** The problem is a tool that needs the server. `Context` is a cloneable handle onto shared notes.

**9.2 Register it beside the arguments.** `tool_ctx` builds the schema from `P` only. The context is created when the tool runs.

**9.3 Build, run, and benchmark.** The test, the demo above, and the timing.

| Section | Step | What it adds |
|---|---|---|
| 9.1–9.3 | `steps/ch09-1-context` | `Context`, `tool_ctx`, in-memory logs and progress |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/context.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/context.py) `Context` (near line 143) is the object a tool receives when it asks for one. `report_progress` (near line 453) sends a progress notification when the request carries a progress token. `info` (near line 834) and `debug` (near line 818) log to the client. `read_resource` (near line 593) reads a URI through the server.
>
> **Adapted.** This chapter records logs and progress in a `Mutex<Vec>`. The wire notification waits until a request carries a progress token. `sample` returns the string `canned`. I searched `context.py` at this commit and did not find a `sample` method, so the canned reply is this chapter's stand-in for a model call, not a line copied from FastMCP. `read_resource` on the context sees the static resources registered before `tool_ctx`. Templates stay on `FastMcp::read`.
>
> **Deferred.** `elicit` is chapter 13. Session state, `set_state`, and the real sampling request to the client wait.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **The context is not an argument.** A client that sends `{"text":"hi"}` is enough. A field named `ctx` in the schema would ask the model to invent a handle.
>
> **Progress here is a record, not a notification.** The list is how the test sees the call. Sending `notifications/progress` on the session waits for a progress token on the request.

[personal note: the first time two tasks wanted the same list, and what the compiler said]

## 9.1 The handle

`Context` holds an `Arc<SharedNotes>` and a copy of the static resource map. `Clone` clones the `Arc`, so two handles push into one list. `info` and `report_progress` lock, push, and unlock.

`Send` means the value can move to another thread. `Sync` means a shared reference can be used from several threads. `Arc<Mutex<...>>` is both, once the contents are `Send`. The tool closure is `'static + Send + Sync` because `tool` stores it for the life of the server, and the server runs on a tokio task. The context has to be the same kind of value, or it could not be built inside that closure.

`sample` ignores the prompt and returns `canned`. The point of the method in this chapter is the shape: a tool can ask, and the test can see the answer, without a network.

## 9.2 Register it beside the arguments

`tool_ctx` takes `Fn(Context, P) -> Result<R, String>`. The schema is `json_schema_of::<P>()`. `Context` never reaches `schemars`.

```rust
let notes = Arc::clone(&self.notes);
let resources = self.resources.clone();
register_ctx(
    description,
    move || Context::new(Arc::clone(&notes), resources.clone()),
    call,
)
```

The closure that builds the context runs on each call. The notes are the server's `Arc`, so `server.logs()` after `dispatch_tool` sees the line the tool pushed. The resource map is the one present when you called `tool_ctx`, which is why the demo registers `file://readme` first.

`FastMcp` is `Clone`. The clone shares `notes` and copies the tool map. The test keeps one clone, moves the other into `serve`, and reads the logs from the one it kept.

## 9.3 Build, run, and benchmark

`tests/context.rs` lists `note` and checks the schema has no `ctx`. The call returns `hi / hello / canned`. The server's log list has one `working` line, and the progress list has `1.0`. Chapter 8's tests still pass in this copy. Sixty-three tests passed: the previous 62, plus this one.

```bash
cargo test -p ch09-1-fastmcp-core
cargo run -p ch09-1-fastmcp-core --example demo
cargo bench -p ch09-1-fastmcp-core --bench context -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
note: "hi / hello / canned"
logs: 1
progress: 1
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
progress_1000           time:   [14.391 µs 14.549 µs 14.696 µs]
```

Local median from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `progress_1000` | 14.37 µs | 14.19 µs | 14.66 µs | < 100 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `progress_1000` | 1,000 `report_progress` calls on one `Context`, then `clear` so the list does not grow across samples | Building the `Context`. No JSON, no tool dispatch. | `progress` is 0 through 999. `total` is `Some(1000)`. The message is `None`. | No socket and no `notifications/progress`. The note is a struct in a `Mutex<Vec>`. | Well under 100 µs. About 14 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI median below is filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `progress_1000` | pending | pending | pending | < 100 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI median once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `canned_sample` stub. `tests/ch09.rs` expects `"canned"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch09-context.rs`](playground/ch09-context.rs) is two clones of one list. The playground does not have this book's crate.

[Run the shared list on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=6a6f0d25005352509385bb2ff2e022ee).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
note: hi / hello / canned
logs: 1
progress: 2
```

## Recap

`tool_ctx` gives a tool a `Context` and leaves the schema as `P`. The context is an `Arc` around a `Mutex` of notes, so a clone shares the list and a test can read it after the call. `read_resource` returns a static body. `sample` returns `canned`.

`cargo run -p ch09-1-fastmcp-core --example demo` prints the note, one log, and a progress value of 1. Locally, a thousand progress notes take 14.37 µs.

[personal note: a lock I held too long, or the moment sharing the list was obvious]

## What's next

Chapter 10 injects the other things a tool needs, a database handle or a token, and keeps those out of the schema too. The mechanism is a trait, `FromContext`, and a `PhantomData` so the type is part of the value even when the value holds no data of that type.
