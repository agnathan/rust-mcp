# Chapter 1: Speaking JSON-RPC

*Draft. Bracketed notes like [personal note] are where my own story goes.*

FastMCP's `mcp.run()` hides the bytes. Before using a protocol crate, I want to see one message: a JSON object with a version, a method, and sometimes an id. This chapter builds that object, parses it back, and answers two MCP methods, `initialize` and `ping`. It does not open a socket. Chapter 2 is the first server a client can start, and that server will use `rmcp` instead of these hand-rolled types.

| Section | Step | What it adds |
|---|---|---|
| 1.1 | `steps/ch01-1-jsonrpc` | JSON-RPC 2.0 requests, notifications, responses, and errors; MCP `initialize` and `ping` |

The code is one library crate, package `ch01-1-fastmcp-core`, library name `fastmcp_core`.

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612) (2026-10-08). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** A request is a message that has an id. [`server/low_level.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/low_level.py) runs `initialize` only when `ctx.method == "initialize"` and `ctx.request_id is not None`. Notifications, including `notifications/initialized`, are a different path, and `ping` is named there as a message with no interior FastMCP handler. This chapter's `Message` enum is that split: request, notification, response. `dispatch` returns `None` for a notification.
>
> **Adopted.** A failed call still has a wire shape. [`exceptions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/exceptions.py) `to_mcp_error` turns a Python exception into an `MCPError` whose code is `INVALID_PARAMS` or `INTERNAL_ERROR`. [`server/mixins/mcp_operations.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/mixins/mcp_operations.py) raises `MCPError(code=INVALID_PARAMS)` when a list cursor is bad. This chapter uses the JSON-RPC codes for parse errors, invalid messages, unknown methods, and bad `initialize` params. An unknown method is a response, not a panic.
>
> **Adapted.** FastMCP does not parse JSON-RPC itself. [`server/low_level.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/low_level.py) rebuilds `mcp_types.InitializeRequest` with `model_validate`, and [`_compat.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/_compat.py) still recognizes the wire names `protocolVersion` and `serverInfo` on `InitializeResult`. [`types.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/types.py) is not those models. It defines `Textarea` for tool UIs. The wire types live in the Python SDK. This chapter hand-rolls the same JSON with serde, camelCase on the wire, so the next chapter can switch to `rmcp` without the envelope being a mystery. [`server/context.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/context.py) sets `protocol_version="2025-06-18"` on a detached request context. This step answers with that same string and does not negotiate.
>
> **Deferred.** [`client/client.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/client.py) `initialize` is idempotent and can end in a `server/discover` session that has no `InitializeResult`. That handshake is chapter 2's server and chapter 12's client. [`server/middleware/ping.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/middleware/ping.py) `PingMiddleware` sends pings on a timer (`interval_ms`, default `30000`, and the value must be positive). Chapter 14 takes that loop. This chapter only answers an inbound `ping` with an empty object, which is what [`server/providers/proxy.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/proxy.py) returns from its `ping` handler (`EmptyResult`) and what `Client.ping` treats as success. The same proxy copies upstream `instructions` onto the initialize result. Composition does that in chapter 15. `to_mcp_error`'s map from `NotFoundError`, `DisabledError`, and `ValidationError` waits for chapter 8.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md). This chapter enforces three wire rules and leaves the rest.
>
> **A request has an id. A notification does not.** `parse_message` looks at which keys exist. `dispatch` writes a response only for a request. `notifications/initialized` produces nothing. That is W1 and W2 in the notes, from `server/low_level.py`.
>
> **One response, and it is either a result or an error.** A message with both `result` and `error` is an invalid request. `dispatch` of `initialize` returns one `Response`. FastMCP's root middleware goes further: if middleware raises after initialize already produced a result, it keeps the committed result so the client does not see two replies (W3, `server/low_level.py`). This chapter has no middleware, so that second half waits.
>
> **Bad input is a JSON-RPC error, not a crash.** Broken JSON is code `-32700`. A JSON value that is not a message is `-32600`. Unknown methods are `-32601` inside a normal response. `initialize` with missing params is `-32602`. The full `to_mcp_error` map (W4) is chapter 8. The ping timer (W5) is chapter 14. Discover-era initialize (W6) and proxy forwarding (W7) are later chapters.

