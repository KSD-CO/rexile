//! Regression tests for Issue #16:
//! “(.){1}” finds nothing (groups with bounded/range quantifiers fail to match).

use regex::Regex;
use rexile::Pattern;

#[test]
fn test_issue_16_dot_exact_quantifier() {
    // Exact case from Issue #16
    let pattern = Pattern::new(r"(.){1}").expect("Pattern should compile");

    assert!(pattern.is_match("a"), "pattern (.){{1}} should match 'a'");
    assert!(
        !pattern.is_match(""),
        "pattern (.){{1}} should not match empty string"
    );
    assert_eq!(pattern.find("a"), Some((0, 1)));

    let captures = pattern.captures("a").expect("Expected captures");
    assert_eq!(captures.get(1), Some("a"));
}

#[test]
fn test_group_bounded_quantifier_variations() {
    // Exactly n times: {n}
    let pat_ab2 = Pattern::new(r"(ab){2}").expect("Pattern should compile");
    assert!(pat_ab2.is_match("abab"));
    assert!(!pat_ab2.is_match("ab"));
    assert!(!pat_ab2.is_match("aba"));
    assert_eq!(pat_ab2.find("abab"), Some((0, 4)));
    let caps = pat_ab2.captures("abab").expect("Expected captures");
    assert_eq!(caps.get(1), Some("ab"));

    // Repeated single-character capture retains last repetition
    let pat_w3 = Pattern::new(r"(\w){3}").expect("Pattern should compile");
    assert!(pat_w3.is_match("xyz"));
    assert!(!pat_w3.is_match("xy"));
    assert_eq!(pat_w3.find("xyz"), Some((0, 3)));
    let caps = pat_w3.captures("xyz").expect("Expected captures");
    assert_eq!(caps.get(1), Some("z"));

    // Range quantifier: {n,m}
    let pat_d24 = Pattern::new(r"(\d){2,4}").expect("Pattern should compile");
    assert!(!pat_d24.is_match("1"));
    assert!(pat_d24.is_match("12"));
    assert!(pat_d24.is_match("123"));
    assert!(pat_d24.is_match("1234"));
    assert_eq!(pat_d24.find("12345"), Some((0, 4)));
    let caps = pat_d24.captures("123").expect("Expected captures");
    assert_eq!(caps.get(1), Some("3"));

    // At least n times: {n,}
    let pat_d2 = Pattern::new(r"(\d){2,}").expect("Pattern should compile");
    assert!(!pat_d2.is_match("1"));
    assert!(pat_d2.is_match("12"));
    assert!(pat_d2.is_match("12345"));
    assert_eq!(pat_d2.find("12345"), Some((0, 5)));

    // Embedded inside sequence
    let pat_seq = Pattern::new(r"prefix(foo){2}suffix").expect("Pattern should compile");
    assert!(pat_seq.is_match("prefixfoofoosuffix"));
    assert!(!pat_seq.is_match("prefixfoosuffix"));
    let caps = pat_seq
        .captures("prefixfoofoosuffix")
        .expect("Expected captures");
    assert_eq!(caps.get(1), Some("foo"));

    // Nested quantified groups
    let pat_nested = Pattern::new(r"((a){2}){3}").expect("Pattern should compile");
    assert!(pat_nested.is_match("aaaaaa"));
    assert!(!pat_nested.is_match("aaaaa"));
    assert_eq!(pat_nested.find("aaaaaa"), Some((0, 6)));

    // Non-capturing group with bounded quantifier combined with capturing group
    let pat_mixed = Pattern::new(r"(?:foo){2}(bar)").expect("Pattern should compile");
    assert!(pat_mixed.is_match("foofoobar"));
    assert!(!pat_mixed.is_match("foobar"));
    let caps_mixed = pat_mixed.captures("foofoobar").expect("Expected captures");
    assert_eq!(caps_mixed.get(1), Some("bar"));
}

