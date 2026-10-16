//! One task from submit to result, and one thousand tasks at once.
//!
//! The runtime and the board are built before the timer. The work closure
//! returns "ok" immediately.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{ExtensionRegistry, TaskBoard, TASKS_EXTENSION_ID};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime")
}

fn board() -> TaskBoard {
    let mut registry = ExtensionRegistry::new();
    registry.insert(TASKS_EXTENSION_ID, ());
    TaskBoard::new(&registry).expect("extension")
}

fn submit_to_result(c: &mut Criterion) {
    let rt = runtime();
    let board = board();
    c.bench_function("submit_to_result", |b| {
        b.iter(|| {
            rt.block_on(async {
                let id = board.submit(|| "ok".to_string());
                let value = board.wait(id).await;
                black_box(value);
            });
        });
    });
}

fn concurrent_1k(c: &mut Criterion) {
    let rt = runtime();
    let board = board();
    c.bench_function("concurrent_1k", |b| {
        b.iter(|| {
            rt.block_on(async {
                let ids: Vec<u64> = (0..1_000)
                    .map(|_| board.submit(|| "ok".to_string()))
                    .collect();
                for id in ids {
                    let value = board.wait(id).await;
                    black_box(value);
                }
            });
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = submit_to_result, concurrent_1k
);
criterion_main!(benches);
