# Chapter 7: Prompts, completions, and pagination

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Tools do work. Resources hold bytes. A prompt is the third thing a server offers: a message the client can ask for by name, with holes the user fills in. `greet` with `name = Ada` becomes `Hello, Ada`. While the user is still typing `A`, the server can offer `Ada` and `Alan`. That offer is a completion.

The catalogs are no longer guaranteed to fit in one response. FastMCP pages `list_*` with a cursor that is an offset in disguise. The client treats the cursor as opaque. This chapter does the same, and it pages tools, resources, and prompts.

## What you'll build

A rendered greeting, the first page of a four-name catalog, and the cursor for the page after it.

```text
greet: Hello, Ada
page: greet, one
next: eyJvIjoyfQ==
```

`eyJvIjoyfQ==` is the URL-safe base64 of `{"o":2}`. The next page starts at offset 2. A cursor that is not that shape is `invalid_params`. A completion for the prefix `A` returns `Ada` and `Alan`, and never more than 100 values.

The snapshot is `steps/ch07-1-prompts`. It is a full copy of chapter 6, plus `pagination.rs` and `prompts.rs`. The crates are `ch07-1-fastmcp-core` and `ch07-1-fastmcp-macros`.

```bash
cargo run -p ch07-1-fastmcp-core --example demo
```

## Goals

- Register a prompt and render it from an argument map.
- Complete one argument by prefix, and cap the reply at 100 values.
- Page a list with an opaque cursor, and reject a cursor that is not a non-negative offset.
- Return `impl Iterator` from the page slice so the caller does not name the iterator type.
- Read the two benchmarks: one render, and one page taken from the middle of 10,000 items.

## Roadmap

**7.1 A prompt, and the names that complete it.** The problem is a message with a hole. The registry stores the function. Completion is a filtered list.

**7.2 A cursor is an offset.** `list_*` returns a page and a `nextCursor`. The encoding is small on purpose, so you can see what the client is not supposed to parse.

**7.3 Build, run, and benchmark.** The tests, the demo above, and the two timings.

| Section | Step | What it adds |
|---|---|---|
| 7.1–7.3 | `steps/ch07-1-prompts` | Prompts, prefix completion, cursor pages on every `list_*` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`utilities/pagination.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/pagination.py) `CursorState.encode` writes URL-safe base64 of `{"o": offset}`. `decode` rejects a bool, a non-integer, and a negative offset. `paginate_sequence` returns the page and the next cursor, or `None` on the last page. [`server/mixins/mcp_operations.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/mixins/mcp_operations.py) `_apply_pagination` turns that `ValueError` into `INVALID_PARAMS`. [`server/completions.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/completions.py) `normalize_completion` (near line 54) caps a completion at 100 values and sets `has_more` when it truncates. [`server/providers/local_provider/decorators/prompts.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/local_provider/decorators/prompts.py) `prompt` (the implementation near line 104) registers a function as a prompt.
>
> **Adapted.** FastMCP returns the whole list when `page_size` is `None`. This chapter always pages, with a default of 50, so a cursor shows up in the tests when you set the page size to 2. `serde_json` writes `{"o":2}` with no space. Python's `json.dumps` writes `{"o": 2}`. The cursor is opaque, so the two encodings do not have to match byte for byte. Both carry the offset. A completion handler that returns a bare `str` is a `TypeError` in Python, because a string would iterate into characters. Here the method takes a list of strings, so that mistake is a compile error. `page_items` returns `impl Iterator<Item = &T>`.
>
> **Deferred.** The completion context of arguments the user already filled in, async completion handlers, and `page_size = None` meaning "send everything" wait. Prompt auth and tags wait with the rest of auth.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **A bad cursor is `invalid_params`.** That is W4's validation half, applied to a cursor. The message starts with `Invalid cursor`. Chapter 8 maps the rest of the exception table.
>
> **A completion stops at 100.** The MCP contract caps `values`. This chapter truncates and sets `has_more`, which is what `normalize_completion` does, so a long candidate list does not become an oversized response.

[personal note: the first cursor I decoded by hand, and what the offset was]

## 7.1 A prompt, and the names that complete it

`greet` needs a `name`. The server stores a closure that receives the argument map and returns the message text. Required arguments are checked before the closure runs. A missing `name` is `invalid_params`, the same code a missing tool field already uses.

```rust
FastMcp::new("fastmcp-rs").prompt(
    "greet",
    "Greet someone",
    &[("name", true)],
    |args| {
        let name = args.get("name").map(String::as_str).unwrap_or("there");
        Ok(format!("Hello, {name}"))
    },
)
```

`prompts/get` turns that string into one user message. `prompts/list` advertises the name, the description, and the arguments. `initialize` now enables prompts and completions alongside tools and resources.

