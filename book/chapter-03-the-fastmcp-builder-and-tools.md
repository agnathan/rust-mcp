# Chapter 3: The `FastMcp` builder and tools

*Draft. Bracketed notes like [personal note] are where my own story goes.*

Chapter 2 can negotiate a protocol version and answer `tools/list` with one tool, `status`. It cannot register a function you wrote, and `tools/call` falls through to `rmcp`'s default, which is method-not-found. This chapter keeps the version choice and the stdio server, and adds a registry. `FastMcp::new("fastmcp-rs").tool(...)` stores a closure. `tools/call` looks the name up and runs it.

The arguments are typed. The closure takes a struct, and serde fills that struct from the JSON object before the closure runs. The JSON Schema for those fields is still an empty object. Chapter 4 derives the schema from the struct.

| Section | Step | What it adds |
|---|---|---|
| 3.1 | `steps/ch03-1-builder` | `FastMcp::tool`, a `HashMap` of closures, `tools/call` |

The package is `ch03-1-fastmcp-core`. The library name is still `fastmcp_core`. It is a full copy of chapter 2 plus this chapter's registry, so the earlier snapshots still build on their own.

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
> **An unknown tool name is `invalid_params`, and it does not panic.** `dispatch_tool` returns `ToolError::Unknown`. `call_tool` turns that into `ErrorData::invalid_params`. FastMCP's `to_mcp_error` maps `NotFoundError` to `INVALID_PARAMS` (W4). `rmcp`'s tool router uses the same code for a missing name. The full exception map is still chapter 8.
>
> **Arguments that do not match `P` never enter the closure.** serde fails first, and that is `ToolError::InvalidParams`, also sent as `invalid_params`. That is the validation half of W4.
>
> **A closure that returns `Err` is a tool result the client can read.** `call_tool` sends `CallToolResult::error`, with `is_error: true`. It does not use a JSON-RPC error for that path. `rmcp` documents the split: a tool failure belongs in the result, a protocol failure belongs in `Err`.
>
> **`status` stays, and it is not a map entry.** `tools/list` puts it first. `tools/call` with the name `status` returns the era note. Real tools are the names you passed to `.tool()`.

