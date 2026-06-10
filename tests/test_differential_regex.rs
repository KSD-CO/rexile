use regex::Regex;
use rexile::Pattern;

#[derive(Debug)]
struct Case {
    pattern: &'static str,
    haystack: &'static str,
}

type SearchResult = (bool, Option<(usize, usize)>, Vec<(usize, usize)>);

fn assert_search_compatible(cases: &[Case]) {
    for case in cases {
        let rexile = rexile_search(case.pattern, case.haystack);
        let regex = regex_search(case.pattern, case.haystack);

        assert_eq!(
            rexile, regex,
            "pattern {:?} on haystack {:?}",
            case.pattern, case.haystack
        );
    }
}

fn rexile_search(pattern: &str, haystack: &str) -> SearchResult {
    let re = Pattern::new(pattern).unwrap_or_else(|err| {
        panic!("rexile failed to compile pattern {pattern:?}: {err}");
    });

    (
        re.is_match(haystack),
        re.find(haystack),
        re.find_all(haystack),
    )
}

fn regex_search(pattern: &str, haystack: &str) -> SearchResult {
    let re = Regex::new(pattern).unwrap_or_else(|err| {
        panic!("regex failed to compile pattern {pattern:?}: {err}");
    });

    (
        re.is_match(haystack),
        re.find(haystack).map(|mat| (mat.start(), mat.end())),
        re.find_iter(haystack)
            .map(|mat| (mat.start(), mat.end()))
            .collect(),
    )
}

#[test]
fn literal_search_matches_regex() {
    assert_search_compatible(&[
        Case {
            pattern: "hello",
            haystack: "well hello there",
        },
        Case {
            pattern: "hello",
            haystack: "no greeting here",
        },
        Case {
            pattern: "ERROR",
            haystack: "INFO ERROR WARN ERROR",
        },
        Case {
            pattern: r"\.rs",
            haystack: "src/lib.rs README.md main.rs",
        },
    ]);
}

#[test]
fn alternation_search_matches_regex() {
    assert_search_compatible(&[
        Case {
            pattern: "foo|bar|baz",
            haystack: "qux bar baz foo",
        },
        Case {
            pattern: "foo|bar|baz",
            haystack: "qux only",
        },
        Case {
            pattern: "aa|a",
            haystack: "aaa",
        },
        Case {
            pattern: "a|aa",
            haystack: "aaa",
        },
        Case {
            pattern: "foo|foobar",
            haystack: "foobar foo",
        },
    ]);
}

#[test]
fn character_classes_match_regex_for_ascii_inputs() {
    assert_search_compatible(&[
        Case {
            pattern: "[a-z]+",
            haystack: "ABC abc xyz 123",
        },
        Case {
            pattern: "[A-Za-z0-9_]+",
            haystack: "--abc_DEF123--",
        },
        Case {
            pattern: "[0-9]+",
            haystack: "abc 123 def 456",
        },
        Case {
            pattern: "[^0-9]+",
            haystack: "abc123def",
        },
        Case {
            pattern: r"[\[\]]+",
            haystack: "a [x] b",
        },
    ]);
}

#[test]
fn escape_classes_match_regex_for_ascii_inputs() {
    assert_search_compatible(&[
        Case {
            pattern: r"\d+",
            haystack: "Order #123 costs 45",
        },
        Case {
            pattern: r"\D+",
            haystack: "123abc456",
        },
        Case {
            pattern: r"\w+",
            haystack: "hello_world 123",
        },
        Case {
            pattern: r"\W+",
            haystack: "abc -- 123",
        },
        Case {
            pattern: r"\s+",
            haystack: "a \t b  c",
        },
        Case {
            pattern: r"\S+",
            haystack: "  abc 123 ",
        },
    ]);
}

#[test]
fn quantifiers_match_regex() {
    assert_search_compatible(&[
        Case {
            pattern: "a*",
            haystack: "baaac",
        },
        Case {
            pattern: "a+",
            haystack: "baaac",
        },
        Case {
            pattern: "a?",
            haystack: "baaac",
        },
        Case {
            pattern: "a{2}",
            haystack: "baaaac",
        },
        Case {
            pattern: "a{2,}",
            haystack: "baaaac",
        },
        Case {
            pattern: "a{2,4}",
            haystack: "baaaaac",
        },
        Case {
            pattern: r"\d*",
            haystack: "a12b",
        },
        Case {
            pattern: r"\d{2,3}",
            haystack: "a1234b",
        },
    ]);
}

#[test]
fn anchors_match_regex() {
    assert_search_compatible(&[
        Case {
            pattern: "^foo",
            haystack: "foobar",
        },
        Case {
            pattern: "^foo",
            haystack: "barfoo",
        },
        Case {
            pattern: "bar$",
            haystack: "foobar",
        },
        Case {
            pattern: "bar$",
            haystack: "barfoo",
        },
        Case {
            pattern: "^exact$",
            haystack: "exact",
        },
        Case {
            pattern: "^exact$",
            haystack: "not exact",
        },
    ]);
}

