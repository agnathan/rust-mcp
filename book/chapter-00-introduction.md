# Chapter 0: Introduction

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

This book is a course in Rust for developers who are new to the language. The program we build is a Rust version of [FastMCP](https://github.com/PrefectHQ/fastmcp), Prefect's Python framework for the [Model Context Protocol](https://modelcontextprotocol.io/). You learn the language by growing one server. A Rust idea shows up in the chapter that first needs it, in a short sidebar, and not before.

[personal note: what got me interested in MCP and in Rust]

### What MCP is

MCP is a way for a program that has tools, files, or prompts to talk to a client. The client is often a language-model application. The conversation on the wire is JSON-RPC 2.0. A session starts with `initialize`. Either side can `ping`. After that the client lists tools, calls one, reads a resource, or asks for a prompt.

The protocol is the envelope and the lifecycle. A useful server is the registry of tools, the schemas, the errors, the auth, and the transport. Those are the framework.

FastMCP 4 speaks the current protocol, `2026-07-28`, and older handshake-era clients, from one deployment. It negotiates the version per connection. This book teaches that in three steps, which are not three chapters in a row: chapter 1 hand-rolls the handshake-era `initialize` exchange and pins it to `2025-06-18`, chapter 2 negotiates a version and moves the transport onto the Rust SDK `rmcp`, and chapter 12 makes that negotiation work per connection over HTTP.

### What FastMCP is

FastMCP's package lives in `fastmcp_slim/fastmcp/`. A server in its README is a name, a function, and `run`:

```python
from fastmcp import FastMCP

mcp = FastMCP("Demo 🚀")

@mcp.tool
def add(a: int, b: int) -> int:
    """Add two numbers"""
    return a + b

if __name__ == "__main__":
    mcp.run()
```

That decorator hides the JSON-RPC. This book starts under it. The Rust we write keeps the idea of a small server you build by registering functions, and it uses `rmcp` for the protocol types and the transports from chapter 2 on. The chapters are about the parts that make FastMCP FastMCP: the builder, the macro, providers, transforms, middleware, and the rest of the [outline](book-outline.md).

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612) (v4.1.0, 2026-10-08). Paths are under `fastmcp_slim/fastmcp/` unless the link says otherwise.
>
> **Adopted.** The picture of the finished server is the [README](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/README.md) example: construct `FastMCP` with a name, register a function, call `run`. [`server/server.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/server.py) is that class. `FastMCP.tool` is the method the `@mcp.tool` decorator calls. This chapter writes no Rust for it. The shape is the target.
>
> **Deferred.** Registering a tool by hand is chapter 3. A `#[tool]` macro is chapter 5. Starting a process, negotiating the protocol version, and speaking stdio are chapter 2. This chapter has no wire rules to enforce.

### Who this book is for

Read in order and type the steps if Rust is new. Chapter 1 uses cargo, structs and enums, serde, `Result` and `?`, and `#[test]`. Async, traits, and macros wait until a later chapter has a reason to introduce them.

If you already know Rust and want the framework, the "How FastMCP does it" box in each chapter names the Python files. The code under `steps/` is the finished snapshot. `fastmcp-rs/` is the crate you fill in.

## Overview

There are 26 chapters, numbered 0 through 25, plus three appendices. The full table, with the FastMCP files and the Rust each chapter introduces, is [book-outline.md](book-outline.md). The tour below is the reading order, in arcs.

**The wire.** Chapter 1 builds a JSON-RPC message by hand and answers `initialize` and `ping`. Chapter 2 accepts a range of protocol versions, chooses one, and switches the transport to `rmcp` over stdio.

**Things a server holds.** Chapters 3 through 8 add the `FastMcp` builder and a tool registry, JSON Schema from types, a `#[tool]` macro, resources and URI templates, prompts with completion and pagination, and the split between a tool error and a protocol error.

**What a tool can see.** Chapters 9 through 11 give a tool a context (logging, progress, sampling, request state), inject dependencies that stay out of the tool schema, and run startup and shutdown around shared resources.

**Both protocol eras on the network.** Chapter 12 serves streamable HTTP, negotiates the version per connection, and keeps sessions. That is the third step of the protocol arc. Chapter 13 asks the user for structured input in the middle of a call.

**Clients and tests.** Chapter 14 is a client and a group of clients. Chapter 15 is an in-memory transport and the test harness, so later work can be checked without a subprocess every time.

**A server made of parts.** Chapters 16 through 20 add a middleware chain, providers (local, mounted, proxy, filesystem), transforms and visibility, tool search and a sandboxed code mode, and background tasks through an extension.

**Running it for someone else.** Chapters 21 through 25 generate tools from an OpenAPI spec, verify JWTs, add authorization, storage, and telemetry, load settings and a CLI, and publish a release. Appendix A is apps, Appendix B is the OAuth proxies and the full OAuth server, and Appendix C is icons and metadata.

### Steps

A step is a directory under `steps/`. It is a full copy of the code at the end of that section, not a patch on the previous directory, so chapter 1 still builds after chapter 2 exists. Every step crate is a member of one cargo workspace at the repo root (`resolver = "3"`). Package names carry the section (`ch01-1-fastmcp-core`) so two steps can be built together. The library name stays `fastmcp_core`, which is what the tests import.

`fastmcp-rs/` is where the exercises go. Its chapter 1 test compiles and then panics in `todo!()` until you replace the stub. `steps/` is the finished snapshot. There is no solutions directory beyond those snapshots.

```
steps/chNN-<slug>/   the code at the end of that section
fastmcp-rs/          the crate you fill in
book/                this mdBook, plus playground files and social drafts
```

Only `steps/ch01-1-jsonrpc` exists so far.

### Tags

When a chapter's pull request merges, the merge commit on `main` gets an annotated tag `step-01`, `step-02`, and so on. The tag is pushed after the merge, not from the chapter branch. The commands for the current tag are in [docs/tags.md](../docs/tags.md). `step-01` is not created yet.

### Benchmarks and the CI dashboard

A chapter that adds work gets a Criterion bench. The timer covers the work the chapter is about, and it leaves out the network. Chapter 0 has no bench. Chapter 1 is the first.

On every pull request and every push to `main`, the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml) runs the benches on Ubuntu. The number this book quotes is the median, `median.point_estimate` in Criterion's `estimates.json`. The `time:` line Criterion prints is the slope, which is a different statistic. The workflow writes the median and its low and high into the job summary and uploads them as an artifact. A slow result does not fail the build, and a bench crash does not fail the workflow.

