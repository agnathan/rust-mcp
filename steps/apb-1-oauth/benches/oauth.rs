//! One S256 challenge.
//!
//! The verifier is built before the timer. The timed work is the hash and
//! the base64url encoding.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use mcp_oauth::code_challenge_s256;

fn pkce_s256(c: &mut Criterion) {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    c.bench_function("pkce_s256", |b| {
        b.iter(|| {
            let challenge = code_challenge_s256(black_box(verifier));
            black_box(challenge);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = pkce_s256
);
criterion_main!(benches);
