//! Turn a 1,000-operation OpenAPI document into tools.
//!
//! The JSON text is built before the timer. The timed work parses it and
//! walks paths.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::tools_from_spec;
use serde_json::Value;

fn spec_text() -> String {
    let mut paths = String::from(r#"{"openapi":"3.0.0","paths":{"#);
    for index in 0..1_000 {
        if index > 0 {
            paths.push(',');
        }
        paths.push_str(&format!(
            r#""/widgets/{index}":{{"get":{{"operationId":"get_widget_{index}"}}}}"#
        ));
    }
    paths.push_str("}}");
    paths
}

fn parse_1k(c: &mut Criterion) {
    let text = spec_text();
    c.bench_function("parse_1k", |b| {
        b.iter(|| {
            let value: Value = serde_json::from_str(&text).expect("json");
            let tools = tools_from_spec(&value).expect("spec");
            black_box(tools.operations.len());
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = parse_1k
);
criterion_main!(benches);
