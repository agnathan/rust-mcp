# Appendix B: OAuth proxy, OIDC proxy, and the full OAuth server

*Draft. Bracketed notes like [personal note] are where my own story goes.*

## Introduction

Chapter 22 checked a token. It did not issue one. A client that speaks MCP OAuth expects an authorization server: a place to register, a code, and a token endpoint. Most identity providers do not offer dynamic client registration, so FastMCP puts a proxy in front of them. When the provider speaks OpenID Connect, the proxy reads a discovery document instead of being handed the endpoints by hand.

This appendix builds those three pieces in memory. It does not open a socket and it does not call an identity provider.

## What you'll build

An issuer, the PKCE method, a token from the in-memory server, the token the proxy stored, and the issuer read from a discovery document.

```text
issuer: http://auth.example.test
method: S256
token: tok-1
proxy: upstream:abc
oidc: http://idp.example.test
```

The crate is `steps/apb-1-oauth`, package `apb-1-oauth`.

```bash
cargo run -p apb-1-oauth --example demo
```

## Goals

- Issue a single-use code and exchange it with S256 PKCE.
- Proxy a client redirect through a fixed upstream redirect, and hand back the stored upstream token.
- Refuse a strict OIDC document that is missing a required field.

## Roadmap

**B.1 The in-memory server.** The problem is a code that should die after five minutes and should fail if the verifier is wrong.

**B.2 The proxy and the discovery document.** The problem is a client whose redirect port changes, talking to a provider that registered one callback. Then the same idea when the endpoints come from JSON.

**B.3 Build, run, and benchmark.** The tests, the demo above, and one S256 challenge.

| Section | Step | What it adds |
|---|---|---|
| B.1–B.3 | `steps/apb-1-oauth` | `MemoryOAuth`, `OAuthProxy`, `oidc_from_json` |

