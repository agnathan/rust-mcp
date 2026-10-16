# Chapter 6: Resources and templates

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A tool answers a call. A resource is something the client reads by URI: a readme, a license, a row in a catalog. FastMCP registers both. A URI with no braces is a static resource. A URI with `{owner}` in it is a template, and the server fills the body from the text that landed in that slot.

This chapter adds both to the server from chapter 5. The interesting Rust is the match. The parameter name has to borrow the pattern, and the captured text has to borrow the URI the client sent. Those are two different lifetimes. The function that renders the body cannot hold either borrow, because that function lives on the server and the request does not.

## What you'll build

One exact URI, and one template that pulls two names out of a path.

```text
readme: hello
repo: acme/widgets
```

`file://readme` is stored as the string `hello`. `res://acme/widgets` matches `res://{owner}/{repo}` and the template function returns `acme/widgets`. A URI that matches nothing is `invalid_params`. A template function that returns `Err` is an internal error.

The snapshot is `steps/ch06-1-resources`. It is a full copy of chapter 5, plus `resources.rs`. The crates are `ch06-1-fastmcp-core` and `ch06-1-fastmcp-macros`. The library names stay `fastmcp_core` and `fastmcp_macros`.

```bash
cargo run -p ch06-1-fastmcp-core --example demo
```

## Goals

- Register a static resource and a `{param}` template on `FastMcp`.
- Match a URI so the parameter name borrows the pattern and the value borrows the URI.
- Explain why the server copies those borrows into `String`s before it calls the template.
- List and read the resources over the same duplex session as the tools.
- Read the two benchmarks: one `HashMap` lookup among 10,000 static URIs, and one scan of 10,000 templates.

## Roadmap

**6.1 The match, and the two borrows.** The problem is a URI with a hole in it. `match_template` walks the pattern. This is the lifetime lesson.

**6.2 Register, list, and read.** Exact URIs live in a `HashMap`. Templates are a list, and the first match wins. `resources/list`, `resources/templates/list`, and `resources/read` are the wire methods.

**6.3 Build, run, and benchmark.** The tests, the demo above, and the two timings.

| Section | Step | What it adds |
|---|---|---|
| 6.1–6.3 | `steps/ch06-1-resources` | Static resources, `{param}` templates, `resources/read` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`resources/template.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/resources/template.py) `build_regex` (near line 160) turns `{var}` into one path segment, `[^/]+`. `{var*}` is a wildcard. `{?query}` is stripped before the path match. `match_uri_template` (near line 197) drops a `#fragment` when the template itself has no `#`, then matches the path. [`server/providers/local_provider/decorators/resources.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/providers/local_provider/decorators/resources.py) `resource` (near line 111) is the decorator: a URI with parameters becomes a template, and a URI without them becomes a static resource. [`resources/function_resource.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/resources/function_resource.py) `resource` (near line 223) is the standalone form of that decorator. It returns the function with metadata attached.
>
> **Adapted.** This chapter's matcher is a small scanner, with no `regex` crate and no cache. `{var*}` and `{?query}` fail the match, so a wildcard never silently captures a slash. The server copies each capture into a `String` before the template function runs. FastMCP's regex groups are already owned strings by the time the function sees them. Listing sorts URIs so the order is stable. An exact URI is a `HashMap` get. A second registration of the same URI replaces the first, the same rule as a tool name.
>
> **Deferred.** Hyphen-to-underscore parameter names, query parameters, `{path*}` wildcards, and the path-security checks in `resources/security.py` wait. Pagination of `resources/list` is chapter 7.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> **An unknown URI is `invalid_params`.** That is the same code an unknown tool name already uses. The message says `unknown resource`. A template function that returns `Err` is `internal_error`. Chapter 8 is where the full exception map from W4 lands. This chapter only splits those two resource outcomes.
>
> **`{param}` is one segment.** `file://{name}` matches `file://readme` and does not match `file://a/b`. A wildcard that swallowed slashes would hand the template a path it did not ask for.

[personal note: a URI I matched wrong the first time, and what the captures actually held]

## 6.1 The match, and the two borrows

You have a pattern, `res://{owner}/{repo}`, and a URI the client sent, `res://acme/widgets`. You want the pairs `("owner", "acme")` and `("repo", "widgets")`. The names are slices of the pattern. The values are slices of the URI. If both borrows used one lifetime, the compiler would tie the names to the request, or the values to the pattern, and one of those strings would have to outlive the other for no reason.

`match_template` says so in the signature:

```rust
pub fn match_template<'p, 'u>(
    pattern: &'p str,
    uri: &'u str,
) -> Option<Vec<(&'p str, &'u str)>>
```

`'p` is the pattern's lifetime. `'u` is the URI's. The `Vec` can hold both because each pair names them separately. A fragment is dropped when the pattern has no `#`, which is what FastMCP's matcher does. A query string is dropped before the walk. `{name}` then captures until the next `/` or until the following literal. An empty capture fails. A capture that contains `/` fails. `{name*}` and `{?q}` fail, so those forms are visible as "not this chapter" rather than as a surprising match.

