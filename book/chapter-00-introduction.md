# Chapter 0: Introduction

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

This is the front door. The book you are holding is a course in Rust for developers who already ship software and are new to the language. The program we build, chapter by chapter, is a Rust version of [FastMCP](https://github.com/PrefectHQ/fastmcp), Prefect's Python framework for the [Model Context Protocol](https://modelcontextprotocol.io/). You learn the language by growing one server. A Rust idea shows up in the chapter that first needs it, in a short explanation beside the code, and not before.

The problem this chapter solves is orientation. MCP, FastMCP, Rust, and a 26-chapter outline are four different maps, and starting at chapter 1 without them means the first JSON object arrives with no reason to care about it. By the end of these pages you should know what the protocol is asking a server to do, what FastMCP adds on top of the protocol, who the book is written for, and how a chapter, a step crate, a benchmark, and the CI jobs fit together.

[personal note: what got me interested in MCP and in Rust]

## What you'll build

There is no server in this chapter. What you leave with is a working toolchain and a picture of the program the rest of the book grows.

The destination, in Python, is the server from FastMCP's README. A name, a function, and `run`:

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

That decorator hides the JSON-RPC. This book starts under it and climbs back to a server you build by registering functions. From chapter 2 on, the protocol types and the transports come from `rmcp`, the Rust MCP SDK. The chapters in between are the parts that make FastMCP FastMCP: the builder, the macro, providers, transforms, middleware, and the rest of the [outline](book-outline.md).

On a machine with Rust installed, the two commands that prove the repo is alive are:

```bash
mdbook build book
cargo test -p ch01-1-fastmcp-core
```

The first builds this book. The second runs chapter 1's tests. Chapter 0 itself has nothing to compile.

## Goals

- Explain what MCP is, in plain language, and what a session does after `initialize`.
- Explain what FastMCP adds on top of the protocol, and which of those parts this book builds.
- Decide how to read the book if Rust is new to you, and how to read it if you already know Rust and want the framework.
- Find a step crate, a benchmark, a playground file, and the CI jobs, and say what each one is for.
- Install the toolchain and build the book.
- Say why a long-lived server that parses untrusted JSON is a good fit for Rust, using chapter 1's measured parse time as the first concrete number.

## Roadmap

**What MCP is.** The protocol, the session, and the three places this book teaches the two protocol eras. You need this before chapter 1's single message makes sense.

**What FastMCP is.** The README server, and the Python files that picture is copied from. This is the target the Rust is growing toward.

**Who this book is for.** How to read if the language is new, and how to read if you came for the framework.

**How the book is put together.** Steps, tags, benchmarks, playground links, the toolchain, and CI. These are the mechanics you will touch in every later chapter.

**Why this book exists.** Performance, safety, and a single binary, and why wrapping `rmcp` in a thin example would skip both halves of the title.

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612) (v4.1.0, 2026-10-08). Paths are under `fastmcp_slim/fastmcp/` unless the link says otherwise.
>
> **Adopted.** The picture of the finished server is the [README](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/README.md) example: construct `FastMCP` with a name, register a function, call `run`. [`server/server.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/server.py) is that class. `FastMCP.tool` is the method the `@mcp.tool` decorator calls. This chapter writes no Rust for it. The shape is the target.
>
> **Deferred.** Registering a tool by hand is chapter 3. A `#[tool]` macro is chapter 5. Starting a process, negotiating the protocol version, and speaking stdio are chapter 2. This chapter has no wire rules to enforce.

## What MCP is

MCP is a way for a program that has tools, files, or prompts to talk to a client. The client is often a language-model application. The conversation on the wire is JSON-RPC 2.0, which chapter 1 builds by hand: one JSON object, a version, a method, and sometimes an id. A session starts with `initialize`. Either side can `ping`. After that the client lists tools, calls one, reads a resource, or asks for a prompt.

The protocol is the envelope and the lifecycle. A useful server is also the registry of tools, the schemas that describe their arguments, the errors, the auth, and the transport. Those are the framework. FastMCP is that framework in Python. This book is that framework, taught as Rust.

FastMCP 4 speaks the current protocol, `2026-07-28`, and older handshake-era clients, from one deployment. It negotiates the version per connection. The book teaches that negotiation in three steps, which are not three chapters in a row. Chapter 1 hand-rolls the handshake-era `initialize` exchange and pins the answer to `2025-06-18`. Chapter 2 negotiates a version and moves the transport onto `rmcp`. Chapter 12 makes that negotiation work per connection over HTTP.

## What FastMCP is

Come back to the README server above for a moment. `FastMCP("Demo 🚀")` constructs the server. `@mcp.tool` registers `add`. `mcp.run()` starts it. A reader who has used FastMCP already knows this shape, and a reader who has not can treat it as the finish line: a small server, a function with a name and a description, and a way to run it.

The Rust we write keeps that shape. The package we are reading lives in `fastmcp_slim/fastmcp/`. The chapters after this one open the files inside it, one area at a time, and say what the Rust adopted, what it adapted, and what it left for a later chapter. The full table, with those files and the Rust each chapter introduces, is [book-outline.md](book-outline.md).

## Who this book is for

Read in order, and type the steps, if Rust is new. Chapter 1 uses Cargo, structs and enums, serde, `Result` and `?`, and `#[test]`. Each of those is explained where it first appears. Async, traits, and macros wait until a later chapter has a reason to introduce them. You do not need to have written Rust before. You do need to be comfortable reading code in some other language, because the explanations assume you already know what a function, a loop, and a test are.

If you already know Rust and you came for the framework, the "How FastMCP does it" box in each chapter names the Python files. The code under `steps/` is the finished snapshot for that section. `fastmcp-rs/` is the crate you fill in, with `todo!()` stubs the tests call. You can skip a sidebar that teaches a word you already use, and you should still read the box, because that is where the Python behavior and the Rust behavior are lined up.

## How the book is put together

There are 26 chapters, numbered 0 through 25, plus three appendices. What follows is the reading order, told as the arcs a server grows through. The outline is the index. This is the tour.

The wire comes first. Chapter 1 builds a JSON-RPC message by hand and answers `initialize` and `ping`. Chapter 2 accepts a range of protocol versions, chooses one, and switches the transport to `rmcp` over stdio.

Then the things a server holds. Chapters 3 through 8 add the `FastMcp` builder and a tool registry, JSON Schema from types, a `#[tool]` macro, resources and URI templates, prompts with completion and pagination, and the split between a tool error and a protocol error.

Then what a tool can see. Chapters 9 through 11 give a tool a context (logging, progress, sampling, request state), inject dependencies that stay out of the tool schema, and run startup and shutdown around shared resources.

Then both protocol eras on the network. Chapter 12 serves streamable HTTP, negotiates the version per connection, and keeps sessions. That is the third step of the protocol arc. Chapter 13 asks the user for structured input in the middle of a call.

Then clients and tests. Chapter 14 is a client and a group of clients. Chapter 15 is an in-memory transport and the test harness, so later work can be checked without starting a subprocess every time.

Then a server made of parts. Chapters 16 through 20 add a middleware chain, providers (local, mounted, proxy, filesystem), transforms and visibility, tool search and a sandboxed code mode, and background tasks through an extension.

Then running it for someone else. Chapters 21 through 25 generate tools from an OpenAPI spec, verify JWTs, add authorization, storage, and telemetry, load settings and a CLI, and publish a release. Appendix A is apps, Appendix B is the OAuth proxies and the full OAuth server, and Appendix C is icons and metadata.

### Steps

A step is a directory under `steps/`. It is a full copy of the code at the end of that section, not a patch on the previous directory, so chapter 1 still builds after chapter 2 exists. Every step crate is a member of one Cargo workspace at the repo root (`resolver = "3"`). Package names carry the section (`ch01-1-fastmcp-core`) so two steps can be built together. The library name stays `fastmcp_core`, which is what the tests import.

`fastmcp-rs/` is where the exercises go. Its chapter 1 test compiles and then panics in `todo!()` until you replace the stub. `steps/` is the finished snapshot. There is no solutions directory beyond those snapshots.

```text
steps/chNN-<slug>/   the code at the end of that section
fastmcp-rs/          the crate you fill in
book/                this mdBook, plus playground files and social drafts
```

The first snapshot you can build is `steps/ch01-1-jsonrpc`. Each later chapter adds its own directory beside it, and the earlier snapshots keep building on their own.

### Tags

When a chapter's pull request merges, the merge commit on `main` gets an annotated tag `step-01`, `step-02`, and so on. The tag is pushed after the merge, not from the chapter branch. The commands for the current tag are in [docs/tags.md](../docs/tags.md). As this chapter is written, `step-01` has not been created yet.

### Benchmarks and the CI dashboard

A chapter that adds work gets a Criterion bench. The timer covers the work the chapter is about, and it leaves out the network. This chapter has no bench. Chapter 1 is the first, and it times parsing and writing a JSON-RPC message.

On every pull request and every push to `main`, the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml) runs the benches on Ubuntu. The number this book quotes is the median, `median.point_estimate` in Criterion's `estimates.json`. The `time:` line Criterion prints is the slope, which is a different statistic, and the chapters say so when they show both. The workflow writes the median and its low and high into the job summary and uploads them as an artifact. A slow result does not fail the build, and a bench crash does not fail the workflow.