[personal note: where I was with Rust when I started this chapter]

## What this chapter decides

- The envelope is hand-rolled. `rmcp` arrives in chapter 2.
- A message is one of three structs: request, notification, response. The parser chooses by keys, not by trying each shape until one fits.
- `ping` returns `{}`. `initialize` returns protocol version `2025-06-18`, empty capabilities, and server info `fastmcp-rs` `0.1.0`. No `instructions` field.
- An unknown method is a successful Rust `Result` whose JSON-RPC body is an error. A parse failure is a Rust error, because there is no id to answer yet.
- JSON arrays are rejected. This chapter does not frame stdin, and it does not implement JSON-RPC batches.

## 1.1 The crate

The workspace root `Cargo.toml` lists every step crate. `resolver = "3"` is the current cargo resolver. The package that matters today:

```bash
cargo test -p ch01-1-fastmcp-core
cargo bench -p ch01-1-fastmcp-core --bench jsonrpc
cargo run -p ch01-1-fastmcp-core --example demo
```

> **Rust sidebar: cargo**
>
> - Cargo builds, fetches dependencies, and runs tests. `cargo test -p NAME` tests one package. `cargo run -p NAME --example demo` runs `examples/demo.rs`. `cargo bench` runs the Criterion files under `benches/`.
> - `Cargo.toml` is the manifest. `[dependencies]` names crates from crates.io. This step depends on `serde` and `serde_json`.
> - A workspace is one lockfile and one `target/` directory for many packages. The root manifest sets `resolver = "3"`.
> - The package name (`ch01-1-fastmcp-core`) must be unique in the workspace. The library name (`fastmcp_core`) is what `use fastmcp_core::...` imports. `publish = false` keeps the step off crates.io. `rust-version = "1.85"` says which compiler the manifest expects.

### Structs for the three messages

A request asks for a response. A notification does not. A response is the reply. Those are three structs, and an enum that holds exactly one of them.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: JsonRpcVersion,
    pub id: RequestId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}
```

`Notification` is the same idea without `id`. `Response` has `id` (which may be JSON `null`), and at most one of `result` and `error`. `ErrorObject` is `code`, `message`, and an optional `data`. `Message` is the enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}
```

`ErrorCode` is the other enum this chapter needs. `ParseError` is `-32700`, `InvalidRequest` is `-32600`, `MethodNotFound` is `-32601`, `InvalidParams` is `-32602`, `InternalError` is `-32603`. Those numbers are the JSON-RPC 2.0 codes. FastMCP's `to_mcp_error` imports the params and internal names from the Python SDK; the integers are not written out in the FastMCP file I read.

> **Rust sidebar: structs and enums**
>
> - A struct groups named fields. `Request` is always a version, an id, a method, and maybe params.
> - An enum is one variant at a time. `Message::Request` and `Message::Notification` cannot both be true. `match` says what to do for each variant.
> - `#[derive(Debug, Clone, PartialEq, Eq)]` asks the compiler to write those pieces. `Debug` is what `{:?}` prints. `PartialEq` is what `assert_eq!` uses.
> - `Option` is an enum the standard library already defined: `Some(value)` or `None`. A missing `params` field is `None`.

MCP `initialize` adds two more structs. `#[serde(rename_all = "camelCase")]` turns `protocol_version` into `protocolVersion` and `client_info` into `clientInfo`. `InitializeResult` does the same for `serverInfo`. `instructions` is optional and is left off the JSON when it is `None`.

### Parsing without guessing

Serde can derive `Deserialize` for each struct. It should not be asked to guess the message kind. A request and a notification both have `method`. If the id failed to parse and we tried "does this fit a notification?" second, a broken request could be accepted as a notification. `parse_message` reads the keys first.

