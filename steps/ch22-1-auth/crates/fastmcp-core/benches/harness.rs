//! A direct tool list, and the same list through the duplex pipe.
//!
//! The duplex is the transport stdio tests use. No process is spawned.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{FastMcp, Harness, McpClient};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime")
}

fn memory_list(c: &mut Criterion) {
    let harness = Harness::new(FastMcp::new("widgets"));
    c.bench_function("memory_list", |b| {
        b.iter(|| {
            let names = harness.tool_names();
            black_box(names);
        });
    });
}

fn duplex_list(c: &mut Criterion) {
    let rt = runtime();
    let client = rt
        .block_on(McpClient::connect_memory(FastMcp::new("widgets")))
        .expect("connect");
    c.bench_function("duplex_list", |b| {
        b.iter(|| {
            rt.block_on(async {
                let names = client.tool_names().await.expect("list");
                black_box(names);
            });
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = memory_list, duplex_list
);
criterion_main!(benches);
