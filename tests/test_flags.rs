use regex::Regex;
use rexile::{Pattern, PatternError};

fn regex_matches(pattern: &str, text: &str) -> Vec<(usize, usize)> {
    Regex::new(pattern)
        .unwrap()
        .find_iter(text)
        .map(|matched| (matched.start(), matched.end()))
        .collect()
}

fn rexile_matches(pattern: &str, text: &str) -> Vec<(usize, usize)> {
    Pattern::new(pattern).unwrap().find_all(text)
}

#[test]
fn multiline_anchors_match_regex() {
    let pattern = r"(?m)^foo$";
    let text = "foo\nbar\nfoo\n";
    let rexile = Pattern::new(pattern).unwrap();
    let regex = Regex::new(pattern).unwrap();

    assert_eq!(rexile.is_match(text), regex.is_match(text));
    assert_eq!(
        rexile.find(text),
        regex
            .find(text)
            .map(|matched| (matched.start(), matched.end()))
    );
    assert_eq!(rexile_matches(pattern, text), regex_matches(pattern, text));
    assert_eq!(
        rexile
            .find_iter(text)
            .map(|matched| (matched.start(), matched.end()))
            .collect::<Vec<_>>(),
        regex_matches(pattern, text)
    );
}

#[test]
fn multiline_anchors_work_in_groups_and_alternation() {
    let text = "foo\nbar\nbaz\nfoo";

    for pattern in [
        r"(?m)(?:^foo$|^bar$)",
        r"(?m)^foo$|^bar$",
        r"(?m)(^foo$|^bar$)",
        r"(?m)^(foo|bar)$",
    ] {
        assert_eq!(rexile_matches(pattern, text), regex_matches(pattern, text));
    }
}

#[test]
fn multiline_empty_anchor_keeps_the_final_line_start() {
    let pattern = r"(?m)^";
    let text = "line\n";

    assert_eq!(rexile_matches(pattern, text), regex_matches(pattern, text));
    assert_eq!(
        Pattern::new(pattern)
            .unwrap()
            .find_iter(text)
            .map(|matched| (matched.start(), matched.end()))
            .collect::<Vec<_>>(),
        regex_matches(pattern, text)
    );
}

#[test]
fn dotall_applies_inside_captures() {
    let pattern = r"(?s)(a.)";
    let text = "a\n";
    let rexile = Pattern::new(pattern).unwrap();
    let regex = Regex::new(pattern).unwrap();

    let rexile_captures = rexile.captures(text).map(|captures| {
        (0..captures.len())
            .map(|index| captures.get(index).map(str::to_string))
            .collect::<Vec<_>>()
    });
    let regex_captures = regex.captures(text).map(|captures| {
        captures
            .iter()
            .map(|capture| capture.map(|matched| matched.as_str().to_string()))
            .collect::<Vec<_>>()
    });

    assert_eq!(rexile_captures, regex_captures);
}

#[test]
fn dotall_applies_inside_quantified_non_capturing_groups() {
    let pattern = r"(?s)(?:a.)+";
    let text = "a\na\n";

    assert_eq!(rexile_matches(pattern, text), regex_matches(pattern, text));
}

#[test]
fn combined_flags_and_multiline_captures_match_regex() {
    let pattern = r"(?im)^(error)$";
    let text = "Error\ninfo\nERROR\n";
    let rexile = Pattern::new(pattern).unwrap();
    let regex = Regex::new(pattern).unwrap();

    let rexile_captures = rexile
        .captures_iter(text)
        .map(|captures| {
            (0..captures.len())
                .map(|index| captures.get(index).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let regex_captures = regex
        .captures_iter(text)
        .map(|captures| {
            captures
                .iter()
                .map(|capture| capture.map(|matched| matched.as_str().to_string()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(rexile_captures, regex_captures);
}

#[test]
fn consecutive_leading_flag_groups_are_global() {
    let pattern = r"(?i)(?m)^error$";
    let text = "info\nError\n";

    assert_eq!(rexile_matches(pattern, text), regex_matches(pattern, text));
}

#[test]
fn multiline_replace_and_split_match_regex() {
    let pattern = r"(?m)^foo$";
    let text = "foo\nbar\nfoo";
    let rexile = Pattern::new(pattern).unwrap();
    let regex = Regex::new(pattern).unwrap();

    assert_eq!(
        rexile.replace_all(text, "X"),
        regex.replace_all(text, "X").to_string()
    );
    assert_eq!(
        rexile.split(text).collect::<Vec<_>>(),
        regex.split(text).collect::<Vec<_>>()
    );
}

#[test]
fn anchors_inside_lookarounds_keep_absolute_context() {
    let pattern = r"(?m)(?=^foo$)";
    let text = "bar\nfoo\nbaz";

    assert_eq!(rexile_matches(pattern, text), vec![(4, 4)]);
}

#[test]
fn lf_multiline_mode_does_not_enable_crlf_mode() {
    let pattern = r"(?m)^foo$";
    let text = "foo\r\n";

    assert_eq!(
        Pattern::new(pattern).unwrap().is_match(text),
        Regex::new(pattern).unwrap().is_match(text)
    );
}

#[test]
fn escaped_and_character_class_anchors_remain_literals() {
    for (pattern, text) in [(r"(?m)\^foo\$", "^foo$"), (r"(?m)[^$]+", "abc")] {
        assert_eq!(rexile_matches(pattern, text), regex_matches(pattern, text));
    }
}

#[test]
fn unsupported_flag_syntax_returns_an_error() {
    for pattern in [
        r"(?x)foo",
        r"(?U)foo",
        r"(?u)foo",
        r"(?R)foo",
        r"(?-i)foo",
        r"(?i:foo)",
        r"foo(?i)bar",
        r"(?mR)^foo$",
    ] {
        assert!(
            matches!(
                Pattern::new(pattern),
                Err(PatternError::UnsupportedFeature(_))
            ),
            "expected unsupported flag error for {pattern:?}"
        );
    }
}