```rust
pub fn parse_message(bytes: &[u8]) -> Result<Message, ProtocolError> {
    let value: Value = serde_json::from_slice(bytes)?;
    message_from_value(value)
}
```

`serde_json::Value` is one JSON value: object, array, string, number, bool, or null. `from_slice` turns bytes into that value. The `?` on that line is the next sidebar. After the version string is `"2.0"`, the function branches:

- `method`, and no `result` or `error`, and an `id` that is not null: a request.
- `method`, and no `id`: a notification.
- no `method`, an `id`, and exactly one of `result` or `error`: a response.
- both `result` and `error`, or anything else: `InvalidRequest`.

A null `result` is rejected. MCP `ping` uses `{}`, not null. An error response may use `"id": null`, and that becomes `Option::None` on `Response.id`.

> **Rust sidebar: serde and serde_json**
>
> - `Serialize` writes a Rust value out as JSON. `Deserialize` reads JSON into a Rust value. `#[derive(Serialize, Deserialize)]` generates both.
> - `#[serde(rename = "2.0")]` stores `JsonRpcVersion::V2` in memory and writes the string `"2.0"`. `rename_all = "camelCase"` renames every field.
> - `#[serde(default, skip_serializing_if = "Option::is_none")]` accepts a missing field and omits it on the way out.
> - `serde_json::from_slice` and `to_vec` are the byte forms. `to_value` and `json!` build a `Value` in memory. `json!({})` is an empty object, which is the body of a ping result.

### `Result` and `?`

`parse_message` and `dispatch` return `Result<T, ProtocolError>`. The `Ok` side is the value. The `Err` side is why it failed. `ProtocolError` carries an `ErrorCode` and a message. `From<serde_json::Error>` turns a serde failure into that type: syntax and a cut-off buffer become a parse error, and other serde failures become an invalid request. That `From` impl is what lets `?` convert the error.

```rust
pub fn reply(bytes: &[u8]) -> Result<Option<Response>, ProtocolError> {
    let message = crate::jsonrpc::parse_message(bytes)?;
    dispatch(&message)
}
```

If `parse_message` returns `Err`, `reply` returns that same error and does not call `dispatch`. If parse succeeds, `?` unwraps the `Message` and continues. `dispatch` matches on the enum. A notification is `Ok(None)`. A response is `Err`, because dispatch only accepts something a peer sent us to handle. A request matches the method string: `ping`, `initialize`, or anything else.

`ping` builds `Response::success` with `json!({})`. `initialize` reads params into `InitializeParams`. Missing params, an empty protocol version, or an empty client name or version is `Err` with `InvalidParams`. A method this chapter does not know is `Ok(Some(error_response))` with code `-32601`. The JSON-RPC error and the Rust error are different. The first is a body we can send. The second means we could not build a body.

> **Rust sidebar: `Result` and `?`**
>
> - `Result<T, E>` is `Ok(T)` or `Err(E)`. Functions in this chapter use `E = ProtocolError`.
> - `?` after a call that returns `Result` does two things. On `Ok`, it gives you the inner value. On `Err`, it returns from the function you are in.
> - The function that uses `?` has to return a `Result` whose error type matches, or a type the error converts into with `From`.
> - `expect` and `unwrap` are for tests and for the demo, where a failure should stop the process. Library code in this chapter returns the `Result` instead.

`initialize_params` does not use `?` on the serde error, because that conversion would mark a bad object as `InvalidRequest`. Bad params are `InvalidParams`, so the function maps the error itself:

```rust
serde_json::from_value(params).map_err(|err| ProtocolError::invalid_params(err.to_string()))
```

### The tests

`#[test]` marks a function cargo should run. Tests next to the code live in a `mod tests` at the bottom of the file, behind `#[cfg(test)]`, so a normal build does not include them. Tests in `tests/handshake.rs` are integration tests: they can only call the public API, the same way another crate would.

