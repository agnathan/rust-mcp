# Appendix A: Apps

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A tool the model calls is not the same thing as a tool the view calls. FastMCP's app keeps both in one provider. The model sees the entry point that opens the view. The view calls the rest through a hashed name, so a transform that renames the public list does not hide the button's target.

This appendix builds that split. It does not render a Prefab page.

## What you'll build

The extension id, the one name the model sees, the hashed backend name, and the body that name returns.

```text
extension: io.modelcontextprotocol/ui
model: show
hashed: deb87552a0f1_save
body: saved
```

The crate is `steps/apa-1-apps`, package `apa-1-apps`. It is not a copy of the server. The hash is the part that has to match FastMCP.

```bash
cargo run -p apa-1-apps --example demo
```

## Goals

- Tell a model-visible tool from an app-visible tool.
- Hash `app_name`, a NUL, and the tool name the way FastMCP does, and call `<hash>_<name>`.
- Default a `ui://` resource to `text/html;profile=mcp-app`.

## Roadmap

**A.1 Two audiences.** The problem is a save button the model should not call by its short name. The model list omits it.

**A.2 The hash and the MIME type.** The problem is finding that button after the public name changed. The hash is stable. The MIME type marks the renderer.

**A.3 Build, run, and benchmark.** The tests, the demo above, and a filter over 10,000 tools.

| Section | Step | What it adds |
|---|---|---|
| A.1–A.3 | `steps/apa-1-apps` | `McpApp`, `hash_tool`, `UI_MIME_TYPE` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`apps/config.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/apps/config.py) sets `UI_EXTENSION_ID` to `io.modelcontextprotocol/ui` (line 18). [`utilities/mime.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/mime.py) sets `UI_MIME_TYPE` to `text/html;profile=mcp-app` (line 7) and `resolve_ui_mime_type` returns that type for a `ui://` URI when no type was given (lines 23–26). [`server/providers/addressing.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/addressing.py) `hash_tool` (line 44) hashes `app_name`, a NUL, and `tool_name` with SHA-256 and keeps `HASH_LENGTH` 12 hex characters (lines 38 and 49). `hashed_backend_name` (line 73) is `<hash>_<local_name>`. [`server/providers/local_provider/decorators/tools.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/local_provider/decorators/tools.py) sets `PREFAB_RENDERER_URI` to `ui://prefab/renderer.html` (line 42).
>
> **Adapted.** [`apps/app.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/apps/app.py) `FastMCPApp` (line 149) is a `Provider`. `@app.tool` defaults visibility to `["app"]` and adds `"model"` when `model=True` (lines 222–224). `@app.ui` sets visibility to `["model"]` (line 340). This crate stores an audience enum and a string body. It does not build a Prefab component. `call_hashed` checks the 12-character prefix and returns the body.
>
> **Deferred.** Prefab components, content-security policy, iframe permissions, and rewriting hashed names as the payload leaves the server. There is no HTML renderer.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This appendix does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **The short name of a backend tool is not in the model list.** `open("save")` fails. The hashed name still reaches it. A client that only has the model list cannot call `save` by that spelling.

[personal note: a save button I left on the model list and then watched the model press it]

## A.1 Two audiences

`McpApp::ui` records a model tool. `McpApp::tool` records an app tool. `model_names` keeps the first kind. `open` returns the title for a model tool and an error for an app tool.

```rust
pub fn model_names(&self) -> Vec<&str> {
    self.tools
        .iter()
        .filter(|tool| tool.audience == Audience::Model)
        .map(|tool| tool.name.as_str())
        .collect()
}
```

The test registers `show` and `save`. The model list is `["show"]`. `open("save")` is an error.

## A.2 The hash and the MIME type

FastMCP's hash is SHA-256 of the app name, a zero byte, and the tool name. The first 12 hex characters are the prefix. `Dashboard` and `save` hash to `deb87552a0f1`. The callable name is `deb87552a0f1_save`.

```rust
pub fn hash_tool(app_name: &str, tool_name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(app_name.as_bytes());
    hasher.update([0]);
    hasher.update(tool_name.as_bytes());
    hex_prefix(&hasher.finalize(), HASH_LENGTH)
}
```

`resolve_ui_mime_type` returns `text/html;profile=mcp-app` for `ui://prefab/renderer.html`. An explicit type wins. A `file://` URI with no type returns nothing.

## A.3 Build, run, and benchmark

4 tests passed. `the_model_sees_the_ui_tool_only` checks the list and the refused short name. `the_hashed_name_calls_the_backend_tool` checks `deb87552a0f1` and the body `saved`. `a_ui_uri_defaults_to_the_app_mime` checks the extension id and the MIME default. `an_app_audience_is_not_the_model` checks the enum.

```bash
cargo test -p apa-1-apps
cargo run -p apa-1-apps --example demo
cargo bench -p apa-1-apps --bench apps -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
extension: io.modelcontextprotocol/ui
model: show
hashed: deb87552a0f1_save
body: saved
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
visible_10k             time:   [9.8925 µs 10.062 µs 10.218 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `visible_10k` | 9.924 µs | 9.820 µs | 10.29 µs | < 50 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `visible_10k` | Filtering 10,000 `(name, audience)` pairs down to the model-visible names | Building the 10,000 pairs | Even indexes are model tools, odd indexes are app tools | No hash, no renderer, no socket | Well under 50 µs. About 10 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `visible_10k` | pending | pending | pending | < 50 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `ui_extension_id` stub. `tests/apa.rs` expects `"io.modelcontextprotocol/ui"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/apa-apps.rs`](playground/apa-apps.rs) prints the demo lines. The playground does not have the `sha2` crate. The hash prefix is the one the crate computed for `Dashboard` and `save`.

[Playground: appendix A apps](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=14f24d720520222adce42036412164b6)

The playground printed:

```text
extension: io.modelcontextprotocol/ui
model: show
hashed: deb87552a0f1_save
body: saved
```

## Recap

The model list holds `show`. `save` is reached as `deb87552a0f1_save`. A `ui://` URI defaults to `text/html;profile=mcp-app`. The extension id is `io.modelcontextprotocol/ui`.

`cargo run -p apa-1-apps --example demo` prints the four lines above. Locally, filtering 10,000 tools takes 9.924 µs.

[personal note: a hash I truncated to 8 characters and then could not find the tool]

## What's next

Appendix B is the OAuth proxy, the OIDC discovery document, and an in-memory authorization server. The app crate stops at the audience split and the hash.
