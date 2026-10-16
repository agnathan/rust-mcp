//! One HS256 verification.
//!
//! The token is signed before the timer. The secret is a test value and is
//! not printed.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::verify;

fn verify_hs256(c: &mut Criterion) {
    let secret = b"bench-secret-not-logged";
    let token =
        fastmcp_core::sign(secret, "http://example.test/mcp", "ada", 4_102_444_800).expect("sign");
    c.bench_function("verify_hs256", |b| {
        b.iter(|| {
            let subject = verify(secret, &token, "http://example.test/mcp").expect("verify");
            black_box(subject);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = verify_hs256
);
criterion_main!(benches);
