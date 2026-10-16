//! Authorization checks, a memory get, and a span with the subscriber off and on.
//!
//! `trace_on` installs a subscriber that writes nowhere, once, before the timer.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{allow_call, traced_call, MemoryStore, Store};

fn auth_1k(c: &mut Criterion) {
    c.bench_function("auth_1k", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                let result = allow_call("add", |name| name == "add");
                black_box(result).expect("allow");
            }
        });
    });
}

fn store_get(c: &mut Criterion) {
    let mut store = MemoryStore::new();
    store.set("banner", "ok");
    c.bench_function("store_get", |b| {
        b.iter(|| {
            let value = store.get("banner");
            black_box(value);
        });
    });
}

fn trace_off(c: &mut Criterion) {
    c.bench_function("trace_off", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                black_box("ok".to_string());
            }
        });
    });
}

fn trace_on(c: &mut Criterion) {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::sink)
        .try_init();
    c.bench_function("trace_on", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                let value = traced_call("add", || "ok".to_string());
                black_box(value);
            }
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = auth_1k, store_get, trace_off, trace_on
);
criterion_main!(benches);
