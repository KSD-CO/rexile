use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rexile::Pattern as RexilePattern;
use regex::Regex;

fn benchmark_literal_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Literal Patterns");
    
    let text = "hello world hello world hello world";
    let patterns = vec![
        ("hello", "Simple literal"),
        ("world", "Another literal"),
        ("hello world", "Multi-word literal"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_anchored_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Anchored Patterns");
    
    let text = "hello world this is a test";
    let patterns = vec![
        ("^hello", "Start anchor"),
        ("test$", "End anchor"),
        ("^hello world", "Start with phrase"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_character_classes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Character Classes");
    
    let text = "abc123xyz789";
    let patterns = vec![
        ("[a-z]+", "Lowercase letters"),
        ("[0-9]+", "Digits"),
        ("[a-zA-Z]+", "All letters"),
        ("[^0-9]+", "Non-digits"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_escape_sequences(c: &mut Criterion) {
    let mut group = c.benchmark_group("Escape Sequences");
    
    let text = "abc 123 def 456 xyz";
    let patterns = vec![
        (r"\d+", "Digits"),
        (r"\w+", "Word chars"),
        (r"\s+", "Whitespace"),
        (r"\d+\s+\w+", "Mixed sequence"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_quantifiers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quantifiers");
    
    let text = "aaabbbcccddd";
    let patterns = vec![
        ("a+", "Plus"),
        ("b*", "Star"),
        ("c?", "Question"),
        ("a+b+c+", "Multiple plus"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_groups(c: &mut Criterion) {
    let mut group = c.benchmark_group("Groups");
    
    let text = "foo bar baz";
    let patterns = vec![
        ("(foo)", "Simple group"),
        ("(foo|bar)", "Alternation group"),
        ("^(hello)", "Anchored group"),
        (r"(\d+)", "Group with escape"),
        (r"(\w+)@", "Group with suffix"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_alternation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Alternation");
    
    let text = "the quick brown fox jumps over the lazy dog";
    let patterns = vec![
        ("cat|dog", "2 alternatives"),
        ("cat|dog|fox", "3 alternatives"),
        ("one|two|three|four|five", "5 alternatives"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_find_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("Find All");
    
    let text = "test1 test2 test3 test4 test5";
    let patterns = vec![
        ("test", "Simple literal"),
        (r"\d+", "Digits"),
        (r"test\d+", "Literal + digit"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.find_all(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                // FIXED: Collect to Vec for fair comparison with rexile
                black_box(re.find_iter(text)
                    .map(|m| (m.start(), m.end()))
                    .collect::<Vec<_>>())
            });
        });
    }
    
    group.finish();
}

fn benchmark_large_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Text");
    
    // Generate large text
    let text = "hello world ".repeat(1000);
    let patterns = vec![
        ("hello", "Simple literal"),
        (r"\w+", "Word pattern"),
        ("hello world", "Multi-word"),
    ];
    
    for (pattern, name) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(&text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(&text)))
            });
        });
    }
    
    group.finish();
}

fn benchmark_real_world_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Real World Patterns");
    
    let email = "user@example.com test admin@domain.org";
    let version = "Version: 1.2.3 and 4.5.6";
    let url = "Visit https://example.com or http://test.org";
    
    let patterns = vec![
        (r"\w+@\w+\.\w+", "Email pattern", email),
        (r"\d+\.\d+\.\d+", "Version number", version),
        (r"https?://\w+\.\w+", "URL pattern", url),
    ];
    
    for (pattern, name, text) in patterns {
        group.bench_with_input(BenchmarkId::new("rexile", name), &pattern, |b, p| {
            let re = RexilePattern::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("regex", name), &pattern, |b, p| {
            let re = Regex::new(p).unwrap();
            b.iter(|| {
                black_box(re.is_match(black_box(text)))
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_literal_patterns,
    benchmark_anchored_patterns,
    benchmark_character_classes,
    benchmark_escape_sequences,
    benchmark_quantifiers,
    benchmark_groups,
    benchmark_alternation,
    benchmark_find_all,
    benchmark_large_text,
    benchmark_real_world_patterns,
);
criterion_main!(benches);
