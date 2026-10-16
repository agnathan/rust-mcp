# Chapter 10: Dependency injection

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A tool that queries a database needs the database. A tool that checks a caller needs the token. Neither of those is something the model should invent and send as JSON. FastMCP keeps them off the schema with `Depends`. The function asks for a type, and the server fills it in before the function runs.

This chapter builds a small version of that. `provide` stores a value by its type. `Dep<T>` pulls it back out. A tuple of dependencies is one argument, so a tool can ask for the database and the token together. The schema is still only the struct the client sends.

## What you'll build

A query, a database name, and a token, and a schema whose only required field is `q`.

```text
who: "main secret widgets"
required: q
```

`Db` and `Token` were stored on the server. They are not in `required`.

The snapshot is `steps/ch10-1-depends`. It is a full copy of chapter 9, plus `inject.rs`. The crates are `ch10-1-fastmcp-core` and `ch10-1-fastmcp-macros`.

```bash
cargo run -p ch10-1-fastmcp-core --example demo
```

## Goals

- Store a value with `provide` and read it back as `Dep<T>`.
- Keep `Dep<T>` out of the tool schema.
- Use `PhantomData<fn() -> T>` on `Dep`, and a tuple `Extract` impl for two dependencies.
- Compare a thousand extractions with a thousand hand-passed `Arc` clones.

## Roadmap

**10.1 A value keyed by its type.** The problem is a handle the model must not send. `TypeId` is the key.

**10.2 Extract a tuple.** `FromContext` builds one `Dep`. `Extract` builds one dependency or a pair.

**10.3 Build, run, and benchmark.** The test, the demo above, and the two timings.

| Section | Step | What it adds |
|---|---|---|
| 10.1–10.3 | `steps/ch10-1-depends` | `provide`, `Dep<T>`, `tool_inject` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`dependencies.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/dependencies.py) re-exports `Depends`, `CallArgument`, `Dependency`, and `Shared` from `uncalled_for` (near line 14), plus `CurrentContext` and `CurrentAccessToken` from the server. [`server/dependencies.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/dependencies.py) `_resolve_fastmcp_dependencies` (near line 820) builds the resolution frame: a cache, an exit stack, and the call's arguments. Parameters that are dependencies are removed from the signature the schema sees.
>
> **Adapted.** This chapter keys the map by `TypeId` and clones the value into `Dep<T>`. There is no `Depends()` annotation and no async exit stack. `PhantomData<fn() -> T>` sits beside the value so `Dep` is invariant in `T`. A missing dependency is a tool failure, `is_error`, with the type name in the message.
>
> **Deferred.** `CurrentRequest`, header injection, and the Docket task dependencies wait. Cycles between dependencies wait with them.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **A dependency is not a parameter.** `required` lists `q`. It does not list `db` or `token`. A model that sees those fields will try to fill them.
>
> **The token stays on the server.** Injection is how it reaches the function. It is not a string the client includes in `arguments`.

[personal note: a schema that accidentally asked for the database, and how I noticed]

## 10.1 A value keyed by its type

`provide` inserts `T` into a `HashMap<TypeId, Arc<dyn Any + Send + Sync>>`. `TypeId::of::<T>()` is the key. Two different types never share a slot. `get::<T>` downcasts. A miss is an error that names the type, so a forgotten `provide` is visible.

`Dep<T>` holds the cloned `T` and a `PhantomData<fn() -> T>`. The marker holds no bytes. `fn() -> T` is invariant in `T`, which stops a `Dep<Cat>` from being treated as a `Dep<Animal>` just because `Cat` is an `Animal`. The value is real. The marker is the type-system half of the same idea.

## 10.2 Extract a tuple

`FromContext` is the trait one dependency implements. `Dep<T>` implements it by cloning out of the map. `Extract` is what the tool asks for. A single `FromContext` type extracts as itself. A pair extracts by extracting each half:

```rust
impl<A: Extract, B: Extract> Extract for (A, B) {
    fn extract(ctx: &InjectCtx) -> Result<Self, String> {
        Ok((A::extract(ctx)?, B::extract(ctx)?))
    }
}
```

`tool_inject` stores `Fn(E, P)`. `E::extract` runs after `P` deserializes and before the function. The schema is `json_schema_of::<P>()`. `Dep<Db>` never reaches `schemars`.

The demo's function takes `(Dep<Db>, Dep<Token>)` and a `Query { q }`. The result is `main secret widgets`. `required` is `q`.

## 10.3 Build, run, and benchmark

`tests/inject.rs` lists `who` and checks the schema has `q` and does not have `db` or `token`. The call returns `main secret widgets`. Chapter 9's tests still pass in this copy. Sixty-four tests passed: the previous 63, plus this one.

```bash
cargo test -p ch10-1-fastmcp-core
cargo run -p ch10-1-fastmcp-core --example demo
cargo bench -p ch10-1-fastmcp-core --bench inject -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
who: "main secret widgets"
required: q
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
inject_1000             time:   [11.932 µs 12.006 µs 12.089 µs]
hand_1000               time:   [15.046 µs 15.174 µs 15.330 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `inject_1000` | 12.03 µs | 11.90 µs | 12.10 µs | < 100 µs |
| `hand_1000` | 15.22 µs | 15.16 µs | 15.45 µs | < 100 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `inject_1000` | 1,000 `Dep::<Db>::from_context` calls, each a `TypeId` lookup, a downcast, a clone of `Db`, and `bias + n` | Building the map. No JSON and no `dispatch_tool`. | `Db { bias: 1 }`. `n` from 0 through 999. | No socket | Well under 100 µs. About 12 µs here. |
| `hand_1000` | 1,000 `Arc::clone` calls of a `Db` the caller already holds, and the same `bias + n` | Building the `Arc`. No map lookup. | The same `Db { bias: 1 }` | No socket | Well under 100 µs. About 15 µs here. On this guest the lookup-and-clone loop was the smaller of the two. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `inject_1000` | pending | pending | pending | < 100 µs |
| `hand_1000` | pending | pending | pending | < 100 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows an `injected_query_field` stub. `tests/ch10.rs` expects `"q"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch10-depends.rs`](playground/ch10-depends.rs) is a `Dep` with `PhantomData`. The playground does not have this book's crate.

[Run the marker on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=42984da312b71af752257008d74c0ffd).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
who: main secret widgets
required: q
```

## Recap

`provide` stores a value by `TypeId`. `Dep<T>` clones it out, and `PhantomData<fn() -> T>` keeps the type invariant. `tool_inject` extracts a dependency tuple and deserializes only `P`, so `required` stays `q`.

`cargo run -p ch10-1-fastmcp-core --example demo` prints `main secret widgets` and `required: q`. Locally, a thousand extractions take 12.03 µs and a thousand hand-passed `Arc` clones take 15.22 µs.

[personal note: a type that landed in the schema, and the field name that gave it away]

## What's next

Chapter 11 starts the server and stops it. A database pool opens before the first request and closes after the last one, and the tools share it through the same kind of handle this chapter injected.