A pull request stops there. A push to `main` appends that run to `benchmarks/data.json` on the `gh-pages` branch and refreshes the dashboard, [benchmarks/index.html](../benchmarks/index.html). The page is a chart of those medians over time. What each bench includes, and what a good result looks like, is [docs/performance.md](../docs/performance.md).

### Playground links

Some chapters include one file under `book/playground/` that you can run in the [Rust Playground](https://play.rust-lang.org/) without cloning the repo. The file uses only crates the playground already has. Chapter 1's file uses `serde` and `serde_json`. The chapter links a gist, and the link in that chapter was executed on the playground before it was published. Chapter 0 has no program, so it has no playground link.

### Toolchain

Install Rust with [rustup](https://rustup.rs). The crates in this repo set `rust-version` to 1.85, so the compiler has to be at least that. Current stable is enough. Add the formatters, and install mdBook if you want to build the HTML locally:

```bash
rustup component add rustfmt clippy
cargo install mdbook
```

From the repo root, the book build and the first test are:

```bash
mdbook build book
cargo test -p ch01-1-fastmcp-core
```

`mdbook build` writes HTML to `book/_build/`, which is not committed. `cargo test` is chapter 1's crate. Chapter 0 has nothing to compile.

[personal note: first impressions of Rust after installing the toolchain]

### What CI checks

[CI](https://github.com/agnathan/rust-mcp/actions/workflows/ci.yml) runs on every push to `main` and on pull requests. It checks `cargo fmt`, runs clippy, builds this book with mdBook, and tests every step crate on Ubuntu. A change that touches a step also tests that step on macOS and Windows. The benchmark job above is a separate workflow. `fastmcp-rs` milestone tests are not part of the required build, so a `todo!()` in the exercise crate does not fail CI.

## Why this book exists

An MCP server sits on a long-lived connection, parses untrusted JSON, and runs tools that touch files, networks, and credentials. Three properties of Rust match that job.

**Performance.** The book measures the hot paths with Criterion instead of assuming them. Chapter 1's first number is already concrete: on the Ubuntu CI runner, the median parse of one `initialize` message is 1.558 µs, and 1,000 messages take 1.569 ms. Later chapters bench tool dispatch, HTTP, middleware, and search the same way. Chapter 25 puts the Rust server next to Python FastMCP and reports that comparison from a run, not from a slogan.

**Safety.** The compiler tracks which part of the program owns each value. A failure in this book is a `Result` the caller handles, which is the same habit as a JSON-RPC error response: the failure has a shape, and it does not tear the process down. When later chapters share a database pool or a session across tasks, the types have to be explicit about sharing. That is the kind of bug a server hits in production, and it is a bug the language can refuse to compile.

**Single binaries.** `cargo build --release` produces one executable. The machine that runs the server does not need a language runtime or a virtualenv beside it. Chapter 24 is the CLI that starts that executable. Chapter 25 is how it gets published, including release binaries.

`rmcp` (the Rust MCP SDK, 3.5.1 as of this outline) already has protocol types, a session, and stdio and HTTP transports. From chapter 2 on, this book uses those. Wrapping that crate in a thin example would speak MCP, and it would skip both halves of the title. FastMCP is the framework around the protocol: a builder, a decorator, providers, transforms, dependency injection, middleware, tasks, auth policy, and a CLI. Those are what the later chapters implement. Chapter 1 hand-rolls the envelope once so that, when `rmcp` arrives, the bytes are already familiar. The outline's note on `rmcp` is in [book-outline.md](book-outline.md). Before chapter 2 is written, that note still has an open check: which protocol versions 3.5.1 negotiates, and whether `2026-07-28` and the tasks extension are included. If they are missing, chapters 2, 12, and 20 implement those parts.

[personal note: why I wanted to write this book, in my own words]

## What's next

Chapter 1 opens one JSON-RPC message. It parses a request, stays quiet for a notification, and answers `initialize` and `ping`. There is no socket yet.
