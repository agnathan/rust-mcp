//! A thousand progress notes recorded in memory.
//!
//! The context is built before `b.iter`. Each sample pushes 1,000 notes and
//! then clears the list, so the allocation does not grow across samples.
//! There is no socket.

use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{Context, SharedNotes};

fn progress_1000(c: &mut Criterion) {
    let ctx = Context::new(Arc::new(SharedNotes::default()), HashMap::new());
    c.bench_function("progress_1000", |b| {
        b.iter(|| {
            for step in 0..1000 {
                ctx.report_progress(black_box(step as f64), black_box(Some(1000.0)), None);
            }
            ctx.clear();
            black_box(&ctx);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = progress_1000
);
criterion_main!(benches);
