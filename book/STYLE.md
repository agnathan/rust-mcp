# Style

This file is how every chapter of *Learning Rust by Building FastMCP* gets written. Read it before a draft, and again before a revision.

The reader is a working developer who is new to Rust. They already ship software in some other language. They picked this book up to learn Rust by building a server in the shape of [FastMCP](https://github.com/PrefectHQ/fastmcp). Write the chapter they would want to read on a weeknight: warm, concrete, and paced so that one idea lands before the next one starts.

`book/STYLE.md` is the source of truth for shape and voice. [docs/chapter-template.md](../docs/chapter-template.md) and [docs/writing-a-chapter.md](../docs/writing-a-chapter.md) point here.

## Voice

Write the way a good O'Reilly or Manning chapter sounds. A person is walking you through a problem they have already hit. Then they show the few lines that solve it. Then they say what those lines just did.

- Start from a problem the reader can picture. A client sends a protocol date the server has never answered. A tool is registered and `tools/call` still says the method is missing. A client cannot tell which fields are required. Name that problem in plain language, then solve it.
- Keep the rhythm even. One idea, then the lines that carry it, then a paragraph of explanation. A wall of code loses the reader. A wall of bullets loses them too.
- Use a concrete example whenever you introduce a mechanism. `add(2, 3)` teaches more than "a binary operation." A single JSON object on the page teaches more than a description of JSON-RPC.
- Define every term the first time the chapter uses it. Struct, enum, trait, closure, `Option`, `Result`, derive, schema: the reader may not have met the word before, or may have met it two chapters ago and forgotten the sidebar. A later chapter gives a one-line reminder at the moment the word comes back.
- Stay technically exact. Warmth is word choice and order. The commands, the real output, the median tables, the FastMCP file citations, and the guardrail rules stay exact.
- The author's own story stays in `[personal note]` placeholders. Do not invent a biography, a job, or a feeling and attribute it to the author.

State the thing the reader should do. A real trap gets its own sentence at the moment the reader could fall into it. Save contrast for that trap.

## How a chapter opens

After the title and the draft line, four sections, in this order, before any long code.

### Introduction

Where we are in the book, the problem this chapter solves, and why that problem matters. Plain language. A method name is fine. A code block is too early.

### What you'll build

A concrete description of the result, then a tiny example of it running. Use the demo's real output, a short command, or both. The reader should be able to point at the example and say "that is the thing I will have."

### Goals

A bulleted list. This is the one place a bullet list is required. Say what the reader will understand and be able to do when the chapter is over. Cover the Rust ideas and the MCP or FastMCP ideas. The recap at the end ties back to these lines.

### Roadmap

One or two sentences for each section that follows. Say what happens there and why, in reading order. A single table that maps a section to its `steps/` directory may follow the prose. The table does not replace the sentences.

## How a section is written

Open with a sentence or two of context: the problem this section is about to take on. Then show only the lines that carry the idea. Explain them afterwards, in prose.

The rest of the function lives in the step crate. Name that file. A step directory under `steps/` is a full copy of the code at the end of the section, so the chapter can stay short while the program stays complete. Show a struct when the section is about that struct. Show a function when the section is about that function. Leave the neighboring types in the crate.

When a Rust idea is needed for the first time, teach it in a short paragraph beside the code that needs it. A sidebar may be a paragraph. It should not be a glossary.

Tests are described in words: what goes in, what comes back, and which failure the test is guarding. Quote a real summary line from the run. The command prints every `PASS` line; the chapter does not need to.

## How a chapter closes

### Recap

Walk back through the goals. Name what the reader can now do, the command that shows it, and the number worth remembering.

### What's next

The next chapter's problem, in a few sentences, so the reader knows why the page turns.

## Keep these, and keep them exact

- The **How FastMCP does it** box. Cite only files you opened. The commit for this book is [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). For each idea say **Adopted**, **Adapted**, or **Deferred**, and name the later chapter that will take a deferred idea. Do not invent a path, a function, or a threshold. `utilities/versions.py` compares component versions. It does not negotiate MCP protocol dates.
- The **Guardrails** box. Point at [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md). Say which rules this chapter enforces and which rules wait.
- The commands a reader can run, copied so they work from the repo root.
- The demo's real output.
- Criterion, told apart on purpose. The `time:` line is the slope. The table is `median.point_estimate` from `estimates.json`, with the low and high of the median's confidence interval. Link the CI run, and record its date and `github.sha`. Quote the median when the chapter summarizes itself. A slow number does not fail the build. The bench's timing table stays: what the timer includes, what it leaves out, the input, what is faked, and what a good result looks like.
- The playground file, the gist link, and the output from executing that gist.
- `[personal note]` placeholders, including the ones in `book/social/`.
- Social drafts, unless a claim in them has become false.

## Leave these in the crate, or in the terminal

- The full source of a module. The step crate already has it.
- A complete nextest listing. Quote the summary (`N tests run: N passed`) and, when a single test carries the story, that test's name.
- Criterion's warmup, sample-count, and outlier chatter. Quote the slope lines.
- A "What this chapter decides" list. Those decisions belong in the introduction and in the section that makes them.
- A pitfalls list. Write the trap as a paragraph where the reader meets it.

## A skeleton

```markdown
# Chapter N: Title

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Where we are, the problem, and why it matters.

## What you'll build

The result, then a tiny real example of it running.

## Goals

- A Rust idea the reader can now use
- An MCP or FastMCP idea the reader can now explain

## Roadmap

**N.1 Title.** One or two sentences on what this section does and why.

> **How FastMCP does it**
>
> Commit, paths, Adopted / Adapted / Deferred.

> **Guardrails in this chapter**
>
> Which rules this chapter enforces, and which wait.

[personal note: …]

## N.1 Title

The problem this section solves.

The few lines that matter, then prose that explains them.

The tests, in words. The commands. The real demo. The slope lines. The local medians. The timing table. The CI medians. The milestone. The playground link and its output.

## Recap

The goals, answered.

[personal note: a moment where the compiler stopped me]

## What's next

The next chapter's problem.
```
