# Chapter 1: Speaking JSON-RPC

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Picture a client that wants to call a tool on your server. Before it can name the tool, the two programs have to agree they are speaking the same language. That language, on the wire, is JSON-RPC 2.0: one JSON object, a version, a method name, and, when a reply is expected, an id.

FastMCP's `mcp.run()` sends and receives those objects for you. This book starts one layer under that call, because every later chapter is a conversation made of the same objects. Tools, schemas, and HTTP are easier once you have held a single message in your hands and watched a reply come back.

You are at the beginning of the server. There is no socket in this chapter, and no client process. Chapter 2 is the first program another process can start, and that program will hand the bytes to the Rust MCP SDK, `rmcp`. FastMCP 4 also speaks a newer protocol revision, `2026-07-28`, which changes how a session begins. Chapter 2 negotiates versions. This chapter answers `initialize` with one fixed string, `2025-06-18`, so the success path fits on a screen.

## What you'll build

A small library that reads one JSON-RPC message and, when the message expects a reply, writes one back. Give it an `initialize` request and it answers with the server's name.

```text
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"clientInfo":{"name":"demo","version":"0.1.0"},"protocolVersion":"2025-06-18"}}
{"jsonrpc":"2.0","id":1,"result":{"capabilities":{},"protocolVersion":"2025-06-18","serverInfo":{"name":"fastmcp-rs","version":"0.1.0"}}}
```

The first line is the request. The second is the response: protocol version `2025-06-18`, empty capabilities, and server info `fastmcp-rs` `0.1.0`. A `ping` comes back as an empty object, `{}`. A notification comes back as silence.

The code lives in one library crate. The package name is `ch01-1-fastmcp-core` and the library name, the name you import, is `fastmcp_core`. The finished snapshot is `steps/ch01-1-jsonrpc`. From the repo root:

```bash
cargo run -p ch01-1-fastmcp-core --example demo
```

## Goals

- Read a JSON-RPC 2.0 object and say whether it is a request, a notification, or a response.
- Explain why a request carries an id, why a notification does not, and what the server sends back in each case.
- Answer the MCP methods `initialize` and `ping`, and turn broken JSON and unknown methods into the standard error codes.
- Use Cargo, a struct, an enum, serde, `Result`, and `?` on a real message.
- Run the chapter's tests and read a Criterion median, including the difference between that median and the slope Criterion prints.

## Roadmap

**1.1 A message is one of three things.** We name the three shapes, because the rest of the parser depends on knowing which one arrived.

**1.2 Look at the keys, then parse.** A broken request must stay a broken request. Guessing the shape by trying each struct in turn would hide that failure.

**1.3 Answer `initialize` and `ping`.** The server writes a response for a request, stays quiet for a notification, and keeps a Rust error separate from an error that has an id and can travel back to the client.

**1.4 Build, run, and benchmark.** The tests, the demo above, and the timings, so the numbers later in the book have a first measurement to sit beside.

| Section | Step | What it adds |
|---|---|---|
| 1.1–1.4 | `steps/ch01-1-jsonrpc` | JSON-RPC 2.0 requests, notifications, responses, and errors; MCP `initialize` and `ping` |