> **Rust sidebar: `#[test]`**
>
> - `#[test]` goes on a function that takes no arguments and returns nothing (or a `Result`).
> - `cargo test` builds those functions and runs them. A panic, or a failed `assert_eq!`, fails the test.
> - `#[cfg(test)]` keeps the unit-test module out of the library you publish to dependents.
> - A file under `tests/` is a separate crate. It uses `fastmcp_core::parse_message`, not the private helpers.

The unit tests check a request round trip, a notification with no `id` key, a string id on an error response, `"id": null` on an error, broken JSON (`{`) as a parse error, version `"1.0"` and a JSON array as invalid requests, both `result` and `error` rejected, and a null id on a request rejected. The MCP tests check ping's empty object, initialize's `serverInfo` and camelCase wire names, silence for `notifications/initialized`, an unknown method as a response, missing initialize params as `InvalidParams`, and `reply` on `not-json` as a parse error.

The integration test runs the handshake in order: initialize, `notifications/initialized`, ping. A second test covers `tools/call` (unknown here) and an initialize object that has a protocol version but no client info. A third parses 1,000 initialize requests and checks each id. That third test is the same size as the parse bench, but it is a correctness check, not a timer.

### Build, run, and benchmark

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

Nextest, after it finished compiling the tests:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.31s
────────────
 Nextest run ID 302e1e52-cb38-49a0-a285-6f429a9ad0c5 with nextest profile: default
    Starting 19 tests across 2 binaries
        PASS [   0.003s] ( 1/19) ch01-1-fastmcp-core jsonrpc::tests::both_result_and_error_is_rejected
        PASS [   0.003s] ( 2/19) ch01-1-fastmcp-core jsonrpc::tests::array_is_an_invalid_request
        PASS [   0.004s] ( 3/19) ch01-1-fastmcp-core jsonrpc::tests::broken_json_is_a_parse_error
        PASS [   0.004s] ( 4/19) ch01-1-fastmcp-core jsonrpc::tests::notification_has_no_id_field
        PASS [   0.003s] ( 5/19) ch01-1-fastmcp-core jsonrpc::tests::null_id_error_response_is_allowed
        PASS [   0.004s] ( 6/19) ch01-1-fastmcp-core jsonrpc::tests::null_request_id_is_rejected
        PASS [   0.004s] ( 7/19) ch01-1-fastmcp-core jsonrpc::tests::request_round_trips
        PASS [   0.004s] ( 8/19) ch01-1-fastmcp-core jsonrpc::tests::string_id_and_error_response_round_trip
        PASS [   0.003s] ( 9/19) ch01-1-fastmcp-core jsonrpc::tests::wrong_version_is_an_invalid_request
        PASS [   0.003s] (10/19) ch01-1-fastmcp-core mcp::tests::initialize_returns_server_info
        PASS [   0.003s] (11/19) ch01-1-fastmcp-core mcp::tests::initialize_without_params_is_invalid_params
        PASS [   0.005s] (12/19) ch01-1-fastmcp-core mcp::tests::handshake_bytes_round_trip_through_reply
        PASS [   0.004s] (13/19) ch01-1-fastmcp-core mcp::tests::initialized_notification_is_silent
        PASS [   0.004s] (14/19) ch01-1-fastmcp-core mcp::tests::reply_uses_question_mark_on_bad_json
        PASS [   0.005s] (15/19) ch01-1-fastmcp-core mcp::tests::ping_returns_an_empty_object
        PASS [   0.005s] (16/19) ch01-1-fastmcp-core mcp::tests::unknown_method_is_a_response_not_a_rust_error
        PASS [   0.005s] (17/19) ch01-1-fastmcp-core::handshake handshake_initialize_then_notification_then_ping
        PASS [   0.003s] (18/19) ch01-1-fastmcp-core::handshake unknown_method_and_bad_initialize_params
        PASS [   0.021s] (19/19) ch01-1-fastmcp-core::handshake parse_one_thousand_initialize_requests
────────────
     Summary [   0.037s] 19 tests run: 19 passed, 0 skipped
