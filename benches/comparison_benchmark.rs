use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rexile::Pattern;
use regex::Regex;

// This benchmark compares ReXile against regex crate for common patterns
// Run with: cargo bench --bench comparison

fn literal_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("literal_comparison");
    
    let text = "The quick brown fox jumps over the lazy dog";
    let pattern_str = "fox";
    
    // ReXile
    group.bench_function("rexile_literal", |b| {
        let pattern = Pattern::new(pattern_str).unwrap();
        b.iter(|| {
            black_box(pattern.is_match(black_box(text)))
        });
    });
    
    // regex crate
    group.bench_function("regex_literal", |b| {
        let re = Regex::new(pattern_str).unwrap();
        b.iter(|| {
            black_box(re.is_match(black_box(text)))
        });
    });
    
    group.finish();
}

fn multi_pattern_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_pattern_comparison");
    
    let text = "import React from 'react'; export default function MyComponent() { return null; }";
    let pattern_str = "import|export|function|return";
    
    // ReXile (uses aho-corasick internally)
    group.bench_function("rexile_alternation", |b| {
        let pattern = Pattern::new(pattern_str).unwrap();
        b.iter(|| {
            black_box(pattern.is_match(black_box(text)))
        });
    });
    
    // regex crate
    group.bench_function("regex_alternation", |b| {
        let re = Regex::new(pattern_str).unwrap();
        b.iter(|| {
            black_box(re.is_match(black_box(text)))
        });
    });
    
    group.finish();
}

fn compilation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation_comparison");
    
    // Test compilation speed
    group.bench_function("rexile_compile_literal", |b| {
        b.iter(|| {
            black_box(Pattern::new("hello").unwrap())
        });
    });
    
    group.bench_function("rexile_compile_multi", |b| {
        b.iter(|| {
            black_box(Pattern::new("import|export|function|class").unwrap())
        });
    });
    
    // regex crate
    group.bench_function("regex_compile_literal", |b| {
        b.iter(|| {
            black_box(Regex::new("hello").unwrap())
        });
    });
    
    group.bench_function("regex_compile_multi", |b| {
        b.iter(|| {
            black_box(Regex::new("import|export|function|class").unwrap())
        });
    });
    
    group.finish();
}

fn memchr_raw_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("memchr_comparison");
    
    let text = "The quick brown fox jumps over the lazy dog";
    let needle = "fox";
    
    // ReXile (wraps memchr)
    group.bench_function("rexile", |b| {
        let pattern = Pattern::new(needle).unwrap();
        b.iter(|| {
            black_box(pattern.is_match(black_box(text)))
        });
    });
    
    // Raw memchr for comparison
    group.bench_function("memchr_raw", |b| {
        use memchr::memmem;
        b.iter(|| {
            black_box(memmem::find(black_box(text.as_bytes()), black_box(needle.as_bytes())).is_some())
        });
    });
    
    group.finish();
}

fn aho_corasick_raw_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("aho_corasick_comparison");
    
    let text = "import React from 'react'; export default function";
    let patterns = vec!["import", "export", "function", "return"];
    
    // ReXile (wraps aho-corasick)
    group.bench_function("rexile", |b| {
        let pattern = Pattern::new("import|export|function|return").unwrap();
        b.iter(|| {
            black_box(pattern.is_match(black_box(text)))
        });
    });
    
    // Raw aho-corasick for comparison
    group.bench_function("aho_corasick_raw", |b| {
        use aho_corasick::AhoCorasick;
        let ac = AhoCorasick::new(&patterns).unwrap();
        b.iter(|| {
            black_box(ac.is_match(black_box(text)))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    literal_comparison,
    multi_pattern_comparison,
    compilation_comparison,
    memchr_raw_comparison,
    aho_corasick_raw_comparison,
);

criterion_main!(benches);