The Python this chapter is copying, and the wire rules it keeps, come next. The Rust starts in 1.1.

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612) (2026-10-08). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** A request is a message that has an id. [`server/low_level.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/low_level.py) runs `initialize` only when `ctx.method == "initialize"` and `ctx.request_id is not None`. Notifications, including `notifications/initialized`, are a different path, and `ping` is named there as a message with no interior FastMCP handler. This chapter's `Message` enum is that split: request, notification, response. `dispatch` returns `None` for a notification.
>
> **Adopted.** A failed call still has a wire shape. [`exceptions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/exceptions.py) `to_mcp_error` turns a Python exception into an `MCPError` whose code is `INVALID_PARAMS` or `INTERNAL_ERROR`. [`server/mixins/mcp_operations.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/mixins/mcp_operations.py) raises `MCPError(code=INVALID_PARAMS)` when a list cursor is bad. This chapter uses the JSON-RPC codes for parse errors, invalid messages, unknown methods, and bad `initialize` params. An unknown method is a response.
>
> **Adapted.** FastMCP does not parse JSON-RPC itself. [`server/low_level.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/low_level.py) rebuilds `mcp_types.InitializeRequest` with `model_validate`, and [`_compat.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/_compat.py) still recognizes the wire names `protocolVersion` and `serverInfo` on `InitializeResult`. [`types.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/types.py) is the module for tool-UI types such as `Textarea`. The wire types live in the Python SDK. This chapter hand-rolls the same JSON with serde, camelCase on the wire, so the next chapter can switch to `rmcp` with the envelope already familiar. [`server/context.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/context.py) sets `protocol_version="2025-06-18"` on a detached request context. This step answers with that same string and does not negotiate.
>
> **Deferred.** [`client/client.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/client.py) `initialize` is idempotent and can end in a `server/discover` session that has no `InitializeResult`. That handshake is chapter 2's server and chapter 14's client. [`server/middleware/ping.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/ping.py) `PingMiddleware` sends pings on a timer (`interval_ms`, default `30000`, and the value must be positive). Chapter 16 takes that loop. This chapter only answers an inbound `ping` with an empty object, which is what [`server/providers/proxy.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/proxy.py) returns from its `ping` handler (`EmptyResult`) and what `Client.ping` treats as success. The same proxy copies upstream `instructions` onto the initialize result. Composition does that in chapter 17. `to_mcp_error`'s map from `NotFoundError`, `DisabledError`, and `ValidationError` waits for chapter 8.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md). This chapter enforces three wire rules and leaves the rest.
>
> **A request has an id. A notification does not.** `parse_message` looks at which keys exist. `dispatch` writes a response only for a request. `notifications/initialized` produces nothing. That is W1 and W2 in the notes, from `server/low_level.py`.
>
> **One response, and it is either a result or an error.** A message with both `result` and `error` is an invalid request. `dispatch` of `initialize` returns one `Response`. FastMCP's root middleware goes further: if middleware raises after initialize already produced a result, it keeps the committed result so the client does not see two replies (W3, `server/low_level.py`). This chapter has no middleware, so that second half waits.
>
> **Bad input is a JSON-RPC error, and the process stays up.** Broken JSON is code `-32700`. A JSON value that is not a message is `-32600`. Unknown methods are `-32601` inside a normal response. `initialize` with missing params is `-32602`. The full `to_mcp_error` map (W4) is chapter 8. The ping timer (W5) is chapter 16. Discover-era initialize (W6) and proxy forwarding (W7) are later chapters.

[personal note: where I was with Rust when I started this chapter]

## 1.1 A message is one of three things

A client can send three kinds of JSON object, and they ask for three different reactions. A request wants a response. A notification is a fire-and-forget, the way `notifications/initialized` tells the server "I heard you" and then expects silence. A response is the reply itself, which this chapter's server receives only so it can refuse to treat it as a new question.

Those are three structs. A struct is a group of named fields that travel together. `Request` is the one to look at, because the other two are the same idea with a field added or removed. The full definitions are in `steps/ch01-1-jsonrpc/crates/fastmcp-core/src/jsonrpc.rs`.

```rust
pub struct Request {
    pub jsonrpc: JsonRpcVersion,
    pub id: RequestId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}
```

`jsonrpc` is the version. This chapter accepts the string `"2.0"` and stores it as the enum value `JsonRpcVersion::V2`. An enum is a type with a fixed set of variants, and a value is exactly one of them. `RequestId` is the other small enum: an id is either a number or a string, because JSON-RPC allows both.

`method` is the name, `"initialize"` or `"ping"`. `params` might be missing. `Option` is the standard library's enum for that situation: `Some(value)` when the field is there, `None` when it is absent. The `serde` attribute on the line above says a missing field becomes `None`, and a `None` is left off the JSON on the way back out.

`Notification` is this struct without `id`. `Response` has an id that may be JSON `null` (stored as `Option`), plus at most one of `result` and `error`. `ErrorObject` carries a numeric `code`, a `message`, and an optional `data`.

The parser has to hand you exactly one of the three. That is another enum:

