use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use regex::Regex;
use rexile::Pattern;

struct SearchWorkload {
    name: &'static str,
    pattern: &'static str,
    text: &'static str,
}

const SEARCH_WORKLOADS: &[SearchWorkload] = &[
    SearchWorkload {
        name: "literal_short",
        pattern: "ERROR",
        text: "INFO ERROR WARN ERROR",
    },
    SearchWorkload {
        name: "literal_long",
        pattern: "needle",
        text: "hay hay hay hay hay hay hay hay hay hay hay hay hay hay hay hay hay needle",
    },
    SearchWorkload {
        name: "alternation_keywords",
        pattern: "import|export|function|return",
        text: "const value = function() { return import_name; } export value;",
    },
    SearchWorkload {
        name: "digit_run",
        pattern: r"\d+",
        text: "Order #12345 costs 67890 units",
    },
    SearchWorkload {
        name: "word_run",
        pattern: r"\w+",
        text: "hello_world 123 next_token",
    },
    SearchWorkload {
        name: "digit_class",
        pattern: "[0-9]+",
        text: "abc 123 def 456 ghi 789",
    },
    SearchWorkload {
        name: "identifier_class",
        pattern: r"[a-zA-Z_]\w*",
        text: "123 _identifier next_value final123",
    },
    SearchWorkload {
        name: "sequence_word_space_digit",
        pattern: r"\w+\s+\d+",
        text: "item 123 next 456 final",
    },
    SearchWorkload {
        name: "sequence_decimal",
        pattern: r"\d+\.\d+",
        text: "version 12.34 done 56.78",
    },
    SearchWorkload {
        name: "bounded_digits",
        pattern: r"\d{4}",
        text: "year 2026 and code 1234",
    },
    SearchWorkload {
        name: "case_insensitive_literal",
        pattern: r"(?i)error",
        text: "INFO Error WARN error",
    },
    SearchWorkload {
        name: "anchored_exact",
        pattern: "^rule 123$",
        text: "rule 123",
    },
    SearchWorkload {
        name: "word_boundaries",
        pattern: r"\btest\b",
        text: "testing test tested test",
    },
    SearchWorkload {
        name: "dot_wildcard",
        pattern: "a.*c",
        text: "a quick brown fox c and abc",
    },
    SearchWorkload {
        name: "lazy_dot_wildcard",
        pattern: "a.*?c",
        text: "a quick brown fox c and abc",
    },
];

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_millis(500));
}

fn compilation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_supported_subset");
    configure_group(&mut group);

    for workload in SEARCH_WORKLOADS {
        group.bench_with_input(
            BenchmarkId::new("rexile", workload.name),
            &workload.pattern,
            |b, &pattern| b.iter(|| black_box(Pattern::new(black_box(pattern)).unwrap())),
        );
        group.bench_with_input(
            BenchmarkId::new("regex", workload.name),
            &workload.pattern,
            |b, &pattern| b.iter(|| black_box(Regex::new(black_box(pattern)).unwrap())),
        );
    }

    group.finish();
}

fn is_match_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_match_supported_subset");
    configure_group(&mut group);

    for workload in SEARCH_WORKLOADS {
        let rexile = Pattern::new(workload.pattern).unwrap();
        let regex = Regex::new(workload.pattern).unwrap();

        group.bench_with_input(
            BenchmarkId::new("rexile", workload.name),
            &workload.text,
            |b, &text| b.iter(|| black_box(rexile.is_match(black_box(text)))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex", workload.name),
            &workload.text,
            |b, &text| b.iter(|| black_box(regex.is_match(black_box(text)))),
        );
    }

    group.finish();
}

fn find_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_supported_subset");
    configure_group(&mut group);

    for workload in SEARCH_WORKLOADS {
        let rexile = Pattern::new(workload.pattern).unwrap();
        let regex = Regex::new(workload.pattern).unwrap();

        group.bench_with_input(
            BenchmarkId::new("rexile", workload.name),
            &workload.text,
            |b, &text| b.iter(|| black_box(rexile.find(black_box(text)))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex", workload.name),
            &workload.text,
            |b, &text| {
                b.iter(|| {
                    black_box(
                        regex
                            .find(black_box(text))
                            .map(|mat| (mat.start(), mat.end())),
                    )
                })
            },
        );
    }

    group.finish();
}

fn find_all_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_all_supported_subset");
    configure_group(&mut group);

    for workload in SEARCH_WORKLOADS {
        let rexile = Pattern::new(workload.pattern).unwrap();
        let regex = Regex::new(workload.pattern).unwrap();

        group.bench_with_input(
            BenchmarkId::new("rexile", workload.name),
            &workload.text,
            |b, &text| b.iter(|| black_box(rexile.find_all(black_box(text)))),
        );
        group.bench_with_input(
            BenchmarkId::new("regex", workload.name),
            &workload.text,
            |b, &text| {
                b.iter(|| {
                    black_box(
                        regex
                            .find_iter(black_box(text))
                            .map(|mat| (mat.start(), mat.end()))
                            .collect::<Vec<_>>(),
                    )
                })
            },
        );
    }

    group.finish();
}

fn replacement_and_split_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("replace_split_supported_subset");
    configure_group(&mut group);

    let replace_text = "a=1 b=22 c=333 d=4444";
    let replace_pattern = r"(\w+)=(\d+)";
    let replacement = "$1:[$2]";
    let rexile_replace = Pattern::new(replace_pattern).unwrap();
    let regex_replace = Regex::new(replace_pattern).unwrap();

    group.bench_function("rexile/replace_all_captures", |b| {
        b.iter(|| {
            black_box(rexile_replace.replace_all(black_box(replace_text), black_box(replacement)))
        })
    });
    group.bench_function("regex/replace_all_captures", |b| {
        b.iter(|| {
            black_box(
                regex_replace
                    .replace_all(black_box(replace_text), black_box(replacement))
                    .to_string(),
            )
        })
    });

    let split_text = "one  two\tthree   four five";
    let split_pattern = r"\s+";
    let rexile_split = Pattern::new(split_pattern).unwrap();
    let regex_split = Regex::new(split_pattern).unwrap();

    group.bench_function("rexile/split_whitespace", |b| {
        b.iter(|| {
            black_box(
                rexile_split
                    .split(black_box(split_text))
                    .collect::<Vec<_>>(),
            )
        })
    });
    group.bench_function("regex/split_whitespace", |b| {
        b.iter(|| black_box(regex_split.split(black_box(split_text)).collect::<Vec<_>>()))
    });

    group.finish();
}

fn cached_api_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("cached_api");
    configure_group(&mut group);

    let text = "this is a test string for pattern matching";
    let _ = rexile::is_match("test", text);

    group.bench_function("rexile/is_match_cached", |b| {
        b.iter(|| black_box(rexile::is_match("test", black_box(text)).unwrap()))
    });

    group.bench_function("rexile/find_cached", |b| {
        b.iter(|| black_box(rexile::find("test", black_box(text)).unwrap()))
    });

    group.finish();
}

criterion_group!(
    benches,
    compilation_benchmark,
    is_match_benchmark,
    find_benchmark,
    find_all_benchmark,
    replacement_and_split_benchmark,
    cached_api_benchmark,
);

criterion_main!(benches);
