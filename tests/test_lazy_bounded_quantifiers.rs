//! Integration tests for lazy bounded quantifiers: {n}?, {n,}?, {n,m}?

use regex::Regex;
use rexile::Pattern;

#[test]
fn test_lazy_bounded_literal() {
    // Range lazy: {n,m}?
    let pat_lazy = Pattern::new(r"a{2,4}?").expect("Pattern should compile");
    assert_eq!(pat_lazy.find("aaaa"), Some((0, 2)));

    let pat_greedy = Pattern::new(r"a{2,4}").expect("Pattern should compile");
    assert_eq!(pat_greedy.find("aaaa"), Some((0, 4)));

    // AtLeast lazy: {n,}?
    let pat_atleast_lazy = Pattern::new(r"a{2,}?").expect("Pattern should compile");
    assert_eq!(pat_atleast_lazy.find("aaaa"), Some((0, 2)));

    let pat_atleast_greedy = Pattern::new(r"a{2,}").expect("Pattern should compile");
    assert_eq!(pat_atleast_greedy.find("aaaa"), Some((0, 4)));

    // Exactly lazy: {n}?
    let pat_exact_lazy = Pattern::new(r"a{3}?").expect("Pattern should compile");
    assert_eq!(pat_exact_lazy.find("aaaa"), Some((0, 3)));
}

#[test]
fn test_lazy_bounded_charclass() {
    // Character class: [0-9]{2,4}?
    let pat_digit_lazy = Pattern::new(r"\d{2,4}?").expect("Pattern should compile");
    assert_eq!(pat_digit_lazy.find("12345"), Some((0, 2)));

    let pat_digit_greedy = Pattern::new(r"\d{2,4}").expect("Pattern should compile");
    assert_eq!(pat_digit_greedy.find("12345"), Some((0, 4)));

    let pat_alpha_lazy = Pattern::new(r"[a-z]{2,5}?").expect("Pattern should compile");
    assert_eq!(pat_alpha_lazy.find("abcdef"), Some((0, 2)));
}

#[test]
fn test_lazy_bounded_backtracking() {
    // Suffix requires extending the lazy match beyond min
    let pat = Pattern::new(r"a{2,5}?b").expect("Pattern should compile");
    assert_eq!(pat.find("aaaaab"), Some((0, 6)));
    assert_eq!(pat.find("aab"), Some((0, 3)));
    assert_eq!(pat.find("aaab"), Some((0, 4)));
    assert_eq!(pat.find("ab"), None); // min 2 'a's required

    let pat_digits = Pattern::new(r"\d{2,4}?px").expect("Pattern should compile");
    assert_eq!(pat_digits.find("1234px"), Some((0, 6)));
    assert_eq!(pat_digits.find("12px"), Some((0, 4)));
}

#[test]
fn test_lazy_bounded_groups() {
    // Capturing group: (ab){2,4}?
    let pat_group_lazy = Pattern::new(r"(ab){2,4}?").expect("Pattern should compile");
    assert_eq!(pat_group_lazy.find("abababab"), Some((0, 4)));
    let caps = pat_group_lazy
        .captures("abababab")
        .expect("Expected captures");
    assert_eq!(caps.get(1), Some("ab"));

    let pat_group_greedy = Pattern::new(r"(ab){2,4}").expect("Pattern should compile");
    assert_eq!(pat_group_greedy.find("abababab"), Some((0, 8)));

    // Non-capturing group: (?:foo){2,4}?
    let pat_nc_lazy = Pattern::new(r"(?:foo){2,4}?").expect("Pattern should compile");
    assert_eq!(pat_nc_lazy.find("foofoofoofoo"), Some((0, 6)));

    // Backtracking with group
    let pat_group_bt = Pattern::new(r"(?:foo){2,4}?bar").expect("Pattern should compile");
    assert_eq!(pat_group_bt.find("foofoofoobar"), Some((0, 12)));
}

#[test]
fn test_lazy_bounded_differential_with_regex() {
    let cases = [
        (r"a{2,4}?", "aaaa"),
        (r"a{2,4}?", "aa"),
        (r"a{2,4}?", "a"),
        (r"a{2,}?", "aaaaa"),
        (r"a{3}?", "aaaaa"),
        (r"(\d){2,4}?", "12345"),
        (r"[a-z]{2,5}?", "abcdef"),
        (r"a{2,5}?b", "aaaaab"),
        (r"a{2,5}?b", "aaab"),
        (r"prefix(foo){2,4}?suffix", "prefixfoofoofoosuffix"),
        (r"(?:a|b){2,4}?", "ababab"),
        (r"\w{2,}?end", "helloend"),
        (r"\d{1,3}?ms", "123ms"),
        (r"\d{1,3}?ms", "1ms"),
    ];

    for (pat, text) in cases {
        let rexile_pat = Pattern::new(pat).unwrap_or_else(|e| {
            panic!("rexile failed to compile `{pat}`: {e}");
        });
        let regex_pat = Regex::new(pat).unwrap_or_else(|e| {
            panic!("regex failed to compile `{pat}`: {e}");
        });

        assert_eq!(
            rexile_pat.is_match(text),
            regex_pat.is_match(text),
            "is_match mismatch on `{pat}` and `{text}`"
        );

        assert_eq!(
            rexile_pat.find(text),
            regex_pat.find(text).map(|m| (m.start(), m.end())),
            "find mismatch on `{pat}` and `{text}`"
        );

        let rexile_caps = rexile_pat.captures(text);
        let regex_caps = regex_pat.captures(text);
        assert_eq!(
            rexile_caps.is_some(),
            regex_caps.is_some(),
            "captures presence mismatch on `{pat}` and `{text}`"
        );

        if let (Some(r_caps), Some(std_caps)) = (rexile_caps, regex_caps) {
            assert_eq!(
                r_caps.len(),
                std_caps.len(),
                "captures len mismatch on `{pat}` and `{text}`"
            );
            for i in 0..std_caps.len() {
                assert_eq!(
                    r_caps.get(i),
                    std_caps.get(i).map(|m| m.as_str()),
                    "capture group {i} mismatch on `{pat}` and `{text}`"
                );
            }
        }
    }
}
