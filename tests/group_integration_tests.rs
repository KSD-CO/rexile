/// Integration tests for groups combined with other ReXile features
use rexile::ReXile;

#[test]
fn test_group_with_escape_sequences() {
    // (\d+|\w+) - match either digits or word characters
    let re = ReXile::new("(\\d+)").unwrap();
    assert!(re.is_match("123"));
    assert!(re.is_match("abc123"));
    assert_eq!(re.find("abc123"), Some((3, 6)));

    let re_word = ReXile::new("(\\w+)").unwrap();
    assert!(re_word.is_match("hello"));
    assert!(re_word.is_match("test123"));
}

#[test]
fn test_group_with_charclass() {
    // ([0-9]+) - match digits in group
    let re = ReXile::new("([0-9]+)").unwrap();
    assert!(re.is_match("123"));
    assert!(re.is_match("abc123"));
    assert_eq!(re.find("abc123"), Some((3, 6)));

    // ([a-z]+) - match lowercase letters
    let re_lower = ReXile::new("([a-z]+)").unwrap();
    assert!(re_lower.is_match("hello"));
    assert!(!re_lower.is_match("HELLO"));
}

#[test]
fn test_quantified_group_alternation() {
    // (foo|bar)+ - one or more foo or bar
    let re = ReXile::new("(foo|bar)+").unwrap();
    assert!(re.is_match("foo"));
    assert!(re.is_match("bar"));
    assert!(re.is_match("foobar"));
    assert!(re.is_match("barfoo"));
    assert!(re.is_match("foofoobar"));
    assert!(!re.is_match("baz"));

    // Find multiple occurrences
    assert_eq!(re.find("test foo baz"), Some((5, 8)));
    assert_eq!(re.find("test bar baz"), Some((5, 8)));
}

#[test]
fn test_group_with_anchors() {
    // ^(hello) - group at start
    let re_start = ReXile::new("^(hello)").unwrap();
    assert!(re_start.is_match("hello world"));
    assert!(!re_start.is_match("say hello"));

    // (world)$ - group at end
    let re_end = ReXile::new("(world)$").unwrap();
    assert!(re_end.is_match("hello world"));
    assert!(!re_end.is_match("world hello"));
}

#[test]
fn test_multiple_groups() {
    // (foo)(bar) - two consecutive groups
    let re = ReXile::new("(foo)(bar)").unwrap();
    assert!(re.is_match("foobar"));
    assert!(!re.is_match("foo"));
    assert!(!re.is_match("bar"));
    assert_eq!(re.find("test foobar end"), Some((5, 11)));
}

#[test]
fn test_group_with_optional() {
    // (test)? - optional group
    let re = ReXile::new("(test)?").unwrap();
    assert!(re.is_match("test"));
    assert!(re.is_match("testing"));
    // Note: Zero-width matches not fully supported yet
}

#[test]
fn test_group_find_all() {
    // Find all protocol occurrences
    let re = ReXile::new("(http|https|ftp)").unwrap();
    let text = "Visit http://example.com or https://secure.com or ftp://files.com";
    let matches = re.find_all(text);

    assert_eq!(matches.len(), 3);
    assert_eq!(&text[matches[0].0..matches[0].1], "http");
    assert_eq!(&text[matches[1].0..matches[1].1], "https"); // Leftmost-longest match
    assert_eq!(&text[matches[2].0..matches[2].1], "ftp");
}

#[test]
fn test_non_capturing_group() {
    // (?:hello) - non-capturing
    let re = ReXile::new("(?:hello)").unwrap();
    assert!(re.is_match("hello world"));
    assert_eq!(re.find("say hello there"), Some((4, 9)));
}

#[test]
fn test_group_alternation_priority() {
    // (foo|fo|f) - should match longest first
    let re = ReXile::new("(foo|fo|f)").unwrap();
    assert!(re.is_match("foo"));
    assert_eq!(re.find("foo"), Some((0, 3)));
}

#[test]
fn test_quantified_group_edge_cases() {
    // (ab)+ - at least one ab
    let re_plus = ReXile::new("(ab)+").unwrap();
    assert!(re_plus.is_match("ab"));
    assert!(re_plus.is_match("abab"));
    assert!(re_plus.is_match("ababab"));
    assert!(!re_plus.is_match("a"));
    assert!(!re_plus.is_match("b"));

    // (ab)* - zero or more ab
    let re_star = ReXile::new("(ab)*").unwrap();
    assert!(re_star.is_match("ab"));
    assert!(re_star.is_match("abab"));
    // Note: Zero-width matches not fully handled

    // (ab)? - zero or one ab
    let re_optional = ReXile::new("(ab)?").unwrap();
    assert!(re_optional.is_match("ab"));
    assert!(re_optional.is_match("abc"));
}

#[test]
fn test_complex_real_world_patterns() {
    // URL protocol
    let re_url = ReXile::new("(http|https)://").unwrap();
    assert!(re_url.is_match("http://example.com"));
    assert!(re_url.is_match("https://secure.com"));
    assert!(!re_url.is_match("ftp://files.com"));

    // Email-like pattern (simplified)
    let re_user = ReXile::new("(\\w+)@").unwrap();
    assert!(re_user.is_match("user@example.com"));
    assert_eq!(re_user.find("contact: user@example.com"), Some((9, 14)));

    // Version pattern like v1.2.3
    let re_version = ReXile::new("v(\\d+)").unwrap();
    assert!(re_version.is_match("v1.0.0"));
    assert!(re_version.is_match("v2.1.5"));
}

#[test]
fn test_group_with_literal_prefix() {
    // prefix(foo|bar) - literal then group
    let re = ReXile::new("prefix(foo|bar)").unwrap();
    assert!(re.is_match("prefixfoo"));
    assert!(re.is_match("prefixbar"));
    assert!(!re.is_match("foo"));
    assert!(!re.is_match("bar"));
}

#[test]
fn test_group_with_literal_suffix() {
    // (foo|bar)suffix - group then literal
    let re = ReXile::new("(foo|bar)suffix").unwrap();
    assert!(re.is_match("foosuffix"));
    assert!(re.is_match("barsuffix"));
    assert!(!re.is_match("foo"));
    assert!(!re.is_match("bar"));
}
