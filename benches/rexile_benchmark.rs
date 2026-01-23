use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rexile::Pattern;

fn literal_search_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("literal_search");

    let text = "The quick brown fox jumps over the lazy dog. The fox is quick and clever.";
    let patterns = vec!["fox", "dog", "quick", "notfound"];

    for pattern_str in patterns {
        group.bench_with_input(
            BenchmarkId::new("rexile", pattern_str),
            &pattern_str,
            |b, &pattern_str| {
                let pattern = Pattern::new(pattern_str).unwrap();
                b.iter(|| black_box(pattern.is_match(black_box(text))));
            },
        );
    }

    group.finish();
}

fn multi_pattern_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_pattern");

    let text = "import React from 'react'; export default function MyComponent() { return null; }";

    // Test with different numbers of alternatives
    let patterns = vec![
        ("2_alternatives", "import|export"),
        ("4_alternatives", "import|export|function|return"),
        (
            "8_alternatives",
            "import|export|function|return|class|const|let|var",
        ),
    ];

    for (name, pattern_str) in patterns {
        group.bench_with_input(
            BenchmarkId::new("rexile", name),
            &pattern_str,
            |b, &pattern_str| {
                let pattern = Pattern::new(pattern_str).unwrap();
                b.iter(|| black_box(pattern.is_match(black_box(text))));
            },
        );
    }

    group.finish();
}

fn anchor_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("anchors");

    let text = "Hello World";

    let patterns = vec![
        ("start_anchor", "^Hello"),
        ("end_anchor", "World$"),
        ("exact_match", "^Hello World$"),
    ];

    for (name, pattern_str) in patterns {
        group.bench_with_input(
            BenchmarkId::new("rexile", name),
            &pattern_str,
            |b, &pattern_str| {
                let pattern = Pattern::new(pattern_str).unwrap();
                b.iter(|| black_box(pattern.is_match(black_box(text))));
            },
        );
    }

    group.finish();
}

fn find_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_operations");

    let text = "needle in a haystack with another needle and yet another needle";
    let pattern = Pattern::new("needle").unwrap();

    group.bench_function("find_first", |b| {
        b.iter(|| black_box(pattern.find(black_box(text))));
    });

    group.bench_function("find_all", |b| {
        b.iter(|| black_box(pattern.find_all(black_box(text))));
    });

    group.finish();
}

fn compilation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation");

    group.bench_function("compile_literal", |b| {
        b.iter(|| black_box(Pattern::new("hello").unwrap()));
    });

    group.bench_function("compile_multi_pattern", |b| {
        b.iter(|| black_box(Pattern::new("import|export|function|class").unwrap()));
    });

    group.bench_function("compile_anchored", |b| {
        b.iter(|| black_box(Pattern::new("^Hello$").unwrap()));
    });

    group.finish();
}

fn cached_api_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("cached_api");

    let text = "this is a test string for pattern matching";

    // First call (will compile and cache)
    let _ = rexile::is_match("test", text);

    group.bench_function("cached_is_match", |b| {
        b.iter(|| black_box(rexile::is_match("test", black_box(text)).unwrap()));
    });

    group.bench_function("cached_find", |b| {
        b.iter(|| black_box(rexile::find("test", black_box(text)).unwrap()));
    });

    group.finish();
}

fn text_size_scaling_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_size_scaling");

    let small_text = "hello world";
    let medium_text = "hello world ".repeat(100);
    let large_text = "hello world ".repeat(1000);

    let pattern = Pattern::new("world").unwrap();

    group.bench_function("small_100_bytes", |b| {
        b.iter(|| black_box(pattern.is_match(black_box(small_text))));
    });

    group.bench_function("medium_1kb", |b| {
        b.iter(|| black_box(pattern.is_match(black_box(&medium_text))));
    });

    group.bench_function("large_10kb", |b| {
        b.iter(|| black_box(pattern.is_match(black_box(&large_text))));
    });

    group.finish();
}

criterion_group!(
    benches,
    literal_search_benchmark,
    multi_pattern_benchmark,
    anchor_benchmark,
    find_operations_benchmark,
    compilation_benchmark,
    cached_api_benchmark,
    text_size_scaling_benchmark,
);

criterion_main!(benches);
