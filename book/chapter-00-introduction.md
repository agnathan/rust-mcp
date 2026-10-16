# Introduction

*Draft. Bracketed notes like [personal note] are where my own story goes.*

This book is a course in Rust. The program we build is a Rust version of [FastMCP](https://github.com/PrefectHQ/fastmcp), Prefect's Python framework for the [Model Context Protocol](https://modelcontextprotocol.io/). Each chapter adds one piece, with tests, and points at the FastMCP file that solves the same problem.

[personal note: what got me interested in MCP and in Rust]

## What MCP is

MCP is a way for a program that has tools, files, or prompts to talk to a client. The client is often a language-model application. The conversation on the wire is JSON-RPC 2.0. A session starts with `initialize`. Either side can `ping`. After that the client lists tools, calls one, reads a resource, or asks for a prompt.

The protocol is the envelope. FastMCP is the framework around it: a server you build by registering functions, a client that speaks several transports, and the extra pieces (middleware, auth, mounts, OpenAPI) that show up once a server is real.

## What FastMCP does

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

That decorator hides the JSON-RPC. This book starts under it. Chapter 1 builds the handshake-era envelope by hand. Chapter 2 negotiates the protocol version and is the first server a client can start. Later chapters add the builder, tools, resources, prompts, errors, context, dependency injection, HTTP for both protocol eras, a client, middleware, providers, and the rest of the outline in [book-outline.md](book-outline.md) (chapters 0 through 25).

The Rust we write is not a line-by-line port. FastMCP already uses the Python MCP SDK for JSON-RPC. From chapter 2 on, we use the Rust SDK `rmcp` the same way. The chapters are about the parts that make FastMCP FastMCP.

## How the repo is laid out

```
steps/chNN-<slug>/   the code at the end of that section
fastmcp-rs/          the crate you fill in; its milestone test starts failing
book/                this mdBook, plus social drafts
```

A step is a full copy, not a patch on the previous directory. Chapter 1 still builds after chapter 2 exists. Every step crate is a member of one cargo workspace at the repo root. Package names carry the section (`ch01-1-fastmcp-core`) so two steps can be built together. The library name stays `fastmcp_core`, which is what the tests import.

`fastmcp-rs/` is where the exercises go. `steps/` is the finished snapshot. There is no solutions directory beyond those snapshots.

When a chapter's pull request merges, the merge commit on `main` gets an annotated tag `step-01`, `step-02`, and so on. The tag is pushed after the merge, not from the chapter branch. The current plan is [docs/tags.md](../docs/tags.md). `step-01` is not created yet.

## Toolchain

Install Rust with [rustup](https://rustup.rs). The crates in this repo set `rust-version` to 1.85, so the compiler has to be at least that. Current stable is enough. Add the formatters:

```bash
rustup component add rustfmt clippy
```

The book is an [mdBook](https://rust-lang.github.io/mdBook/). From the repo root:

```bash
mdbook build book
```

The HTML lands in `book/_build/`, which is not committed. Chapter 1 is where the first `cargo test` and `cargo bench` commands are.

[personal note: first impressions of Rust after installing the toolchain]

## How to read this book

If you are new to Rust, read in order and type the steps. A Rust idea shows up in the chapter that first needs it, in a short sidebar, and not before. Chapter 1 uses cargo, structs and enums, serde, `Result` and `?`, and `#[test]`. Async, traits, and macros wait until a later chapter has a reason to introduce them.

If you already know Rust and want the framework, the "How FastMCP does it" box in each chapter names the Python files.

Let's open a JSON-RPC message.
