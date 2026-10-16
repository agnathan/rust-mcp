//! One thousand calls with no hooks, and the same thousand with five hooks.
//!
//! The five hooks all continue. The cache is not in this timer, because a
//! hit would skip the tool.

use std::hint::black_box;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{CallLog, FastMcp, RateLimit, ResponseLimit, Timing};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
struct Echo {
    text: String,
}

fn server(layered: bool) -> FastMcp {
    let server = FastMcp::new("widgets").tool("echo", "echo", |args: Echo| Ok(args));
    if !layered {
        return server;
    }
    server
        .with_hook(Arc::new(CallLog::default()))
        .with_hook(Arc::new(Timing::default()))
        .with_hook(Arc::new(RateLimit::new(1.0e15, 1.0e15)))
        .with_hook(Arc::new(ResponseLimit::new(1_000_000).expect("limit")))
        .with_hook(Arc::new(CallLog::default()))
}

fn layers_0(c: &mut Criterion) {
    let server = server(false);
    let args = serde_json::json!({"text": "hi"});
    c.bench_function("layers_0", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let value = server.invoke("echo", args.clone()).expect("call");
                black_box(value);
            }
        });
    });
}

fn layers_5(c: &mut Criterion) {
    let server = server(true);
    let args = serde_json::json!({"text": "hi"});
    c.bench_function("layers_5", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let value = server.invoke("echo", args.clone()).expect("call");
                black_box(value);
            }
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = layers_0, layers_5
);
criterion_main!(benches);
