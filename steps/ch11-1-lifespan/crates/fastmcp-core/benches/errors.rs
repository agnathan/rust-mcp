//! A thousand successful dispatches, next to a thousand lookups that miss.
//!
//! The server and the argument JSON are built before `b.iter`. The error
//! path never enters the closure.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::FastMcp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
struct Add {
    left: i64,
    right: i64,
}

#[derive(Serialize, JsonSchema)]
struct Sum {
    total: i64,
}

fn server() -> (FastMcp, serde_json::Value) {
    let server = FastMcp::new("fastmcp-rs").tool("add", "add", |args: Add| {
        Ok(Sum {
            total: args.left + args.right,
        })
    });
    (server, serde_json::json!({"left": 2, "right": 3}))
}

fn dispatch_ok_1000(c: &mut Criterion) {
    let (server, arguments) = server();
    c.bench_function("dispatch_ok_1000", |b| {
        b.iter(|| {
            let mut last = None;
            for _ in 0..1000 {
                last = Some(server.dispatch_tool(black_box("add"), arguments.clone()));
            }
            black_box(last);
        });
    });
}

fn dispatch_missing_1000(c: &mut Criterion) {
    let (server, arguments) = server();
    c.bench_function("dispatch_missing_1000", |b| {
        b.iter(|| {
            let mut last = None;
            for _ in 0..1000 {
                last = Some(server.dispatch_tool(black_box("missing"), arguments.clone()));
            }
            black_box(last);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = dispatch_ok_1000, dispatch_missing_1000
);
criterion_main!(benches);
