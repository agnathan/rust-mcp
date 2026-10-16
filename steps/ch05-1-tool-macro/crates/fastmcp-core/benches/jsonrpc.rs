//! Criterion benches for chapter 1.
//!
//! Each function builds its input before `b.iter`. The timed region is only
//! the closure passed to `iter`. `black_box` keeps the compiler from deleting
//! the work.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use fastmcp_core::{dispatch, parse_message, Message, Request, RequestId, PROTOCOL_VERSION};
use serde_json::json;

fn initialize_request(id: i64) -> Request {
    Request::new(
        RequestId::Number(id),
        "initialize",
        Some(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "bench", "version": "0.1.0"}
        })),
    )
}

fn one_initialize_bytes() -> Vec<u8> {
    serde_json::to_vec(&initialize_request(1)).expect("fixture")
}

fn thousand_initialize_bytes() -> Vec<Vec<u8>> {
    (0..1000)
        .map(|id| serde_json::to_vec(&initialize_request(id)).expect("fixture"))
        .collect()
}

fn thousand_messages() -> Vec<Message> {
    (0..1000)
        .map(|id| Message::Request(initialize_request(id)))
        .collect()
}

fn parse_1_message(c: &mut Criterion) {
    let bytes = one_initialize_bytes();
    c.bench_function("parse_1_message", |b| {
        b.iter(|| {
            let message = parse_message(black_box(&bytes)).expect("parse");
            black_box(message);
        });
    });
}

fn parse_1000_messages(c: &mut Criterion) {
    let batch = thousand_initialize_bytes();
    c.bench_function("parse_1000_messages", |b| {
        b.iter(|| {
            let mut count = 0;
            for bytes in &batch {
                let message = parse_message(black_box(bytes)).expect("parse");
                black_box(message);
                count += 1;
            }
            black_box(count);
        });
    });
}

fn serialize_1_message(c: &mut Criterion) {
    let message = Message::Request(initialize_request(1));
    c.bench_function("serialize_1_message", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(black_box(&message)).expect("serialize");
            black_box(bytes);
        });
    });
}

fn serialize_1000_messages(c: &mut Criterion) {
    let messages = thousand_messages();
    c.bench_function("serialize_1000_messages", |b| {
        b.iter(|| {
            let mut total = 0usize;
            for message in &messages {
                let bytes = serde_json::to_vec(black_box(message)).expect("serialize");
                total += bytes.len();
                black_box(bytes);
            }
            black_box(total);
        });
    });
}

fn dispatch_initialize(c: &mut Criterion) {
    let message = Message::Request(initialize_request(1));
    c.bench_function("dispatch_initialize", |b| {
        b.iter(|| {
            let response = dispatch(black_box(&message)).expect("dispatch");
            black_box(response);
        });
    });
}

criterion_group!(
    benches,
    parse_1_message,
    parse_1000_messages,
    serialize_1_message,
    serialize_1000_messages,
    dispatch_initialize
);
criterion_main!(benches);
