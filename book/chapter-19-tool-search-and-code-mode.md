# Chapter 19: Tool search and code mode

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A catalog of ten thousand tools is too big to hand to a model on every turn. FastMCP hides that catalog behind a search tool. The query is ordinary words. BM25 ranks the tools whose names and descriptions share those words, and the client sees a short list.

The next step is a script. Instead of a model turn for every call, one block of code calls the tools it already found. The block runs in a sandbox, and it may only call tools so many times.

## What you'll build

A search hit, two calls, and their sum.

```text
hits: add
calls: 2
answer: 5
```

The snapshot is `steps/ch19-1-search`. It is a full copy of chapter 18, plus `search.rs`. The crates are `ch19-1-fastmcp-core` and `ch19-1-fastmcp-macros`.

```bash
cargo run -p ch19-1-fastmcp-core --example demo
```

## Goals

- Tokenize tool text into lowercase words of two or more characters.
- Rank documents with BM25 Okapi, `k1` 1.5 and `b` 0.75, and keep scores above zero.
- Expose that search under the name `search_tools`.
- Run a rhai script whose `call_tool(name, value)` returns the value when the name is known.
- Stop the script when it passes the call limit.

## Roadmap

**19.1 Rank the catalog.** The problem is a list that does not fit in a prompt. The index scores each document and returns the best names.

**19.2 One script, many calls.** The problem is a model that takes a turn per tool. The script calls `call_tool` until the limit.

**19.3 Build, run, and benchmark.** The tests, the demo above, a search over 10,000 tools, and eight calls in one script next to eight host lookups.

| Section | Step | What it adds |
|---|---|---|
| 19.1–19.3 | `steps/ch19-1-search` | `Bm25`, `execute`, `SEARCH_TOOL_NAME` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/transforms/search/bm25.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/search/bm25.py) `_tokenize` (line 20) keeps tokens of length at least 2. `_BM25Index` (line 26) defaults `k1` to 1.5 and `b` to 0.75. The idf term is `log((n - df + 0.5) / (df + 0.5) + 1)` (line 67). `BM25SearchTransform` (line 89) defaults `max_results` to 5, `search_tool_name` to `search_tools`, and `call_tool_name` to `call_tool`. [`server/transforms/search/base.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/search/base.py) `BaseSearchTransform` (line 291) replaces the tool list with pinned tools, the search tool, and a call proxy. [`server/transforms/catalog.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/catalog.py) `CatalogTransform` (line 66) is the base that can see the real catalog. [`experimental/transforms/code_mode.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/experimental/transforms/code_mode.py) `CodeMode` (line 611) collapses tools into discovery plus `execute`. `max_tool_calls` defaults to 50 (line 631). The inner `call_tool` (line 722) raises when the count passes that limit (line 726).
>
> **Adapted.** Tokens here are ASCII letters and digits. FastMCP normalizes with NFKC and casefold, then keeps Unicode alphanumeric tokens. The idf and the tf term follow the same formula. The script engine is rhai. FastMCP runs Python and requires `pydantic-monty` (line 222). `call_tool` in this chapter returns the integer the script passed, once the name is in the map. FastMCP forwards the call to the real tool.
>
> **Deferred.** The catalog hash that rebuilds the index (`_catalog_hash`, line 83), `always_visible` pins, auth filtering on search results, and Monty's resource limits. A full Python sandbox is not in this crate.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **A script stops at the call limit.** Two calls with a limit of 1 return an error whose text contains `tool call limit exceeded`. An unknown name is an error too. The tool does not run past the cap.

[personal note: a search query that was one character long, and the empty token list it produced]

## 19.1 Rank the catalog

```rust
let idf = ((n as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();
```

`search` drops a document whose score is zero, sorts the rest best first, and keeps `top_k`. The demo asks for five and gets `add`, because `weather` shares no token with `add numbers`.

## 19.2 One script, many calls

`execute` builds a rhai engine and registers `call_tool`. The counter increments before the call. When it is already at the limit, the function returns the error and the script stops. A name that is not in the map is `unknown tool`.

`tests/search.rs` checks the ranking, a script that adds 2 and 3, the limit, and the unknown name. Chapter 18's tests still pass in this copy. One hundred eight tests passed: the previous 105, plus these three.

## 19.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch19-1-fastmcp-core
cargo run -p ch19-1-fastmcp-core --example demo
cargo bench -p ch19-1-fastmcp-core --bench search -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
hits: add
calls: 2
answer: 5
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
search_10k              time:   [181.46 µs 182.77 µs 184.66 µs]
script_8                time:   [153.45 µs 157.21 µs 159.85 µs]
turns_8                 time:   [88.837 ns 91.050 ns 95.733 ns]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `search_10k` | 181.6 µs | 181.1 µs | 182.2 µs | < 1 ms |
| `script_8` | 154.1 µs | 153.1 µs | 160.0 µs | < 1 ms |
| `turns_8` | 89.57 ns | 88.37 ns | 94.31 ns | < 1 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `search_10k` | One BM25 query and the top 5 names | Building the index of 10,000 documents | Query `word7`. Document `i` contains `word{i % 50}` | No socket | Well under 1 ms. About 182 µs here. |
| `script_8` | Parsing and running one rhai script that calls `call_tool` eight times, including a new engine | Building the tool map and the script string | Eight tools, each call passes `1`, limit 50 | The tool returns the integer the script passed. No model | Well under 1 ms. About 154 µs here. |
| `turns_8` | Eight host lookups that sum the same eight values | Building the map | The same eight names | No engine and no model. This is the in-process cost of eight separate calls, not eight network round trips | Well under 1 µs. About 90 ns here. |

The script is the stand-in for one model turn that calls eight tools. Eight host lookups are cheaper because they skip the engine. They are in the table so the script's cost has a neighbor.

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `search_10k` | pending | pending | pending | < 1 ms |
| `script_8` | pending | pending | pending | < 1 ms |
| `turns_8` | pending | pending | pending | < 1 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `search_tool_name` stub. `tests/ch19.rs` expects `"search_tools"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch19-search.rs`](playground/ch19-search.rs) ranks two documents by shared tokens and adds 2 and 3 under a call cap. The playground does not have this book's crate, and it does not have rhai. The crate demo runs BM25 and a rhai script.

[Run the overlap on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=0420ccffc4c285a8cb669d13ea984b30).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
hits: add
calls: 2
answer: 5
```

## Recap

`tokenize` keeps words of two or more characters. `Bm25::search` scores with the Okapi formula and drops zeros. `execute` runs a rhai script. `call_tool` returns the passed integer when the name is known, and it refuses the call that would pass the limit.

`cargo run -p ch19-1-fastmcp-core --example demo` prints the three lines above. Locally, a search over 10,000 tools takes 181.6 µs, eight calls in one script take 154.1 µs, and eight host lookups take 89.57 ns.

[personal note: the script that called the same tool in a loop, and the limit that stopped it]

## What's next

Chapter 20 lets a tool run after the call has returned. The client submits a task, polls it, and reads the result. An extension id is how the server says it speaks that protocol.
