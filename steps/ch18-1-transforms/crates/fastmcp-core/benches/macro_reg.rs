//! Registration the macro emits, next to a direct call of the same function.
//!
//! The proc macro already ran, at compile time. This bench times what is left
//! at runtime: building the schema inside `.tool()`, and calling the function
//! the macro left in place.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::FastMcp;
use fastmcp_macros::tool;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema, Clone, Copy)]
pub struct Sum {
    total: i64,
}

#[tool(description = "add two integers")]
fn add(left: i64, right: i64) -> Sum {
    Sum {
        total: left + right,
    }
}

fn register_macro_1(c: &mut Criterion) {
    c.bench_function("register_macro_1", |b| {
        b.iter(|| black_box(FastMcp::new("fastmcp-rs").tool_decl::<AddTool>()));
    });
}

fn call_fn_1000(c: &mut Criterion) {
    c.bench_function("call_fn_1000", |b| {
        b.iter(|| {
            let mut total = 0;
            for n in 0..1000 {
                total += black_box(add(n, 1)).total;
            }
            black_box(total);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = register_macro_1, call_fn_1000
);
criterion_main!(benches);
