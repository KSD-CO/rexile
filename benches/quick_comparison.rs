use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rexile::Pattern;
use regex::Regex;
use std::time::Duration;

// Quick comparison benchmark - runs faster than full benchmark
// Run with: cargo bench --bench quick_comparison

fn quick_literal_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_literal");
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(20);
    
    let text = "The quick brown fox jumps over the lazy dog";
    let pattern_str = "fox";
    
    group.bench_function("rexile", |b| {
        let pattern = Pattern::new(pattern_str).unwrap();
        b.iter(|| black_box(pattern.is_match(black_box(text))));
    });
    
    group.bench_function("regex", |b| {
        let re = Regex::new(pattern_str).unwrap();
        b.iter(|| black_box(re.is_match(black_box(text))));
    });
    
    group.finish();
}

fn quick_multi_pattern_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_multi_pattern");
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(20);
    
    let text = "import React from 'react'; export default function MyComponent() { return null; }";
    
    // Test with different numbers of alternations
    let tests = vec![
        ("2_patterns", "import|export"),
        ("4_patterns", "import|export|function|return"),
        ("8_patterns", "import|export|function|return|class|const|let|var"),
    ];
    
    for (name, pattern_str) in tests {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern_str, |b, pattern_str| {
            let pattern = Pattern::new(pattern_str).unwrap();
            b.iter(|| black_box(pattern.is_match(black_box(text))));
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern_str, |b, pattern_str| {
            let re = Regex::new(pattern_str).unwrap();
            b.iter(|| black_box(re.is_match(black_box(text))));
        });
    }
    
    group.finish();
}

fn quick_compilation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_compilation");
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(20);
    
    group.bench_function("rexile_literal", |b| {
        b.iter(|| black_box(Pattern::new("hello").unwrap()));
    });
    
    group.bench_function("regex_literal", |b| {
        b.iter(|| black_box(Regex::new("hello").unwrap()));
    });
    
    group.bench_function("rexile_multi", |b| {
        b.iter(|| black_box(Pattern::new("import|export|function|class").unwrap()));
    });
    
    group.bench_function("regex_multi", |b| {
        b.iter(|| black_box(Regex::new("import|export|function|class").unwrap()));
    });
    
    group.finish();
}

fn quick_anchor_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_anchors");
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(20);
    
    let text = "Hello World";
    
    // Start anchor
    group.bench_function("rexile_start", |b| {
        let pattern = Pattern::new("^Hello").unwrap();
        b.iter(|| black_box(pattern.is_match(black_box(text))));
    });
    
    group.bench_function("regex_start", |b| {
        let re = Regex::new("^Hello").unwrap();
        b.iter(|| black_box(re.is_match(black_box(text))));
    });
    
    // End anchor
    group.bench_function("rexile_end", |b| {
        let pattern = Pattern::new("World$").unwrap();
        b.iter(|| black_box(pattern.is_match(black_box(text))));
    });
    
    group.bench_function("regex_end", |b| {
        let re = Regex::new("World$").unwrap();
        b.iter(|| black_box(re.is_match(black_box(text))));
    });
    
    group.finish();
}

criterion_group!(
    benches,
    quick_literal_comparison,
    quick_multi_pattern_comparison,
    quick_compilation_comparison,
    quick_anchor_comparison,
);

criterion_main!(benches);