```rust
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}
```

`Message::Request` and `Message::Notification` are different values. A `match` later in the chapter says what to do for each. The `untagged` attribute matters in section 1.2: serde will not look for an extra tag field in the JSON. We decide the variant ourselves, from the keys.

`#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` on these types asks the compiler to write the boring pieces. `Debug` is what `{:?}` prints. `Clone` makes a copy. `PartialEq` is what `assert_eq!` uses in the tests. `Serialize` and `Deserialize` belong to serde, which is the next section.

The error codes are an enum too, and the numbers are the JSON-RPC 2.0 codes. `ParseError` is `-32700`. `InvalidRequest` is `-32600`. `MethodNotFound` is `-32601`. `InvalidParams` is `-32602`. `InternalError` is `-32603`. FastMCP's `to_mcp_error` imports the params and internal names from the Python SDK. The integers themselves are not written out in the FastMCP file this chapter read. They live here so a test can name them.

MCP `initialize` adds two more structs, in `src/mcp.rs`. The wire uses camelCase (`protocolVersion`, `clientInfo`) and the Rust fields use snake_case. `#[serde(rename_all = "camelCase")]` is the whole translation:

```rust
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: Value,
    pub client_info: Implementation,
}
```

`InitializeResult` does the same for `serverInfo`. An optional `instructions` string is left off the JSON when it is `None`. This chapter's result leaves it off.

One trap is worth naming now, before the parser. `"id": null` is legal on an error response, for the case where the id of a failed request could not be read. It is illegal on a request. A request with a null id is an invalid request, and the tests keep it that way.

## 1.2 Look at the keys, then parse

Serde can fill in a struct from JSON. The trouble is asking it to guess which struct. A request and a notification both have `method`. If you try "does this fit a request?" and, on failure, "does this fit a notification?", a request with a bad id can fall through and get accepted as a notification. The client would wait for a reply that never comes. `parse_message` refuses that guess. It reads the keys first, then deserializes the struct it has already chosen.

```rust
pub fn parse_message(bytes: &[u8]) -> Result<Message, ProtocolError> {
    let value: Value = serde_json::from_slice(bytes)?;
    message_from_value(value)
}
```

`serde_json::Value` is one JSON value: an object, an array, a string, a number, a bool, or null. `from_slice` turns bytes into that value. The `?` on that line returns early when the bytes are not JSON at all. Section 1.3 explains `?` on the function that callers actually use. `message_from_value` is the rest of the function, in the same file. After it has checked that the version string is `"2.0"`, it branches on keys:

- `method` is present, `result` and `error` are absent, and `id` is present and not null: a request.
- `method` is present and `id` is absent: a notification.
- `method` is absent, `id` is present, and exactly one of `result` or `error` is present: a response.
- Both `result` and `error`, a JSON array, a version other than `"2.0"`, or anything else: `InvalidRequest` (`-32600`).

A null `result` is rejected. MCP `ping` uses `{}`, an empty object. An error response may use `"id": null`, and that becomes `None` on `Response.id`.

Serde is the crate doing the translation, and `serde_json` is the crate that speaks JSON specifically. `Serialize` writes a Rust value out as JSON. `Deserialize` reads JSON into a Rust value. The derive on `Request` generates both. `#[serde(rename = "2.0")]` on `JsonRpcVersion::V2` stores the enum in memory and writes the string `"2.0"`. `serde_json::to_vec` is the byte form of the other direction. `json!({})` builds an empty object in memory, which is the body of a ping result.

A JSON array is an invalid request in this chapter. JSON-RPC batches exist in the wild. This server answers one object at a time, and newline framing of a stdio stream waits until chapter 2.

## 1.3 Answer `initialize` and `ping`

Parsing gives you a `Message`. Something still has to decide what to send back. That function is `reply`, and it is the one to read twice, because it is where Rust's `?` earns its place. It lives in `src/mcp.rs`.

```rust
pub fn reply(bytes: &[u8]) -> Result<Option<Response>, ProtocolError> {
    let message = crate::jsonrpc::parse_message(bytes)?;
    dispatch(&message)
}
```

