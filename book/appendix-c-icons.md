# Appendix C: Icons and metadata

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A client that lists your server has a name, and it may have a picture. FastMCP takes a list of icons on the server and on each component. The picture is a URI. It is not a file the server opens.

This appendix checks that URI and builds the small JSON card a client can show. It does not fetch the image.

## What you'll build

The server name, the website, the icon source, and the theme.

```text
name: widgets
website: https://example.test
src: https://example.test/icon.png
theme: light
```

The crate is `steps/apc-1-icons`, package `apc-1-icons`.

```bash
cargo run -p apc-1-icons --example demo
```

## Goals

- Accept an icon whose `src` is `https`, `http`, or `data`, and refuse the others.
- Require each size to be `WxH` or `any`.
- Put the name, the website, and the icons that passed the check into one JSON object.

## Roadmap

**C.1 The icon.** The problem is a `javascript:` URL sitting where a picture should be. The check refuses it.

**C.2 The server card.** The problem is handing a client one object. Bad icons are dropped. The rest stay.

**C.3 Build, run, and benchmark.** The tests, the demo above, and a check of 1,000 icons.

| Section | Step | What it adds |
|---|---|---|
| C.1–C.3 | `steps/apc-1-icons` | `Icon`, `check_icon`, `ServerCard` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/server.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/server.py) `FastMCP.__init__` takes `website_url` (line 326) and `icons` (line 327). [`utilities/components.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/components.py) stores `icons` on a component (line 133), described as an optional list for user interfaces. [`utilities/inspect.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/utilities/inspect.py) copies each tool icon into the inspect payload with `model_dump` (lines 129–134).
>
> **Adapted.** An icon here has `src`, an optional MIME type, sizes, and an optional theme of `light` or `dark`. `check_icon` allows `https://`, `http://`, and `data:`. A size is `WxH` or `any`. `server_card_json` keeps the icons that pass and drops the rest. The JSON keys are `websiteUrl`, `mimeType`, and `theme`.
>
> **Deferred.** Fetching the image, sanitizing SVG, and putting the icons on the `tools/list` wire message. The inspect command from chapter 24 does not print this card.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This appendix does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **`javascript:` is not an icon.** `check_icon` returns `icon src must be https, http, or data`. A `file://` URL is the same refusal. The card builder drops those icons instead of serializing them.

[personal note: an icon URL I pointed at a file on my laptop]

## C.1 The icon

`check_icon` looks at the prefix of `src`, then at each size. `48x48` passes. `any` passes. `big` does not. The theme is an enum, so a third spelling does not construct.

```rust
let allowed = icon.src.starts_with("https://")
    || icon.src.starts_with("http://")
    || icon.src.starts_with("data:");
```

The test accepts `https://example.test/icon.png` and a `data:` URI, and it rejects `javascript:alert(1)`.

## C.2 The server card

`ServerCard` holds the name, an optional website, and the icons. `server_card_json` runs `check_icon` and skips a failure. Two icons go in, one `file://` icon, and the JSON array has length 1. The theme string is `light`.

## C.3 Build, run, and benchmark

2 tests passed. `an_https_icon_is_kept_and_a_script_src_is_not` checks the prefixes and a bad size. `the_server_card_drops_a_bad_icon` checks the name, the website, the remaining icon, and `theme` `light`.

```bash
cargo test -p apc-1-icons
cargo run -p apc-1-icons --example demo
cargo bench -p apc-1-icons --bench icons -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
name: widgets
website: https://example.test
src: https://example.test/icon.png
theme: light
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
check_1k                time:   [7.1768 µs 7.2438 µs 7.3307 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `check_1k` | 7.219 µs | 7.149 µs | 7.359 µs | < 50 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `check_1k` | `check_icon` on 1,000 icons, counting the ones that pass | Building the 1,000 icons | Each `src` is `https://example.test/icon-{n}.png`, size `48x48`, theme `light` | No fetch and no JSON | Well under 50 µs. About 7 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `check_1k` | pending | pending | pending | < 50 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows an `icon_theme_light` stub. `tests/apc.rs` expects `"light"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/apc-icons.rs`](playground/apc-icons.rs) checks the `https` prefix, refuses `javascript:`, and prints the demo lines. The playground does not have this crate.

[Playground: appendix C icons](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=88431839c82ed863f8e1ee214d439c57)

The playground printed:

```text
name: widgets
website: https://example.test
src: https://example.test/icon.png
theme: light
```

## Recap

An icon source is `https`, `http`, or `data`. A size is `WxH` or `any`. The server card keeps the icons that pass and names the website.

`cargo run -p apc-1-icons --example demo` prints the four lines above. Locally, checking 1,000 icons takes 7.219 µs.

[personal note: a data URI so long it pushed the name off the card]

## What's next

The steps stop here. The server card is the last piece this book builds. A later release can put that card on the wire. This appendix does not.
