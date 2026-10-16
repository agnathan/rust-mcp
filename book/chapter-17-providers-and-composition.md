# Chapter 17: Providers and composition

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A server that only knows the tools you typed into `main` gets awkward the moment a second service shows up. FastMCP treats a tool list as something a provider can answer. One provider is local. Another joins several lists. A mount puts a prefix on every name so two `status` tools can live side by side. A proxy forwards the call, forwards the ping, and keeps the upstream instructions when it has none of its own.

A directory of files is a provider too. Scan it, and the tools appear without a second registration call.

## What you'll build

Two mounts, one proxy, and the instructions that came from upstream.

```text
tools: api_status, local_status
ping: ok
instructions: be brief
```

The snapshot is `steps/ch17-1-providers`. It is a full copy of chapter 16, plus `provider.rs`. The crates are `ch17-1-fastmcp-core` and `ch17-1-fastmcp-macros`.

```bash
cargo run -p ch17-1-fastmcp-core --example demo
```

## Goals

- Hold a tool list in a `LocalProvider` and call a name that is present.
- Join providers in an aggregate that asks in order and keeps the first hit.
- Prefix every name with `{namespace}_` and strip that prefix on the call.
- Forward ping through a proxy, and copy instructions only when the proxy has none.
- Read `*.tool` files from a directory, in sorted order, and skip a name that starts with `_`.

## Roadmap

**17.1 Local, aggregate, and mount.** The problem is two tools with the same name. The mount makes the public name `api_status`. The aggregate lists both.

**17.2 A proxy and a directory.** The problem is a ping that dies at the proxy, and a folder of tools nobody registered. The proxy forwards the ping and copies instructions. `discover_tools` reads the files.

**17.3 Build, run, and benchmark.** The tests, the demo above, a direct call, the same call through a proxy, and a list across ten providers.

| Section | Step | What it adds |
|---|---|---|
| 17.1–17.3 | `steps/ch17-1-providers` | `Provider`, `AggregateProvider`, `MountedProvider`, `ProxyProvider`, `discover_tools` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/providers/base.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/base.py) `Provider` (line 77) is the base. `get_*` returns `None` to mean “I don't have it.” Providers are queried in registration order, and the first non-`None` wins. [`aggregate.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/aggregate.py) `AggregateProvider` (line 57) combines providers. A namespace is a transform on the wrapped provider, so `add_provider(api_provider, namespace="api")` exposes `api_foo`. [`local_provider/local_provider.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/local_provider/local_provider.py) `LocalProvider` (line 51) holds components registered in this process. [`proxy.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/proxy.py) `ProxyProvider` (line 852) fetches from a client factory. `FastMCPProxy` (line 1438) installs `_setup_proxy_ping_handler` (line 1496). `ping_remote` (line 1497) awaits `client.ping()` and returns `EmptyResult` (line 1504). `ProxyInitializeMiddleware` (line 263) copies upstream `instructions` onto the result when the proxy's own instructions are `None` (line 323). [`filesystem.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/filesystem.py) `FileSystemProvider` (line 50) scans a directory. [`filesystem_discovery.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/filesystem_discovery.py) `discover_files` (line 35) returns `[]` when the root is missing (line 47) and sorts the paths it finds.
>
> **Adapted.** `call` returns `Option<String>`. `None` means this provider does not have the name. The aggregate walks in order and keeps the first `Some`. FastMCP queries `get_*` in parallel and returns the highest version. The mount is its own provider. FastMCP applies a `Namespace` transform instead. Discovery reads `*.tool` files whose first line is the tool name. FastMCP imports Python modules and registers the objects it finds. The proxy here is an `Arc` to another provider in this process. FastMCP opens a client. The default list cache of 300 seconds (`_DEFAULT_CACHE_TTL`, line 849) is not in this proxy.
>
> **Deferred.** A remote client factory, the 300-second list cache, and `task_config.mode="forbidden"` on every proxied component (line 858). Tasks cannot run through a FastMCP proxy. The task protocol is chapter 20. Version-aware lookup is chapter 23. OpenAPI as a provider is chapter 21.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **W7.** `ProxyProvider::ping` calls the inner provider and returns `Ok(())` when that call succeeds. `instructions` returns the proxy's own text when it has one, and the upstream text when it does not.
>
> **A mount does not answer the bare name.** `api_status` reaches the inner `status` tool. `status` on that mount is `None`.

