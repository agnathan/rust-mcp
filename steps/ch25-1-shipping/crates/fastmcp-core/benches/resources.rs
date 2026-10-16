//! Exact lookup among 10,000 static resources, and a scan of 10,000 templates.
//!
//! The servers are built before `b.iter`. `resource_lookup_10k` is one
//! `HashMap` get. `template_scan_10k` walks every template; the URI matches
//! the last pattern, so the scan does not stop early.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::FastMcp;

fn ten_thousand_static() -> FastMcp {
    let mut server = FastMcp::new("fastmcp-rs");
    for index in 0..10_000 {
        server = server.resource(
            format!("file://doc-{index}"),
            format!("doc-{index}"),
            "text/plain",
            "body",
        );
    }
    server
}

fn ten_thousand_templates() -> FastMcp {
    let mut server = FastMcp::new("fastmcp-rs");
    for index in 0..10_000 {
        server = server.template(
            format!("res://item-{index}/{{name}}"),
            format!("item-{index}"),
            "text/plain",
            |_| Ok("found".to_string()),
        );
    }
    server
}

fn resource_lookup_10k(c: &mut Criterion) {
    let server = ten_thousand_static();
    let uri = "file://doc-5000";
    c.bench_function("resource_lookup_10k", |b| {
        b.iter(|| black_box(server.read(black_box(uri))));
    });
}

fn template_scan_10k(c: &mut Criterion) {
    let server = ten_thousand_templates();
    let uri = "res://item-9999/readme";
    c.bench_function("template_scan_10k", |b| {
        b.iter(|| black_box(server.read(black_box(uri))));
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = resource_lookup_10k, template_scan_10k
);
criterion_main!(benches);