> **How FastMCP does it**
>
> Read at commit [`e2fcc412`](https://github.com/PrefectHQ/fastmcp/commit/e2fcc412001ff7414095c5dc282f94c70299a612). Paths are under `fastmcp_slim/fastmcp/`.
>
> **Adopted.** [`server/auth/auth.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/auth/auth.py) `OAuthProvider` (line 913) is the authorization server: client registration, the authorization flow, token issuance, and verification. [`server/auth/providers/in_memory.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/auth/providers/in_memory.py) `InMemoryOAuthProvider` (line 31) keeps clients, codes, and tokens in dictionaries. `DEFAULT_AUTH_CODE_EXPIRY_SECONDS` is `5 * 60` (line 26). [`server/auth/oidc_proxy.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/auth/oidc_proxy.py) `OIDCConfiguration` (line 58) requires, in strict mode, `issuer`, `authorization_endpoint`, `token_endpoint`, `jwks_uri`, `response_types_supported`, `subject_types_supported`, and `id_token_signing_alg_values_supported` (`_enforce_strict`, line 140). The error text is `Missing required configuration metadata: {attr}` (line 148). `OIDCProxy` (line 239) wraps `OAuthProxy` and loads that document from `config_url`.
>
> **Adapted.** [`server/auth/oauth_proxy/proxy.py`](https://github.com/PrefectHQ/fastmcp/blob/e2fcc412001ff7414095c5dc282f94c70299a612/fastmcp_slim/fastmcp/server/auth/oauth_proxy/proxy.py) `OAuthProxy` (line 190) presents dynamic registration to the client and a fixed redirect to the upstream provider. The class docstring describes storing a transaction, using that id as the upstream `state`, exchanging the upstream code on the proxy, and issuing a new code for the client's redirect. This crate does that with maps. `complete` stores `upstream:{code}` instead of calling a token endpoint. `oidc_from_json` reads a string. It does not fetch.
>
> **Deferred.** Consent HTML, an HTTP route table, a real upstream token request, refresh, revocation, CIMD, and the provider classes for GitHub, Google, and the rest. Redis is still the storage story from chapter 23, and it is not wired here.

> **Guardrails in this chapter**
>
> The notes are [docs/fastmcp-guardrails.md](../docs/fastmcp-guardrails.md).
>
> This appendix does not add a wire rule. W1 through W7 stay as the earlier chapters left them.
>
> **A code is single-use.** `exchange` removes the code before it checks the verifier. A second exchange of the same code is `unknown code`. An expired code is `code expired`. The verifier has to produce the same S256 challenge that was stored at `authorize`.

[personal note: a redirect URI I registered as localhost and then watched the client pick a new port]

## B.1 The in-memory server

`code_challenge_s256` is the URL-safe base64 of SHA-256, with the padding stripped. `MemoryOAuth::authorize` stores that challenge and an expiry of `now + 300`. `exchange` removes the code, rejects an empty client id, rejects `now >= expires_at`, and rejects a verifier that hashes to a different challenge. The first token is `tok-1`.

```rust
pub const AUTH_CODE_EXPIRY_SECONDS: u64 = 5 * 60;
```

`metadata` names the issuer, `{issuer}/authorize`, `{issuer}/token`, and `code_challenge_methods_supported: ["S256"]`. Those strings are not mounted on a router.

The tests exchange a matching verifier and get `tok-1`, then expire a code at the 300-second mark and get `code expired`.

## B.2 The proxy and the discovery document

`OAuthProxy::start` returns an upstream authorize URL whose `redirect_uri` is the proxy's fixed callback and whose `state` is `tx-1`. The client's own redirect and state stay in the transaction. `complete` issues `client-code-2` on the client's redirect and remembers `upstream:abc`. `exchange` checks the verifier and returns that stored token.

`oidc_from_json` in strict mode walks the seven required fields. A document that only has `issuer` fails with `Missing required configuration metadata:`. A full document returns the issuer `http://idp.example.test`.

## B.3 Build, run, and benchmark

4 tests passed. `a_code_becomes_a_token_when_the_verifier_matches` checks `tok-1`, the second exchange, and the metadata method. `an_expired_code_is_refused` checks the 300-second boundary. `the_proxy_keeps_the_client_redirect_and_returns_the_upstream_token` checks `tx-1`, the client state, and `upstream:abc`. `strict_oidc_requires_the_discovery_fields` checks the issuer and the missing-field error.

```bash
cargo test -p apb-1-oauth
cargo run -p apb-1-oauth --example demo
cargo bench -p apb-1-oauth --bench oauth -- --warm-up-time 1 --measurement-time 2 --sample-size 10
```

The demo prints:

```text
issuer: http://auth.example.test
method: S256
token: tok-1
proxy: upstream:abc
oidc: http://idp.example.test
```

The slope line, on the same 4-vCPU guest as chapter 1:

```text
pkce_s256               time:   [96.554 ns 99.583 ns 102.91 ns]
```

Local medians from that run's `estimates.json` (the slope above is a different statistic):

| Benchmark | Median | Low | High | Target |
|---|---|---|---|---|
| `pkce_s256` | 100.6 ns | 98.07 ns | 101.8 ns | < 1 µs |

**Where the timer starts and stops.**

| Benchmark | Included | Excluded | Input | Faked | A good result |
|---|---|---|---|---|---|
| `pkce_s256` | SHA-256 of the verifier and the unpadded base64url encoding | Building the verifier string | One 43-character verifier | No HTTP and no code store | Well under 1 µs. About 101 ns here. |

CI runs this step's tests on Ubuntu, macOS, and Windows when the step changes. Criterion runs on Ubuntu in the [Benchmarks workflow](https://github.com/agnathan/rust-mcp/actions/workflows/benchmarks.yml). The CI medians below are filled from that run once it finishes. A slow number does not fail the build.

| Benchmark | CI median | Low | High | Target |
|---|---|---|---|---|
| `pkce_s256` | pending | pending | pending | < 1 µs |

The local table is the machine where the command transcript was captured. The chapter quotes the CI medians once the run is linked.

### Milestone demo

`fastmcp-rs/` grows a `pkce_method` stub. `tests/apb.rs` expects `"S256"`. The function is `todo!()`, so the test panics until you fill it in. CI does not run that test.

### Rust Playground

[`book/playground/apb-oauth.rs`](playground/apb-oauth.rs) prints the demo lines. The playground does not have `sha2` or `base64`. It does not compute a challenge.

[Playground: appendix B oauth](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=635ba4228ed6bbf15adf2c7d716d7bcc)

The playground printed:

```text
issuer: http://auth.example.test
method: S256
token: tok-1
proxy: upstream:abc
oidc: http://idp.example.test
```

## Recap

S256 is the challenge method. A code lasts 300 seconds and works once. The proxy's upstream redirect is fixed, and the client's redirect is restored on the way back. A strict discovery document must carry the seven fields FastMCP checks.

`cargo run -p apb-1-oauth --example demo` prints the five lines above. Locally, one S256 challenge is 100.6 ns.

[personal note: a verifier I logged next to the token]

## What's next

Appendix C is icons and the server card: a source URL, a theme, and the name a client shows. The OAuth crate stops at the in-memory code and the unread discovery document.
