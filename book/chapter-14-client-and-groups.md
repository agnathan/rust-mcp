# Chapter 14: The client and client groups

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

The server has had the pipe to itself. A client is the other end. It has to decide what a string means before it opens anything: a script, an HTTP URL, the older SSE URL, an in-memory server, or a config that names several servers.

One client talks to one server. A group holds several clients and lists their tools together. Each public name is the server name, an underscore, and the tool name. Two servers that would publish the same public name are a collision, and the list stops.

## What you'll build

A memory client, a ping, and the tools from two servers.

```text
kind: memory
ping: ok
tools: gadgets_status, widgets_status
```

The snapshot is `steps/ch14-1-client`. It is a full copy of chapter 13, plus `client.rs`. The crates are `ch14-1-fastmcp-core` and `ch14-1-fastmcp-macros`.

```bash
cargo run -p ch14-1-fastmcp-core --example demo
```

## Goals

- Classify a string as stdio, HTTP, SSE, memory, or config.
- Connect to a server over an in-memory duplex and ping it.
- See that a discover session has no initialize result, and that a handshake session returns the stored version on every call.
- List eight servers at once with `JoinSet`, and reject a colliding public name.

## Roadmap

**14.1 Infer, then connect.** The problem is a string that could be any transport. Classification is separate from opening the pipe.

**14.2 A group of clients.** The problem is ten tools that share one short name. The group prefixes and checks for a collision.

**14.3 Build, run, and benchmark.** The tests, the demo above, and two list timings.