#[test]
fn dot_and_dotall_match_regex() {
    assert_search_compatible(&[
        Case {
            pattern: "a.c",
            haystack: "abc axc a\nc",
        },
        Case {
            pattern: "a.*c",
            haystack: "abc axxc",
        },
        Case {
            pattern: "a.+c",
            haystack: "ac abc",
        },
        Case {
            pattern: r"(?s)a.*c",
            haystack: "a\n\nc",
        },
    ]);
}

#[test]
fn lazy_quantifiers_match_regex() {
    assert_search_compatible(&[
        Case {
            pattern: "a.*?c",
            haystack: "abc axxc",
        },
        Case {
            pattern: "a.+?c",
            haystack: "abc axxc",
        },
        Case {
            pattern: "a??",
            haystack: "baaac",
        },
    ]);
}

#[test]
fn boundaries_match_regex_for_ascii_inputs() {
    assert_search_compatible(&[
        Case {
            pattern: r"\btest\b",
            haystack: "a test testing test!",
        },
        Case {
            pattern: r"\Btest\B",
            haystack: "atestb test",
        },
        Case {
            pattern: r"\b\w+\b",
            haystack: "one two_three 123",
        },
    ]);
}

#[test]
fn sequences_match_regex_for_ascii_inputs() {
    assert_search_compatible(&[
        Case {
            pattern: r"\w+\s+\d+",
            haystack: "item 123 next",
        },
        Case {
            pattern: r"\d+\.\d+",
            haystack: "version 12.34 done 56.78",
        },
        Case {
            pattern: r"[A-Z][a-z]+\d?",
            haystack: "Alice2 bob Carol",
        },
    ]);
}

#[test]
fn case_insensitive_literals_match_regex_for_ascii_inputs() {
    assert_search_compatible(&[
        Case {
            pattern: r"(?i)error",
            haystack: "INFO Error WARN error",
        },
        Case {
            pattern: r"(?i)get|post",
            haystack: "GET post Put",
        },
    ]);
}

#[test]
fn captures_match_regex_for_simple_ascii_patterns() {
    let cases = [
        Case {
            pattern: r"(\w+)=(\d+)",
            haystack: "a=1 b=22",
        },
        Case {
            pattern: r"(\w+)@(\w+)\.(\w+)",
            haystack: "email test@example.com done",
        },
    ];

    for case in &cases {
        let rexile = Pattern::new(case.pattern).unwrap();
        let regex = Regex::new(case.pattern).unwrap();

        let rexile_captures = rexile.captures(case.haystack).map(|captures| {
            (0..captures.len())
                .map(|index| captures.get(index).map(str::to_string))
                .collect::<Vec<_>>()
        });
        let regex_captures = regex.captures(case.haystack).map(|captures| {
            captures
                .iter()
                .map(|capture| capture.map(|mat| mat.as_str().to_string()))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            rexile_captures, regex_captures,
            "pattern {:?} on haystack {:?}",
            case.pattern, case.haystack
        );
    }
}

#[test]
fn replacement_matches_regex_for_simple_ascii_patterns() {
    let cases = [
        (r"\d+", "Order #123 costs 45", "X"),
        (r"(\w+)=(\d+)", "a=1 b=22", "$1:[$2]"),
    ];

    for (pattern, haystack, replacement) in cases {
        let rexile = Pattern::new(pattern).unwrap();
        let regex = Regex::new(pattern).unwrap();

        assert_eq!(
            rexile.replace(haystack, replacement),
            regex.replace(haystack, replacement).to_string(),
            "replace with pattern {pattern:?} on haystack {haystack:?}"
        );
        assert_eq!(
            rexile.replace_all(haystack, replacement),
            regex.replace_all(haystack, replacement).to_string(),
            "replace_all with pattern {pattern:?} on haystack {haystack:?}"
        );
    }
}

#[test]
fn split_matches_regex_for_simple_ascii_patterns() {
    let cases = [(r"\s+", "a  b\tc"), (",", "a,b,,c,")];

    for (pattern, haystack) in cases {
        let rexile = Pattern::new(pattern).unwrap();
        let regex = Regex::new(pattern).unwrap();

        let rexile_parts: Vec<_> = rexile.split(haystack).collect();
        let regex_parts: Vec<_> = regex.split(haystack).collect();

        assert_eq!(
            rexile_parts, regex_parts,
            "split with pattern {pattern:?} on haystack {haystack:?}"
        );
    }
}

#[test]
fn invalid_patterns_fail_to_compile() {
    for pattern in ["[", "(", "a{", "a{2,1}"] {
        assert!(
            Pattern::new(pattern).is_err(),
            "rexile unexpectedly compiled invalid pattern {pattern:?}"
        );
        assert!(
            Regex::new(pattern).is_err(),
            "regex unexpectedly compiled invalid pattern {pattern:?}"
        );
    }
}