`Result<T, E>` is the standard library's enum for an operation that can fail. `Ok(T)` is the value. `Err(E)` is why it failed. Here `E` is `ProtocolError`, which carries an `ErrorCode` and a message. `From<serde_json::Error>` turns a serde failure into that type: a syntax error or a cut-off buffer becomes a parse error (`-32700`), and other serde failures become an invalid request. That conversion is what lets `?` change the error type on the way out.

`?` after a call that returns `Result` does two things. On `Ok`, it unwraps the inner value and the function continues. On `Err`, it returns from the function you are in. If `parse_message` fails, `reply` returns that error and never calls `dispatch`. The function that uses `?` has to return a `Result` whose error matches, or a type the error converts into. `expect` and `unwrap` are for tests and for the demo, where a failure should stop the process. Library code in this chapter returns the `Result`.

`dispatch` matches on the enum. A notification is `Ok(None)`: success, and nothing to write. A response is an error, because dispatch only accepts something a peer sent us to handle. A request matches the method string:

```rust
match request.method.as_str() {
    "ping" => Ok(Response::success(request.id.clone(), json!({}))),
    "initialize" => initialize_response(request),
    other => Ok(Response::failure(
        Some(request.id.clone()),
        ErrorObject::from_code(
            ErrorCode::MethodNotFound,
            format!("Method not found: {other}"),
        ),
    )),
}
```

`ping` returns `{}`. `initialize` reads params into `InitializeParams`. Missing params, an empty protocol version, or an empty client name or version is `Err` with `InvalidParams` (`-32602`). The success body is protocol version `2025-06-18`, empty capabilities, and server info `fastmcp-rs` `0.1.0`. There is no `instructions` field.

A method this chapter does not know, including `tools/call`, is `Ok(Some(error_response))` with code `-32601`. Read that twice. The JSON-RPC error and the Rust error are different kinds of failure. The JSON-RPC error is a body we can send, because we still have an id. The Rust error means we could not build a body at all, which is what happens when the bytes are not JSON, or when `initialize` params are missing. The tool registry that would make `tools/call` a real method is chapter 3. The error response is the point of this chapter.

Bad `initialize` params take a careful path. Mapping a serde error through the usual `From` would mark a bad object as `InvalidRequest`. Bad params are `InvalidParams`, so `initialize_params` maps the error itself:

```rust
serde_json::from_value(params).map_err(|err| ProtocolError::invalid_params(err.to_string()))
```

The tests sit next to this code and in `tests/handshake.rs`. `#[test]` marks a function Cargo should run. A panic, or a failed `assert_eq!`, fails the test. Tests in a `mod tests` at the bottom of the file are behind `#[cfg(test)]`, so a normal build leaves them out. A file under `tests/` is a separate crate. It can call `fastmcp_core::parse_message` and the other public functions, the same way another crate would, and it cannot see the private helpers.

The unit tests check a request round trip, a notification with no `id` key, a string id on an error response, `"id": null` on an error, broken JSON (`{`) as a parse error, version `"1.0"` and a JSON array as invalid requests, both `result` and `error` rejected, and a null id on a request rejected. The MCP tests check ping's empty object, initialize's `serverInfo` and camelCase wire names, silence for `notifications/initialized`, an unknown method as a response, missing initialize params as `InvalidParams`, and `reply` on `not-json` as a parse error.

The integration test runs the handshake in order: initialize, `notifications/initialized`, ping. A second test covers `tools/call` (unknown here) and an initialize object that has a protocol version and no client info. A third parses 1,000 initialize requests and checks each id. That third test is the same size as the parse bench. It checks correctness. The bench, next, checks time.

## 1.4 Build, run, and benchmark

Cargo is the build tool. It fetches dependencies, compiles, and runs tests. `cargo test -p NAME` tests one package. `cargo run -p NAME --example demo` runs `examples/demo.rs`. `cargo bench` runs the Criterion files under `benches/`. `Cargo.toml` is the manifest. This step depends on `serde` and `serde_json`.

