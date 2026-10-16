//! Four threads read one RwLock.
//!
//! The lock is built before `b.iter`. Each sample spawns four threads. Each
//! thread reads the banner 1,000 times. Spawn and join are inside the timer,
//! because that is the cost of sharing the state across threads.

use std::hint::black_box;
use std::sync::{Arc, RwLock};
use std::thread;

use criterion::{criterion_group, criterion_main, Criterion};

fn contended_reads(c: &mut Criterion) {
    let banner = Arc::new(RwLock::new(String::from("ready")));
    c.bench_function("contended_reads", |b| {
        b.iter(|| {
            let mut handles = Vec::with_capacity(4);
            for _ in 0..4 {
                let banner = Arc::clone(&banner);
                handles.push(thread::spawn(move || {
                    let mut last = 0;
                    for _ in 0..1000 {
                        last += black_box(banner.read().expect("read").len());
                    }
                    last
                }));
            }
            let total: usize = handles
                .into_iter()
                .map(|handle| handle.join().expect("join"))
                .sum();
            black_box(total);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = contended_reads
);
criterion_main!(benches);