A pull request stops there. A push to `main` appends that run to `benchmarks/data.json` on the `gh-pages` branch and refreshes the dashboard, [benchmarks/index.html](../benchmarks/index.html). The page is a chart of those medians over time. What each bench includes, and what a good result looks like, is [docs/performance.md](../docs/performance.md).

### Playground links

Some chapters include one file under `book/playground/` that you can run in the [Rust Playground](https://play.rust-lang.org/) without cloning the repo. The file uses only crates the playground already has. Chapter 1's file uses `serde` and `serde_json`. The chapter links a gist, and the link in that chapter was executed on the playground before it was published. This chapter has no program, so it has no playground link.

### Toolchain

Install Rust with [rustup](https://rustup.rs). The crates in this repo set `rust-version` to 1.85, so the compiler has to be at least that. Current stable is enough. Add the formatters, and install mdBook if you want to build the HTML locally:

```bash
rustup component add rustfmt clippy
cargo install mdbook
```

From the repo root:

```bash
mdbook build book
cargo test -p ch01-1-fastmcp-core
```

`mdbook build` writes HTML to `book/_build/`, which is not committed. `cargo test` is chapter 1's crate. This chapter has nothing of its own to compile.

[personal note: first impressions of Rust after installing the toolchain]

### What CI checks

[CI](https://github.com/agnathan/rust-mcp/actions/workflows/ci.yml) runs on every push to `main` and on pull requests. It checks `cargo fmt`, runs clippy, builds this book with mdBook, and tests every step crate on Ubuntu. A change that touches a step also tests that step on macOS and Windows. The benchmark job above is a separate workflow. `fastmcp-rs` milestone tests are not part of the required build, so a `todo!()` in the exercise crate does not fail CI.

## Why this book exists

An MCP server sits on a long-lived connection, parses untrusted JSON, and runs tools that touch files, networks, and credentials. Three properties of Rust match that job, and each one shows up as something you can point at later in the book.

Performance is measured, not assumed. The book times the hot paths with Criterion. Chapter 1's first number is already concrete: on the Ubuntu CI runner, the median parse of one `initialize` message is 1.558 µs, and 1,000 messages take 1.569 ms ([run 38843114413](https://github.com/agnathan/rust-mcp/actions/runs/38843114413)). Later chapters bench tool dispatch, HTTP, middleware, and search the same way. Chapter 25 puts the Rust server next to Python FastMCP and reports that comparison from a run, not from a slogan.

Safety here means the compiler tracks which part of the program owns each value. A failure in this book is a `Result` the caller handles. That is the same habit as a JSON-RPC error response: the failure has a shape, and the process stays up. When later chapters share a database pool or a session across tasks, the types have to say how that sharing works. That is the kind of bug a server hits in production, and it is a bug the language can refuse to compile.

A single binary is what `cargo build --release` produces. The machine that runs the server does not need a language runtime or a virtualenv beside it. Chapter 24 is the CLI that starts that executable. Chapter 25 is how it gets published, including release binaries.

`rmcp` 3.5.1 already has protocol types, a session, and stdio and HTTP transports. From chapter 2 on, this book uses those. Wrapping that crate in a thin example would speak MCP, and it would skip the framework the title is about. FastMCP around the protocol is a builder, a decorator, providers, transforms, dependency injection, middleware, tasks, auth policy, and a CLI. Those are what the later chapters implement. Chapter 1 hand-rolls the envelope once so that, when `rmcp` arrives, the bytes are already familiar.

Chapter 2 opens the crate and records what 3.5.1 can already say. `ProtocolVersion::V_2026_07_28` is the crate's latest revision, and that revision has no `initialize` handshake. The tasks methods are on the server handler, gated by the `io.modelcontextprotocol/tasks` capability. The book still writes the handshake-versus-modern branch itself, because that note is the framework's decision, and it leaves the tasks runner to chapter 20. HTTP for both eras is chapter 12. The outline's note on `rmcp` is in [book-outline.md](book-outline.md).

[personal note: why I wanted to write this book, in my own words]

## Recap

MCP is a JSON-RPC conversation between a client and a program that has tools, files, or prompts. A session starts with `initialize`, either side can `ping`, and then the client lists and calls tools. FastMCP is the framework around that conversation. The README server, a name plus a decorated function plus `run`, is the shape this book is growing in Rust.

If Rust is new, read in order and type the steps. Chapter 1 teaches the language it uses. If you already know Rust, the FastMCP box and the `steps/` snapshots are the path through the framework, and `fastmcp-rs/` is the crate you fill in.

A step directory is a full copy, so earlier chapters keep building. Benchmarks quote Criterion's median, and the printed slope is a different number. Playground files use crates the playground already has. `mdbook build book` and `cargo test -p ch01-1-fastmcp-core` are the two commands that show the repo is alive.

Rust fits this server because the hot path can be measured (1.558 µs to parse one `initialize` on the Ubuntu runner), because a failure is a `Result` with a shape, and because `cargo build --release` is one executable. `rmcp` carries the protocol from chapter 2 on. The framework around it is what the book implements, and chapter 2 has already confirmed that 3.5.1 speaks `2026-07-28`.

## What's next

Chapter 1 opens one JSON-RPC message. It parses a request, stays quiet for a notification, and answers `initialize` and `ping`. There is no socket yet. The two lines it prints are the envelope every later chapter is filling in.
