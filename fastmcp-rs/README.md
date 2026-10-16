# fastmcp-rs

This is the crate you implement while you read the book. The finished snapshot for each chapter stays under `steps/`. Each `tests/chNN.rs` compiles, then panics in `todo!()` until you fill in that chapter's function.

CI does not run these tests. The required build is the step crates.

```bash
cargo test -p fastmcp-rs --test ch01
cargo test -p fastmcp-rs --test ch02
cargo test -p fastmcp-rs --test ch03
cargo test -p fastmcp-rs --test ch04
```
