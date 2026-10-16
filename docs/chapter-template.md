# Chapter template

The chapter shape and the voice live in [book/STYLE.md](../book/STYLE.md). Follow that file. The skeleton below is the same opening and closing, copied so this page still stands alone. The FastMCP box is filled only after the files have been read. See [Writing a chapter](writing-a-chapter.md).

```markdown
# Chapter N: Title

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Where we are in the book, the problem this chapter solves, and why it matters. Plain language, before any code.

## What you'll build

The concrete result, then a tiny example of it running.

## Goals

- What the reader will understand and be able to do (Rust)
- What the reader will understand and be able to do (MCP / FastMCP)

## Roadmap

One or two sentences per section: what happens there, and why.

> **How FastMCP does it**
>
> Cite `path` under `fastmcp_slim/fastmcp/`, with a link to the commit. **Adopted:** … **Adapted:** … **Deferred:** … (and the chapter that will take it).

> **Guardrails in this chapter**
>
> Which wire or safety rule this chapter enforces, and which it leaves for a later chapter. See [fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).

[personal note: …]

## N.1 Section title

A sentence or two of context, then only the lines that carry the idea, then prose that explains them. The rest of the code stays in the step crate. A Rust explanation sits where the term is first needed, as a paragraph.

### The tests

What goes in, what comes back, and which failure the test guards. Quote the real summary line.

### Build, run, and benchmark

The real commands, the real demo output, the Criterion slope lines, and a table whose Median column is `median.point_estimate` from `estimates.json`. The `time:` line is the slope.

### Milestone demo

### Rust Playground

One file, crates the playground already has, and a play.rust-lang.org link that has been executed.

## Recap

Tie the result back to the goals.

## What's next
```

Social drafts live in `book/social/chapter-NN.md` and end the personal beat with `[personal note]`.
