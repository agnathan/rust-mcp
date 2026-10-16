//! One direct call, the same call through a proxy, and a list across ten providers.
//!
//! The providers are built before the timer. No socket is opened.

use std::hint::black_box;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{AggregateProvider, LocalProvider, Provider, ProxyProvider};

fn direct_call(c: &mut Criterion) {
    let provider = LocalProvider::new(vec![("status", "ok")]);
    c.bench_function("direct_call", |b| {
        b.iter(|| {
            let body = provider.call("status");
            black_box(body);
        });
    });
}

fn proxy_call(c: &mut Criterion) {
    let inner: Arc<dyn Provider> = Arc::new(LocalProvider::new(vec![("status", "ok")]));
    let proxy = ProxyProvider::new(inner);
    c.bench_function("proxy_call", |b| {
        b.iter(|| {
            let body = proxy.call("status");
            black_box(body);
        });
    });
}

fn aggregate_10(c: &mut Criterion) {
    let parts: Vec<Arc<dyn Provider>> = (0..10)
        .map(|index| {
            let provider: Arc<dyn Provider> =
                Arc::new(LocalProvider::new(vec![(format!("tool{index}"), "ok")]));
            provider
        })
        .collect();
    let combined = AggregateProvider::new(parts);
    c.bench_function("aggregate_10", |b| {
        b.iter(|| {
            let names = combined.list_tools();
            black_box(names);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = direct_call, proxy_call, aggregate_10
);
criterion_main!(benches);
