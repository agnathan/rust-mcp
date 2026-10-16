# fastmcp-rs

This is the crate you implement while you read the book. The finished snapshot for each chapter stays under `steps/`. `tests/ch01.rs` compiles, then panics in `todo!()` until `ping_response_json` returns a JSON-RPC ping response.

CI does not run these tests. The required build is the step crates.

```bash
cargo test -p fastmcp-rs --test ch01
```
