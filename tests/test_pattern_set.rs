use std::ops::ControlFlow;
use std::sync::Arc;

use regex::{Regex, RegexSet};

use rexile::{Pattern, PatternSet, PatternSetError, SetCache, SetSearchMode};

type Hit = (usize, usize, usize);
type CaptureHit = (usize, Vec<Option<(usize, usize)>>);

fn assert_equivalent(patterns: &[&str], text: &str) {
    let set = PatternSet::new(patterns).unwrap();
    // Rexile's existing word boundaries are ASCII, even in UTF-8 text.
    // Make that documented semantic difference explicit in the oracle.
    let oracle_patterns: Vec<_> = patterns
        .iter()
        .map(|pattern| {
            pattern
                .replace(r"\b", r"(?-u:\b)")
                .replace(r"\B", r"(?-u:\B)")
        })
        .collect();
    let baseline = RegexSet::new(&oracle_patterns).unwrap();
    let regexes: Vec<_> = oracle_patterns
        .iter()
        .map(|pattern| Regex::new(pattern).unwrap())
        .collect();
    assert_eq!(set.len(), patterns.len());
    assert_eq!(
        set.is_match(text),
        baseline.is_match(text),
        "{patterns:?} / {text:?}"
    );
    let ids: Vec<_> = baseline.matches(text).iter().collect();
    assert_eq!(set.matches(text).iter().collect::<Vec<_>>(), ids);
    let mut cache = set.create_cache();
    assert_eq!(
        set.matches_with_cache(text, &mut cache)
            .iter()
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(set.is_match_with_cache(text, &mut cache), !ids.is_empty());

    let mut expected: Vec<Hit> = regexes
        .iter()
        .enumerate()
        .flat_map(|(id, re)| {
            re.find_iter(text)
                .map(move |hit| (id, hit.start(), hit.end()))
        })
        .collect();
    expected.sort_unstable_by_key(|&(id, start, _)| (start, id));
    let actual: Vec<_> = set
        .find_iter(text)
        .map(|hit| {
            assert_eq!(hit.as_str(), &text[hit.range()]);
            (hit.pattern_id(), hit.start(), hit.end())
        })
        .collect();
    assert_eq!(actual, expected, "{patterns:?} / {text:?}");
    assert_eq!(
        set.find(text)
            .map(|hit| (hit.pattern_id(), hit.start(), hit.end())),
        expected.first().copied()
    );
    let expected_each: Vec<_> = regexes
        .iter()
        .enumerate()
        .filter_map(|(id, re)| re.find(text).map(|hit| (id, hit.start(), hit.end())))
        .collect();
    assert_eq!(
        set.find_each(text)
            .iter()
            .map(|hit| (hit.pattern_id(), hit.start(), hit.end()))
            .collect::<Vec<_>>(),
        expected_each
    );

    let mut expected_captures: Vec<CaptureHit> = regexes
        .iter()
        .enumerate()
        .flat_map(|(id, re)| {
            re.captures_iter(text).map(move |hit| {
                (
                    id,
                    hit.iter()
                        .map(|m| m.map(|m| (m.start(), m.end())))
                        .collect(),
                )
            })
        })
        .collect();
    expected_captures.sort_by_key(|(id, slots)| (slots[0].unwrap().0, *id));
    let actual_captures: Vec<_> = set
        .captures_iter(text)
        .map(|hit| {
            (
                hit.pattern_id(),
                (0..hit.captures().len())
                    .map(|i| hit.pos(i))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        actual_captures, expected_captures,
        "captures: {patterns:?} / {text:?}"
    );
    let expected_capture_each: Vec<_> = regexes
        .iter()
        .enumerate()
        .filter_map(|(id, re)| {
            re.captures(text).map(|hit| {
                (
                    id,
                    hit.iter()
                        .map(|m| m.map(|m| (m.start(), m.end())))
                        .collect::<Vec<_>>(),
                )
            })
        })
        .collect();
    assert_eq!(
        set.captures_each(text)
            .iter()
            .map(|hit| {
                (
                    hit.pattern_id(),
                    (0..hit.captures().len())
                        .map(|i| hit.pos(i))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
        expected_capture_each
    );
    assert_eq!(
        set.captures(text).map(|hit| (hit.pattern_id(), hit.pos(0))),
        expected_captures.first().map(|(id, slots)| (*id, slots[0]))
    );

    for mode in [SetSearchMode::All, SetSearchMode::FirstPerPattern] {
        let mut visited = Vec::new();
        let result = set.visit_matches(text, &mut cache, mode, |hit| {
            visited.push((hit.pattern_id(), hit.start(), hit.end()));
            ControlFlow::<()>::Continue(())
        });
        assert_eq!(result, ControlFlow::Continue(()));
        let mut desired = if mode == SetSearchMode::All {
            expected.clone()
        } else {
            expected_each.clone()
        };
        desired.sort_unstable_by_key(|&(id, start, _)| (start, id));
        assert_eq!(visited, desired);
        let mut visited_captures = Vec::new();
        let _ = set.visit_captures(text, &mut cache, mode, |hit| {
            for i in 0..hit.len() {
                assert_eq!(hit.get(i), hit.pos(i).map(|(s, e)| &text[s..e]));
            }
            visited_captures.push((
                hit.pattern_id(),
                (0..hit.len()).map(|i| hit.pos(i)).collect::<Vec<_>>(),
            ));
            ControlFlow::<()>::Continue(())
        });
        let mut desired = if mode == SetSearchMode::All {
            expected_captures.clone()
        } else {
            expected_capture_each.clone()
        };
        desired.sort_by_key(|(id, slots)| (slots[0].unwrap().0, *id));
        assert_eq!(visited_captures, desired);
    }
}

#[test]
fn empty_and_utf8_progress() {
    for text in ["", "a", "é", "abé", "→ a ", "\n"] {
        assert_equivalent(&["", "a*", "(a*)", "$", "^", "a?", r"\b", r"\B"], text);
    }
}

#[test]
fn overlaps_duplicates_and_prefix_order() {
    for text in [
        "",
        "abcd aaa",
        "xx abcX abc12 a123 abc45",
        "foo123 foobar12 bar3",
    ] {
        assert_equivalent(
            &[
                "a",
                "abcd",
                "b",
                "abc",
                "a",
                r"abc(\d+)",
                r"a(\d+)",
                r"(?:foo|foobar)(\d+)",
            ],
            text,
        );
    }
    let long = format!("a{}", "b".repeat(100));
    assert_equivalent(&[&long, "b", "ab", "a"], &format!("{long} {long}"));
}

#[test]
fn internal_literal_candidates_preserve_earliest_start_and_context() {
    for text in [
        "alice:123;tag0000",
        "→ café:123;tag0000 alice:4;tag0000",
        "aaaaab aab ab",
        "fooaaa:tag foox:tag",
        "a".repeat(200).as_str(),
    ] {
        assert_equivalent(
            &[
                r"([a-z]+):([0-9]+);tag0000",
                r"a{2,3}b",
                r"foo(a+):tag",
                r"(a+)(b)",
                r"ab",
                "tag0000",
            ],
            text,
        );
    }
}

#[test]
fn candidate_scanner_switches_without_losing_overlaps_or_duplicates() {
    let text = format!(
        "{}{}{}",
        "ababab ".repeat(12),
        " ".repeat(600),
        "ababab ".repeat(12)
    );
    assert_equivalent(&["aba", "bab", "aba"], &text);
    let text = format!(
        "{}{}{}",
        "éαx éαy ".repeat(12),
        " ".repeat(600),
        "éαx éαy ".repeat(12)
    );
    assert_equivalent(&["éαx", "éαy", "éαx"], &text);
    let long = "abcdefghijklmnopq";
    let other = "abcdefghijklmnopr";
    let text = format!(
        "{}{}{}",
        format!("{long} {other} ").repeat(12),
        " ".repeat(600),
        format!("{long} {other} ").repeat(12)
    );
    assert_equivalent(&[long, other, long], &text);
    assert_equivalent(&["a\0", "ab", "a\0"], "a\0ab a\0");
    assert_equivalent(
        &["aba", "bab", "axa", "bxb", "aya", "byb", "aba"],
        &"aba bab axa bxb aya byb aza bzb ".repeat(20),
    );
    assert_equivalent(
        &[r"key=([0-9]+)", r"key=([a-z]+)", "key="],
        &"key=123 key=abc key=!!! ".repeat(20),
    );
    let text = format!("{}{}", "aba bab ".repeat(20), " ".repeat(1500));
    assert_equivalent(&["aba", "bab", "aba"], &text);
}

#[test]
fn captures_and_branch_rollback() {
    assert_equivalent(
        &[
            r"user=(\w+)",
            r"code=(\d+)",
            r"(a)|(b)",
            r"x((.).);",
            r"((a)|(b))+",
            r"(a)?b",
            r"foo(a+?)b",
            r"(a|ab)c",
            r"(a+)(a)",
            r"(a*?)(b)",
        ],
        "code=42 user=alice code=7 xab; aabc abc b aaaab fooaaab",
    );
}

#[test]
fn flags_boundaries_and_unicode_context() {
    assert_equivalent(
        &[
            r"(?m)^ERROR$",
            r"(?im)^error$",
            r"(?s)(BEGIN.*END)",
            r"\bfoo(\d+)",
            r"(?i)(hello)",
            r"đi(\d+)",
            r"(?i)(é)",
            r"(?i)(k)",
        ],
        "é HELLO xfoo1 foo12\nERROR\nerror\nBEGIN\npayload\nEND\n→ đi12 É K",
    );
}

#[test]
fn unicode_and_negated_classes_keep_lazy_capture_lengths() {
    for text in ["éαabc", "aéαx", "xx→éα", ""] {
        assert_equivalent(
            &[r"([^x]+?)", r"([éα]+?)", r"([^x]{1,3})", r"([a-z]+?)"],
            text,
        );
    }
}

#[test]
fn seeded_differential_patterns() {
    let pool = [
        "a",
        "b",
        "",
        "ab",
        "abcdef",
        "[0-9]+",
        "(a)",
        "(a+)",
        "(a*)",
        "a+?",
        "a??",
        "a{2,3}",
        "ab?c",
        "(a|b)",
        "(a)?b",
        "foo[0-9]+",
        "foo([0-9]+)",
        "foo([a-z]+):([0-9]+)",
        "^a",
        "b$",
        r"\ba",
        "(?i)abc",
        "((a)b)",
        "(?:a|ab)c",
    ];
    let alphabet = ["a", "b", "c", " ", "1", "2", "foo", ":", "é", "\n", "ABC"];
    let mut seed = 0x726578696c65u64;
    for _ in 0..160 {
        let mut step = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 32) as usize
        };
        let patterns: Vec<_> = (0..6).map(|_| pool[step() % pool.len()]).collect();
        let text: String = (0..12).map(|_| alphabet[step() % alphabet.len()]).collect();
        assert_equivalent(&patterns, &text);
    }
}

#[test]
fn extensions_preserve_full_context() {
    let set = PatternSet::new([r"foo(?=\d)", r"(?<=9)abc(\d+)", r"(foo)\1(\d+)"]).unwrap();
    let text = "fooX foo1 9abc23 foofoo45";
    assert_eq!(
        set.find_each(text)
            .iter()
            .map(|m| (m.pattern_id(), m.as_str()))
            .collect::<Vec<_>>(),
        [(0, "foo"), (1, "abc23"), (2, "foofoo45")]
    );
    let captures = set.captures_each(text);
    assert_eq!(captures[2].get(1), Some("foo"));
    assert_eq!(captures[2].get(2), Some("45"));
}

#[test]
fn cache_reuse_errors_cancellation_and_threads() {
    let set = Arc::new(PatternSet::new(["foo", r"foo(\d+)", ""]).unwrap());
    let clone = set.clone();
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let set = clone.clone();
            std::thread::spawn(move || {
                let mut cache = set.create_cache();
                for _ in 0..20 {
                    assert_eq!(
                        set.matches_with_cache("foo1", &mut cache)
                            .iter()
                            .collect::<Vec<_>>(),
                        [0, 1, 2]
                    );
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    let mut cache = SetCache::default();
    assert_eq!(
        set.visit_captures("foo1", &mut cache, SetSearchMode::All, |m| {
            ControlFlow::Break(m.pattern_id())
        }),
        ControlFlow::Break(0)
    );
    let empty = PatternSet::new(std::iter::empty::<&str>()).unwrap();
    assert!(empty.is_empty());
    assert!(empty.matches_with_cache("foo1", &mut cache).is_empty());
    assert!(!empty.is_match_with_cache("", &mut cache));
    assert_eq!(set.matches_with_cache("foo1", &mut cache).len(), 3);
    assert!(!set.matches("foo1").matched(usize::MAX));
    assert!(matches!(
        PatternSet::new(["foo", "(?x)a"]),
        Err(PatternSetError::Pattern { pattern_id: 1, .. })
    ));
    let mut iterator = set.find_iter("");
    assert_eq!(iterator.next().unwrap().pattern_id(), 2);
    assert!(iterator.next().is_none());
    assert!(iterator.next().is_none());
}

#[test]
fn ordinary_identifier_boolean_queries() {
    let pattern = Pattern::new(r"[a-zA-Z_]\w*").unwrap();
    let regex = Regex::new(r"(?-u:[a-zA-Z_]\w*)").unwrap();
    for text in ["", "123", "éΩ", "123_", "42 A9", "é_name9Ω", "\0z", "a"] {
        assert_eq!(pattern.is_match(text), regex.is_match(text), "{text:?}");
        assert_eq!(
            pattern.find(text),
            regex.find(text).map(|hit| (hit.start(), hit.end())),
            "{text:?}",
        );
    }
}

#[test]
fn ordinary_pattern_iterator_regressions() {
    let lazy = Pattern::new("a.*?b\nc").unwrap();
    assert_eq!(lazy.find("a x b\nc later"), Some((0, 7)));
    assert_eq!(lazy.find("a\nx b\nc"), None);
    for pattern in ["", "a*", "(a*)", "(?i)(é)", "(?i)(k)", "(?i)(İ)"] {
        for text in ["", "é", "aé", "→ É K İ"] {
            let pattern = Pattern::new(pattern).unwrap();
            let matches: Vec<_> = pattern
                .find_iter(text)
                .map(|m| (m.start(), m.end()))
                .collect();
            let captures: Vec<_> = pattern
                .captures_iter(text)
                .map(|c| c.pos(0).unwrap())
                .collect();
            assert_eq!(matches, captures, "{pattern:?} / {text:?}");
        }
    }
    for source in [r"\b", r"\B", r"\b[a-z]+", r"([a-z]+)\b"] {
        let pattern = Pattern::new(source).unwrap();
        let regex = Regex::new(
            &source
                .replace(r"\b", r"(?-u:\b)")
                .replace(r"\B", r"(?-u:\B)"),
        )
        .unwrap();
        for text in ["ab cd", "abé cd", ""] {
            assert_eq!(
                pattern
                    .find_iter(text)
                    .map(|m| m.range())
                    .collect::<Vec<_>>(),
                regex.find_iter(text).map(|m| m.range()).collect::<Vec<_>>(),
                "{source} / {text}",
            );
        }
    }
    for source in ["(?i)k", "(?i)é", "(?i)(İ)", "(?i)K|z"] {
        let pattern = Pattern::new(source).unwrap();
        for text in ["K k", "k K", "É é", "é É", "İ i", "i İ"] {
            assert_eq!(
                pattern.find(text),
                pattern.captures(text).and_then(|c| c.pos(0))
            );
            assert_eq!(pattern.is_match(text), pattern.captures(text).is_some());
        }
    }
    let lookbehind = Pattern::new(r"(?<=a)b").unwrap();
    assert_eq!(
        lookbehind
            .find_iter("ab ab")
            .map(|m| m.range())
            .collect::<Vec<_>>(),
        [1..2, 4..5]
    );
}
