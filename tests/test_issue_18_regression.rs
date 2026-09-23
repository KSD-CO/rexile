use rexile::{Pattern, PatternSet};

fn assert_search(pattern: &str, text: &str, expected: Option<((usize, usize), &str)>) {
    let pattern = Pattern::new(pattern).expect("pattern should compile");
    let expected_range = expected.map(|(range, _)| range);

    assert_eq!(pattern.is_match(text), expected.is_some());
    assert_eq!(pattern.find(text), expected_range);
    assert_eq!(
        pattern
            .find_iter(text)
            .next()
            .map(|matched| matched.range()),
        expected_range.map(|(start, end)| start..end)
    );
    assert_eq!(
        pattern.captures(text).as_ref().and_then(|caps| caps.pos(0)),
        expected_range
    );
    assert_eq!(
        pattern.captures(text).as_ref().and_then(|caps| caps.get(1)),
        expected.map(|(_, group)| group)
    );
}

#[test]
fn quantified_backreferences_match_the_captured_text() {
    for (source, text, expected) in [
        (r"(.)\1+", "aaa", Some(((0, 3), "a"))),
        (r"(a)\1+", "aa+", Some(((0, 2), "a"))),
        (r"(a)\1\+", "aa+", Some(((0, 3), "a"))),
        (r"(ab)\1+", "zababab!", Some(((1, 7), "ab"))),
        (r"(a|ab)\1+", "ababab", Some(((0, 6), "ab"))),
        (r"^(ab)\1+$", "ababab", Some(((0, 6), "ab"))),
        (r"(a)\1+(?=b)", "aaab", Some(((0, 3), "a"))),
        (r"(ab)\1{2}", "ababab", Some(((0, 6), "ab"))),
        (r"(ab)\1{1,2}?", "ababab", Some(((0, 4), "ab"))),
        (r"(.)\1+?", "aaaa", Some(((0, 2), "a"))),
        (r"(.)\1*", "ab", Some(((0, 1), "a"))),
        (r"(.)\1?", "aa", Some(((0, 2), "a"))),
        (r"(.)\1{0}", "abc", Some(((0, 1), "a"))),
        (r"(.)\1+", "abc", None),
        (r"(é)\1+", "ééé", Some(((0, 6), "é"))),
        (r"(?i)(a)\1+", "AaA", Some(((0, 3), "A"))),
        (r"(?i)(é)\1+", "ÉéÉ", Some(((0, 6), "É"))),
    ] {
        assert_search(source, text, expected);
    }
}

#[test]
fn quantified_backreferences_backtrack_for_suffixes() {
    assert_search(r"(a)\1+?b", "aaab", Some(((0, 4), "a")));
    assert_search(r"(a)\1{1,3}a", "aaaa", Some(((0, 4), "a")));
    assert_search(r"(ab)(\1+)", "ababab", Some(((0, 6), "ab")));
}

#[test]
fn backreference_search_apis_agree_for_grouped_and_ungrouped_forms() {
    for source in [r"(.)\1+", r"(.)(\1)+"] {
        let pattern = Pattern::new(source).unwrap();
        let text = "aaa bbb";
        let expected = vec![(0, 3), (4, 7)];

        assert_eq!(pattern.find_all(text), expected);
        assert_eq!(
            pattern
                .find_iter(text)
                .map(|m| (m.start(), m.end()))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            pattern
                .captures_iter(text)
                .filter_map(|caps| caps.pos(0))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(pattern.replace_all(text, "x"), "x x");
        assert_eq!(pattern.replace_all(text, "<$1>"), "<a> <b>");
        assert_eq!(pattern.split(text).collect::<Vec<_>>(), ["", " ", ""]);
    }
}

#[test]
fn pattern_set_matches_quantified_backreferences() {
    let set = PatternSet::new([r"(ab)\1+"]).unwrap();
    assert!(set.is_match("zababab!"));
    assert_eq!(set.find_each("zababab!")[0].as_str(), "ababab");
    assert_eq!(set.captures_each("zababab!")[0].get(1), Some("ab"));
    assert_eq!(
        set.captures_iter("zababab!").next().unwrap().get(1),
        Some("ab")
    );
}

#[test]
fn empty_backreference_repetition_makes_progress() {
    let pattern = Pattern::new(r"()\1*").unwrap();
    let ranges: Vec<_> = pattern
        .find_iter("é")
        .map(|matched| (matched.start(), matched.end()))
        .collect();
    assert_eq!(ranges, [(0, 0), (2, 2)]);
    assert_eq!(pattern.find_all("é"), ranges);
}

#[test]
fn alternation_keeps_branches_without_backreferences() {
    let pattern = Pattern::new(r"(a)\1+|b").unwrap();
    assert_eq!(pattern.find("b"), Some((0, 1)));
    assert_eq!(pattern.captures("b").unwrap().get(1), None);
}
