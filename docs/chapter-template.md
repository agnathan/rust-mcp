# Chapter template

Copy this shape. The FastMCP box is filled only after the files have been read. See [Writing a chapter](writing-a-chapter.md).

```markdown
# Chapter N: Title

*Draft. Bracketed notes like [personal note] are where my own story goes.*

Two or three paragraphs: what the previous chapter cannot do, what this one adds, what it leaves for later.

| Section | Step | What it adds |
|---|---|---|
| N.1 | `steps/chNN-1-slug` | … |

> **How FastMCP does it**
>
> Cite `path` under `fastmcp_slim/fastmcp/`, with a link to the commit. **Adopted:** … **Adapted:** … **Deferred:** … (and the chapter that will take it).

> **Guardrails in this chapter**
>
> Which wire or safety rule this chapter enforces, and which it leaves for a later chapter. See [fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).

[personal note: …]

## What this chapter decides

## N.1 Section title

Walk the key function. A Rust sidebar sits where the type is first needed.

### The tests

### Build, run, and benchmark

A real command, the real test summary, the real demo output, the Criterion slope, and a table whose Median column is `estimates.json`.

### Milestone demo

### Rust Playground

One file, crates the playground already has, and a play.rust-lang.org link that has been executed.

## Pitfalls

## Chapter summary

## What's next
```

Social drafts live in `book/social/chapter-NN.md` and end the personal beat with `[personal note]`.