#[test]
fn test_non_capturing_group_bounded_quantifier() {
    let pat_nc = Pattern::new(r"(?:ab){2}").expect("Pattern should compile");
    assert!(pat_nc.is_match("abab"));
    assert!(!pat_nc.is_match("ab"));
    assert_eq!(pat_nc.find("abab"), Some((0, 4)));

    let pat_nc_alt = Pattern::new(r"(?:a|b){3}").expect("Pattern should compile");
    assert!(pat_nc_alt.is_match("aba"));
    assert!(pat_nc_alt.is_match("bbb"));
    assert!(!pat_nc_alt.is_match("ab"));

    let pat_nc_seq = Pattern::new(r"x(?:y){2,3}z").expect("Pattern should compile");
    assert!(pat_nc_seq.is_match("xyyz"));
    assert!(pat_nc_seq.is_match("xyyyz"));
    assert!(!pat_nc_seq.is_match("xyz"));
    assert!(!pat_nc_seq.is_match("xyyyyz"));

    let pat_nested_bounded = Pattern::new(r"(?:a{2}){2}").expect("Pattern should compile");
    assert_eq!(pat_nested_bounded.find("aaaa"), Some((0, 4)));

    let pat_zero = Pattern::new(r"(?:a){0}").expect("Pattern should compile");
    assert!(pat_zero.is_match("b"));
    assert_eq!(pat_zero.find("b"), Some((0, 0)));
}

#[test]
fn test_invalid_group_quantifiers_fail_to_compile() {
    let invalid_patterns = [
        r"(a){2,1}",
        r"(?:a){2,1}",
        r"(a){",
        r"(?:a){",
        r"(a){invalid}",
        r"(a){1,2,3}",
    ];

    for pat in invalid_patterns {
        let res = Pattern::new(pat);
        assert!(
            res.is_err(),
            "Invalid pattern `{}` should fail to compile, but got {:?}",
            pat,
            res
        );
    }
}

#[test]
fn test_issue_16_differential_with_regex() {
    let cases = [
        (r"(.){1}", "a"),
        (r"(.){1}", ""),
        (r"(.){1}", "hello"),
        (r"(ab){2}", "abab"),
        (r"(ab){2}", "aba"),
        (r"(ab){2}", "ababab"),
        (r"([a-z]){3}", "hello"),
        (r"([a-z]){3}", "ab"),
        (r"(\d){2,4}", "1"),
        (r"(\d){2,4}", "12"),
        (r"(\d){2,4}", "12345"),
        (r"(?:foo){2}", "foofoo"),
        (r"(?:foo){2}", "foo"),
        (r"(foo|bar){2}", "foobar"),
        (r"(foo|bar){2}", "barfoo"),
        (r"start_(a|b){2}_end", "start_ab_end"),
        (r"start_(a|b){2}_end", "start_a_end"),
        (r"(é){2}", "éé"),
        (r"(é){2}", "é"),
        (r"(?:a{2}){2}", "aaaa"),
        (r"(?:a|ab){1}c", "abc"),
        (r"(?:a){0}", "b"),
    ];

    for (pat, text) in cases {
        let rexile_pat = Pattern::new(pat).unwrap_or_else(|e| {
            panic!("rexile failed to compile `{pat}`: {e}");
        });
        let regex_pat = Regex::new(pat).unwrap_or_else(|e| {
            panic!("regex failed to compile `{pat}`: {e}");
        });

        // is_match
        assert_eq!(
            rexile_pat.is_match(text),
            regex_pat.is_match(text),
            "is_match mismatch on pattern `{pat}` and text `{text}`"
        );

        // find
        assert_eq!(
            rexile_pat.find(text),
            regex_pat.find(text).map(|m| (m.start(), m.end())),
            "find mismatch on pattern `{pat}` and text `{text}`"
        );

        // captures
        let rexile_caps = rexile_pat.captures(text);
        let regex_caps = regex_pat.captures(text);
        assert_eq!(
            rexile_caps.is_some(),
            regex_caps.is_some(),
            "captures presence mismatch on pattern `{pat}` and text `{text}`"
        );

        if let (Some(r_caps), Some(std_caps)) = (rexile_caps, regex_caps) {
            assert_eq!(
                r_caps.len(),
                std_caps.len(),
                "captures len mismatch on pattern `{pat}` and text `{text}`"
            );
            for i in 0..std_caps.len() {
                assert_eq!(
                    r_caps.get(i),
                    std_caps.get(i).map(|m| m.as_str()),
                    "capture group {i} mismatch on pattern `{pat}` and text `{text}`"
                );
            }
        }
    }
}