The workspace root `Cargo.toml` lists every step crate and sets `resolver = "3"`, the current Cargo feature resolver. A workspace is one lockfile and one `target/` directory for many packages. The package name `ch01-1-fastmcp-core` has to be unique in the workspace. The library name `fastmcp_core` is what `use fastmcp_core::...` imports. `publish = false` keeps the step off crates.io. `rust-version = "1.85"` says which compiler the manifest expects.

From the repository root. Nothing in this step reads an API key or opens a socket.

```bash
cargo build -p ch01-1-fastmcp-core
cargo nextest run -p ch01-1-fastmcp-core
cargo run -p ch01-1-fastmcp-core --example demo
cargo bench -p ch01-1-fastmcp-core --bench jsonrpc -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

I ran those commands on a 4-vCPU Intel Xeon guest under KVM, with 16 GiB of RAM, Linux, rustc 1.99.1, and cargo-nextest 0.9.148. The library rebuild finished with:

```text
   Compiling ch01-1-fastmcp-core v0.1.0 (/workspace/steps/ch01-1-jsonrpc/crates/fastmcp-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
```

Nextest prints a `PASS` line per test. The line that matters is the summary, from run `302e1e52-cb38-49a0-a285-6f429a9ad0c5`:

```text
     Summary [   0.037s] 19 tests run: 19 passed, 0 skipped
```

The demo prints one initialize request and the success response:

```text
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"clientInfo":{"name":"demo","version":"0.1.0"},"protocolVersion":"2025-06-18"}}
{"jsonrpc":"2.0","id":1,"result":{"capabilities":{},"protocolVersion":"2025-06-18","serverInfo":{"name":"fastmcp-rs","version":"0.1.0"}}}
```

Criterion prints a slope on the `time:` line. That slope is a different statistic from the median in the table below. The table is `median.point_estimate` from `target/criterion/<name>/new/estimates.json` on this same run. The first optimized build on this machine compiled the bench dependencies, which took 16.73 s. The slope lines:

```text
parse_1_message         time:   [1.1739 µs 1.1891 µs 1.2033 µs]
parse_1000_messages     time:   [1.2917 ms 1.3082 ms 1.3263 ms]
serialize_1_message     time:   [179.49 ns 181.79 ns 183.81 ns]
serialize_1000_messages time:   [177.14 µs 178.81 µs 180.63 µs]
dispatch_initialize     time:   [738.41 ns 743.92 ns 751.87 ns]
```

Local medians from that run's `estimates.json` (the slope above is a different column):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `parse_1_message` | 1.191 µs | 1.149 µs | 1.207 µs | < 1 ms |
| `parse_1000_messages` | 1.294 ms | 1.271 ms | 1.312 ms | < 50 ms |
| `serialize_1_message` | 179.5 ns | 178.7 ns | 181.6 ns | < 1 ms |
| `serialize_1000_messages` | 178.9 µs | 176.8 µs | 181.7 µs | < 50 ms |
| `dispatch_initialize` | 748.8 ns | 741.2 ns | 780.3 ns | < 1 ms |

**Where the timer starts and stops.** Each bench builds its input before `b.iter`. The timed region is only the closure passed to `iter`. `black_box` keeps the compiler from deleting the result.

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `parse_1_message` | `parse_message` on one buffer | Building the request and encoding the fixture | One `initialize`, id `1`, `protocolVersion` `2025-06-18`, empty capabilities, client `bench` `0.1.0`. The fixture is 154 bytes. The same buffer is reused. | No socket, no sleep, no `rmcp` session | Well under 1 ms. About a microsecond on this machine. |
| `parse_1000_messages` | A loop of 1,000 `parse_message` calls and a count | Building the 1,000 buffers | Ids `0` through `999`, otherwise the same object. One buffer per message, not one JSON array. | The buffers are already in memory. No stdin framing. | Under 50 ms. About 1.3 ms here. |
| `serialize_1_message` | `serde_json::to_vec` of one `Message`, including the output allocation | Building the struct | The same initialize request, already a `Message` | No socket | Well under 1 ms. About 180 ns here. |
| `serialize_1000_messages` | `to_vec` of each of 1,000 messages, plus summing the lengths | Building the 1,000 structs | Ids `0` through `999` | No batching into one array | Under 50 ms. About 179 µs here. |
| `dispatch_initialize` | `dispatch`: match the method, parse params, build `InitializeResult`, `to_value` it | Parsing bytes | One already-parsed initialize, id `1` | Server name `fastmcp-rs`, version `0.1.0`, empty capabilities, no session and no middleware | Well under 1 ms. About 750 ns here. |

CI runs this step's tests on Ubuntu, macOS, and Windows. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38843114413](https://github.com/agnathan/rust-mcp/actions/runs/38843114413) (2026-10-16T02:58:15Z). GitHub's `github.sha` for that pull request run is the merge commit `7e42523ddcf9765dbb7fc2e0350dc9f250db6025`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `parse_1_message` | 1.558 µs | 1.556 µs | 1.561 µs | < 1 ms |
| `parse_1000_messages` | 1.569 ms | 1.567 ms | 1.574 ms | < 50 ms |
| `serialize_1_message` | 249.3 ns | 249.2 ns | 249.7 ns | < 1 ms |
| `serialize_1000_messages` | 249.8 µs | 249.0 µs | 250.1 µs | < 50 ms |
| `dispatch_initialize` | 863.7 ns | 863.1 ns | 865.1 ns | < 1 ms |

On the Ubuntu runner, one message is still well under a millisecond, and 1,000 messages are 1.569 ms to parse and 249.8 µs to serialize. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

The example is the demo. It prints one initialize request and the response.

```bash
cargo run -p ch01-1-fastmcp-core --example demo
```

`fastmcp-rs/` is the crate you implement. `tests/ch01.rs` calls `ping_response_json`, which is still `todo!()`. CI does not run that test. The snapshot under `steps/ch01-1-jsonrpc` is the reference.

### Rust Playground

[`book/playground/ch01-speaking-json-rpc.rs`](playground/ch01-speaking-json-rpc.rs) is the same lesson in one file: parse `initialize`, stay silent for `notifications/initialized`, answer `ping`, and reject `{` as a parse error. It uses `serde` and `serde_json`, which are on the [playground crate list](https://play.rust-lang.org/meta/crates) (serde 1.0.229 and serde_json 1.0.151 when this chapter was checked). Criterion is not on that list, so the file has no bench.

[Run it on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=dbfb903c747272f8d8383595f50aa03b).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
{"jsonrpc":"2.0","id":1,"result":{"capabilities":{},"protocolVersion":"2025-06-18","serverInfo":{"name":"fastmcp-rs","version":"0.1.0"}}}
(no response)
{"jsonrpc":"2.0","id":"ping-1","result":{}}
parse error -32700 (EOF while parsing an object at line 1 column 1)
```

## Recap

You can take a JSON-RPC 2.0 object and say which of the three shapes it is, because the parser looks at the keys before it builds a struct. A request has an id and gets one response. A notification, including `notifications/initialized`, gets silence. `ping` returns `{}`. `initialize` returns protocol version `2025-06-18`, empty capabilities, and server info `fastmcp-rs` `0.1.0`.

Broken JSON is `-32700`. A value that is not a message is `-32600`. An unknown method is a response whose code is `-32601`, which is a Rust `Ok` because the id is still there. Bad `initialize` params are `-32602`, a Rust `Err`, because the body could not be built. You have used Cargo, structs, enums, serde, `Result`, and `?` on that path.

`cargo run -p ch01-1-fastmcp-core --example demo` prints the two lines from the start of the chapter. On the Ubuntu CI run linked above, the medians are 1.558 µs to parse one message, 1.569 ms to parse 1,000, 249.3 ns to serialize one, 249.8 µs to serialize 1,000, and 863.7 ns to dispatch `initialize`. All five sit under their targets. The slope on Criterion's `time:` line is a different number. The table is the median.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 2 starts a process. A small server speaks stdio, chooses a protocol version from a fixed list, and answers `initialize` and `tools/list`. You can point the MCP Inspector at it. That chapter is the first one that needs async, Tokio, and `rmcp`. The types in this chapter stay as the picture of the wire. The running server will let the SDK parse.
