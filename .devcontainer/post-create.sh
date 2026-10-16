#!/usr/bin/env bash
# Stable Rust comes from the devcontainer feature. This script adds the
# components and tools the book uses, then fetches the workspace crates.
set -euo pipefail
rustup component add clippy rustfmt
cargo install cargo-nextest --version 0.9.148 --locked
cargo install mdbook --version 0.4.52 --locked
cargo fetch --locked
