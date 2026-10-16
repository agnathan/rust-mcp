# Chapter 18: Transforms and visibility

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A provider answers “what tools do I have?” A transform answers “what should the client see?” The same `add` can leave the building as `math_add`. One argument can leave under a new name. A tag can hide a tool, and a later rule can show it again. A prompt and a resource can show up as tools for a client that only speaks tools.

When the visible list changes, something has to remember that it changed. FastMCP sends a list-changed notification. This chapter sets a flag the notification would carry.

## What you'll build

A prefixed name, a filtered list, and the two prompt tools.

```text
name: math_add
visible: add
prompts: list_prompts, get_prompt
changed: true
```

The snapshot is `steps/ch18-1-transforms`. It is a full copy of chapter 17, plus `transform.rs`. The crates are `ch18-1-fastmcp-core` and `ch18-1-fastmcp-macros`.

```bash
cargo run -p ch18-1-fastmcp-core --example demo
```

## Goals

- Prefix a tool name and a resource URI, and borrow the name when the prefix is empty.
- Rename one tool and one argument key.
- Disable every tool that carries a tag, then let a later enable put one name back.
- Expose prompts as `list_prompts` and `get_prompt`, and resources as `list_resources` and `read_resource`.
- Set `list_changed` when the visible catalog changes.

## Roadmap

**18.1 Names and arguments.** The problem is two catalogs that both contain `add`. `Cow` owns the prefixed name and borrows the original when there is nothing to add.

**18.2 What the client is allowed to see.** The problem is a tool that should disappear, and a client that cannot call prompts. Visibility marks the catalog. Prompts and resources become tools. The flag records the change.

**18.3 Build, run, and benchmark.** The tests, the demo above, and a filtered list of 10,000 tools.

| Section | Step | What it adds |
|---|---|---|
| 18.1–18.3 | `steps/ch18-1-transforms` | `namespace_name`, `rename_argument`, `Catalog` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/transforms/__init__.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/__init__.py) `Transform` (line 68) takes a sequence and returns a sequence. [`namespace.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/namespace.py) `Namespace` (line 28) turns a tool name into `namespace_name` and a resource URI `protocol://path` into `protocol://namespace/path`. [`tool_transform.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/tool_transform.py) `ToolTransform` (line 16) wraps [`tools/tool_transform.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/tools/tool_transform.py) `ToolTransformConfig` (line 973), including argument renames. [`visibility.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/visibility.py) `Visibility` (line 40) marks components. A later transform overrides an earlier mark. `is_enabled` (line 285) reads the mark, and the default is enabled. `save_visibility_rules` sends `ToolListChangedNotification` (line 339). [`prompts_as_tools.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/prompts_as_tools.py) `PromptsAsTools` (line 39) adds `list_prompts` and `get_prompt`. [`resources_as_tools.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/transforms/resources_as_tools.py) `ResourcesAsTools` (line 42) adds `list_resources` and `read_resource`. The client handles `ToolListChangedNotification` in [`client/messages.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/client/messages.py) `on_tool_list_changed` (line 71).
>
> **Adapted.** The catalog is a `Vec` of names, tags, and an enabled flag. FastMCP stores the mark in component meta and filters at the provider. `namespace_name` returns `Cow`: an empty prefix borrows the original. Argument rename copies one JSON object and changes one key. `list_changed` is a `bool` on the catalog. FastMCP sends the notification on the session.
>
> **Deferred.** Session-scoped visibility rules, the wire notification, schema rewrites beyond one argument key, and version matching. Component versions are chapter 23.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as chapters 1, 8, 14, 16, and 17 left them.
>
> **A disabled tool is absent from `visible`.** `secret` carries `internal`, so it drops out. `safe` was disabled by the same tag and put back by `enable_name`. The list the client would see is the filtered one.

[personal note: a tag I meant as documentation, and the tool that vanished from the client]

## 18.1 Names and arguments

```rust
if prefix.is_empty() {
    Cow::Borrowed(name)
} else {
    Cow::Owned(format!("{prefix}_{name}"))
}
```

`namespace_uri` splits on `://`. `file://data.txt` with prefix `math` becomes `file://math/data.txt`. A string with no scheme is returned borrowed.

`rename_argument` clones the object and writes `old_arg` under `new_arg`. `rename_tool` replaces the catalog name, so the old name is gone.

## 18.2 What the client is allowed to see

`disable_tag` sets `enabled` to false on every tool that has the tag. `enable_name` sets it back to true. FastMCP matches every specified criterion, and a later mark wins. `visible` keeps the tools that are still enabled.

`prompts_as_tools` pushes `list_prompts` and `get_prompt`. `resources_as_tools` pushes `list_resources` and `read_resource`. Each of those methods sets `list_changed`.

`tests/transform.rs` checks the prefix, the rename, the override, and the four generated tools. Chapter 17's tests still pass in this copy. One hundred five tests passed: the previous 100, plus one unit test and these four.

## 18.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch18-1-fastmcp-core
cargo run -p ch18-1-fastmcp-core --example demo
cargo bench -p ch18-1-fastmcp-core --bench transform -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
name: math_add
visible: add
prompts: list_prompts, get_prompt
changed: true
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
filter_10k              time:   [126.21 µs 127.91 µs 129.25 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `filter_10k` | 127.2 µs | 125.9 µs | 129.2 µs | < 1 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `filter_10k` | `visible` over 10,000 tools, allocating the kept names | Building the catalog and `disable_tag` | Even indexes tagged `public`, odd indexes tagged `internal` and already disabled | No socket and no notification | Well under 1 ms. About 127 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The numbers below are `median.point_estimate` and the median confidence interval from `estimates.json` on [run 38962377031](https://github.com/agnathan/rust-mcp/actions/runs/38962377031) (2026-10-16T20:27:32Z). GitHub's `github.sha` for that pull request run is the merge commit `e6badb1977bb36d37706a85e7102d1e9f27f071e`. A push to `main` appends a later run to the dashboard sources in `benchmarks/`. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `filter_10k` | 137.9 µs | 137.8 µs | 138.1 µs | < 1 ms |

On the Ubuntu runner, a filtered list of 10,000 tools is 137.9 µs. That is the median this chapter quotes. The local table above is the machine where the command transcript was captured.

### Milestone demo

`fastmcp-rs/` grows a `namespaced_tool` stub. `tests/ch18.rs` expects `"math_add"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch18-transforms.rs`](playground/ch18-transforms.rs) prints the demo lines. The playground does not have this book's crate.

[Run the prefix on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=153233b28ae0d02b7640934abdb674a8).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
name: math_add
visible: add
prompts: list_prompts, get_prompt
changed: true
```

## Recap

`namespace_name` returns a `Cow`. `namespace_uri` inserts the prefix after the scheme. A rename changes one tool and one argument key. A tag disable hides tools, and a later enable puts one back. Prompts and resources can appear as tools. `list_changed` records that the list moved.

`cargo run -p ch18-1-fastmcp-core --example demo` prints the four lines above. Locally, a filtered list of 10,000 tools takes 127.2 µs.

[personal note: the prefix I applied twice, and the name that became `math_math_add`]

## What's next

Chapter 19 searches that catalog. A BM25 index ranks tools by the words in their names and descriptions. A small script then calls the tools it found, with a limit on how many calls it may make.
