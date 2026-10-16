# Chapter 4: Schemas from types

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Chapter 3 can call `add`. A client that asks `tools/list` still sees an empty object for every tool. It learns the name and the description, and it cannot tell that `left` and `right` are required integers, or that a field called `label` may be left out. The client has to guess, and a guess that omits `right` fails inside the server after the request has already been sent.

FastMCP solves this by building a JSON Schema from the function's types at registration time. The input schema says which arguments the client must send. The output schema says what a successful call returns. This chapter keeps the builder and asks the `schemars` crate to write both documents from the structs you already have. The same `Option` that means "this field may be missing" in Rust becomes "this field is not in `required`" on the wire.

## What you'll build

`add` still adds two integers. The difference is that the tool now carries a schema, and the value it returns is a struct the client can see.

```text
add(2, 3) = {"total":5}
required: ["left","right"]
label: {"type":["string","null"]}
```

`required` lists `left` and `right`. `label` is an optional string: the property allows a string or JSON `null`, and omitting the key still runs the tool. The return value is `{"total": 5}`, and `tools/list` advertises an output schema whose `total` is an integer. `status` stays the era note from chapter 2, with an empty input object and no output schema.

The package is `ch04-1-fastmcp-core`. The library name is still `fastmcp_core`. The snapshot is `steps/ch04-1-schema`, a full copy of chapter 3 plus schema generation. `schemars` 1.2.3 is a direct dependency so `#[derive(JsonSchema)]` can see a crate named `schemars`. `rmcp`'s server feature uses that same crate. Re-exporting it as `rmcp::schemars` is enough to call the trait, and the derive looks up the crate by its own name, so the package depends on `schemars` directly.

```bash
cargo run -p ch04-1-fastmcp-core --example demo
```

## Goals

- Generate a JSON Schema from an argument struct and from the struct a tool returns.
- Explain `#[derive(JsonSchema)]`, and why `schemars` has to be a direct dependency.
- Say what `Option<String>` becomes in the schema: absent from `required`, and typed as a string or null.
- Keep a missing required field out of the closure, and let a missing optional field through.
- Read the schema benchmarks, and know they measure generation with no cache.

## Roadmap

**4.1 Ask the type for its schema.** The input struct, the output struct, and `json_schema_of`, which writes a draft 2020-12 document and drops the title the tool does not need. This is where `Option` becomes a field the client may omit.

**4.2 Show that document on `tools/list`.** Registration stores both schemas. A call still deserializes first, and the return struct is what `structuredContent` carries.

**4.3 Build, run, and benchmark.** The tests, the demo above, and the cost of building the document once and a thousand times, with no cache in the way.

| Section | Step | What it adds |
|---|---|---|
| 4.1–4.3 | `steps/ch04-1-schema` | `JsonSchema` on the argument struct and the return struct; `Option` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`tools/function_parsing.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/tools/function_parsing.py) builds the input schema with `input_type_adapter.json_schema()` (around line 366) and then `compress_schema(..., prune_titles=True)`. The output schema is another `TypeAdapter.json_schema` (around line 515), also passed through `compress_schema` (line 538). This chapter generates both documents at `.tool()` time and stores them on the registered tool.
>
> **Adapted.** FastMCP's schemas come from Pydantic. These come from `schemars` and JSON Schema draft 2020-12. [`utilities/json_schema.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/json_schema.py) `compress_schema` (line 881) also prunes nested titles, unused `$defs`, and parameters you name. This chapter only removes the top-level `title` and `description`, which are the struct's name and its doc comment. An `Option` field is omitted from `required`, and its property type is `["string", "null"]` for `Option<String>`. FastMCP wraps a non-object return value in an object and sets `x-fastmcp-wrap-result` (`function_parsing.py`, the `wrap_non_object_output_schema` branch). The tools in this chapter return a struct, so the output schema is already an object. The wrap marker is not copied.
>
> **Deferred.** Docstring text copied onto each property, and `Field(title=...)`, stay in `function_parsing.py` for a later pass. Parameters injected from `Context` instead of the schema are chapters 9 and 10. `rmcp` caches a schema per `TypeId`. `json_schema_of` does not, so the bench is the cost of building the document.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **A field in `required` is a field serde will not default.** `left` and `right` are in `required`. Omitting `right` is `ToolError::InvalidParams` and the closure does not run. That is the validation half of W4, now visible on `tools/list`.
>
> **An `Option` field is not in `required`.** Omitting `label` deserializes as `None` and the tool runs. The schema says the property may also be JSON `null`.
>
> **The return struct has an output schema. `status` does not.** `tools/list` sets `outputSchema` from `Sum`. `status` is still the era note, with an empty input object and no output schema.
>
> Unknown names and the closure's `Err` are unchanged from chapter 3. The full `to_mcp_error` map is still chapter 8.