The template function is stored as `Arc<dyn Fn(&[(String, String)]) -> Result<String, String> + Send + Sync>`. That function outlives the request. It cannot borrow the URI. `UriTemplate::render` copies each capture into a `String`, then calls the function. The copy is the point of the lifetime split: the match is cheap and borrowed, and the value the server keeps is owned.

## 6.2 Register, list, and read

`resource` inserts into a `HashMap` keyed by the URI. `template` pushes onto a `Vec`. `read` checks the map first. If the URI is absent, it walks templates in registration order and uses the first pattern that matches.

```rust
pub fn read(&self, uri: &str) -> Result<(String, String), ToolError> {
    if let Some(resource) = self.resources.get(uri) {
        return Ok((resource.body.clone(), resource.mime.clone()));
    }
    for template in &self.templates {
        if let Some(rendered) = template.render(uri) {
            return rendered
                .map(|body| (body, template.mime.clone()))
                .map_err(ToolError::Failed);
        }
    }
    Err(ToolError::Unknown)
}
```

The `Ok` pair is the body and the MIME type. The demo registers the readme and the repo template, then prints both bodies.

`initialize` now advertises resources as well as tools: `ServerCapabilities::builder().enable_tools().enable_resources()`. `list_resources` and `list_resource_templates` sort by URI so two runs agree. `read_resource` turns a hit into `ResourceContents::text`. `ToolError::Unknown` becomes `ErrorData::invalid_params`. `ToolError::Failed` becomes `ErrorData::internal_error`.

Tools still work. `status` is still first in `tools/list`. The macro from chapter 5 still registers `add`.

## 6.3 Build, run, and benchmark

`tests/resources.rs` speaks `resources/list`, `resources/templates/list`, and `resources/read` over the duplex pipe. The static URIs come back sorted. `res://acme/widgets` renders `acme/widgets`. `file://missing` is `invalid_params`. `fail://readme` hits a template that returns `Err` and is `internal_error`. The matcher unit tests cover one segment, two segments, a slash that does not fit, and a fragment that is ignored. Chapter 5's tests still pass in this copy. Forty-seven tests passed: the previous 39, plus these eight.

```bash
cargo test -p ch06-1-fastmcp-core
cargo run -p ch06-1-fastmcp-core --example demo
cargo bench -p ch06-1-fastmcp-core --bench resources -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
readme: hello
repo: acme/widgets
```

The slope lines, on the same 4-vCPU guest as chapter 1:

```text
resource_lookup_10k    time:   [38.701 ns 38.924 ns 39.182 ns]
template_scan_10k       time:   [259.95 µs 262.02 µs 264.90 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `resource_lookup_10k` | 38.75 ns | 38.69 ns | 39.14 ns | < 1 µs |
| `template_scan_10k` | 264.2 µs | 260.0 µs | 266.8 µs | < 1 ms |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `resource_lookup_10k` | One `read` of `file://doc-5000` on a server that already holds 10,000 static resources | Building those 10,000 resources. No template walk, because the URI hits the map. | URIs `file://doc-0` through `file://doc-9999`, body `body` | No socket | Well under 1 µs. About 39 ns here, one `HashMap` get and two `String` clones. |
| `template_scan_10k` | One `read` of `res://item-9999/readme`. The pattern that matches is registered last, so the scan walks all 10,000 templates, copies the capture, and runs the closure. | Building the 10,000 templates | Patterns `res://item-0/{name}` through `res://item-9999/{name}` | No socket. The closure returns `"found"` and does no I/O. | Well under 1 ms. About 264 µs here. The cost is the scan, then one copy. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `resource_lookup_10k` | pending | pending | pending | < 1 µs |
| `template_scan_10k` | pending | pending | pending | < 1 ms |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `static_resource_uri` stub. `tests/ch06.rs` expects `"file://readme"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch06-resources.rs`](playground/ch06-resources.rs) is the matcher on its own: two lifetimes, one pattern, one URI. The playground does not have this book's crate.

[Run the matcher on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=396501c631a32dbab59ad198f4f0ae6d).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
owner=acme
repo=widgets
```

## Recap

A static resource is a URI in a `HashMap`. A template is a pattern with `{param}` holes, matched so the name borrows the pattern and the value borrows the URI, then copied into `String`s before the function runs. `resources/read` returns the body. An unknown URI is `invalid_params`. `{param}` captures one segment.

`cargo run -p ch06-1-fastmcp-core --example demo` prints `hello` and `acme/widgets`. Locally, one lookup among 10,000 static resources is 38.75 ns, and a scan of 10,000 templates is 264.2 µs.

[personal note: the moment the two lifetimes clicked, or the error that made them obvious]

## What's next

Chapter 7 adds prompts, argument completion, and a cursor. `list_*` can no longer pretend the whole catalog fits in one response.
