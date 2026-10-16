# Chapter 3: The `FastMcp` builder and tools

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Chapter 2 can negotiate a protocol version and answer `tools/list` with one tool, `status`, whose description is the era of the connection. That tool is a signpost. It is not a function you wrote. Ask the server to call anything else and `rmcp`'s default answer is method-not-found. The Python server from the introduction, the one that registers `add` and returns `a + b`, is still out of reach.

This chapter keeps the version choice and the stdio server, and adds the registry FastMCP is known for. You hand `FastMcp` a name, a description, and a function. `tools/call` looks the name up and runs the function. The arguments are typed: the function takes a struct, and serde fills that struct from the JSON object before the function runs. The JSON Schema for those fields is still an empty object. Chapter 4 derives the schema from the struct, so a client can see which fields are required. Here the job is to store the function and call it.

## What you'll build

A server you assemble by chaining `.tool(...)`, and a call that adds two integers.

```text
add(2, 3) = 5
has add: true
has status in the map: false
```

`add` is in the registry. `status` is still listed and still callable, and it is not one of the names you registered. `tools/list` puts `status` first, then your tools in sorted order. An unknown name comes back as a protocol error. A function that returns a failure comes back as a tool result the client can read, with `is_error: true`.

The package is `ch03-1-fastmcp-core`. The library name is still `fastmcp_core`. The snapshot is `steps/ch03-1-builder`, a full copy of chapter 2 plus the registry, so the earlier snapshots still build on their own.

```bash
cargo run -p ch03-1-fastmcp-core --example demo
```

## Goals

- Register a function with `FastMcp::new("fastmcp-rs").tool(...)` and call it by name.
- Explain why a `HashMap` of tools stores `Arc<dyn Fn>` once a second function shows up.
- Deserialize arguments into a struct before the function runs, and keep a bad object out of the function.
- Tell a tool error (`is_error: true`) from a protocol error (`invalid_params`).
- Read the lookup and dispatch benchmarks, and know that neither one opens a socket.

## Roadmap

**3.1 Put a function in the registry.** The `add` example, the builder that returns `Self`, and the wrapper that lets two different functions share one map. This is the storage problem.

**3.2 Call it, and leave `status` beside it.** `tools/call` for a known name, an unknown name, bad JSON, and a function that fails on purpose. `status` still reports the era, and it stays out of the map.

**3.3 Build, run, and benchmark.** The tests, the demo above, and the timings for one lookup among 1,000 names and for calling all 1,000.

