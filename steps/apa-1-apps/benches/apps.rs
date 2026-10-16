//! Filter model-visible names out of 10,000 tools.
//!
//! The catalog is built before the timer. Odd indexes are app tools.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use mcp_apps::{visible_model, Audience};

fn visible_10k(c: &mut Criterion) {
    let tools: Vec<(String, Audience)> = (0..10_000)
        .map(|index| {
            let audience = if index % 2 == 0 {
                Audience::Model
            } else {
                Audience::App
            };
            (format!("tool_{index}"), audience)
        })
        .collect();
    c.bench_function("visible_10k", |b| {
        b.iter(|| {
            let names = visible_model(black_box(&tools));
            black_box(names);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = visible_10k
);
criterion_main!(benches);
