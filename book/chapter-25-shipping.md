# Chapter 25: Shipping it

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

The server from the last chapter can parse a command. It still cannot be handed to someone else as a crate. Publishing is the step that turns the package metadata into a version other people can depend on, and a release is the step that turns that version into binaries for more than one machine.

This chapter does not upload anything. It checks the metadata, names the archives, and times one tool call against the Python server this book has been reading.

## What you'll build

A dry-run publish plan, three archive names, and the result of `add(2, 3)`.

```text
version: 0.1.0
command: cargo publish --dry-run
targets: 3
rust: 5
```

The snapshot is `steps/ch25-1-shipping`. It is a full copy of chapter 24, plus `shipping.rs`. The crates are `ch25-1-fastmcp-core` and `ch25-1-fastmcp-macros`.

```bash
cargo run -p ch25-1-fastmcp-core --example demo
```

## Goals

- Read a `MAJOR.MINOR.PATCH` version with the `semver` crate, and refuse `0.0.0`.
- Plan `cargo publish --dry-run` without running it.
- Name three release archives the way a cargo-dist layout does.
- Time one in-process `add` call, and put that time next to Python FastMCP on the same machine.

## Roadmap

**25.1 A version you can publish.** The problem is a package that still says `0.0.0`. That string is a placeholder. The plan refuses it.

**25.2 Archives and the Python call.** The problem is knowing what a release would ship, and whether the Rust call is in the same neighborhood as FastMCP. The archives are names. The call is real.

**25.3 Build, run, and benchmark.** The tests, the demo above, the Rust median, and the Python transcript.