[personal note: the first struct whose schema surprised me]

## 4.1 Ask the type for its schema

`Add` is the input. `Sum` is the output. The new piece on each struct is `JsonSchema`, next to the serde derive you already used.

```rust
#[derive(Deserialize, JsonSchema)]
struct Add {
    left: i64,
    right: i64,
    label: Option<String>,
}

#[derive(Serialize, JsonSchema)]
struct Sum {
    total: i64,
}
```

`#[derive(JsonSchema)]` is an instruction to a macro that runs while the program is compiling. The macro reads the fields and writes the `impl JsonSchema for Add` you would otherwise write by hand. `Deserialize` and `Serialize` are the same kind of macro, from serde. You name the traits. The compiler fills in the methods. If a field's type has no `JsonSchema` implementation, the derive fails at compile time, which is the error you want: a schema that cannot describe one of its fields should not wait until a client asks.

`Option<String>` is either `Some(text)` or `None`. Serde treats a missing JSON key as `None`, and a present JSON `null` as `None` too. A present string is `Some`. Forgetting `Option` puts the field in `required`. The client must send it, and a missing key fails before the closure. With `Option`, `schemars` leaves `label` out of the `required` array. The property itself allows a string or null: `{"type": ["string", "null"]}`.

`json_schema_of` is the whole generator. It lives in `steps/ch04-1-schema/crates/fastmcp-core/src/schema.rs`.

```rust
pub fn json_schema_of<T: JsonSchema>() -> Arc<Map<String, Value>> {
    let generator = SchemaSettings::draft2020_12().into_generator();
    let schema = generator.into_root_schema_for::<T>();
    let Value::Object(mut object) = serde_json::to_value(schema).expect("schema is JSON") else {
        panic!("schema is a JSON object");
    };
    object.remove("title");
    object.remove("description");
    Arc::new(object)
}
```

`JsonSchema` is the trait the derive implements. `SchemaSettings::draft2020_12()` picks the dialect. `into_root_schema_for::<T>()` walks `T` and returns a schema value. `serde_json::to_value` turns that into the map MCP stores on the tool. The top-level title would be the struct's name, `Add` or `Sum`. MCP already has the tool's name and description, so the title and the top-level description come off. `Arc` is the same shared pointer as chapter 3: the document is built once, at registration, and every clone of the server shares it.

A return type whose schema is not an object, a bare `i64`, is legal in `rmcp`, and it is the case FastMCP wraps in an object with `x-fastmcp-wrap-result`. This chapter's `Sum` is an object so `tools/list` can show `total`. The wrap marker is not copied.

FastMCP's `compress_schema` also prunes nested titles, unused `$defs`, and parameters you name. This function removes the top-level title and description. That is the part a client needs in order to see `required` and the property types. The deeper prune waits.

## 4.2 Show that document on `tools/list`

`register` stores `json_schema_of::<P>()` as the input schema and `json_schema_of::<R>()` as the output schema. `.tool()` calls each once. `tools/list` passes the output schema to `Tool::with_raw_output_schema`. The closure still runs only after `P` deserializes, and its `Ok(R)` is serialized to JSON for `structuredContent`.

So the path a client sees is: list the tool, read `required`, send `left` and `right`, and receive `{"total": 5}`. Send `left` alone and the closure does not run. Send `left`, `right`, and no `label`, and the tool runs with `label` as `None`.

`status` is unchanged. It is still listed first, still answers with the era note, and it still has an empty input object and no output schema. It was never a registered closure, so it never went through `json_schema_of`.

## 4.3 Build, run, and benchmark

`option_field_is_not_required` builds a schema for a struct with `name: String` and `title: Option<String>`. The root type is `object`. `required` contains `name` and does not contain `title`. The `title` property's JSON contains `null`.

`omitted_label_runs_and_stays_out_of_required` calls `add` with `left` and `right` only. The sum is `{"total": 5}`. The registered input schema lists `left` and omits `label` from `required`. The output schema's `total` property has type `integer`.

