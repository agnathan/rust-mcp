# Contributing

Chapters are written by reading FastMCP first. The voice and the chapter shape are in [book/STYLE.md](book/STYLE.md). The standing rule, and where a deferred idea gets recorded, are in [docs/writing-a-chapter.md](docs/writing-a-chapter.md).

Before writing or revising a chapter:

1. Read the current FastMCP files for that area on [PrefectHQ/fastmcp](https://github.com/PrefectHQ/fastmcp), under `fastmcp_slim/fastmcp/`.
2. Implement the worthwhile ideas in the chapter's step crates.
3. In the "How FastMCP does it" box, cite the files, link the commit, and say what was adopted, adapted, or deferred, and why.
4. Check that every path in the box exists.

`cargo fmt --all`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `mdbook build book` should pass. CI stays keyless.

After a chapter pull request merges, tag `step-NN` on the merge commit. Do not push that tag from the pull request. See [docs/tags.md](docs/tags.md).
