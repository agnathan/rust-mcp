//! One in-process elicitation round.
//!
//! The desk is built inside the sample. The timer includes the channel, the
//! wait, and the submit. There is no socket.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::Desk;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime")
}

fn elicit_round(c: &mut Criterion) {
    let rt = runtime();
    c.bench_function("elicit_round", |b| {
        b.iter(|| {
            rt.block_on(async {
                let desk = Desk::default();
                let answerer = desk.clone();
                let pending = tokio::spawn(async move { answerer.submit("yes").await });
                let value = desk.ask("Ship it?").await.expect("ask");
                pending.await.expect("join").expect("submit");
                black_box(value);
            });
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = elicit_round
);
criterion_main!(benches);
