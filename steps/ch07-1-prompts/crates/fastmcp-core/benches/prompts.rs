//! One prompt render, and one page taken from the middle of 10,000 items.
//!
//! The server, the argument map, the 10,000 strings, and the cursor are built
//! before `b.iter`.

use std::collections::HashMap;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{encode_cursor, paginate_sequence, FastMcp};

fn greet() -> (FastMcp, HashMap<String, String>) {
    let server =
        FastMcp::new("fastmcp-rs").prompt("greet", "Greet someone", &[("name", true)], |args| {
            let name = args.get("name").map(String::as_str).unwrap_or("there");
            Ok(format!("Hello, {name}"))
        });
    let mut arguments = HashMap::new();
    arguments.insert("name".into(), "Ada".into());
    (server, arguments)
}

fn prompt_render_1(c: &mut Criterion) {
    let (server, arguments) = greet();
    c.bench_function("prompt_render_1", |b| {
        b.iter(|| black_box(server.render_prompt(black_box("greet"), black_box(&arguments))));
    });
}

fn paginate_10k(c: &mut Criterion) {
    let items: Vec<String> = (0..10_000).map(|index| format!("item-{index}")).collect();
    let cursor = encode_cursor(5_000);
    c.bench_function("paginate_10k", |b| {
        b.iter(|| {
            black_box(paginate_sequence(
                black_box(&items),
                black_box(Some(cursor.as_str())),
                black_box(50),
            ))
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = prompt_render_1, paginate_10k
);
criterion_main!(benches);
