# Social posts: Chapter 0, "Introduction"

## X

Chapter 0 of Learning Rust by Building FastMCP is up: Introduction.

The book is a Rust course for people new to the language. The program is a Rust FastMCP: MCP servers, built the way Prefect's Python framework builds them.

26 chapters. Chapter 1 hand-rolls the JSON-RPC envelope. Chapter 2 negotiates the protocol and picks up `rmcp`. The rest is the framework: tools, resources, HTTP, a client, providers, auth, a CLI.

[personal note]

Steps are cumulative copies, tags are `step-NN` on the merge commit, benches quote the Criterion median, and a few chapters link the Rust Playground.

#rustlang #MCP

## LinkedIn

Chapter 0 of Learning Rust by Building FastMCP is out: Introduction.

This is a book for developers who are new to Rust. The project is a Rust version of FastMCP, Prefect's Python framework for the Model Context Protocol. You learn the language by growing one server, and a Rust feature shows up in the chapter that first needs it.

The tour is 26 chapters. The wire comes first: a hand-rolled JSON-RPC `initialize` in chapter 1, then version negotiation and `rmcp` over stdio in chapter 2, then HTTP for both protocol eras in chapter 12. In between and after that, the framework: a builder, schemas, a `#[tool]` macro, resources, prompts, errors, context, HTTP sessions, a client, middleware, providers, OpenAPI, auth, and a CLI.

The repo is set up so each chapter still builds on its own. `steps/` holds a full copy at the end of each section. `fastmcp-rs/` is the crate you fill in. A merged chapter gets an annotated `step-NN` tag on `main`. Criterion runs in CI, the book quotes the median, and a push to `main` updates the benchmark dashboard. A few chapters include a Rust Playground link that was executed before it was published.

`rmcp` carries the protocol types and the transports from chapter 2. The book still builds the FastMCP-shaped pieces, because those are the design, and because that is the Rust practice.

[personal note]

Next: chapter 1, Speaking JSON-RPC. One request, one notification, no socket.

#Rust #MCP #LearningInPublic #SoftwareEngineering
