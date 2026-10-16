# Chapter 2: Negotiating the protocol

*Draft. Bracketed notes like [personal note] are where my own story goes.*

Chapter 1 can parse one JSON-RPC message and answer `initialize` with the string `2025-06-18`. It cannot speak to a client that asks for a different date, and it never opens a pipe. This chapter keeps those types, then does two new things. It chooses a protocol version from a fixed list, and it runs an `rmcp` server on stdio.

`rmcp` 3.5.1 already knows `2026-07-28`. I checked the crate: `ProtocolVersion::V_2026_07_28` is `ProtocolVersion::LATEST`, and `has_initialize()` is false for that revision. The tasks methods `tasks/get`, `tasks/update`, and `tasks/cancel` are on `ServerHandler`, gated by the `io.modelcontextprotocol/tasks` capability. This chapter does not turn that capability on. FastMCP's `tool(task=true)` runner is chapter 20. What `rmcp` does not decide is our era note: handshake versus modern. That branch is the code in `negotiate.rs`.

| Section | Step | What it adds |
|---|---|---|
| 2.1 | `steps/ch02-1-negotiate` | Version choice, `ProtocolEra`, an `rmcp` `ServerHandler`, stdio |

The package is `ch02-1-fastmcp-core`. The library name is still `fastmcp_core`. It is a full copy of chapter 1 plus this chapter's modules, so `cargo test -p ch01-1-fastmcp-core` still builds the earlier snapshot on its own.

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/low_level.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/low_level.py) sends both `initialize` and `server/discover` through the root middleware. On `2026-07-28`, client capabilities ride on each request's `_meta` (`client_supports_extension`). This server advertises one supported list and lets the handshake and `server/discover` each read it. [`server/mixins/transport.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/mixins/transport.py) `run` is the synchronous entry; `run_stdio_async` opens a stdio pair and runs the session. `FastMcp::run_stdio` is that entry on `rmcp::transport::stdio`.
>
> **Adapted.** [`utilities/versions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/versions.py) compares component versions (PEP 440 strings such as `"1.0"`), not MCP protocol dates. This chapter does not call it. Protocol dates stay on `rmcp::model::ProtocolVersion`. [`server/server.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/server.py) is the `FastMCP` class. `run` is mixed in from `TransportMixin`, not defined in that file.
>
> **Deferred.** The banner, lifespan, and HTTP transports in `run_stdio_async` / `run_http_async` wait. HTTP for both eras is chapter 12. Registering real tools is chapter 3. The tasks extension runner is chapter 20.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **A handshake version we support is echoed. `2026-07-28` is not an `initialize` answer.** `choose_version` returns the requested version only when `has_initialize()` is true and the const slice contains it. A client that asks for `2026-07-28` on `initialize` is answered with `2025-11-25`, the newest handshake version in the slice. Agreeing to `2026-07-28` there would name a revision that has no handshake.
>
> **No handshake version in the slice is an error, not a panic.** `choose_version` returns `ErrorData` with code `UNSUPPORTED_PROTOCOL_VERSION` (`-32022`).
>
> **`initialize` is still a request. `notifications/initialized` is still not.** `rmcp` keeps that split (W1, W2). This chapter does not parse those bytes itself. A `tools/call` in this chapter hits `rmcp`'s default, which is method-not-found. Calling a tool is chapter 3. The full `to_mcp_error` map (W4) is still chapter 8.

[personal note: the first time async code compiled and I was not sure what the runtime was doing]

## What this chapter decides

- `rmcp` 3.5.1 is the protocol crate from here on. Chapter 1's serde types stay in the crate as the picture of the envelope.
- Supported versions are a `const` slice, oldest first, from `2024-11-05` through `2026-07-28`.
- `ProtocolEra::Handshake` is every version with `initialize`. `ProtocolEra::Modern` is `2026-07-28`.
- The stdio server's name is `fastmcp-rs` and its version string is `0.1.0`. It advertises the tools capability. `tools/list` returns one tool, `status`, whose description is the era note. Nothing calls it yet.
- The bench that says "stdio" uses `tokio::io::duplex`, the same async read/write transport as stdin/stdout. It does not spawn a process and it does not touch the terminal's real stdin.