Completion is a table of candidate strings, keyed by the prompt name and the argument name. `complete_values` keeps the ones that start with the text the client has typed, then stops at 100. The `u32` it returns is the count before the cap, so `total` can say how many matches existed. `has_more` is true when the cap cut the list.

A prompt reference and a resource-template reference share that table. The key is the prompt name, or the template URI. An unknown key is an empty list: the user is still typing, and an empty suggestion is a complete answer.

## 7.2 A cursor is an offset

`encode_cursor(2)` is `eyJvIjoyfQ==`. You can paste that into a base64 decoder and read `{"o":2}`. The client is not supposed to. It sends the string back on the next `list_*`.

`decode_cursor` accepts only a JSON object whose `o` is a non-negative integer. `true` is a JSON bool, and `-1` is negative. Both fail. `_apply_pagination` maps that failure to `invalid_params`, and so does `turn_page`.

The slice itself is an iterator:

```rust
pub fn page_items<T>(
    items: &[T],
    offset: usize,
    page_size: usize,
) -> impl Iterator<Item = &T> {
    items.iter().skip(offset).take(page_size.max(1))
}
```

`impl Iterator` means "some type that yields `&T`". The concrete type is `Skip<Take<slice::Iter<T>>>`, and naming it would tie every caller to that shape. `paginate_sequence` collects the iterator into the page and, when more items remain, encodes `offset + page_size` as `next_cursor`.

`with_page_size` sets the size for every list: tools, resources, templates, and prompts. The default is 50, which is why chapter 6's three-tool list still comes back in one response. A second page is what you get when the catalog is longer than the page.

## 7.3 Build, run, and benchmark

`tests/prompts.rs` lists a page of two prompts, follows the cursor, and checks that `list_all_prompts` walks until the names are `extra-0`, `extra-1`, `extra-2`, and `greet`. `prompts/get` renders `Hello, Ada`. The prefix `A` completes to `Ada` and `Alan`. The cursor `nope` is `invalid_params`. The pagination unit tests cover a round trip, a three-page walk, a bool offset, and a negative offset. Chapter 6's tests still pass in this copy. Fifty-six tests passed: the previous 47, plus these nine.

```bash
cargo test -p ch07-1-fastmcp-core
cargo run -p ch07-1-fastmcp-core --example demo
cargo bench -p ch07-1-fastmcp-core --bench prompts -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
greet: Hello, Ada
page: greet, one
next: eyJvIjoyfQ==
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
prompt_render_1         time:   [61.765 ns 63.236 ns 64.238 ns]
paginate_10k            time:   [2.4377 µs 2.4841 µs 2.5173 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `prompt_render_1` | 62.14 ns | 60.92 ns | 63.27 ns | < 1 µs |
| `paginate_10k` | 2.482 µs | 2.420 µs | 2.509 µs | < 50 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `prompt_render_1` | One `render_prompt("greet", {name: Ada})`: the required-argument check and the format | Building the server and the argument map | The prompt `greet`, one required argument `name` | No socket | Well under 1 µs. About 62 ns here. |
| `paginate_10k` | Decode the cursor, `skip` to offset 5,000, `take` 50 items, clone them, and encode the next cursor | Building the 10,000 strings and encoding the starting cursor | Items `item-0` through `item-9999`. Page size 50. Cursor offset 5,000. | No socket and no `rmcp` session. This is the pager, not a round trip. | Well under 50 µs. About 2.5 µs here. The cost is cloning 50 strings, not walking the wire. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `prompt_render_1` | pending | pending | pending | < 1 µs |
| `paginate_10k` | pending | pending | pending | < 50 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `sample_prompt_name` stub. `tests/ch07.rs` expects `"greet"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch07-prompts.rs`](playground/ch07-prompts.rs) renders the greeting, takes the first page, and encodes the cursor with a small base64 function. The playground does not have this book's crate.

[Run the page on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=840bb71284269d4a707baed36111a034).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
greet: Hello, Ada
page: greet, one
next: eyJvIjoyfQ==
```

## Recap

A prompt is a name, a description, arguments, and a function from those arguments to a message. Completion filters a candidate list by prefix and stops at 100. Every `list_*` returns a page. The cursor is base64 of `{"o": offset}`, and a cursor that is not a non-negative integer is `invalid_params`.

`cargo run -p ch07-1-fastmcp-core --example demo` prints `Hello, Ada`, the page `greet, one`, and the cursor `eyJvIjoyfQ==`. Locally, one render is 62.14 ns and one page from the middle of 10,000 items is 2.482 µs.

[personal note: a page that came back empty, and the cursor that caused it]

## What's next

Chapter 8 sorts the errors. A missing argument, a tool that returns `Err`, and a panic-shaped failure inside the server are three different outcomes, and the client should be able to tell them apart.
