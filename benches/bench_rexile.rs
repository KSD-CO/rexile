use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_rexile_is_match(c: &mut Criterion) {
    let pat = "^rule\\s+\\d+$";
    let text = "rule 12345";

    c.bench_function("rexile::is_match_cached", |b| {
        b.iter(|| {
            let _ = rexile::is_match(black_box(pat), black_box(text)).unwrap();
        })
    });
}

fn bench_regex_compile_and_match(c: &mut Criterion) {
    let pat = "^rule\\s+\\d+$";
    let text = "rule 12345";

    c.bench_function("regex::compile_and_match", |b| {
        b.iter(|| {
            let r = regex::Regex::new(black_box(pat)).unwrap();
            let _ = r.is_match(black_box(text));
        })
    });
}

fn bench_regex_compile_once_and_match(c: &mut Criterion) {
    let pat = "^rule\\s+\\d+$";
    let text = "rule 12345";

    let r = regex::Regex::new(pat).unwrap();
    c.bench_function("regex::compile_once_and_match", |b| {
        b.iter(|| {
            let _ = r.is_match(black_box(text));
        })
    });
}

criterion_group!(
    benches,
    bench_rexile_is_match,
    bench_regex_compile_and_match,
    bench_regex_compile_once_and_match
);
criterion_main!(benches);
