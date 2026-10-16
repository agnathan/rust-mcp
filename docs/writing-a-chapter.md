# Writing a chapter

Follow this before writing or revising any chapter.

## Research FastMCP first

Before the Rust, read how [FastMCP](https://github.com/PrefectHQ/fastmcp) implements the same area. Look at the current files on `main`, not a memory of an older tree. The package lives in `fastmcp_slim/fastmcp/`. Paths in the book are relative to that directory.

From that reading, name the best and most important ideas. Implement the worthwhile ones in this chapter's Rust. Leave a short note for each idea you do not take.

## The "How FastMCP does it" box

Every chapter has the box. Cite the specific FastMCP files, and link to the commit you opened. Check that each path exists. For each idea say one of:

- **Adopted.** The Rust does the same thing, and why it belongs in this chapter.
- **Adapted.** The Rust keeps the idea and changes the shape, and why.
- **Deferred.** The idea is real and named, and which later chapter will take it. Add that sentence to the later chapter's outline row in `book/book-outline.md` in the same change.

Do not invent a file path, a function name, or a threshold. If the source was not opened, the box does not mention it.

## What else a chapter contains

Use [the chapter template](chapter-template.md). In short: a decision section, a walkthrough of the key function, a Rust sidebar at the moment the feature needs the language, the tests in words, a Build/run/benchmark section with a real command and the real output, a milestone demo, a summary, and social posts with a `[personal note]` placeholder. Criterion tables use the median from `estimates.json` (`median.point_estimate`). The printed `time:` line is the slope. CI stays keyless.

Teach a Rust concept in the chapter that first needs it. Do not introduce it earlier.

## Step crates

Each section is its own directory under `steps/`, a copy of the previous section plus the new behavior. Register it in the workspace `Cargo.toml`, `.github/chapters.json`, `.github/path-filters.yml`, and `.github/workflows/benchmarks.yml`. Tests run on Ubuntu, macOS, and Windows. The bench for that section runs on Ubuntu. After the chapter's pull request merges, tag `step-NN` on `main`. Do not push that tag from the pull request. The plan for the current tag is [tags.md](tags.md).
