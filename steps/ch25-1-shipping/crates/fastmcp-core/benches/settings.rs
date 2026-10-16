//! Parse `inspect` and load settings.
//!
//! The timed work builds a plan from scratch each iteration. The OnceLock
//! cache is not on this path.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::plan;

fn cold_start(c: &mut Criterion) {
    c.bench_function("cold_start", |b| {
        b.iter(|| {
            let planned = plan(&["inspect"], None).expect("plan");
            black_box(planned);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = cold_start
);
criterion_main!(benches);
