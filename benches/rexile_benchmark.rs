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
        name: "literal_log",
        pattern: "ERROR",
        text: "2024-01-15 ERROR [main] Connection timeout after 30s retry=3 user=admin@example.com",
    },
    SearchWorkload {
        name: "literal_code",
        pattern: "calculate_total",
        text: "fn calculate_total(items: Vec<Item>) -> Result<f64, Error> { Ok(0.0) }",
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
        name: "multiline_line_anchor",
        pattern: r"(?m)^ERROR$",
        text: "INFO\nERROR\nWARN\nERROR\n",
    },
    SearchWorkload {
        name: "combined_case_multiline_anchor",
        pattern: r"(?im)^error$",
        text: "info\nError\nWARN\nERROR\n",
    },
    SearchWorkload {
        name: "dotall_capture",
        pattern: r"(?s)(BEGIN.*END)",
        text: "prefix BEGIN\npayload\nEND suffix",
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

fn captures_iter_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("captures_iter");
    configure_group(&mut group);

    for (name, pattern, text) in [
        ("flat", r"(\w+)=(\d+)", "a=1 b=22 c=333 d=4444"),
        ("nested", r"x((.).);", "xab;xcd;xef;xgh;"),
        (
            "dotall",
            r"(?s)(BEGIN.*END)",
            "prefix BEGIN\npayload\nEND suffix",
        ),
    ] {
        let rexile = Pattern::new(pattern).unwrap();
        let regex = Regex::new(pattern).unwrap();

        group.bench_with_input(BenchmarkId::new("rexile", name), &text, |b, &text| {
            b.iter(|| black_box(rexile.captures_iter(black_box(text)).count()))
        });
        group.bench_with_input(BenchmarkId::new("regex", name), &text, |b, &text| {
            b.iter(|| black_box(regex.captures_iter(black_box(text)).count()))
        });
    }

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

fn prefix_churn_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("prefilter_prefix_churn");
    configure_group(&mut group);
    let no_match_rexile = Pattern::new(r"abc\d+").unwrap();
    let no_match_regex = Regex::new(r"abc\d+").unwrap();

    for (name, repeats) in [("512b", 128usize), ("2k", 512), ("8k", 2_048)] {
        let text = "abcX".repeat(repeats);

        group.bench_with_input(
            BenchmarkId::new("is_match_no_match/rexile", name),
            &text,
            |b, text| b.iter(|| black_box(no_match_rexile.is_match(black_box(text)))),
        );
        group.bench_with_input(
            BenchmarkId::new("is_match_no_match/regex", name),
            &text,
            |b, text| b.iter(|| black_box(no_match_regex.is_match(black_box(text)))),
        );
        group.bench_with_input(
            BenchmarkId::new("find_no_match/rexile", name),
            &text,
            |b, text| b.iter(|| black_box(no_match_rexile.find(black_box(text)))),
        );
        group.bench_with_input(
            BenchmarkId::new("find_no_match/regex", name),
            &text,
            |b, text| {
                b.iter(|| {
                    black_box(
                        no_match_regex
                            .find(black_box(text))
                            .map(|matched| (matched.start(), matched.end())),
                    )
                })
            },
        );
    }

    let tail_text = format!("{}abc123", "abcX".repeat(2_048));
    let tail_rexile = Pattern::new(r"abc\d+").unwrap();
    let tail_regex = Regex::new(r"abc\d+").unwrap();
    group.bench_function("find_tail_match/rexile", |b| {
        b.iter(|| black_box(tail_rexile.find(black_box(&tail_text))))
    });
    group.bench_function("find_tail_match/regex", |b| {
        b.iter(|| {
            black_box(
                tail_regex
                    .find(black_box(&tail_text))
                    .map(|matched| (matched.start(), matched.end())),
            )
        })
    });

    let case_insensitive_text = "AbCX".repeat(2_048);
    let case_insensitive_rexile = Pattern::new(r"(?i)abc\d+").unwrap();
    let case_insensitive_regex = Regex::new(r"(?i)abc\d+").unwrap();
    group.bench_function("case_insensitive_no_match/rexile", |b| {
        b.iter(|| black_box(case_insensitive_rexile.is_match(black_box(&case_insensitive_text))))
    });
    group.bench_function("case_insensitive_no_match/regex", |b| {
        b.iter(|| black_box(case_insensitive_regex.is_match(black_box(&case_insensitive_text))))
    });

    group.finish();
}

fn flagged_context_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flags_context");
    configure_group(&mut group);

    let no_match_text = "INFO\n".repeat(2_048);
    let tail_match_text = format!("{}ERROR\n", no_match_text);
    let multiline_pattern = r"(?m)^ERROR$";
    let rexile_multiline = Pattern::new(multiline_pattern).unwrap();
    let regex_multiline = Regex::new(multiline_pattern).unwrap();

    group.bench_function("multiline_no_match/rexile", |b| {
        b.iter(|| black_box(rexile_multiline.is_match(black_box(&no_match_text))))
    });
    group.bench_function("multiline_no_match/regex", |b| {
        b.iter(|| black_box(regex_multiline.is_match(black_box(&no_match_text))))
    });
    group.bench_function("multiline_tail_find/rexile", |b| {
        b.iter(|| black_box(rexile_multiline.find(black_box(&tail_match_text))))
    });
    group.bench_function("multiline_tail_find/regex", |b| {
        b.iter(|| {
            black_box(
                regex_multiline
                    .find(black_box(&tail_match_text))
                    .map(|matched| (matched.start(), matched.end())),
            )
        })
    });

    let dotall_text = format!("BEGIN\n{}END", "payload\n".repeat(512));
    let dotall_pattern = r"(?s)(BEGIN.*END)";
    let rexile_dotall = Pattern::new(dotall_pattern).unwrap();
    let regex_dotall = Regex::new(dotall_pattern).unwrap();

    group.bench_function("dotall_capture/rexile", |b| {
        b.iter(|| black_box(rexile_dotall.captures(black_box(&dotall_text))))
    });
    group.bench_function("dotall_capture/regex", |b| {
        b.iter(|| black_box(regex_dotall.captures(black_box(&dotall_text))))
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
    captures_iter_benchmark,
    cached_api_benchmark,
    prefix_churn_benchmark,
    flagged_context_benchmark,
);

criterion_main!(benches);
