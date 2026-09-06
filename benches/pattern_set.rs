use std::ops::ControlFlow;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use regex::{Regex, RegexSet};

use rexile::{PatternSet, SetSearchMode};

#[path = "support/regex_capture_visitor.rs"]
mod regex_capture_visitor;
#[path = "support/pattern_set_workloads.rs"]
mod workloads;

fn first_regex<'h>(
    regexes: &[Regex],
    matched: Option<&regex::SetMatches>,
    text: &'h str,
) -> Option<(usize, regex::Match<'h>)> {
    let mut first: Option<(usize, regex::Match<'h>)> = None;
    for (id, regex) in regexes.iter().enumerate() {
        if matched.is_some_and(|matched| !matched.matched(id)) {
            continue;
        }
        if let Some(hit) = regex.find(text) {
            if first
                .as_ref()
                .map_or(true, |(_, best)| hit.start() < best.start())
            {
                first = Some((id, hit));
            }
            // IDs are ascending, so no remaining rule can improve this hit.
            if hit.start() == 0 {
                break;
            }
        }
    }
    first
}

fn first_regex_captures<'h>(
    regexes: &[Regex],
    matched: Option<&regex::SetMatches>,
    text: &'h str,
) -> Option<(usize, regex::Captures<'h>)> {
    let mut first: Option<(usize, regex::Captures<'h>)> = None;
    for (id, regex) in regexes.iter().enumerate() {
        if matched.is_some_and(|matched| !matched.matched(id)) {
            continue;
        }
        if let Some(hit) = regex.captures(text) {
            let start = hit.get(0).unwrap().start();
            if first
                .as_ref()
                .map_or(true, |(_, best)| start < best.get(0).unwrap().start())
            {
                first = Some((id, hit));
            }
            if start == 0 {
                break;
            }
        }
    }
    first
}

fn benchmark(c: &mut Criterion) {
    let mut compile = c.benchmark_group("pattern_set_compile");
    compile
        .sample_size(100)
        .warm_up_time(Duration::from_millis(50))
        .measurement_time(Duration::from_millis(100))
        .nresamples(10_000);
    for &family in workloads::FAMILIES {
        for &count in workloads::COUNTS {
            let patterns = workloads::patterns(family, count);
            let id = format!("{family}/{count}");
            compile.bench_with_input(BenchmarkId::new("rexile", &id), &patterns, |b, p| {
                b.iter(|| black_box(PatternSet::new(black_box(p)).unwrap()))
            });
            compile.bench_with_input(BenchmarkId::new("regex_set", &id), &patterns, |b, p| {
                b.iter(|| black_box(RegexSet::new(black_box(p)).unwrap()))
            });
            compile.bench_with_input(BenchmarkId::new("regex_vec", &id), &patterns, |b, p| {
                b.iter(|| black_box(p.iter().map(|p| Regex::new(p).unwrap()).collect::<Vec<_>>()))
            });
            compile.bench_with_input(BenchmarkId::new("regex_set_vec", &id), &patterns, |b, p| {
                b.iter(|| {
                    black_box((
                        RegexSet::new(p).unwrap(),
                        p.iter().map(|p| Regex::new(p).unwrap()).collect::<Vec<_>>(),
                    ))
                })
            });
        }
    }
    compile.finish();
    for &family in workloads::FAMILIES {
        for &count in workloads::COUNTS {
            let patterns = workloads::patterns(family, count);
            let rexile = PatternSet::new(&patterns).unwrap();
            let regex_set = RegexSet::new(&patterns).unwrap();
            let regexes: Vec<_> = patterns.iter().map(|p| Regex::new(p).unwrap()).collect();
            for &length in workloads::LENGTHS {
                for &density in workloads::DENSITIES {
                    let text = workloads::haystack(family, count, length, density);
                    let id = format!("{family}/{count}/{length}/{density}");
                    assert_eq!(
                        rexile.matches(&text).iter().collect::<Vec<_>>(),
                        regex_set.matches(&text).iter().collect::<Vec<_>>()
                    );
                    let expected_first = rexile
                        .find(&text)
                        .map(|hit| (hit.pattern_id(), hit.start(), hit.end()));
                    let expected_captures = rexile.captures(&text).map(|hit| {
                        (
                            hit.pattern_id(),
                            (0..hit.captures().len())
                                .map(|group| hit.pos(group))
                                .collect::<Vec<_>>(),
                        )
                    });
                    let matched = regex_set.matches(&text);
                    for ids in [None, Some(&matched)] {
                        assert_eq!(
                            first_regex(&regexes, ids, &text).map(|(id, hit)| (
                                id,
                                hit.start(),
                                hit.end()
                            )),
                            expected_first
                        );
                        assert_eq!(
                            first_regex_captures(&regexes, ids, &text).map(|(id, hit)| {
                                (
                                    id,
                                    hit.iter()
                                        .map(|group| group.map(|hit| (hit.start(), hit.end())))
                                        .collect::<Vec<_>>(),
                                )
                            }),
                            expected_captures
                        );
                    }
                    let mut group = c.benchmark_group(format!("pattern_set_search/{id}"));
                    group
                        .sample_size(100)
                        .warm_up_time(Duration::from_millis(50))
                        .measurement_time(Duration::from_millis(100))
                        .nresamples(10_000);
                    group.bench_function("is_match/rexile", |b| {
                        b.iter(|| black_box(rexile.is_match(black_box(&text))))
                    });
                    group.bench_function("is_match/regex_set", |b| {
                        b.iter(|| black_box(regex_set.is_match(black_box(&text))))
                    });
                    let mut cache = rexile.create_cache();
                    group.bench_function("is_match_cached/rexile", |b| {
                        b.iter(|| {
                            black_box(rexile.is_match_with_cache(black_box(&text), &mut cache))
                        })
                    });
                    group.bench_function("matches/rexile", |b| {
                        b.iter(|| black_box(rexile.matches(black_box(&text))))
                    });
                    group.bench_function("matches/regex_set", |b| {
                        b.iter(|| black_box(regex_set.matches(black_box(&text))))
                    });
                    group.bench_function("matches_cached/rexile", |b| {
                        b.iter(|| {
                            black_box(
                                rexile
                                    .matches_with_cache(black_box(&text), &mut cache)
                                    .len(),
                            )
                        })
                    });
                    group.bench_function("find_each/rexile", |b| {
                        b.iter(|| black_box(rexile.find_each(black_box(&text))))
                    });
                    group.bench_function("find_each/regex_vec", |b| {
                        b.iter(|| {
                            black_box(
                                regexes
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(id, re)| {
                                        re.find(black_box(&text)).map(|m| (id, m.start(), m.end()))
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                    });
                    group.bench_function("find_each/regex_set_vec", |b| {
                        b.iter(|| {
                            black_box(
                                regex_set
                                    .matches(black_box(&text))
                                    .iter()
                                    .filter_map(|id| {
                                        regexes[id].find(&text).map(|m| (id, m.start(), m.end()))
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                    });
                    group.bench_function("captures_each/rexile", |b| {
                        b.iter(|| black_box(rexile.captures_each(black_box(&text))))
                    });
                    group.bench_function("captures_each/regex_vec", |b| {
                        b.iter(|| {
                            black_box(
                                regexes
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(id, re)| {
                                        re.captures(black_box(&text)).map(|m| (id, m))
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                    });
                    group.bench_function("captures_each/regex_set_vec", |b| {
                        b.iter(|| {
                            black_box(
                                regex_set
                                    .matches(black_box(&text))
                                    .iter()
                                    .filter_map(|id| regexes[id].captures(&text).map(|m| (id, m)))
                                    .collect::<Vec<_>>(),
                            )
                        })
                    });
                    group.bench_function("find_iter/rexile", |b| {
                        b.iter(|| {
                            black_box(
                                rexile
                                    .find_iter(black_box(&text))
                                    .map(|m| (m.pattern_id(), m.start(), m.end()))
                                    .collect::<Vec<_>>(),
                            )
                        })
                    });
                    for use_set in [false, true] {
                        let baseline = if use_set {
                            "regex_set_vec"
                        } else {
                            "regex_vec"
                        };
                        group.bench_function(format!("find_iter/{baseline}"), |b| {
                            b.iter(|| {
                                let matched = use_set.then(|| regex_set.matches(&text));
                                let ids = (0..count).filter(|&id| {
                                    matched.as_ref().map_or(true, |matches| matches.matched(id))
                                });
                                let mut hits: Vec<_> = ids
                                    .into_iter()
                                    .flat_map(|id| {
                                        regexes[id]
                                            .find_iter(black_box(&text))
                                            .map(move |m| (id, m.start(), m.end()))
                                    })
                                    .collect();
                                hits.sort_unstable_by_key(|&(id, start, _)| (start, id));
                                black_box(hits)
                            })
                        });
                    }
                    group.bench_function("captures_iter/rexile", |b| {
                        b.iter(|| {
                            black_box(rexile.captures_iter(black_box(&text)).collect::<Vec<_>>())
                        })
                    });
                    for use_set in [false, true] {
                        let baseline = if use_set {
                            "regex_set_vec"
                        } else {
                            "regex_vec"
                        };
                        group.bench_function(format!("captures_iter/{baseline}"), |b| {
                            b.iter(|| {
                                let matched = use_set.then(|| regex_set.matches(&text));
                                let ids = (0..count).filter(|&id| {
                                    matched.as_ref().map_or(true, |matches| matches.matched(id))
                                });
                                let mut hits: Vec<_> = ids
                                    .into_iter()
                                    .flat_map(|id| {
                                        regexes[id]
                                            .captures_iter(black_box(&text))
                                            .map(move |m| (id, m))
                                    })
                                    .collect();
                                hits.sort_by_key(|(id, caps)| (caps.get(0).unwrap().start(), *id));
                                black_box(hits)
                            })
                        });
                    }
                    group.bench_function("find/rexile", |b| {
                        b.iter(|| black_box(rexile.find(black_box(&text))))
                    });
                    group.bench_function("captures/rexile", |b| {
                        b.iter(|| black_box(rexile.captures(black_box(&text))))
                    });
                    for use_set in [false, true] {
                        let baseline = if use_set {
                            "regex_set_vec"
                        } else {
                            "regex_vec"
                        };
                        group.bench_function(format!("find/{baseline}"), |b| {
                            b.iter(|| {
                                let matched = use_set.then(|| regex_set.matches(black_box(&text)));
                                black_box(
                                    first_regex(&regexes, matched.as_ref(), black_box(&text))
                                        .map(|(id, hit)| (id, hit.start(), hit.end())),
                                )
                            })
                        });
                        group.bench_function(format!("captures/{baseline}"), |b| {
                            b.iter(|| {
                                let matched = use_set.then(|| regex_set.matches(black_box(&text)));
                                let first =
                                    first_regex(&regexes, matched.as_ref(), black_box(&text));
                                black_box(first.and_then(|(id, hit)| {
                                    regexes[id]
                                        .captures_at(&text, hit.start())
                                        .map(|caps| (id, caps))
                                }))
                            })
                        });
                        group.bench_function(format!("captures/{baseline}_direct"), |b| {
                            b.iter(|| {
                                let matched = use_set.then(|| regex_set.matches(black_box(&text)));
                                black_box(first_regex_captures(
                                    &regexes,
                                    matched.as_ref(),
                                    black_box(&text),
                                ))
                            })
                        });
                    }
                    let mut visitor = regex_capture_visitor::Visitor::new(&regexes);
                    let reference = visitor.visit(&regexes, None, &text);
                    let mut actual = 0;
                    let _ = rexile.visit_captures(&text, &mut cache, SetSearchMode::All, |hit| {
                        actual += hit.pattern_id();
                        for group in 0..hit.len() {
                            if let Some((start, end)) = hit.pos(group) {
                                actual += start + end;
                            }
                        }
                        ControlFlow::<()>::Continue(())
                    });
                    assert_eq!(actual, reference, "visitor checksum: {id}");
                    group.bench_function("visit_captures/rexile", |b| {
                        b.iter(|| {
                            let mut checksum = 0;
                            let _ = rexile.visit_captures(
                                black_box(&text),
                                &mut cache,
                                SetSearchMode::All,
                                |hit| {
                                    checksum += hit.pattern_id();
                                    for group in 0..hit.len() {
                                        if let Some((start, end)) = hit.pos(group) {
                                            checksum += start + end;
                                        }
                                    }
                                    ControlFlow::<()>::Continue(())
                                },
                            );
                            black_box(checksum)
                        })
                    });
                    group.bench_function("visit_captures/regex_vec", |b| {
                        b.iter(|| black_box(visitor.visit(&regexes, None, black_box(&text))))
                    });
                    group.bench_function("visit_captures/regex_set_vec", |b| {
                        b.iter(|| {
                            black_box(visitor.visit(&regexes, Some(&regex_set), black_box(&text)))
                        })
                    });
                    group.finish();
                }
            }
        }
    }
}

fn controls(c: &mut Criterion) {
    let cases = [
        (
            "small_set",
            vec![r"foo([0-9]+)", "bar", r"[0-9]+"],
            "foo12 bar 3".to_owned(),
        ),
        (
            "unicode",
            vec![r"đi([0-9]+)", r"(?i)(é)", r"([éα]+?)"],
            "→ đi12 É αé đi3".to_owned(),
        ),
        (
            "prefix_churn",
            vec![r"abc([0-9]+)", r"abcd([a-z]+)", r"(?:abc|abcd)([0-9]+)"],
            format!("{}abc12", "abcX".repeat(1024)),
        ),
        (
            "nested_captures",
            vec![r"x((.).);", r"((a)|(b))+", r"foo(a+?)b"],
            "xab;bbbafooaaab".to_owned(),
        ),
        (
            "rexile_extensions",
            vec![r"foo(?=[0-9])", r"(?<=9)abc([0-9]+)", r"(foo)\1([0-9]+)"],
            "fooX foo1 9abc23 foofoo45".to_owned(),
        ),
    ];
    for (name, patterns, text) in cases {
        let rexile = PatternSet::new(&patterns).unwrap();
        let regexes = patterns
            .iter()
            .map(|pattern| Regex::new(pattern))
            .collect::<Result<Vec<_>, _>>();
        let mut group = c.benchmark_group(format!("pattern_set_control/{name}"));
        group
            .sample_size(100)
            .warm_up_time(Duration::from_millis(50))
            .measurement_time(Duration::from_millis(100))
            .nresamples(10_000);
        group.bench_function("is_match/rexile", |b| {
            b.iter(|| black_box(rexile.is_match(black_box(&text))))
        });
        group.bench_function("captures_iter/rexile", |b| {
            b.iter(|| black_box(rexile.captures_iter(black_box(&text)).collect::<Vec<_>>()))
        });
        if let Ok(regexes) = regexes {
            let regex_set = RegexSet::new(&patterns).unwrap();
            assert_eq!(rexile.is_match(&text), regex_set.is_match(&text), "{name}");
            let mut expected: Vec<_> = regexes
                .iter()
                .enumerate()
                .flat_map(|(id, re)| {
                    re.captures_iter(&text).map(move |caps| {
                        (
                            id,
                            caps.iter()
                                .map(|cap| cap.map(|cap| (cap.start(), cap.end())))
                                .collect::<Vec<_>>(),
                        )
                    })
                })
                .collect();
            expected.sort_by_key(|(id, slots)| (slots[0].unwrap().0, *id));
            assert_eq!(
                rexile
                    .captures_iter(&text)
                    .map(|hit| (
                        hit.pattern_id(),
                        (0..hit.captures().len())
                            .map(|i| hit.pos(i))
                            .collect::<Vec<_>>()
                    ))
                    .collect::<Vec<_>>(),
                expected,
                "{name}"
            );
            group.bench_function("is_match/regex_set", |b| {
                b.iter(|| black_box(regex_set.is_match(black_box(&text))))
            });
            for use_set in [false, true] {
                let label = if use_set {
                    "regex_set_vec"
                } else {
                    "regex_vec"
                };
                group.bench_function(format!("captures_iter/{label}"), |b| {
                    b.iter(|| {
                        let matched = use_set.then(|| regex_set.matches(&text));
                        let mut hits: Vec<_> = regexes
                            .iter()
                            .enumerate()
                            .filter(|(id, _)| {
                                matched
                                    .as_ref()
                                    .map_or(true, |matches| matches.matched(*id))
                            })
                            .flat_map(|(id, re)| {
                                re.captures_iter(black_box(&text))
                                    .map(move |caps| (id, caps))
                            })
                            .collect();
                        hits.sort_by_key(|(id, caps)| (caps.get(0).unwrap().start(), *id));
                        black_box(hits)
                    })
                });
            }
        } else {
            assert_eq!(name, "rexile_extensions");
        }
        group.finish();
    }
}

criterion_group!(benches, benchmark, controls);
criterion_main!(benches);
