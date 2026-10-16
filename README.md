# rust-mcp

[![CI](https://github.com/agnathan/rust-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/agnathan/rust-mcp/actions/workflows/ci.yml)

*Learning Rust by Building FastMCP.* A Rust MCP framework, with a book on how to build it step by step.

This is a hands-on course for people **new to Rust**. You learn the language by growing one server, not by a tour of the language. The design is informed by Prefect's [FastMCP](https://github.com/PrefectHQ/fastmcp) (Python). Each chapter points at the FastMCP files that solve the same problem.

## Layout

```
steps/chNN-<slug>/   finished code at the end of that section (cumulative copies)
fastmcp-rs/          your crate; the milestone test starts out failing
book/                mdBook source and social drafts
benchmarks/          dashboard sources (data.json, descriptions.json, index.html)
```

A step is the code as of that section, with its tests. Later chapters are new copies under `steps/`, so chapter 1 still builds after chapter 2 exists. They share one cargo workspace at the repo root (`resolver = "3"`). Package names carry the section (`ch01-1-fastmcp-core`). The library each test imports is still `fastmcp_core`.

`fastmcp-rs/` is where the implementation happens. `tests/ch01.rs` compiles, then panics with `todo!()` until you replace the stub. The finished snapshot stays in `steps/`. There is no separate solutions directory.

## How to start

1. Install Rust with [rustup](https://rustup.rs). You need 1.85 or newer (`rustc --version`).
2. Add the usual components: `rustup component add rustfmt clippy`.
3. Read [Chapter 1](book/chapter-01-speaking-json-rpc.md). Run the finished step:

   ```bash
   cargo test -p ch01-1-fastmcp-core
   cargo run -p ch01-1-fastmcp-core --example demo
   ```

4. The exercise crate fails until you fill it in:

   ```bash
   cargo test -p fastmcp-rs --test ch01
   ```

Chapters 0 through 20. Chapter 1 is written. Later rows are the plan in [book/book-outline.md](book/book-outline.md), not empty crates. The same list is `.github/chapters.json`.

| Ch | Title | Step | You add |
|---|---|---|---|
| 0 | Introduction | — | what MCP and FastMCP are, how steps and tags work |
| 1 | Speaking JSON-RPC | `steps/ch01-1-jsonrpc` | JSON-RPC 2.0, `initialize`, `ping` |
| 2 | First server over stdio | `steps/ch02-stdio` | not written |
| 3–20 | See the outline | — | not written |

After chapter 1 merges to `main`, tag that merge commit `step-01`. The tag is not pushed from the chapter pull request. See [docs/tags.md](docs/tags.md).

The book builds with [mdBook](https://rust-lang.github.io/mdBook/): `mdbook build book` (output in `book/_build/`, not committed).

## Continuous integration

[CI](https://github.com/agnathan/rust-mcp/actions/workflows/ci.yml) runs on every push to `main` and on pull requests:

- `cargo fmt --check` and `cargo clippy --workspace --all-targets` (warnings do not fail the build)
- `cargo test` for every step, each step as its own job, on Ubuntu, driven by `.github/scripts/list_steps.py`
- the same tests on Ubuntu and Windows for steps whose files changed
- macOS for those steps, at most four jobs at a time, and one automatic retry when the first wave does not succeed
- an mdBook build of `book/`

[Benchmarks](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml) run on pull requests and on pushes to `main`. They execute the Criterion benches on Ubuntu. The job summary prints the Criterion median with its low and high. A push to `main` appends that run to [`benchmarks/data.json`](benchmarks/data.json) on the `gh-pages` branch and refreshes the dashboard (`benchmarks/index.html`). A slow result does not fail the run, and the bench job does not fail the workflow.

`fastmcp-rs` milestone tests are not part of that required build.

## License

[MIT](LICENSE) © 2026 Daniel Holmlund
