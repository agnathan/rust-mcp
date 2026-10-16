//! Search over 10,000 tools, and eight calls inside one script versus eight turns.
//!
//! The index and the rhai engine's tools are built before the timer.
//! `script_8` still parses and runs the script. `turns_8` is eight host lookups.

use std::collections::HashMap;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{execute, turn_calls, Bm25};

fn search_10k(c: &mut Criterion) {
    let docs: Vec<(String, String)> = (0..10_000)
        .map(|index| {
            (
                format!("tool{index}"),
                format!("tool {index} does word{} work", index % 50),
            )
        })
        .collect();
    let index = Bm25::build(docs);
    c.bench_function("search_10k", |b| {
        b.iter(|| {
            let hits = index.search("word7", 5);
            black_box(hits);
        });
    });
}

fn tools() -> HashMap<String, i64> {
    (0..8).map(|index| (format!("t{index}"), 1)).collect()
}

fn script_8(c: &mut Criterion) {
    let tools = tools();
    let script = (0..8)
        .map(|index| format!(r#"call_tool("t{index}", 1)"#))
        .collect::<Vec<_>>()
        .join(" + ");
    c.bench_function("script_8", |b| {
        b.iter(|| {
            let total = execute(&script, &tools, 50).expect("script");
            black_box(total);
        });
    });
}

fn turns_8(c: &mut Criterion) {
    let tools = tools();
    let names: Vec<(String, i64)> = (0..8).map(|index| (format!("t{index}"), 1)).collect();
    let borrowed: Vec<(&str, i64)> = names
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect();
    c.bench_function("turns_8", |b| {
        b.iter(|| {
            let total = turn_calls(&tools, &borrowed);
            black_box(total);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = search_10k, script_8, turns_8
);
criterion_main!(benches);