| Section | Step | What it adds |
|---|---|---|
| 3.1–3.3 | `steps/ch03-1-builder` | `FastMcp::tool`, a `HashMap` of closures, `tools/call` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/server.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/server.py) `FastMCP.tool` (the method that starts near line 1787) delegates to `self._local_provider.add_tool`. [`server/providers/local_provider/decorators/tools.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/local_provider/decorators/tools.py) `add_tool` turns a callable into a `Tool` with `Tool.from_function` and stores it. [`server/providers/local_provider/local_provider.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/local_provider/local_provider.py) keeps `_components: dict[str, FastMCPComponent]` and `_add_component` writes `self._components[component.key] = component`. This chapter's `HashMap<String, RegisteredTool>` is that dict for tools only.
>
> **Adapted.** [`tools/function_tool.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/tools/function_tool.py) `FunctionTool.from_function` holds the Python function on the tool object. Rust closures each have their own type, so the map stores `Arc<dyn Fn>`. Arguments are a serde struct, which is the stand-in for the Pydantic model FastMCP builds from the function signature. A second `.tool()` with the same name replaces the first. FastMCP's `on_duplicate` branch in `add_tool` can ignore the new tool instead; that policy waits.
>
> **Deferred.** The `@tool` decorator is chapter 5. JSON Schema from the parameter type is chapter 4. `Context` and injected dependencies are chapters 9 and 10. `task=true` is chapter 20. The `timeout` and `auth` parameters on `FastMCP.tool` wait with those chapters.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **An unknown tool name is `invalid_params`, and the process stays up.** `dispatch_tool` returns `ToolError::Unknown`. `call_tool` turns that into `ErrorData::invalid_params`. FastMCP's `to_mcp_error` maps `NotFoundError` to `INVALID_PARAMS` (W4). `rmcp`'s tool router uses the same code for a missing name. The full exception map is still chapter 8.
>
> **Arguments that do not match `P` never enter the closure.** serde fails first, and that is `ToolError::InvalidParams`, also sent as `invalid_params`. That is the validation half of W4.
>
> **A closure that returns `Err` is a tool result the client can read.** `call_tool` sends `CallToolResult::error`, with `is_error: true`. It does not use a JSON-RPC error for that path. `rmcp` documents the split: a tool failure belongs in the result, a protocol failure belongs in `Err`.
>
> **`status` stays, and it is not a map entry.** `tools/list` puts it first. `tools/call` with the name `status` returns the era note. Real tools are the names you passed to `.tool()`.

[personal note: the first closure I stored, and the moment the map's type stopped compiling]

## 3.1 Put a function in the registry

Start with the call you want to be able to write. `Add` is the argument struct. Its field names are the JSON keys. The closure, the `|args: Add| { ... }` expression, is a function with no name. The compiler gives each closure its own unique type.

```rust
#[derive(Deserialize)]
struct Add {
    left: i64,
    right: i64,
}

let server = FastMcp::new("fastmcp-rs").tool("add", "add two integers", |args: Add| {
    Ok(Value::from(args.left + args.right))
});
```

`Ok` is the tool's value. `Err("nope")` would be the tool's failure message. `tool` can accept this closure because the method is generic: the type parameters `P` and `F` are filled in at the call, `P` from the argument (`Add`) and `F` from the closure itself.

`FastMcp::new` returns a server with an empty map. `tool` takes that server by value, inserts one entry, and returns the server. The next `.tool()` is a method call on the value that came back, so the calls chain: `FastMcp::new("fastmcp-rs").tool(...).tool(...)`. Nothing is borrowed across the chain.

The signature is the contract. It lives on `FastMcp::tool` in `steps/ch03-1-builder/crates/fastmcp-core/src/server.rs`.

```rust
pub fn tool<P, F>(mut self, name: impl Into<String>, description: impl Into<String>, call: F) -> Self
where
    P: DeserializeOwned + Send + 'static,
    F: Fn(P) -> Result<Value, String> + Send + Sync + 'static,
```

`DeserializeOwned` means serde can build a `P` from JSON without borrowing the input buffer. `Send + Sync` means another thread may hold the closure and call it. `'static` means the closure does not borrow a local variable that would disappear when `tool` returns. `Fn(P) -> Result<Value, String>` is the shape every tool in this chapter has: arguments in, a JSON value or an error string out.

A `HashMap<String, RegisteredTool>` looks a name up without walking a list. `insert` replaces a key that is already there, so a second `.tool("add", ...)` keeps the second closure and the second description. `contains_key` is the lookup bench. `get` is the dispatch path. Listing sorts the keys so `tools/list` is stable. A `HashMap` does not remember insertion order, which is why the sort is there.

Here is the wall you hit if you try to store `F` directly. The first closure has one type. The second closure, even with the same argument struct, has another. A `HashMap` wants one value type. `register`, in `src/tools.rs`, erases `P` and `F` behind a wrapper that takes a JSON `Value`:

```rust
call: Arc::new(move |value| {
    let params = serde_json::from_value::<P>(value)
        .map_err(|err| ToolError::InvalidParams(err.to_string()))?;
    call(params).map_err(ToolError::Failed)
}),
```

`dyn Fn(Value) -> Result<Value, ToolError>` is one type that can point at any closure with that signature. The `P` you wrote is hidden inside the wrapper. `Arc` is a shared pointer: cloning the server clones the pointer, not the closure. `rmcp` clones the handler onto the connection, so a `Box` (one owner) would not be enough. `Box<dyn Fn>` is the same erasure with a single owner. This server needs the shared one.

A missing JSON field fails on the `from_value` line, before `call(params)`. Logging inside the closure will not show that request. The closure only runs after `P` exists.

## 3.2 Call it, and leave `status` beside it

`call_tool` handles `status` before it touches the map. The name `status` returns the era note from chapter 2, as text. Every other name goes through `dispatch_tool`.

`Unknown` and `InvalidParams` become `ErrorData::invalid_params`, a protocol error with code `-32602`. The message for a missing name contains `unknown tool`. `Failed`, the closure's own `Err`, becomes `CallToolResult::error`, a normal result with `is_error: true`. `Ok(Value)` becomes `CallToolResult::structured`.

That split is the one to keep straight. Returning `Err` from the closure is a tool error the client reads inside a successful JSON-RPC response. Returning `Err(ErrorData)` from `call_tool` is a protocol error. Unknown names and bad JSON take the protocol path. The closure's own `Err` takes the tool path. `rmcp` draws the same line: a tool failure belongs in the result, a protocol failure belongs in `Err`.

`has_tool("status")` is false. `has_tool("add")` is true. `tools/list` still shows `status` first, then the registered names, sorted. The bench that says 1,000 tools is counting `tool-0` through `tool-999`. `status` is not in that loop.

The schema stored next to each closure is an empty object. A client listing tools learns the name and the description, and it does not yet learn that `left` and `right` are required. Chapter 4 fills that document in.

## 3.3 Build, run, and benchmark

Unit tests call `dispatch_tool` with no socket. `add` of 2 and 3 is `5`. A missing name is `ToolError::Unknown`. `{"left": 1}` on `Add` is `InvalidParams`, and the closure does not run. A closure that returns `Err("nope")` is `ToolError::Failed`. Registering `add` twice leaves the second closure and the second description.

`tests/tools.rs` uses the same `tokio::io::duplex` as chapter 2. `tools/list` names `status`, then `add`, then `fail`. `tools/call` of `add` with `left: 2` and `right: 3` returns structured content `5` and `is_error: false`. `tools/call` of `missing` is a protocol error, code `INVALID_PARAMS` (`-32602`), and the message contains `unknown tool`. `tools/call` of `fail` is a normal result with `is_error: true`. `tools/call` of `status` returns the handshake sentence.

Chapter 2's session tests still pass.

```bash
cargo test -p ch03-1-fastmcp-core
cargo run -p ch03-1-fastmcp-core --example demo
cargo bench -p ch03-1-fastmcp-core --bench tools -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

`cargo test` printed `ok` for all 35 tests: 24 in the library, 3 in `tests/handshake.rs`, 3 in `tests/session.rs`, 5 in `tests/tools.rs`. The demo prints:

```text
add(2, 3) = 5
has add: true
has status in the map: false
```

`examples/stdio.rs` registers the same `add` tool and waits on stdin. The demo above is the command that exits.

The bench command's slope, on the same 4-vCPU guest as chapter 1:

```text
lookup_1                time:   [12.424 ns 12.505 ns 12.605 ns]
dispatch_1000           time:   [59.413 µs 59.734 µs 60.355 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `lookup_1` | 12.51 ns | 12.34 ns | 12.64 ns | < 1 µs |
| `dispatch_1000` | 59.96 µs | 59.47 µs | 61.34 µs | < 1 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `lookup_1` | One `HashMap::contains_key` for `tool-500` | Building the server and the 1,000 closures | A server with tools `tool-0` through `tool-999`. The name is `tool-500`. | No JSON, no socket, no `tools/call` | Well under a microsecond. About 13 ns here. |
| `dispatch_1000` | 1,000 lookups, a clone of the argument JSON, serde into `Echo`, and the closure that returns `n` | Building the server, the name list, and the argument JSON | Tools `tool-0` through `tool-999`. Each call gets `{"n": 1}`. | No socket and no `rmcp` session. `status` is not in this loop. | Under a millisecond. About 60 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38856477535](https://github.com/agnathan/rust-mcp/actions/runs/38856477535) (2026-10-16T05:34:43Z). GitHub's `github.sha` for that pull request run is the merge commit `8159c536d796b488fab17233fbfcf7cb4877a717`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `lookup_1` | 17.14 ns | 17.13 ns | 17.16 ns | < 1 µs |
| `dispatch_1000` | 82.97 µs | 82.81 µs | 83.08 µs | < 1 ms |

On the Ubuntu runner, one lookup among 1,000 tools is 17.14 ns, and dispatching all 1,000 is 82.97 µs. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows a `registered_tool_name` stub. `tests/ch03.rs` expects `"add"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch03-the-fastmcp-builder-and-tools.rs`](playground/ch03-the-fastmcp-builder-and-tools.rs) is the `HashMap`, the closure, and `Arc<dyn Fn>`, with `i64` arguments. The [playground crate list](https://play.rust-lang.org/meta/crates) does not include `rmcp`, so the stdio server and the JSON struct stay in the crate.

[Run the registry on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=915630a0f407bcf05cb790ab103c8ed6).

## Recap

`FastMcp::new("fastmcp-rs").tool("add", "...", |args: Add| ...)` puts a closure in a `HashMap`. Two closures can share that map because each one is wrapped in `Arc<dyn Fn>` after serde has built the argument struct. `tools/call` runs the closure and returns structured content, or a tool error when the closure returns `Err`. An unknown name, and JSON that does not match the struct, are `invalid_params`, and the closure does not run. `status` still reports the protocol era. `has_tool("status")` stays false.

`cargo run -p ch03-1-fastmcp-core --example demo` prints `add(2, 3) = 5`. On the Ubuntu run linked above, one lookup among 1,000 tools is 17.14 ns, and dispatching all 1,000 is 82.97 µs.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 4 builds the JSON Schema from `Add` and from the value the tool returns. A client listing tools will see that `left` and `right` are required. An `Option` field is the one the client may leave out.
