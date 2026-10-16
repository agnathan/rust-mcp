//! Schema generation for one tool parameter struct, then for 1,000.
//!
//! `json_schema_of` does not cache. Each call builds a new schema.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::json_schema_of;
use schemars::JsonSchema;

#[derive(JsonSchema)]
#[allow(dead_code)]
struct Greet {
    name: String,
    title: Option<String>,
}

fn schema_one(c: &mut Criterion) {
    c.bench_function("schema_1", |b| {
        b.iter(|| black_box(json_schema_of::<Greet>()));
    });
}

fn schema_thousand(c: &mut Criterion) {
    c.bench_function("schema_1000", |b| {
        b.iter(|| {
            let mut last = None;
            for _ in 0..1000 {
                last = Some(json_schema_of::<Greet>());
            }
            black_box(last);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = schema_one, schema_thousand
);
criterion_main!(benches);