## 2.1 Choosing a version

`ProtocolVersion` in `rmcp` is a small wrapper around a date string. The dates sort in chronological order because they are `YYYY-MM-DD`. `has_initialize` is `version < "2026-07-28"`.

```rust
pub const SUPPORTED_VERSIONS: &[ProtocolVersion] = &[
    ProtocolVersion::V_2024_11_05,
    ProtocolVersion::V_2025_03_26,
    ProtocolVersion::V_2025_06_18,
    ProtocolVersion::V_2025_11_25,
    ProtocolVersion::V_2026_07_28,
];
```

> **Rust: `const` and a shared slice.** `const` means the value is computed at compile time and baked into the binary. The type `&[ProtocolVersion]` is a borrowed slice: a pointer and a length, not an owned `Vec`. `Cow::Borrowed(SUPPORTED_VERSIONS)` hands that slice to `rmcp` without cloning the five versions on each call.

`era_of` looks at `has_initialize`. `era_note` is the `match` that turns the enum into a sentence. `match` must cover every variant. Add a third era later and this function stops compiling until you write the new arm.

> **Rust: `match`.** `match era` picks one arm by the enum variant. The arms here return `&'static str`, a string that lives for the whole process. No allocation.

`choose_version` is the whole negotiation rule:

1. If the client asked for a handshake version that is in the slice, return that version.
2. Otherwise return the newest handshake version in the slice (`max_by` on the date string).
3. If the slice has no handshake version, return `Err`.

`initialize` on `FastMcp` calls `choose_version` and stores the era note in `instructions`. `rmcp` then runs the same rule again on the way out, using our returned version as the fallback preference. The two agree, which the duplex tests check.

`list_tools` reads `context.protocol_version()`. After a handshake that is the version `initialize` agreed. After `server/discover` it is the modern version the client put on the request. The `status` tool's description is `era_note(era_of(version))`.

> **Rust: a trait.** `impl ServerHandler for FastMcp` promises the methods `rmcp` will call. `get_info` and `supported_protocol_versions` are ordinary functions. `initialize` and `list_tools` are `async fn`: they return a future, and the runtime polls it. We do not write the poll loop. `rmcp` and Tokio do.

> **Rust: `async` and `#[tokio::main]`.** `run_stdio` is `async fn`. It `.await`s `serve`, then `.await`s `waiting`, which blocks until the client closes the pipe. `#[tokio::main]` on `examples/stdio.rs` starts a Tokio runtime and runs that future. A runtime is the thread (or threads) that polls futures. Without one, an `async fn` never runs.

### The tests

Unit tests call `choose_version` with no socket. `2025-06-18` comes back unchanged. `2026-07-28` on that function falls back to `2025-11-25`. A slice that contains only `2026-07-28` returns the unsupported-version error.

`tests/session.rs` builds a `tokio::io::duplex`: two ends of an in-memory pipe. One end is `FastMcp::serve`. The other is an `rmcp` client whose `get_info` asks for a chosen version. `#[tokio::test]` starts a runtime for that one test.

- Handshake `2025-06-18`: the peer info version is `2025-06-18`, and `status`'s description is the handshake sentence.
- Handshake `2026-07-28`: the peer info version is `2025-11-25`.
- `ClientLifecycleMode::Discover` with preferred version `2026-07-28`: `status`'s description is the modern sentence.

Chapter 1's handshake tests still pass inside this copy. Twenty-five tests passed: 19 in the library, 3 in `tests/handshake.rs`, 3 in `tests/session.rs`.

### Build, run, and benchmark

