# Chapter 13: Elicitation and interactive tools

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Some tools cannot finish from the arguments alone. "Ship it?" needs a person. The tool asks, then it waits. Another task accepts with a string, or it declines, or it cancels. The tool's result is that decision.

The wait is a `oneshot` channel: one sender, one receiver, one answer. The phase of that wait is an enum, so the desk is always in exactly one state.

## What you'll build

One question, one answer, and the phase after the answer lands.

```text
ask: Ship it?
answer: yes
phase: accepted
```

The snapshot is `steps/ch13-1-elicit`. It is a full copy of chapter 12, plus `elicit.rs`. The crates are `ch13-1-fastmcp-core` and `ch13-1-fastmcp-macros`.

```bash
cargo run -p ch13-1-fastmcp-core --example demo
```

## Goals

- Ask on a `Desk` and answer from a second task with `submit`, `decline`, or `cancel`.
- Read the phase: idle, waiting, accepted, declined, cancelled.
- Reject an elicitation schema that nests an object.
- Ask a second time after the first answer, on a new channel.

## Roadmap

**13.1 The desk.** The problem is a tool that needs a person. `ask` stores a `oneshot` sender and waits.

**13.2 A flat schema.** The problem is a form the client cannot render. The schema is an object of primitive fields.

**13.3 Build, run, and benchmark.** The tests, the demo above, and one round-trip timing.

| Section | Step | What it adds |
|---|---|---|
| 13.1–13.3 | `steps/ch13-1-elicit` | `Desk`, `Phase`, a flat elicitation schema |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/elicitation.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/elicitation.py) `get_elicitation_schema` (line 400) builds a JSON schema for the answer type. `validate_elicitation_json_schema` (line 426) requires an object schema whose fields are primitive: string, number, integer, or boolean. A nested object is a `TypeError` (line 536). `handle_elicit_accept` (line 337) validates the content the client sent back. [`client/elicitation.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/elicitation.py) `ElicitationHandler` (line 26) is the callback that receives the message and returns the answer.
>
> **Adapted.** This chapter's answer travels on a `tokio::sync::oneshot` inside the process. `Phase` names the state. A second `ask` after the first one settles opens a new channel, which is the second round. The schema helper checks the object type, nested objects, arrays of objects, primitives, `enum`, and `const`. FastMCP also walks `$ref` and `oneOf`. Those branches stay in the Python function.
>
> **Deferred.** [`tools/function_tool.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/tools/function_tool.py) `FunctionTool.run` (line 411) returns an `InputRequiredResult` for SEP-2322, and the client comes back later with `input_responses`. [`server/context.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/context.py) `input_responses` (line 378) is that later round. The wire request, URL elicitation, and the sealed `request_state` wait. `rmcp`'s `Peer::elicit` needs the `elicitation` feature and a client handler. This chapter answers in-process so the bench has no socket.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **One answer closes the channel.** A second value needs a second `ask`. The first sender is already used.
>
> **Decline is a tool result with `is_error`.** The call finished. The person said no. That is different from a protocol error, which chapter 8 kept on the error object.
>
> **The schema stays flat.** A nested object is rejected before anyone is asked.

[personal note: a prompt I shipped that asked for a nested object, and the client that refused to draw it]

## 13.1 The desk

`FastMcp::elicit("confirm", "Ship it?")` records a tool whose only job is to ask. `call_tool` hits that name, calls `Desk::ask`, and waits. The message is stored, the phase becomes `Waiting`, and a `oneshot` sender sits in a mutex until another task takes it.

```rust
let (sender, receiver) = oneshot::channel();
*self.inner.pending.lock().expect("pending") = Some(sender);
*self.inner.phase.lock().expect("phase") = Phase::Waiting;
self.inner.notify.notify_one();
```

`submit("yes")` waits until that sender exists, then sends `Accepted`. `ask` receives it and sets the phase to `Accepted`. `decline` and `cancel` are the other two sends. If the asker is gone, `send` fails and the answerer sees `ElicitError::Cancelled`.

`Notify` covers the race where the answer arrives in the same breath as the question. A permit is remembered if `notify_one` runs before `notified`. The answerer does not miss the wakeup, and it does not spin.

A tool result of `{"answer":"yes"}` means the person accepted. A decline comes back as a tool error whose text is `declined`. The call itself succeeded as JSON-RPC. `is_error` is true.

The rest of the desk is in `steps/ch13-1-elicit/crates/fastmcp-core/src/elicit.rs`.

## 13.2 A flat schema

`validate_elicitation_schema` reads a JSON value. The top-level `type` has to be `object`. A property whose `type` is `object` is a nested object, and the function returns an error that names the field. A string, number, integer, or boolean field is allowed. `enum` and `const` are allowed too.

```rust
if property_type == "object" {
    return Err(format!("field {name} is a nested object"));
}
```

The helper does not ask anyone. It only decides whether the form is one the chapter is willing to send.

## 13.3 Build, run, and benchmark

`tests/elicit.rs` calls `confirm` through the in-memory pipe. One task submits `yes` while the tool waits. The structured result is `{"answer":"yes"}`. A second test declines, and `is_error` is true. The unit tests ask twice on one desk and reject a nested object. Chapter 12's tests still pass in this copy. Seventy-seven tests passed: the previous 71, plus three unit tests and these three.

```bash
cargo test -p ch13-1-fastmcp-core
cargo run -p ch13-1-fastmcp-core --example demo
cargo bench -p ch13-1-fastmcp-core --bench elicit -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
ask: Ship it?
answer: yes
phase: accepted
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
elicit_round            time:   [14.732 µs 15.577 µs 16.635 µs]
```

Local median from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `elicit_round` | 15.11 µs | 13.56 µs | 16.05 µs | < 100 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `elicit_round` | Building the desk, spawning the answerer, `ask`, `submit("yes")`, and joining the task | Any socket, any `tools/call`, any schema check | The message `Ship it?` and the value `yes` | The person is a spawned task. The answer is already the string `yes`. | Well under 100 µs. About 15 µs here. Spawn and the channel are part of the number. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI median below is filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `elicit_round` | pending | pending | pending | < 100 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI median once the run is linked.

### Milestone demo

`fastmcp-rs/` grows an `elicit_phase` stub. `tests/ch13.rs` expects `"accepted"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch13-elicit.rs`](playground/ch13-elicit.rs) walks the phase names with no runtime. The `oneshot` desk needs tokio, which the playground does not have.

[Run the phases on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e0feb867907c30f06619406f7f3058f8).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
ask: Ship it?
answer: yes
phase: accepted
```

## Recap

`ask` sets the phase to `Waiting` and waits on a `oneshot`. `submit` sends the string and the phase becomes `Accepted`. `decline` finishes the tool with `is_error`. A nested object never reaches the desk. A second question uses a new channel.

`cargo run -p ch13-1-fastmcp-core --example demo` prints the three lines above. Locally, one round is 15.11 µs.

[personal note: the second answer I sent into a channel that was already closed]

## What's next

Chapter 14 is the other side of the pipe. A client picks a transport from a string, and a group of clients lists every server's tools under one name each.
