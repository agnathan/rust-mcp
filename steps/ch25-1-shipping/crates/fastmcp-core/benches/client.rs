//! One in-memory tools/list, and a fan-out across eight servers.
//!
//! The servers are connected before `b.iter`. The timer is the list. No
//! process is spawned.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{ClientGroup, FastMcp, McpClient};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime")
}

fn list_once(c: &mut Criterion) {
    let rt = runtime();
    let client = rt
        .block_on(McpClient::connect_memory(FastMcp::new("widgets")))
        .expect("connect");
    c.bench_function("list_once", |b| {
        b.iter(|| {
            rt.block_on(async {
                let names = client.tool_names().await.expect("list");
                black_box(names);
            });
        });
    });
}

fn fanout_8(c: &mut Criterion) {
    let rt = runtime();
    let members = rt.block_on(async {
        let mut members = Vec::with_capacity(8);
        for index in 0..8 {
            let client = McpClient::connect_memory(FastMcp::new(format!("s{index}")))
                .await
                .expect("connect");
            members.push((format!("s{index}"), client));
        }
        ClientGroup::new(members).expect("group")
    });
    c.bench_function("fanout_8", |b| {
        b.iter(|| {
            rt.block_on(async {
                let names = members.list_tools().await.expect("list");
                black_box(names.len());
            });
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = list_once, fanout_8
);
criterion_main!(benches);
