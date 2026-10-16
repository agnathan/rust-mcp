//! Extracting a Dep versus cloning an Arc the caller already holds.
//!
//! Neither timer deserializes JSON or opens a socket. The work after the
//! value is in hand is the same addition.

use std::hint::black_box;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{Dep, FromContext, InjectCtx};

#[derive(Clone)]
struct Db {
    bias: i64,
}

fn inject_1000(c: &mut Criterion) {
    let mut ctx = InjectCtx::default();
    ctx.insert(Db { bias: 1 });
    c.bench_function("inject_1000", |b| {
        b.iter(|| {
            let mut total = 0;
            for n in 0..1000 {
                let dep = Dep::<Db>::from_context(black_box(&ctx)).expect("db");
                total += dep.get().bias + n;
            }
            black_box(total);
        });
    });
}

fn hand_1000(c: &mut Criterion) {
    let db = Arc::new(Db { bias: 1 });
    c.bench_function("hand_1000", |b| {
        b.iter(|| {
            let mut total = 0;
            for n in 0..1000 {
                let db = Arc::clone(black_box(&db));
                total += db.bias + n;
            }
            black_box(total);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = inject_1000, hand_1000
);
criterion_main!(benches);
