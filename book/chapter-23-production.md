# Chapter 23: Production

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A server that works on a laptop still has to refuse a call, tell two versions of a tool apart, remember a value, and leave a trace. FastMCP splits those jobs. An authorization check returns yes or no. A version filter keeps the newest component. Storage is a trait with more than one backend. Telemetry is a span around the request. A fingerprint is a hash of the tool's name and schema, so a client can see that the tool changed.

Component versions are strings on a tool. A protocol date such as `2025-11-25` is a different idea, from chapter 2. This chapter compares component versions.

## What you'll build

An allowed call, the newer version, a stored value, and the start of a SHA-256 fingerprint.

```text
allowed: add
version: 10
stored: ok
algo: sha256
digest: 12090edc3a45…
```

The snapshot is `steps/ch23-1-production`. It is a full copy of chapter 22, plus `production.rs`. The crates are `ch23-1-fastmcp-core` and `ch23-1-fastmcp-macros`.

```bash
cargo run -p ch23-1-fastmcp-core --example demo
```

## Goals

- Refuse a call when a check returns false.
- Keep the highest component version of each name, and compare `10` as newer than `2`.
- Store a string in memory and in a JSON file.
- Open a tracing span around a call, and hash the name plus the schema with SHA-256.

## Roadmap

**23.1 Who may call, and which version.** The problem is a tool that should be hidden from one caller, and two copies of `add`. The check runs first. The version filter keeps one.

**23.2 Remember, trace, fingerprint.** The problem is a value that has to survive a restart, a request that has to show up in a trace, and a schema that changed without a new name.

**23.3 Build, run, and benchmark.** The tests, the demo above, a thousand checks, one memory get, and a thousand calls with the span off and on.

| Section | Step | What it adds |
|---|---|---|
| 23.1–23.3 | `steps/ch23-1-production` | `allow_call`, `highest_versions`, `Store`, `traced_call`, `fingerprint` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`utilities/authorization.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/authorization.py) `run_auth_checks` (line 304) runs checks with AND logic and stops at the first failure. [`server/transforms/version_filter.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/version_filter.py) `VersionFilter` (line 24) keeps components in a version range. Unversioned components are included by default, and the highest version of each name is what the client sees. [`utilities/versions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/versions.py) parses those strings as PEP 440 when it can (line 4). `VersionSpec` (line 31) is the range. A date-shaped string that is not valid PEP 440 compares as text (line 11). [`telemetry.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/telemetry.py) `telemetry_mode` (line 86) resolves native, propagation-only, or off. [`server/telemetry.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/telemetry.py) `seam_span` (line 164) opens a span named after the method.
>
> **Adapted.** `allow_call` is one closure. It returns `not authorized` when the closure is false. Version comparison here is dotted integers, then a string compare. `10` is newer than `2`. Unversioned sorts lowest, so a numbered copy wins. `Store` has `MemoryStore` and `FileStore`. The file is one JSON object. `traced_call` uses the `tracing` crate. `fingerprint` is SHA-256 of the name, a newline, and the schema text.
>
> **Deferred.** Scope and role checks, PEP 440 pre-releases, a Redis backend, and an OpenTelemetry exporter. FastMCP's spans are OpenTelemetry spans. This chapter records a `tracing` span and does not ship a collector. The response cache from chapter 16 is a different map.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **A refused call does not run.** `allow_call("secret", |_| false)` is `not authorized`. **Component versions are not protocol dates.** `highest_versions` keeps `10` over `2` and over an unversioned `add`. The protocol list from chapter 2 is untouched.

[personal note: a version string `10` that sorted before `2` because I compared the text]

## 23.1 Who may call, and which version

`allow_call` runs the closure and returns `Ok(())` or `not authorized`. `highest_versions` groups by name and keeps the greater version. Dotted numbers compare numerically. Anything else compares as a string. `None` is the oldest.

## 23.2 Remember, trace, fingerprint

`MemoryStore` is a `HashMap`. `FileStore::open` reads a JSON object, and `set` writes it back. `traced_call` enters an `info_span` named `tool` and runs the closure inside it. `fingerprint` updates a SHA-256 hasher and prints the digest as hex. `FINGERPRINT_ALGO` is `sha256`.

`tests/production.rs` refuses a call, keeps version `10`, round-trips memory and a file, and checks that the digest is 64 hex characters. Chapter 22's tests still pass in this copy. One hundred twenty-two tests passed: the previous 118, plus these four.

## 23.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch23-1-fastmcp-core
cargo run -p ch23-1-fastmcp-core --example demo
cargo bench -p ch23-1-fastmcp-core --bench production -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
allowed: add
version: 10
stored: ok
algo: sha256
digest: 12090edc3a45…
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
auth_1k                 time:   [333.08 ns 359.03 ns 389.67 ns]
store_get               time:   [19.948 ns 20.339 ns 21.180 ns]
trace_off               time:   [8.7018 µs 8.9927 µs 9.3928 µs]
trace_on                time:   [214.12 µs 216.51 µs 218.07 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `auth_1k` | 381.4 ns | 332.8 ns | 413.4 ns | < 5 µs |
| `store_get` | 19.94 ns | 19.83 ns | 20.19 ns | < 1 µs |
| `trace_off` | 8.701 µs | 8.615 µs | 8.863 µs | < 50 µs |
| `trace_on` | 213.4 µs | 212.9 µs | 216.9 µs | < 1 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `auth_1k` | 1,000 `allow_call` checks that return ok | Nothing else. The closure is `|name| name == "add"` | The name `add` | The check is a boolean. No token parse | Well under 5 µs. About 381 ns here. |
| `store_get` | One `get` on a `MemoryStore` | The `set` that stored `ok` | Key `banner` | No disk | Well under 1 µs. About 20 ns here. |
| `trace_off` | 1,000 allocations of the string `ok`, with no span | A subscriber | The string `ok` | No span and no collector | Well under 50 µs. About 8.7 µs here. |
| `trace_on` | 1,000 `traced_call` spans around the same string | Installing the subscriber | Tool name `add` | The subscriber writes to `std::io::sink`. No OTLP exporter | Well under 1 ms. About 213 µs here. The span costs more than the bare string. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `auth_1k` | pending | pending | pending | < 5 µs |
| `store_get` | pending | pending | pending | < 1 µs |
| `trace_off` | pending | pending | pending | < 50 µs |
| `trace_on` | pending | pending | pending | < 1 ms |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `fingerprint_algo` stub. `tests/ch23.rs` expects `"sha256"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch23-production.rs`](playground/ch23-production.rs) keeps the larger of versions 2 and 10 and prints four lines. The playground does not have this book's crate, and it does not hash. The crate demo also prints a 12-character prefix of the SHA-256 digest.

[Run the version pick on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=36c9e4a3eeeaae12f3f71c8f539ca581).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
allowed: add
version: 10
stored: ok
algo: sha256
```

## Recap

`allow_call` can refuse a name. `highest_versions` keeps the newest component version, and `10` beats `2`. `MemoryStore` and `FileStore` share a `Store` trait. `traced_call` enters a span. `fingerprint` is SHA-256.

`cargo run -p ch23-1-fastmcp-core --example demo` prints the five lines above. Locally, a thousand allows take 381.4 ns, one memory get takes 19.94 ns, a thousand bare strings take 8.701 µs, and the same thousand inside spans take 213.4 µs.

[personal note: the file store I pointed at a directory, and the write that failed because the path was not a file]

## What's next

Chapter 24 reads settings from the environment and a config file, then parses `run`, `dev`, `inspect`, and `install` without starting a server for inspect.