| Section | Step | What it adds |
|---|---|---|
| 14.1–14.3 | `steps/ch14-1-client` | `infer_transport`, `McpClient`, `ClientGroup` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`client/transports/inference.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/transports/inference.py) `infer_transport` (line 88) picks a transport from the value it is given. An HTTP URL is Streamable HTTP unless it ends in `/sse`, which is the older SSE transport (the branch near line 177). [`client/client.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/client.py) stores that transport at line 463. `initialize` (line 893) is idempotent: the docstring at line 900 says a second call returns the first result. A discover-era session has no `InitializeResult`, and the docstring at line 907 says this method raises in that case. `ping` (line 1399) treats an `EmptyResult` as success (line 1402). [`client/group.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/group.py) `ClientGroup.__init__` (line 46) rejects an empty mapping. `list_tools` (line 183) gathers every client. The public name is `f"{server_name}_{tool.name}"` (line 206). A name already in the route map raises `ValueError` (line 208).
>
> **Adapted.** This chapter's `infer_transport` takes a `&str`. The word `memory` selects the in-memory duplex. FastMCP selects memory when the value is a server object, which a string cannot be. A `.py` or `.js` suffix is stdio, and this chapter does not spawn that process. SSE is a kind, and this chapter does not open an SSE socket. `ClientGroup::list_tools` uses `tokio::task::JoinSet` for the fan-out. `initialize` returns the protocol version stored when the handshake client connected. A second call returns that same value.
>
> **Deferred.** A real child process, a Streamable HTTP client (the server side is chapter 12), an SSE connection, and a config that opens every `mcpServers` entry. The ping *timer* on the server is chapter 16. This chapter only sends one ping.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **W6.** `ping` is success when the server answers with an empty result. `initialize` on a handshake client returns the stored version every time. `initialize` on a discover client returns an error: that session has no `InitializeResult`.
>
> **A group needs one client.** An empty list is an error before any list runs.
>
> **A public name is unique.** `widgets_status` and `gadgets_status` can sit together. Two servers both named `one` collide on `one_status`.

[personal note: two servers that both exported `search`, and the call that hit the wrong one]

## 14.1 Infer, then connect

`infer_transport` looks at the string and returns a `TransportKind`. It does not open a socket and it does not spawn a process.

```rust
if input.starts_with("http://") || input.starts_with("https://") {
    if input.ends_with("/sse") {
        return Ok(TransportKind::Sse);
    }
    return Ok(TransportKind::Http);
}
```

`memory` is the kind the demo connects. `McpClient::connect_memory` builds a `tokio::io::duplex`, serves `FastMcp` on one end, and runs the rmcp client on the other. The handshake version is stored. `initialize` returns that store. Calling it again returns the same version, because the method does not send a second handshake.

`connect_discover` uses the discover lifecycle from chapter 2. `initialize` on that client returns `ClientError::NoInitializeResult`.

`ping` sends a ping request. The match is `ServerResult::EmptyResult`. That arm is `true`. Anything else is `false`.

A URL that ends in `/sse` is the older transport from chapter 12's `create_sse_app`. This client records the kind and leaves the socket closed.

## 14.2 A group of clients

`ClientGroup::new` rejects an empty vec. `list_tools` clones each peer and spawns a task on a `JoinSet`. `JoinSet` is a set of tasks you can wait on as they finish, in whatever order they finish. Each task lists one server and prefixes every tool.

```rust
names.push(format!("{name}_{}", tool.name));
```

After the set is drained, a `HashSet` records the public names. A name that is already present is `ClientError::Collision`. The surviving names are sorted, so the demo line is stable.

The server name in the prefix is the name you gave the group, which can differ from the name inside `FastMcp::new`. The demo uses `widgets` and `gadgets` for both.

## 14.3 Build, run, and benchmark

`tests/client.rs` checks ping, the stored handshake, a discover client with no initialize result, the prefixed list, a collision, an empty group, and the SSE and stdio kinds. Chapter 13's tests still pass in this copy. Eighty-four tests passed: the previous 77, plus one unit test and these six.

```bash
cargo test -p ch14-1-fastmcp-core
cargo run -p ch14-1-fastmcp-core --example demo
cargo bench -p ch14-1-fastmcp-core --bench client -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
kind: memory
ping: ok
tools: gadgets_status, widgets_status
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
list_once               time:   [35.841 µs 36.166 µs 36.488 µs]
fanout_8                time:   [77.701 µs 78.327 µs 79.316 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `list_once` | 36.40 µs | 35.53 µs | 36.56 µs | < 200 µs |
| `fanout_8` | 78.98 µs | 77.76 µs | 79.79 µs | < 500 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `list_once` | One `tools/list` on an already connected memory client, and collecting the names | The duplex connect and the handshake | One server. The only tool is `status`. | No socket, no process, no HTTP | Well under 200 µs. About 36 µs here. |
| `fanout_8` | Eight `tools/list` calls on a `JoinSet`, the prefixes, the collision check, and the sort | Connecting the eight servers | Eight servers named `s0` through `s7`. Each has `status`. | No process and no HTTP. The peers are already up. | Well under 500 µs. About 79 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI median below is filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `list_once` | pending | pending | pending | < 200 µs |
| `fanout_8` | pending | pending | pending | < 500 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI median once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `grouped_tool_name` stub. `tests/ch14.rs` expects `"widgets_status"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch14-client.rs`](playground/ch14-client.rs) classifies three strings. The duplex client needs tokio and rmcp, which the playground does not have.

[Run the classifier on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ff2b7bf27ef513c1ab1129699fda3260).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
kind: memory
http: http
sse: sse
tools: gadgets_status, widgets_status
```

## Recap

`infer_transport` names the kind and opens nothing. `connect_memory` handshakes on a duplex, `ping` wants an empty result, and `initialize` returns the stored version. A discover client has nothing to return. `ClientGroup` lists with a `JoinSet` and prefixes each tool. An empty group and a colliding name are errors.

`cargo run -p ch14-1-fastmcp-core --example demo` prints the three lines above. Locally, one list is 36.40 µs and eight servers are 78.98 µs.

[personal note: the SSE URL I sent to a Streamable HTTP client, and the handshake that never started]

## What's next

Chapter 15 is how you test this without a process: an in-memory transport you can call from `cargo test`, and snapshots of the messages those tests see.
