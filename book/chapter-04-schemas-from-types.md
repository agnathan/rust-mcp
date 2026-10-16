# Chapter 4: Schemas from types

*Draft. Bracketed notes like [personal note] are where my own story goes.*

Chapter 3 can call `add` with a struct, and `tools/list` still advertises an empty object for every tool. A client cannot see that `left` and `right` are required, or that `label` may be left out. This chapter keeps the builder and asks `schemars` to write that document from the struct. The same call writes a schema for the value the tool returns.

| Section | Step | What it adds |
|---|---|---|
| 4.1 | `steps/ch04-1-schema` | `JsonSchema` on the argument struct and the return struct; `Option` |

The package is `ch04-1-fastmcp-core`. The library name is still `fastmcp_core`. It is a full copy of chapter 3 plus schema generation. `schemars` 1.2.3 is a direct dependency so `#[derive(JsonSchema)]` can see the `schemars` crate. `rmcp`'s server feature uses that same crate.

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

## What this chapter decides

- `#[derive(Deserialize, JsonSchema)]` on the argument struct. `#[derive(Serialize, JsonSchema)]` on the return struct.
- `json_schema_of::<T>()` uses `SchemaSettings::draft2020_12()`, serializes the schema, and removes the top-level title and description.
- `.tool()` calls that once for `P` and once for `R`, and stores both `Arc`s.
- `Add` has `label: Option<String>`. `required` is `["left", "right"]`. `label`'s schema is `{"type": ["string", "null"]}`.
- The bench calls `json_schema_of` directly. Registration is not inside the timer. There is no cache, so 1,000 iterations are 1,000 generations of the same struct.

## 4.1 A schema from a struct

`Add` is the input. `Sum` is the output.

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

> **Rust: a derive macro.** `#[derive(JsonSchema)]` is an instruction to a macro that runs at compile time. The macro reads the struct's fields and writes the `impl JsonSchema for Add` you would otherwise write by hand. `Deserialize` and `Serialize` are the same kind of macro, from serde. You name the traits. The compiler fills in the methods. If a field's type has no `JsonSchema` impl, the derive fails at compile time.

> **Rust: `Option`.** `Option<String>` is either `Some(text)` or `None`. serde treats a missing JSON key as `None`. A present JSON `null` is also `None`. A present string is `Some`. `schemars` sees `Option` and leaves `label` out of the `required` array. The property itself allows a string or null: `{"type": ["string", "null"]}`.

`json_schema_of` is the whole generator:

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

> **Rust: `schemars`.** `JsonSchema` is the trait the derive implements. `SchemaSettings::draft2020_12()` picks the dialect. `into_root_schema_for::<T>()` walks `T` and returns a schema value. `serde_json::to_value` turns that into the `Map` MCP stores on the tool. The top-level title would be the struct name (`Add`). MCP already has the tool's name and description, so the title is removed.

`register` stores `json_schema_of::<P>()` as the input schema and `json_schema_of::<R>()` as the output schema. `tools/list` passes the output schema to `Tool::with_raw_output_schema`. The closure still runs only after `P` deserializes, and its `Ok(R)` is serialized to JSON for `structuredContent`.

### The tests

`option_field_is_not_required` builds a schema for a struct with `name: String` and `title: Option<String>`. The root type is `object`. `required` contains `name` and does not contain `title`. The `title` property's JSON contains `null`.

`omitted_label_runs_and_stays_out_of_required` calls `add` with `left` and `right` only. The sum is `{"total": 5}`. The registered input schema lists `left` and omits `label`. The output schema's `total` property has type `integer`.

`missing_field_does_not_run_the_tool` still rejects `{"left": 1}`.

`tests/tools.rs` checks the same documents on the wire. `tools/list` for `add` has `label` outside `required` and an output schema whose `total` is `integer`. `status` has no output schema. `tools/call` of `add` with `left: 2` and `right: 3` returns structured content `{"total": 5}`.

Thirty-seven tests passed: 26 in the library, 3 in `tests/handshake.rs`, 3 in `tests/session.rs`, 5 in `tests/tools.rs`.

### Build, run, and benchmark

```bash
cargo test -p ch04-1-fastmcp-core
cargo run -p ch04-1-fastmcp-core --example demo
cargo bench -p ch04-1-fastmcp-core --bench schema -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

`cargo test` printed `ok` for all 37 tests. The demo prints:

```text
add(2, 3) = {"total":5}
required: ["left","right"]
label: {"type":["string","null"]}
```

`examples/stdio.rs` registers the same `add` tool and waits on stdin. The demo above is the command that exits.

The bench command's slope, on the same 4-vCPU guest as chapter 1:

```text
schema_1                time:   [1.5947 µs 1.6048 µs 1.6129 µs]
schema_1000             time:   [1.8501 ms 1.8807 ms 1.9153 ms]
```

Local medians from that run's `estimates.json` (the slope above is not this column):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `schema_1` | 1.596 µs | 1.592 µs | 1.610 µs | < 100 µs |
| `schema_1000` | 1.854 ms | 1.827 ms | 1.876 ms | < 50 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `schema_1` | One `json_schema_of::<Greet>()`: draft 2020-12 generation, `serde_json::to_value`, and removal of the top-level title and description | Storing the `Arc` on a tool. No `.tool()` call. | `Greet { name: String, title: Option<String> }` | No socket, no `tools/list`, no cache | Well under 100 µs. About 1.6 µs here. |
| `schema_1000` | 1,000 calls of that same function, each building a new schema | A cache. `rmcp` would hit a `TypeId` cache after the first call. This loop does not. | The same `Greet` struct, a thousand times | No socket | Under 50 ms. About 1.9 ms here. A cache would make this look like one generation. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians for this section are filled from this pull request's Benchmarks run once that job has finished. Until that table is updated, the numbers above are the local run only.

### Milestone demo

`fastmcp-rs/` grows an `optional_argument_name` stub. `tests/ch04.rs` expects `"label"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch04-schemas-from-types.rs`](playground/ch04-schemas-from-types.rs) prints the `required` array and the `label` property, and it treats a missing key as omitted. The [playground crate list](https://play.rust-lang.org/meta/crates) has `schemars` 1.2.2 built without the `derive` feature, so `#[derive(JsonSchema)]` does not compile there. The derive runs in this chapter's crate.

[Run the schema document on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=1759c3f47f771664dee7a57b7894d8f5).

## Pitfalls

- `#[derive(JsonSchema)]` looks up a crate named `schemars`. Re-exporting the crate as `rmcp::schemars` is enough to call the trait, and it is not enough for the derive. The package depends on `schemars` directly.
- Forgetting `Option` puts the field in `required`. The client must send it, and a missing key fails before the closure.
- The bench is uncached on purpose. A later cache would make `schema_1000` tell you about the map, not about `schemars`.
- A return type whose schema is not an object (a bare `i64`) is legal in `rmcp` and is the case FastMCP wraps. This chapter's `Sum` is an object so `tools/list` can show `total`.

## Chapter summary

`.tool()` now stores a JSON Schema for the argument struct and a JSON Schema for the return struct. `required` lists the fields the client must send. `label: Option<String>` is the field they may leave out, and the schema marks it as a string or null. The documents are built when the tool is registered.

[personal note: a moment where the compiler stopped me, and how I worked out what it wanted]

## What's next

Chapter 5 turns `.tool(|args: Add| ...)` into a `#[tool]` attribute, closer to FastMCP's decorator. The schema rules from this chapter stay.