[personal note: the first closure I stored, and the moment the map's type stopped compiling]

## What this chapter decides

- `FastMcp::tool` takes `self` by value and returns `Self`. Calls chain: `FastMcp::new("fastmcp-rs").tool(...).tool(...)`.
- The map key is the tool name. The value is the description, an empty object schema, and the closure.
- The closure is `Fn(P) -> Result<Value, String>`. `P` is any type serde can deserialize. `Ok` becomes structured tool content. `Err(String)` becomes a tool error.
- `status` is still listed, ahead of the registered names, which are sorted.
- The lookup bench is `HashMap::contains_key` on 1,000 names. The dispatch bench calls all 1,000. Neither opens a socket.

## 3.1 Registering a closure

`Add` is the argument struct. The field names are the JSON keys.

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

> **Rust: a closure.** `|args: Add| { ... }` is a function with no name. The compiler gives it a unique type. You can pass it to `tool` because `tool` is generic over that type. The body can return `Result`. `Ok` is the tool's value. `Err("nope")` is the tool's failure message.

`tool` cannot store `F` in the `HashMap`. Every closure is a different `F`, and a `HashMap` wants one value type.

```rust
pub fn tool<P, F>(mut self, name: impl Into<String>, description: impl Into<String>, call: F) -> Self
where
    P: DeserializeOwned + Send + 'static,
    F: Fn(P) -> Result<Value, String> + Send + Sync + 'static,
```

> **Rust: a generic function.** `<P, F>` are filled in at the call. `P` comes from the closure's argument (`Add`). `F` is the closure's own type. `DeserializeOwned` means serde can build a `P` from JSON without borrowing the input. `Send + Sync` means another thread may hold the closure and call it. `'static` means the closure does not borrow a local that would disappear when `tool` returns.

`register` is the line that erases `P` and `F`:

```rust
call: Arc::new(move |value| {
    let params = serde_json::from_value::<P>(value)
        .map_err(|err| ToolError::InvalidParams(err.to_string()))?;
    call(params).map_err(ToolError::Failed)
}),
```

> **Rust: `Arc<dyn Fn>`.** `dyn Fn(Value) -> Result<Value, ToolError>` is one type that can point at any closure with that signature. The `P` you wrote is hidden inside the wrapper. `Arc` is a shared pointer: cloning the server clones the pointer, not the closure. `rmcp` clones the handler onto the connection, so `Box` (one owner) would not be enough. `Box<dyn Fn>` is the same erasure with a single owner. This server needs the shared one.

> **Rust: `HashMap`.** `HashMap<String, RegisteredTool>` looks a name up without walking the whole list. `insert` replaces a key that is already there. `contains_key` is the lookup bench. `get` is the dispatch path. Listing sorts the keys so `tools/list` is stable. A `HashMap` does not keep insertion order.

> **Rust: the builder.** `new` returns a server with an empty map. `tool` takes that server by value, inserts one entry, and returns the server. The next `.tool()` is a method call on the value that came back. Nothing is borrowed across the chain.

`call_tool` handles `status` before the map. Every other name goes through `dispatch_tool`. `Unknown` and `InvalidParams` are `ErrorData::invalid_params`. `Failed` is `CallToolResult::error`. `Ok(Value)` is `CallToolResult::structured`.

### The tests

Unit tests call `dispatch_tool` with no socket. `add` of 2 and 3 is `5`. A missing name is `ToolError::Unknown`. `{"left": 1}` on `Add` is `InvalidParams`, and the closure does not run. A closure that returns `Err("nope")` is `ToolError::Failed`. Registering `add` twice leaves the second closure and the second description.

`tests/tools.rs` uses the same `tokio::io::duplex` as chapter 2.

- `tools/list` names are `status`, then `add`, then `fail`.
- `tools/call` of `add` with `left: 2` and `right: 3` returns structured content `5` and `is_error: false`.
- `tools/call` of `missing` is a protocol error, code `INVALID_PARAMS` (`-32602`), and the message contains `unknown tool`.
- `tools/call` of `fail` is a normal result with `is_error: true`.
- `tools/call` of `status` returns the handshake sentence.

Chapter 2's session tests still pass. Thirty-five tests passed: 24 in the library, 3 in `tests/handshake.rs`, 3 in `tests/session.rs`, 5 in `tests/tools.rs`.

### Build, run, and benchmark

```bash
cargo test -p ch03-1-fastmcp-core
cargo run -p ch03-1-fastmcp-core --example demo
cargo bench -p ch03-1-fastmcp-core --bench tools -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

`cargo test` printed `ok` for all 35 tests. The demo prints:

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

Local medians from that run's `estimates.json` (the slope above is not this column):

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

## Pitfalls

- Storing `F` in the `HashMap` fails as soon as you register a second closure. The wrapper closure, the one that takes `Value`, is the type the map holds.
- A missing JSON field is `invalid_params` before your function runs. Logging inside the closure will not show that call.
- Returning `Err` from the closure is a tool error (`is_error: true`). Returning `Err(ErrorData)` from `call_tool` is a protocol error. Unknown names and bad JSON take the protocol path. The closure's own `Err` takes the tool path.
- `status` is listed and callable, and `has_tool("status")` is false. The bench that says 1,000 tools is counting `tool-0` through `tool-999`.

## Chapter summary

`FastMcp::new("fastmcp-rs").tool("add", "...", |args: Add| ...)` puts a closure in a `HashMap`. `tools/call` deserializes the arguments, runs the closure, and returns either structured content or a tool error. An unknown name is `invalid_params`. `status` still reports the protocol era, beside the tools you registered.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 4 builds the JSON Schema from `Add` and from the value the tool returns. An `Option` field is the one the client may leave out.