```bash
cargo test -p ch02-1-fastmcp-core
cargo run -p ch02-1-fastmcp-core --example demo
cargo bench -p ch02-1-fastmcp-core --bench negotiate -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

`cargo test` printed `ok` for all 25 tests. The demo prints:

```text
requested 2025-06-18 -> 2025-06-18 (handshake era: this session negotiated with initialize)
requested 2025-11-25 -> 2025-11-25 (handshake era: this session negotiated with initialize)
requested 2026-07-28 -> 2025-11-25 (handshake era: this session negotiated with initialize)
a discover client can still be offered 2026-07-28
```

The long-running server is `cargo run -p ch02-1-fastmcp-core --example stdio`. It waits on stdin. A closed pipe ends it. The demo above is the command that exits.

The bench command's slope, on the same 4-vCPU guest as chapter 1:

```text
negotiate_1             time:   [10.441 ns 10.483 ns 10.563 ns]
negotiate_1000          time:   [11.765 µs 11.794 µs 11.817 µs]
stdio_initialize_list   time:   [113.95 µs 115.54 µs 116.92 µs]
```

Local medians from that run's `estimates.json` (the slope above is not this column):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `negotiate_1` | 10.46 ns | 10.43 ns | 10.53 ns | < 1 µs |
| `negotiate_1000` | 11.78 µs | 11.75 µs | 11.81 µs | < 1 ms |
| `stdio_initialize_list` | 113.9 µs | 113.1 µs | 117.3 µs | < 5 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `negotiate_1` | One `choose_version` call | Building the const slice | Requested `2025-06-18`, the five-version slice | No socket, no `rmcp` serve loop | Well under a microsecond. About 10 ns here. |
| `negotiate_1000` | 1,000 `choose_version` calls. Every 5th request is `2026-07-28`; the rest are `2025-06-18`. | Building that `Vec` | The vec above, reused | No session | Under a millisecond. About 12 µs here. |
| `stdio_initialize_list` | One duplex pipe, `serve` on one end, client `initialize` for `2025-06-18`, one `tools/list`, then `cancel` | Building the Tokio runtime. No process spawn. | Server name `fastmcp-rs`, version `0.1.0` | In-memory duplex, not the terminal's stdin. Same `AsyncRead`/`AsyncWrite` transport as `rmcp::transport::stdio` | Under 5 ms. About 114 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38856055391](https://github.com/agnathan/rust-mcp/actions/runs/38856055391) (2026-10-16T05:29:54Z). GitHub's `github.sha` for that pull request run is the merge commit `ee7a1bd5b624f3ac203c5fc461977d052db27246`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `negotiate_1` | 16.66 ns | 16.52 ns | 17.14 ns | < 1 µs |
| `negotiate_1000` | 21.83 µs | 21.78 µs | 21.84 µs | < 1 ms |
| `stdio_initialize_list` | 236.2 µs | 235.0 µs | 238.8 µs | < 5 ms |

On the Ubuntu runner, one version choice is 16.66 ns, a thousand choices are 21.83 µs, and one in-memory initialize plus `tools/list` is 236.2 µs. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows a `negotiated_version_label` stub. `tests/ch02.rs` expects `"2025-06-18"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch02-negotiating-the-protocol.rs`](playground/ch02-negotiating-the-protocol.rs) is the version choice with date strings and a `match`, and no `rmcp`. The [playground crate list](https://play.rust-lang.org/meta/crates) has Tokio and does not have `rmcp`, so the stdio server cannot run there.

[Run the version choice on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=196b3da398ed6527cb59f13ea1b6d6dd).

## Pitfalls

- Echoing `2026-07-28` from `initialize` looks helpful and breaks the handshake. That revision has no `initialize`.
- `utilities/versions.py` is the wrong "version" module for this chapter. It sorts component versions such as `"1.0"`.
- `max_by` on the date string works because the dates are `YYYY-MM-DD`. A version named `"latest"` would sort in the wrong place.
- The duplex bench is not a measurement of process startup. Spawning `examples/stdio` would time the OS, not `choose_version`.

## Chapter summary

The server accepts five protocol dates. `initialize` echoes a handshake date it supports and falls back to `2025-11-25` when the client asks for `2026-07-28`. `server/discover` can still speak `2026-07-28`, and `tools/list` shows which era the connection is in. The bytes move through `rmcp` on stdio.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 3 puts tools in a `HashMap` and calls them. `FastMcp::new("fastmcp-rs").tool(...)` is the builder. The `status` description from this chapter stays, as a reminder of the era, and real tools sit beside it.
