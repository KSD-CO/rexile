//! Cover perf_compare's literals without changing the original regression harness.

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use regex::Regex;
use rexile::Pattern;

const CASES: &[(&str, &str, &str)] = &[
    (
        "literal_log",
        "ERROR",
        "2024-01-15 ERROR [main] Connection timeout after 30s retry=3 user=admin@example.com",
    ),
    (
        "literal_code",
        "calculate_total",
        "fn calculate_total(items: Vec<Item>) -> Result<f64, Error> { Ok(0.0) }",
    ),
];

fn literal_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_literals");
    group.sample_size(100);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_millis(500));

    for &(name, pattern, text) in CASES {
        let rexile = Pattern::new(pattern).unwrap();
        let regex = Regex::new(pattern).unwrap();
        group.bench_function(BenchmarkId::new("rexile", name), |b| {
            b.iter(|| black_box(rexile.is_match(black_box(text))))
        });
        group.bench_function(BenchmarkId::new("regex", name), |b| {
            b.iter(|| black_box(regex.is_match(black_box(text))))
        });
    }
    group.finish();
}

criterion_group!(benches, literal_queries);
criterion_main!(benches);
