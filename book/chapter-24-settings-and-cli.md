# Chapter 24: Settings and CLI

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A server needs a host and a log level before it listens, and those values should come from the environment or a file, not from a constant buried in `main`. FastMCP reads `FASTMCP_` variables and an env file. The command line then decides what to do: run, reload while developing, inspect the tools, or write an install snippet.

Inspect should not open a socket. It answers “what tools does this server have?” and stops. Dev is the same server with reload turned on.

## What you'll build

Defaults, the inspect command, and the built-in tool name.

```text
host: 127.0.0.1
level: INFO
command: inspect
tools: status
```

The snapshot is `steps/ch24-1-cli`. It is a full copy of chapter 23, plus `settings.rs`. The crates are `ch24-1-fastmcp-core` and `ch24-1-fastmcp-macros`.

```bash
cargo run -p ch24-1-fastmcp-core --example demo
```

## Goals

- Load a host and a log level from defaults, then a JSON file, then `FASTMCP_` environment variables.
- Keep the first successful load in a `OnceLock`.
- Parse `run`, `dev`, `inspect`, and `install`.
- `inspect` lists tool names and does not bind a port. `dev` sets reload. `install` returns a JSON snippet.

## Roadmap

**24.1 Settings.** The problem is three places a host can come from. The file overrides the default. The environment overrides the file.

**24.2 Commands.** The problem is a developer who wants the tool list without starting the server. `inspect` is that command.

**24.3 Build, run, and benchmark.** The tests, the demo above, and a cold parse of `inspect`.

| Section | Step | What it adds |
|---|---|---|
| 24.1–24.3 | `steps/ch24-1-cli` | `load_settings`, `plan`, `install_json`, `SETTINGS_ENV_PREFIX` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`settings.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/settings.py) `Settings` (line 34) uses `env_prefix` `FASTMCP_` (line 38), an env file, and `extra="ignore"` (line 40). [`cli/cli.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/cli/cli.py) `run` (line 410) takes `--transport` / `-t` and `--host`, and the help text says the http default host is `127.0.0.1` (line 424). `inspect` (line 795) takes `--format` / `-f`. `dev_app` (line 141) is the `dev` group, and `inspector` (line 145) is `fastmcp dev inspector`. [`cli/run.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/cli/run.py) `load_mcp_server_config` (line 124) reads a config file. `run_with_reload` (line 384) is the reload path. [`cli/install/mcp_json.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/cli/install/mcp_json.py) `install_mcp_json` (line 21) writes an MCP JSON snippet.
>
> **Adapted.** `load_settings` merges defaults, an optional JSON file, and `Env::prefixed("FASTMCP_")`. Unknown keys are not fields on `Settings`, so they are not applied. `plan` uses clap. `run` returns the host and does not bind. `dev` sets `reload` to true. `inspect` calls `tool_names` on a new `FastMcp` and does not listen. `install` returns JSON with `mcpServers`, the name, command `fastmcp`, and args `["run"]`. `cached_settings` is a `OnceLock`. The cold-start bench calls `plan` directly, so the cache is not on that path.
>
> **Deferred.** The Cyclopts app, `fastmcp dev inspector`, editable installs, clipboard copy, transport selection, and actually binding `127.0.0.1`. Reload is a flag. The file watcher is not started.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **Inspect does not listen.** `plan(&["inspect"], None)` returns the tool name `status` and `reload` is false. The test does not open a socket. `dev` is the command that sets reload.

[personal note: a `FASTMCP_HOST` I set in the shell and then could not explain why the file was ignored]

## 24.1 Settings

```rust
figment
    .merge(Serialized::defaults(Settings::default()))
    .merge(Json::file(path))
    .merge(Env::prefixed(SETTINGS_ENV_PREFIX))
```

The default host is `127.0.0.1` and the default level is `INFO`. A JSON file replaces those. `FASTMCP_HOST` replaces the file's host and leaves the file's log level alone when that variable is unset.

## 24.2 Commands

`plan` parses the arguments with clap. `inspect` builds an in-memory server and joins `tool_names`. The only name on a new server is `status`. `install_json` is the snippet a client config would store.

`tests/settings.rs` checks the default, the file, the environment override, inspect, dev's reload flag, and the install snippet. Chapter 23's tests still pass in this copy. One hundred twenty-four tests passed: the previous 122, plus these two.

## 24.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch24-1-fastmcp-core
cargo run -p ch24-1-fastmcp-core --example demo
cargo bench -p ch24-1-fastmcp-core --bench settings -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
host: 127.0.0.1
level: INFO
command: inspect
tools: status
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
cold_start              time:   [8.3540 µs 8.3950 µs 8.4398 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `cold_start` | 8.384 µs | 8.352 µs | 8.442 µs | < 50 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `cold_start` | clap parse of `inspect`, figment load of the defaults, and `tool_names` on a new server | The `OnceLock` cache. It is not called here | The argument list `inspect`. No config file | No socket and no reload watcher | Well under 50 µs. About 8.4 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `cold_start` | pending | pending | pending | < 50 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `settings_env_prefix` stub. `tests/ch24.rs` expects `"FASTMCP_"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch24-settings.rs`](playground/ch24-settings.rs) prints the demo lines. The playground does not have this book's crate, and it does not parse arguments. The crate uses clap and figment.

[Run the lines on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=869357da234959de56d0a275bcdcf5a9).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
host: 127.0.0.1
level: INFO
command: inspect
tools: status
```

## Recap

`load_settings` merges defaults, a JSON file, and `FASTMCP_` variables. `plan` parses `run`, `dev`, `inspect`, and `install`. Inspect lists `status` and does not listen. Dev sets reload. Install returns a JSON snippet. `cached_settings` stores the first load.

`cargo run -p ch24-1-fastmcp-core --example demo` prints the four lines above. Locally, one cold `inspect` takes 8.384 µs.

[personal note: an inspect command that bound a port because I reused the run path]

## What's next

Chapter 25 is the shipping chapter: a small server you can point a client at, with the pieces from this part of the book in one place. The steps so far stop at the command parser.
