//! Criterion benches for chapter 2.
//!
//! Fixtures are built before `b.iter`. The timed region is the closure
//! passed to `iter`.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{choose_version, FastMcp, SUPPORTED_VERSIONS};
use rmcp::model::ProtocolVersion;
use rmcp::{model::ClientConfig, ClientHandler, ServiceExt};

fn negotiate_one(c: &mut Criterion) {
    let requested = ProtocolVersion::V_2025_06_18;
    c.bench_function("negotiate_1", |b| {
        b.iter(|| {
            let chosen =
                choose_version(black_box(&requested), SUPPORTED_VERSIONS).expect("version");
            black_box(chosen);
        });
    });
}

fn negotiate_thousand(c: &mut Criterion) {
    let requested: Vec<ProtocolVersion> = (0..1000)
        .map(|index| {
            if index % 5 == 0 {
                ProtocolVersion::V_2026_07_28
            } else {
                ProtocolVersion::V_2025_06_18
            }
        })
        .collect();
    c.bench_function("negotiate_1000", |b| {
        b.iter(|| {
            let mut last = None;
            for version in &requested {
                last =
                    Some(choose_version(black_box(version), SUPPORTED_VERSIONS).expect("version"));
            }
            black_box(last);
        });
    });
}

struct BenchClient;

impl ClientHandler for BenchClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = ProtocolVersion::V_2025_06_18;
        info
    }
}

fn stdio_initialize_list(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");
    c.bench_function("stdio_initialize_list", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let (server_io, client_io) = tokio::io::duplex(64 * 1024);
                tokio::spawn(async move {
                    let running = FastMcp::new("fastmcp-rs")
                        .serve(server_io)
                        .await
                        .expect("server");
                    let _ = running.waiting().await;
                });
                let client = BenchClient.serve(client_io).await.expect("client");
                let tools = client
                    .peer()
                    .list_tools(Default::default())
                    .await
                    .expect("list");
                client.cancel().await.expect("cancel");
                black_box(tools.tools.len());
            });
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = negotiate_one, negotiate_thousand, stdio_initialize_list
);
criterion_main!(benches);