```

The demo:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/examples/demo`
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"clientInfo":{"name":"demo","version":"0.1.0"},"protocolVersion":"2025-06-18"}}
{"jsonrpc":"2.0","id":1,"result":{"capabilities":{},"protocolVersion":"2025-06-18","serverInfo":{"name":"fastmcp-rs","version":"0.1.0"}}}
```

Criterion prints a slope on the `time:` line. That slope is not the number in the table. The table is the median point estimate from `target/criterion/<name>/new/estimates.json` on this same run. The first optimized build on this machine compiled the bench dependencies, which is the 16.73 s in the log.

```text
    Finished `bench` profile [optimized] target(s) in 16.73s
     Running benches/jsonrpc.rs (target/release/deps/jsonrpc-7119a1c6f1cdd032)
Gnuplot not found, using plotters backend
Benchmarking parse_1_message
Benchmarking parse_1_message: Warming up for 1.0000 s
Benchmarking parse_1_message: Collecting 10 samples in estimated 2.0000 s (1.7M iterations)
Benchmarking parse_1_message: Analyzing
parse_1_message         time:   [1.1739 µs 1.1891 µs 1.2033 µs]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) low mild

Benchmarking parse_1000_messages
Benchmarking parse_1000_messages: Warming up for 1.0000 s
Benchmarking parse_1000_messages: Collecting 10 samples in estimated 2.0169 s (1595 iterations)
Benchmarking parse_1000_messages: Analyzing
parse_1000_messages     time:   [1.2917 ms 1.3082 ms 1.3263 ms]
Found 3 outliers among 10 measurements (30.00%)
  1 (10.00%) low severe
  1 (10.00%) low mild
  1 (10.00%) high severe

Benchmarking serialize_1_message
Benchmarking serialize_1_message: Warming up for 1.0000 s
Benchmarking serialize_1_message: Collecting 10 samples in estimated 2.0000 s (11M iterations)
Benchmarking serialize_1_message: Analyzing
serialize_1_message     time:   [179.49 ns 181.79 ns 183.81 ns]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild

Benchmarking serialize_1000_messages
Benchmarking serialize_1000_messages: Warming up for 1.0000 s
Benchmarking serialize_1000_messages: Collecting 10 samples in estimated 2.0091 s (11k iterations)
Benchmarking serialize_1000_messages: Analyzing
serialize_1000_messages time:   [177.14 µs 178.81 µs 180.63 µs]

Benchmarking dispatch_initialize
Benchmarking dispatch_initialize: Warming up for 1.0000 s
Benchmarking dispatch_initialize: Collecting 10 samples in estimated 2.0000 s (2.7M iterations)
Benchmarking dispatch_initialize: Analyzing
dispatch_initialize     time:   [738.41 ns 743.92 ns 751.87 ns]
```

Local medians from that run's `estimates.json` (the slope above is not this column):

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

## Pitfalls

- Classifying a message by "try each struct" will accept a request with a bad id as a notification, because a notification ignores an `id` field. Check the keys, then deserialize the struct you already chose.
- `"id": null` is legal on an error response when the id could not be read. It is not legal on a request.
- A response with both `result` and `error` is invalid even if both values would otherwise deserialize.
- `Option` turns JSON `null` into `None`. This chapter rejects a null result instead of treating it as an empty object.
- A JSON array is an invalid request here. Newline framing of stdio is chapter 2.
- `tools/list` is an unknown method. The error response is the point. The tool registry is chapter 3.

## Chapter summary

The crate can parse and emit a JSON-RPC 2.0 message, and it can answer `initialize` and `ping`. Requests have ids. Notifications do not, and they get no response. Errors use the standard codes. On the Ubuntu CI run linked above, the medians are 1.558 µs to parse one message, 1.569 ms to parse 1,000, 249.3 ns to serialize one, 249.8 µs to serialize 1,000, and 863.7 ns to dispatch `initialize`. All five sit under the targets.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 2 starts a process. A small server speaks stdio, answers `initialize` and `tools/list`, and you can point the MCP Inspector at it. That chapter is the first one that needs async, `tokio`, and `rmcp`. The types in this chapter stay as the picture of the wire. The running server will not keep parsing by hand.