`missing_field_does_not_run_the_tool` still rejects `{"left": 1}`.

`tests/tools.rs` checks the same documents on the wire. `tools/list` for `add` has `label` outside `required` and an output schema whose `total` is `integer`. `status` has no output schema. `tools/call` of `add` with `left: 2` and `right: 3` returns structured content `{"total": 5}`.

```bash
cargo test -p ch04-1-fastmcp-core
cargo run -p ch04-1-fastmcp-core --example demo
cargo bench -p ch04-1-fastmcp-core --bench schema -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

`cargo test` printed `ok` for all 37 tests: 26 in the library, 3 in `tests/handshake.rs`, 3 in `tests/session.rs`, 5 in `tests/tools.rs`. The demo prints:

```text
add(2, 3) = {"total":5}
required: ["left","right"]
label: {"type":["string","null"]}
```

`examples/stdio.rs` registers the same `add` tool and waits on stdin. The demo above is the command that exits.

The bench calls `json_schema_of` directly. Registration is outside the timer. There is no cache, so 1,000 iterations are 1,000 generations of the same struct. `rmcp` would hit a `TypeId` cache after the first call. This loop does not, on purpose: a cache would make `schema_1000` tell you about the map, not about `schemars`.

The bench command's slope, on the same 4-vCPU guest as chapter 1:

```text
schema_1                time:   [1.5947 µs 1.6048 µs 1.6129 µs]
schema_1000             time:   [1.8501 ms 1.8807 ms 1.9153 ms]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `schema_1` | 1.596 µs | 1.592 µs | 1.610 µs | < 100 µs |
| `schema_1000` | 1.854 ms | 1.827 ms | 1.876 ms | < 50 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `schema_1` | One `json_schema_of::<Greet>()`: draft 2020-12 generation, `serde_json::to_value`, and removal of the top-level title and description | Storing the `Arc` on a tool. No `.tool()` call. | `Greet { name: String, title: Option<String> }` | No socket, no `tools/list`, no cache | Well under 100 µs. About 1.6 µs here. |
| `schema_1000` | 1,000 calls of that same function, each building a new schema | A cache. `rmcp` would hit a `TypeId` cache after the first call. This loop does not. | The same `Greet` struct, a thousand times | No socket | Under 50 ms. About 1.9 ms here. A cache would make this look like one generation. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38857080367](https://github.com/agnathan/rust-mcp/actions/runs/38857080367) (2026-10-16T05:41:43Z). GitHub's `github.sha` for that pull request run is the merge commit `4e2a37e58ef911f020756b0349703c1b1f633433`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `schema_1` | 1.863 µs | 1.861 µs | 1.865 µs | < 100 µs |
| `schema_1000` | 1.916 ms | 1.913 ms | 1.920 ms | < 50 ms |

On the Ubuntu runner, one schema is 1.863 µs, and a thousand uncached generations are 1.916 ms. Those are the medians this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows an `optional_argument_name` stub. `tests/ch04.rs` expects `"label"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch04-schemas-from-types.rs`](playground/ch04-schemas-from-types.rs) prints the `required` array and the `label` property, and it treats a missing key as omitted. The [playground crate list](https://play.rust-lang.org/meta/crates) has `schemars` 1.2.2 built without the `derive` feature, so `#[derive(JsonSchema)]` does not compile there. The derive runs in this chapter's crate. The playground file prints the schema document with `serde_json` instead.

[Run the schema document on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=1759c3f47f771664dee7a57b7894d8f5).

## Recap

`.tool()` stores a JSON Schema for the argument struct and a JSON Schema for the return struct, both generated when the tool is registered. `#[derive(JsonSchema)]` writes the implementation, and it needs a direct `schemars` dependency. `required` lists the fields the client must send. `label: Option<String>` is the field they may leave out, and the schema marks it as a string or null. Omitting `right` still fails before the closure. Omitting `label` runs the tool.

`cargo run -p ch04-1-fastmcp-core --example demo` prints `{"total": 5}` and the `label` schema. On the Ubuntu run linked above, one schema is 1.863 µs and a thousand uncached generations are 1.916 ms. The bench has no cache, so that second number is the cost of building the document.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 5 turns `.tool(|args: Add| ...)` into a `#[tool]` attribute, closer to FastMCP's decorator. The schema rules from this chapter stay: required fields, optional fields, and an output schema taken from the return type.
