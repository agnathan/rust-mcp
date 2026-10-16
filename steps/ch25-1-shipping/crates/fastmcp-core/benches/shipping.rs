//! One in-process `add` call on the book's server.
//!
//! The server is built before the timer. The timed work is `dispatch_tool`.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{add_server, call_add};

fn rust_call(c: &mut Criterion) {
    let server = add_server();
    c.bench_function("rust_call", |b| {
        b.iter(|| {
            let value = call_add(black_box(&server)).expect("add");
            black_box(value);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = rust_call
);
criterion_main!(benches);
