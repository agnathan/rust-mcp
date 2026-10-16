//! One loopback round trip, and one discover next to one initialize.
//!
//! The listener is bound before `b.iter`. Each sample opens its own TCP
//! connection, writes one HTTP request, and reads the reply. A handshake
//! sample also deletes the session so the worker does not stay up.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{delete_mcp, post_mcp, FastMcp};

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"bench","version":"0.1.0"}}}"#;

const LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

const DISCOVER: &str = r#"{"jsonrpc":"2.0","id":3,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"bench","version":"0.1.0"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime")
}

fn http_roundtrip(c: &mut Criterion) {
    let rt = runtime();
    let server = rt
        .block_on(FastMcp::new("fastmcp-rs").serve_http())
        .expect("bind");
    let url = server.url().to_string();
    c.bench_function("http_roundtrip", |b| {
        b.iter(|| {
            rt.block_on(async {
                let init = post_mcp(&url, &[("Mcp-Method", "initialize")], INIT)
                    .await
                    .expect("initialize");
                let session = init.header("mcp-session-id").expect("session").to_string();
                let listed = post_mcp(
                    &url,
                    &[
                        ("Mcp-Session-Id", &session),
                        ("MCP-Protocol-Version", "2025-06-18"),
                        ("Mcp-Method", "tools/list"),
                    ],
                    LIST,
                )
                .await
                .expect("tools/list");
                delete_mcp(
                    &url,
                    &[
                        ("Mcp-Session-Id", &session),
                        ("MCP-Protocol-Version", "2025-06-18"),
                    ],
                )
                .await
                .expect("delete");
                black_box(listed.status);
            });
        });
    });
    server.shutdown();
}

fn initialize_once(c: &mut Criterion) {
    let rt = runtime();
    let server = rt
        .block_on(FastMcp::new("fastmcp-rs").serve_http())
        .expect("bind");
    let url = server.url().to_string();
    c.bench_function("initialize_once", |b| {
        b.iter(|| {
            rt.block_on(async {
                let init = post_mcp(&url, &[("Mcp-Method", "initialize")], INIT)
                    .await
                    .expect("initialize");
                let session = init.header("mcp-session-id").expect("session").to_string();
                delete_mcp(
                    &url,
                    &[
                        ("Mcp-Session-Id", &session),
                        ("MCP-Protocol-Version", "2025-06-18"),
                    ],
                )
                .await
                .expect("delete");
                black_box(init.status);
            });
        });
    });
    server.shutdown();
}

fn discover_once(c: &mut Criterion) {
    let rt = runtime();
    let server = rt
        .block_on(FastMcp::new("fastmcp-rs").serve_http())
        .expect("bind");
    let url = server.url().to_string();
    c.bench_function("discover_once", |b| {
        b.iter(|| {
            rt.block_on(async {
                let reply = post_mcp(
                    &url,
                    &[
                        ("MCP-Protocol-Version", "2026-07-28"),
                        ("Mcp-Method", "server/discover"),
                    ],
                    DISCOVER,
                )
                .await
                .expect("discover");
                black_box(reply.status);
            });
        });
    });
    server.shutdown();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = http_roundtrip, initialize_once, discover_once
);
criterion_main!(benches);
