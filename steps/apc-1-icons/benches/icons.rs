//! Check 1,000 https icons.
//!
//! The icons are built before the timer. The timed work is `count_ok`.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use mcp_icons::{count_ok, Icon, IconTheme};

fn check_1k(c: &mut Criterion) {
    let icons: Vec<Icon> = (0..1_000)
        .map(|index| Icon {
            src: format!("https://example.test/icon-{index}.png"),
            mime_type: Some("image/png".to_string()),
            sizes: vec!["48x48".to_string()],
            theme: Some(IconTheme::Light),
        })
        .collect();
    c.bench_function("check_1k", |b| {
        b.iter(|| {
            let count = count_ok(black_box(&icons));
            black_box(count);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = check_1k
);
criterion_main!(benches);
