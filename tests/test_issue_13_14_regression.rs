//! Regression tests for Issue #13 (invalid `(?` pattern accepted)
//! and Issue #14 (valid `(é)` pattern rejected due to UTF-8 char boundary panic).

use rexile::Pattern;

#[test]
fn test_issue_13_invalid_qmark_paren_rejected() {
    // Issue #13: The pattern "(?" is invalid and must be rejected.
    let pattern = Pattern::new(r"(?");
    assert!(
        pattern.is_err(),
        "Pattern `(?` should be rejected as Err, got {:?}",
        pattern
    );
}

#[test]
fn test_unclosed_qmark_paren_variations_rejected() {
    let invalid_patterns = [
        r"(?",
        r"(?a",
        r"a(?",
        r"(a)(?",
        r"(?:foo)(?",
        r"(?*",
        r"(?+",
        r"(??",
        r"(?#",
        r"(?>",
        r"(?<",
        r"(?P",
        r"(?=",
        r"(?!",
        r"(?<=",
        r"(?<!",
        r"hello (?",
        r"a(?b",
        r"(a)(?b",
    ];

    for pat in invalid_patterns {
        let res = Pattern::new(pat);
        assert!(
            res.is_err(),
            "Pattern `{}` should be rejected as Err, got {:?}",
            pat,
            res
        );
    }
}

#[test]
fn test_malformed_and_unsupported_group_syntax_rejected() {
    let invalid_patterns = [
        r"(?)",              // empty group with question mark
        r"(?a)",             // unknown group / unsupported flag
        r"(??)",             // invalid group
        r"(?*)",             // invalid group
        r"(?+)",             // invalid group
        r"(?#comment)",      // comment group (unsupported)
        r"(?>atomic)",       // atomic group (unsupported)
        r"(?P<name>abc)",    // named capture group (unsupported)
        r"(?<name>abc)",     // named capture group (unsupported)
        r"(?|branch|reset)", // branch reset group (unsupported)
        r"abc(?def)ghi",     // invalid group in sequence
        r"abc(?=def)ghi",    // lookahead with suffix (unsupported combined pattern)
    ];

    for pat in invalid_patterns {
        let res = Pattern::new(pat);
        assert!(
            res.is_err(),
            "Pattern `{}` should be rejected as Err, got {:?}",
            pat,
            res
        );
    }
}

#[test]
fn test_issue_14_unicode_capture_accepted() {
    // Issue #14: The pattern `(é)` should be accepted and match `é`.
    let pattern = Pattern::new(r"(é)").expect("Pattern `(é)` should compile");
    assert!(pattern.is_match("é"));
    assert!(!pattern.is_match("a"));

    let caps = pattern.captures("é").expect("Should capture");
    assert_eq!(caps.get(1), Some("é"));
}

#[test]
fn test_unicode_capture_variations() {
    // Multi-byte 2-byte UTF-8
    let p1 = Pattern::new(r"a(é)b").expect("a(é)b");
    assert!(p1.is_match("aéb"));
    assert_eq!(p1.captures("aéb").unwrap().get(1), Some("é"));

    let p2 = Pattern::new(r"é(a)é").expect("é(a)é");
    assert!(p2.is_match("éaé"));
    assert_eq!(p2.captures("éaé").unwrap().get(1), Some("a"));

    let p3 = Pattern::new(r"(?:é)").expect("(?:é)");
    assert!(p3.is_match("é"));

    let p4 = Pattern::new(r"([é])").expect("([é])");
    assert!(p4.is_match("é"));

    let p5 = Pattern::new(r"(é|ø)").expect("(é|ø)");
    assert!(p5.is_match("ø"));
    assert!(p5.is_match("é"));

    let p6 = Pattern::new(r"(é*)").expect("(é*)");
    assert!(p6.is_match("éé"));

    let p7 = Pattern::new(r"(é+)?").expect("(é+)?");
    assert!(p7.is_match("é"));

    // Multi-byte 3-byte UTF-8 (CJK: 日本語)
    let p8 = Pattern::new(r"(日本語)").expect("(日本語)");
    assert!(p8.is_match("日本語"));
    assert_eq!(p8.captures("日本語").unwrap().get(1), Some("日本語"));

    // Multi-byte 4-byte UTF-8 (Emoji: 🦀)
    let p9 = Pattern::new(r"(🦀)").expect("(🦀)");
    assert!(p9.is_match("🦀"));
    assert_eq!(p9.captures("🦀").unwrap().get(1), Some("🦀"));

    let p10 = Pattern::new(r"🦀(🦀)🦀").expect("🦀(🦀)🦀");
    assert!(p10.is_match("🦀🦀🦀"));
    assert_eq!(p10.captures("🦀🦀🦀").unwrap().get(1), Some("🦀"));

    let p11 = Pattern::new(r"(🦀foo|🦀bar)").expect("(🦀foo|🦀bar)");
    assert!(p11.is_match("🦀bar"));
    assert!(p11.is_match("🦀foo"));
    assert!(!p11.is_match("🦀baz"));

    let p12 = Pattern::new(r"🦀|🦁").expect("🦀|🦁");
    assert!(p12.is_match("🦁"));
    assert!(p12.is_match("🦀"));
}
