//! A filtered list of 10,000 tools.
//!
//! Half the tools are already disabled. The timer is the walk that keeps
//! the enabled names.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::Catalog;

fn filter_10k(c: &mut Criterion) {
    let mut catalog = Catalog::new(
        (0..10_000)
            .map(|index| {
                let tag = if index % 2 == 0 { "public" } else { "internal" };
                (format!("tool{index}"), vec![tag])
            })
            .collect::<Vec<_>>(),
    );
    catalog.disable_tag("internal");
    c.bench_function("filter_10k", |b| {
        b.iter(|| {
            let names = catalog.visible();
            black_box(names);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = filter_10k
);
criterion_main!(benches);
