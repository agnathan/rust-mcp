# Chapter 22: Authentication

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

A tool call can carry a bearer token. The server has to decide whether that token is the one it expects before the tool runs. FastMCP verifies JWTs with a public key, a shared secret, or a JWKS document. This chapter verifies the shared-secret case: HS256. The audience claim has to match the resource.

Beside the check, the server can publish a small document that says which authorization server issues tokens for this resource. That is the protected-resource metadata. The OAuth proxy and the full authorization server are a later appendix. This chapter stops at verify, a key id, and the metadata fields.

## What you'll build

The algorithm, the subject inside a valid token, and the resource URL. The token and the secret stay off the page.

```text
alg: HS256
subject: ada
resource: http://example.test/mcp
```

The snapshot is `steps/ch22-1-auth`. It is a full copy of chapter 21, plus `auth.rs`. The crates are `ch22-1-fastmcp-core` and `ch22-1-fastmcp-macros`.

```bash
cargo run -p ch22-1-fastmcp-core --example demo
```

## Goals

- Sign and verify an HS256 token with a shared secret.
- Reject a bad signature and a wrong audience with the same message, and keep the secret out of that message.
- Read the first `kid` from a JWKS document without fetching it.
- Advertise a resource, its authorization servers, and the scopes it supports.

## Roadmap

**22.1 Verify.** The problem is a token that looks plausible and is not. The check is the signature and the audience.

**22.2 Metadata.** The problem is a client that does not know where to get a token. The metadata names the authorization server.

**22.3 Build, run, and benchmark.** The tests, the demo above, and one verification.

| Section | Step | What it adds |
|---|---|---|
| 22.1–22.3 | `steps/ch22-1-auth` | `sign`, `verify`, `jwks_kid`, `ResourceMetadata` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/auth/providers/jwt.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/auth/providers/jwt.py) `JWTVerifier` (line 207) accepts RS, ES, PS, EdDSA, and HS256/384/512. The constructor takes `jwks_uri`, `issuer`, `audience`, and `algorithm` (line 227). The default algorithm is `RS256` (line 249). `jwks_refresh_interval` defaults to 30 seconds (line 239). [`server/auth/auth.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/auth/auth.py) `RemoteAuthProvider` (line 522) is a resource server that verifies tokens from known authorization servers and advertises RFC 9728 protected-resource metadata. `get_routes` (line 609) builds those routes from `authorization_servers` (line 633) and `scopes_supported`.
>
> **Adapted.** This chapter verifies HS256 with `jsonwebtoken` and the `rust_crypto` feature. The audience is one string. A failure is the text `token rejected`. The secret is not interpolated into the error. `jwks_kid` reads `keys[0].kid` from a JSON value the caller already has. `ResourceMetadata` holds `resource`, `authorization_servers`, and `scopes_supported`.
>
> **Deferred.** RS256 and the other asymmetric algorithms, fetching a JWKS URI, issuer checks, required scopes on the token, and the HTTP route that serves the metadata. The OAuth proxy, the OIDC proxy, and the full authorization server are Appendix B. This chapter does not write that appendix.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This chapter does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **A rejected token does not reveal the secret.** The error string is `token rejected`. The test checks that the word `secret` is absent from it. A token signed for `http://example.test/mcp` fails when the expected audience is different. The demo prints the algorithm, the subject, and the resource. It does not print the token or the secret.

[personal note: a log line that included the bearer token, and the rotation that followed]

## 22.1 Verify

```rust
validation.set_audience(&[audience]);
let data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
    .map_err(|_| "token rejected".to_string())?;
```

`sign` is how the tests and the bench mint a token. `verify` returns the `sub` claim. HS256 is `AUTH_ALGORITHM`.

## 22.2 Metadata

`ResourceMetadata::new` stores the resource URL, the authorization servers, and the scopes. `jwks_kid` is a local lookup. FastMCP would fetch `jwks_uri` on an interval. This chapter does not.

`tests/auth.rs` accepts a good token, rejects a tampered one and a wrong audience, reads `test-key` from a JWKS document, and checks the metadata fields. Chapter 21's tests still pass in this copy. One hundred eighteen tests passed: the previous 115, plus these three.

## 22.3 Build, run, and benchmark

From the repo root:

```bash
cargo test -p ch22-1-fastmcp-core
cargo run -p ch22-1-fastmcp-core --example demo
cargo bench -p ch22-1-fastmcp-core --bench auth -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
alg: HS256
subject: ada
resource: http://example.test/mcp
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
verify_hs256            time:   [1.5281 µs 1.5313 µs 1.5352 µs]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `verify_hs256` | 1.530 µs | 1.524 µs | 1.534 µs | < 20 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `verify_hs256` | One `verify` of an HS256 token, including the audience check | Signing the token | Audience `http://example.test/mcp`, subject `ada`, expiry far in the future | No HTTP, no JWKS fetch, no log of the secret | Well under 20 µs. About 1.5 µs here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `verify_hs256` | pending | pending | pending | < 20 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows an `auth_algorithm` stub. `tests/ch22.rs` expects `"HS256"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/ch22-auth.rs`](playground/ch22-auth.rs) prints the three demo lines. The playground does not have this book's crate, and it does not verify a signature. The crate does.

[Run the lines on the playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=9392a39e3cc4bdfb659997325f9a8f4d).

I executed that gist on the playground's stable channel, debug mode, edition 2021. It exited 0 and printed:

```text
alg: HS256
subject: ada
resource: http://example.test/mcp
```

## Recap

`sign` and `verify` use HS256. A bad signature and a wrong audience both become `token rejected`, and that string does not carry the secret. `jwks_kid` reads a key id from a document already in hand. `ResourceMetadata` names the resource and the authorization servers.

`cargo run -p ch22-1-fastmcp-core --example demo` prints the three lines above. Locally, one verification takes 1.530 µs.

[personal note: an audience I set to the public URL and a token that still said localhost]

## What's next

Chapter 23 is the production set: a check that can refuse a call, a version filter on components, a storage trait with memory and file backends, a tracing span, and a fingerprint of a tool's name and schema.
