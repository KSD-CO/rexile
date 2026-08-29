use regex::Regex;
use rexile::Pattern;

fn assert_search_compatible(pattern: &str, text: &str) {
    let rexile = Pattern::new(pattern).unwrap();
    let regex = Regex::new(pattern).unwrap();

    assert_eq!(rexile.is_match(text), regex.is_match(text), "{pattern:?}");
    assert_eq!(
        rexile.find(text),
        regex
            .find(text)
            .map(|matched| (matched.start(), matched.end())),
        "{pattern:?} on {text:?}",
    );
    assert_eq!(
        rexile.find_all(text),
        regex
            .find_iter(text)
            .map(|matched| (matched.start(), matched.end()))
            .collect::<Vec<_>>(),
        "{pattern:?} on {text:?}",
    );
}

#[test]
fn prefix_churn_no_match_is_compatible() {
    let text = "abcX".repeat(128);
    assert_search_compatible(r"abc\d+", &text);
}

#[test]
fn prefix_churn_finds_the_first_verified_candidate() {
    let text = format!("{}abc123 {}abc45", "abcX".repeat(64), "abcX".repeat(8));
    assert_search_compatible(r"abc\d+", &text);
}

#[test]
fn prefix_analysis_handles_groups_and_optional_prefixes() {
    assert_search_compatible(r"(?:foo|bar)\d+", "skip fooX bar12 foo34");
    assert_search_compatible(r"(?:x)?foo\d+", "foo1 xfoo2");
}

#[test]
fn ascii_case_insensitive_prefix_uses_exact_candidate_matching() {
    assert_search_compatible(r"(?i)abc\d+", "abcX AbC12 abc34");
}

#[test]
fn unicode_case_insensitive_prefix_falls_back_without_false_negative() {
    assert_search_compatible(r"(?i)über\d+", "ÜBER12 über34");
}

#[test]
fn unicode_prefix_candidates_keep_byte_offsets() {
    assert_search_compatible(r"đi\d+", "→ đi12, điX, đi34");
}

#[test]
fn prefix_prefilter_matches_regex_for_supported_variants() {
    for (pattern, text) in [
        (r"\babc\d+", "xabc1 abc12"),
        (r"ab+c\d+", "ac1 abbc23"),
        (r"foo\+\d+", "foo+X foo+12"),
        (r"foo(?:bar)?\d+", "fooX foobar12 foo34"),
        (r"(?:foo|bar)\d+", "barX foo12 bar34"),
        (r"(?i)abc\d+", "AbCX abc12 ABC34"),
    ] {
        assert_search_compatible(pattern, text);
    }

    let end_anchored = Pattern::new(r"abc\d+$").unwrap();
    assert!(end_anchored.is_match("abc1 abc23"));
    assert_eq!(end_anchored.find("abc1 abc23"), Some((5, 10)));
}

#[test]
fn exact_candidate_matching_keeps_lookaround_and_backreference_context() {
    let lookahead = Pattern::new(r"foo(?=\d+)").unwrap();
    assert_eq!(lookahead.find("fooX foo12"), Some((5, 8)));

    let lookbehind = Pattern::new(r"(?<=\d)abc\d+").unwrap();
    assert_eq!(lookbehind.find("xabc1 9abc23"), Some((7, 12)));

    let backreference = Pattern::new(r"(foo)\1\d+").unwrap();
    assert_eq!(backreference.find("foofooX foofoo12"), Some((8, 16)));
}
