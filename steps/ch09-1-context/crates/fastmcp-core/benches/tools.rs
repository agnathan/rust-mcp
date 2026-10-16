//! Lookup and dispatch over a registry of 1,000 tools.
//!
//! The server and the argument JSON are built before `b.iter`.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::FastMcp;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct Echo {
    n: i64,
}

fn thousand() -> (FastMcp, Vec<String>, serde_json::Value) {
    let mut server = FastMcp::new("fastmcp-rs");
    let mut names = Vec::with_capacity(1000);
    for index in 0..1000 {
        let name = format!("tool-{index}");
        server = server.tool(name.clone(), "echo n", |args: Echo| Ok(args.n));
        names.push(name);
    }
    (server, names, serde_json::json!({"n": 1}))
}

fn lookup_one(c: &mut Criterion) {
    let (server, names, _) = thousand();
    let name = names[500].clone();
    c.bench_function("lookup_1", |b| {
        b.iter(|| black_box(server.has_tool(black_box(&name))));
    });
}

fn dispatch_thousand(c: &mut Criterion) {
    let (server, names, arguments) = thousand();
    c.bench_function("dispatch_1000", |b| {
        b.iter(|| {
            let mut last = None;
            for name in &names {
                last = Some(server.dispatch_tool(black_box(name), arguments.clone()));
            }
            black_box(last);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = lookup_one, dispatch_thousand
);
criterion_main!(benches);