| Section | Step | What it adds |
|---|---|---|
| 25.1–25.3 | `steps/ch25-1-shipping` | `parse_release`, `publish_plan`, `release_archives`, `call_add` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612).
>
> **Adopted.** The root [`pyproject.toml`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/pyproject.toml) names the project `fastmcp` (line 2), sets `license` to `Apache-2.0` (line 7), and points Homepage, Repository, and Documentation at `https://gofastmcp.com` and the GitHub repo (lines 32–34). The version is dynamic. `[tool.uv-dynamic-versioning]` uses `pep440` (line 68) and `fallback-version = "0.0.0"` (line 70). The build backend is hatchling (lines 37–38). [`fastmcp_slim/pyproject.toml`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/pyproject.toml) repeats the Apache-2.0 license (line 18) and installs the `fastmcp` console script (line 48). [`server/server.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/server.py) `call_tool` (line 1357) runs a tool by name and takes `run_middleware`.
>
> **Adapted.** `Package::book` holds the name `fastmcp-core`, version `0.1.0`, license `Apache-2.0`, and rust-version `1.88`. `publish_plan` returns the string `cargo publish --dry-run` and the docs.rs cfg `--cfg docsrs`. It refuses `0.0.0`, an uppercase crate name, and an empty license or description. `release_archives` names three host archives and does not build them. `call_add` is `dispatch_tool("add", {"a": 2, "b": 3})`. The Python script calls `call_tool` with `run_middleware=False`, so both sides skip the hook chain.
>
> **Deferred.** Running `cargo publish`, a docs.rs build, installing cargo-dist, and uploading archives. This checkout of FastMCP has no `.github` workflows directory, so a release workflow is not cited. The step crate stays `publish = false`.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **The dry run is a string.** `publish_plan` does not spawn `cargo`. The step crate's manifest says `publish = false`, so this directory is not the one you upload. Copy the metadata into a crate that is allowed to publish, then run the command yourself.

[personal note: a version I almost tagged 0.0.0 because the fallback string looked like a real release]

## 25.1 A version you can publish

`semver::Version::parse` accepts `1.2.3` and also `1.2.3-alpha`. A release this chapter will plan is only `MAJOR.MINOR.PATCH`. `0.0.0` parses, and then the plan refuses it. That is FastMCP's fallback when git cannot see a tag. It is not a version to upload.

```rust
pub fn publish_plan(package: &Package) -> Result<PublishPlan, String> {
    let version = parse_release(&package.version)?;
    if version.major == 0 && version.minor == 0 && version.patch == 0 {
        return Err("refusing to publish 0.0.0".to_string());
    }
    Ok(PublishPlan {
        command: PUBLISH_COMMAND.to_string(),
        docs_rs: docs_rs_cfg().to_string(),
    })
}
```

`PUBLISH_COMMAND` is `cargo publish --dry-run`. `docs_rs_cfg` is `--cfg docsrs`, the flag docs.rs sets so a crate can hide items from the rendered docs. The function returns those strings. It does not start a process.

The tests ask for a plan of `0.1.0` and get the dry-run command, then ask for `0.0.0` and get `refusing to publish 0.0.0`. An uppercase name is refused with `crate name must be lowercase`.

## 25.2 Archives and the Python call

cargo-dist names an archive with the package, the version, and the target triple. Linux and macOS get `.tar.gz`. Windows gets `.zip`. This chapter builds the three names and stops.

```rust
pub fn release_archives(name: &str, version: &str) -> Vec<String> {
    vec![
        format!("{name}-{version}-x86_64-unknown-linux-gnu.tar.gz"),
        format!("{name}-{version}-aarch64-apple-darwin.tar.gz"),
        format!("{name}-{version}-x86_64-pc-windows-msvc.zip"),
    ]
}
```

The call that the benchmark times is the book's own server. `add_server` registers `add`. `call_add` passes `a` 2 and `b` 3. The value that comes back is the JSON number `5`. Hooks from chapter 16 are not on `dispatch_tool`.

The Python side is `steps/ch25-1-shipping/python/call_add.py`. It builds a `FastMCP("widgets")`, registers the same `add`, and calls `call_tool` with `run_middleware=False`. Ten samples of 1,000 calls, after 50 warmup calls. The median sample, divided by 1,000, is one call.

## 25.3 Build, run, and benchmark

129 tests passed: the previous 124, plus these 5. `a_release_version_plans_a_dry_run` checks the command and the license. `zero_version_is_refused` and `an_uppercase_name_is_refused` check the two refusals. `three_archives_name_the_host` checks the Linux and Windows suffixes. `add_returns_five` checks the JSON number `5`.

```bash
cargo test -p ch25-1-fastmcp-core
cargo run -p ch25-1-fastmcp-core --example demo
cargo bench -p ch25-1-fastmcp-core --bench shipping -- --warm-up-time 1 --measurement-time 2 --sample-size 10
python3 steps/ch25-1-shipping/python/call_add.py
```

The demo prints:

```text
version: 0.1.0
command: cargo publish --dry-run
targets: 3
rust: 5
```

The Python script needs the `server` extra of `fastmcp-slim` at commit `e2fcc412`. On this machine that interpreter printed:

```text
python: 5
middleware: off
per_call_ns: 159518.6
low_ns: 151279.6
high_ns: 178212.3
samples: 10
loops: 1000
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
rust_call               time:   [97.885 ns 98.409 ns 99.084 ns]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `rust_call` | 98.31 ns | 97.26 ns | 98.71 ns | < 1 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `rust_call` | One `dispatch_tool("add")` that deserializes `{a, b}` and returns the sum | Building the `FastMcp` and registering `add` | `a` is 2 and `b` is 3 | No socket, no publish, no Python | Well under 1 µs. About 98 ns here. |

The Python number is not a Criterion median. Ten samples of 1,000 calls, middleware off, median sample per call: 159.5 µs (151.3 µs to 178.2 µs). Dividing that median by the Rust median is 1,623. Both calls returned 5. The CI table below is the Rust call only. The runner does not install FastMCP.

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `rust_call` | pending | pending | pending | < 1 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `publish_command` stub. `tests/ch25.rs` expects `"cargo publish --dry-run"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch25-shipping.rs`](playground/ch25-shipping.rs) prints the demo lines. The playground does not have this book's crate, and it does not have the `semver` crate. The addition is `2 + 3`.

[Playground: ch25 shipping](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=0a461e7f5cb4f1771f981ce576897d68)

The playground printed:

```text
version: 0.1.0
command: cargo publish --dry-run
targets: 3
rust: 5
```

## Recap

`parse_release` keeps `MAJOR.MINOR.PATCH`. `publish_plan` refuses `0.0.0` and returns `cargo publish --dry-run` without running it. `release_archives` names three hosts. `call_add` returns 5.

`cargo run -p ch25-1-fastmcp-core --example demo` prints the four lines above. Locally, one Rust call is 98.31 ns. The Python call on this machine, middleware off, is 159.5 µs.

[personal note: a dry run I treated as an upload because the command still said publish]

## What's next

Appendix A is the interactive app: a tool the model calls to open a view, and tools the view calls that the model does not see. The shipping steps stop at the plan and the comparison.