[personal note: two servers that both exported `status`, and the call that hit the wrong one]

## 17.1 Local, aggregate, and mount

`Provider` is a trait. `list_tools` returns the public names. `call` returns `None` when the name is absent, so the next provider can try.

```rust
fn call(&self, name: &str) -> Option<String> {
    self.parts.iter().find_map(|part| part.call(name))
}
```

`MountedProvider` lists `{namespace}_{name}`. On a call it strips that prefix and asks the inner provider. A name without the prefix is `None`.

`tests/providers.rs` checks a local list, an aggregate that keeps the first `status` body, and a mount that answers `api_status` and refuses `status`. Chapter 16's tests still pass in this copy. One hundred tests passed: the previous 94, plus one unit test and these five.

## 17.2 A proxy and a directory

```rust
fn instructions(&self) -> Option<String> {
    self.instructions
        .clone()
        .or_else(|| self.inner.instructions())
}
```

`ping` forwards and then returns `Ok(())`. That is the empty success the wire uses for ping. A proxy that sets its own instructions keeps them.

`discover_tools` returns an empty list when the directory is missing. It sorts directory entries, keeps files that end in `.tool`, and skips a file whose name starts with `_`. The first line is the tool name. The rest is the body.

## 17.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch17-1-fastmcp-core
cargo run -p ch17-1-fastmcp-core --example demo
cargo bench -p ch17-1-fastmcp-core --bench providers -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
tools: api_status, local_status
ping: ok
instructions: be brief
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
direct_call             time:   [15.789 ns 16.025 ns 16.397 ns]
proxy_call              time:   [17.563 ns 17.594 ns 17.640 ns]
aggregate_10            time:   [446.81 ns 448.33 ns 450.10 ns]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `direct_call` | 15.91 ns | 15.74 ns | 16.11 ns | < 1 µs |
| `proxy_call` | 17.58 ns | 17.56 ns | 17.63 ns | < 1 µs |
| `aggregate_10` | 448.1 ns | 446.0 ns | 450.0 ns | < 5 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `direct_call` | One `call("status")` on a `LocalProvider` | Building the provider | One tool, body `ok` | No socket and no proxy | Well under 1 µs. About 16 ns here. |
| `proxy_call` | One `call("status")` through a `ProxyProvider` | Building the inner provider and the proxy | The same tool | No remote client and no list cache | Well under 1 µs. About 18 ns here. A proxy hop costs a little more than the direct call. |
| `aggregate_10` | `list_tools` across ten providers | Building the ten providers | Each provider has one tool | No socket. The list is not filtered | Well under 5 µs. About 448 ns here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `direct_call` | pending | pending | pending | < 1 µs |
| `proxy_call` | pending | pending | pending | < 1 µs |
| `aggregate_10` | pending | pending | pending | < 5 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `mounted_tool_name` stub. `tests/ch17.rs` expects `"api_status"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch17-providers.rs`](playground/ch17-providers.rs) prints the demo lines. The playground does not have this book's crate.

[Run the prefix on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=59ce5490407ca4a744c4db8ddfff79f2).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
tools: api_status, local_status
ping: ok
instructions: be brief
```

## Recap

A provider lists names and calls one. An aggregate asks in order. A mount prefixes `{namespace}_` and strips it on the way in. A proxy forwards the call and the ping, and copies instructions when it has none. `discover_tools` reads sorted `*.tool` files and skips a leading underscore.

`cargo run -p ch17-1-fastmcp-core --example demo` prints the three lines above. Locally, a direct call takes 15.91 ns, the same call through a proxy takes 17.58 ns, and a list across ten providers takes 448.1 ns.

[personal note: the instructions field I dropped on the proxy, and the model that lost the upstream hint]

## What's next

Chapter 18 changes what a list returns without rewriting the provider. A namespace can sit on the catalog. A rename can change one argument. Tags can hide a tool. A prompt or a resource can show up as a tool. A change to the list sets a flag the client can hear.
