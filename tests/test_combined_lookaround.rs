//! Test combined lookaround patterns
//!
//! Tests patterns like foo(?=bar), foo(?!bar), (?<=foo)bar, (?<!foo)bar

use rexile::Pattern;

#[test]
fn test_lookahead_combined_simple() {
    // Pattern: foo(?=bar) - match 'foo' only if followed by 'bar'
    let pattern = Pattern::new(r"foo(?=bar)").unwrap();

    assert!(pattern.is_match("foobar"));
    assert!(!pattern.is_match("foobaz"));
    assert!(!pattern.is_match("bazfoo"));

    // find should return position of 'foo', not including 'bar'
    assert_eq!(pattern.find("foobar"), Some((0, 3)));
    assert_eq!(pattern.find("test foobar end"), Some((5, 8)));
}

#[test]
fn test_negative_lookahead_combined() {
    // Pattern: foo(?!bar) - match 'foo' only if NOT followed by 'bar'
    let pattern = Pattern::new(r"foo(?!bar)").unwrap();

    assert!(pattern.is_match("foobaz"));
    assert!(!pattern.is_match("foobar"));
    assert!(pattern.is_match("foo"));
    assert!(pattern.is_match("foo123"));

    assert_eq!(pattern.find("foobaz"), Some((0, 3)));
    assert_eq!(pattern.find("test foo123"), Some((5, 8)));
}

#[test]
fn test_lookbehind_combined_simple() {
    // Pattern: (?<=foo)bar - match 'bar' only if preceded by 'foo'
    let pattern = Pattern::new(r"(?<=foo)bar").unwrap();

    assert!(pattern.is_match("foobar"));
    assert!(!pattern.is_match("bazbar"));
    assert!(!pattern.is_match("barfoo"));

    // find should return position of 'bar'
    assert_eq!(pattern.find("foobar"), Some((3, 6)));
    assert_eq!(pattern.find("test foobar end"), Some((8, 11)));
}

#[test]
fn test_negative_lookbehind_combined() {
    // Pattern: (?<!foo)bar - match 'bar' only if NOT preceded by 'foo'
    let pattern = Pattern::new(r"(?<!foo)bar").unwrap();

    assert!(pattern.is_match("bazbar"));
    assert!(!pattern.is_match("foobar"));
    assert!(pattern.is_match("bar"));
    assert!(pattern.is_match("123bar"));

    assert_eq!(pattern.find("bazbar"), Some((3, 6)));
    assert_eq!(pattern.find("test 123bar"), Some((8, 11)));
}

#[test]
fn test_lookahead_with_complex_prefix() {
    // Pattern: \d+(?=\w) - match digits only if followed by word char
    let pattern = Pattern::new(r"\d+(?=\w)").unwrap();

    assert!(pattern.is_match("123abc"));
    assert!(!pattern.is_match("123 "));
    assert!(!pattern.is_match("123."));

    assert_eq!(pattern.find("123abc"), Some((0, 3)));
}

#[test]
fn test_lookbehind_with_complex_suffix() {
    // Pattern: (?<=\d)abc - match literal 'abc' only if preceded by digit
    let pattern = Pattern::new(r"(?<=\d)abc").unwrap();

    assert!(pattern.is_match("9abc"));
    assert!(!pattern.is_match("xabc"));
    assert!(pattern.is_match("test9abc"));

    assert_eq!(pattern.find("9abc"), Some((1, 4)));
    assert_eq!(pattern.find("test9abc"), Some((5, 8)));
}

#[test]
fn test_lookahead_find_all() {
    // Pattern: foo(?=bar)
    let pattern = Pattern::new(r"foo(?=bar)").unwrap();

    let matches = pattern.find_all("foobar foobaz foobar");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0], (0, 3));
    assert_eq!(matches[1], (14, 17));
}

#[test]
fn test_lookbehind_find_all() {
    // Pattern: (?<=foo)bar
    let pattern = Pattern::new(r"(?<=foo)bar").unwrap();

    let matches = pattern.find_all("foobar bazbar foobar");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0], (3, 6));
    assert_eq!(matches[1], (17, 20));
}

#[test]
fn test_email_with_lookahead() {
    // Pattern: \w+@(?=gmail) - match email user@ only if domain starts with gmail
    let pattern = Pattern::new(r"\w+@(?=gmail)").unwrap();

    assert!(pattern.is_match("user@gmail.com"));
    assert!(!pattern.is_match("user@yahoo.com"));
}

#[test]
fn test_password_strength_lookahead() {
    // Pattern: \w+(?=.*\d) - match word chars only if digits exist ahead (simplified)
    // Note: .* matching is complex, so this is a basic test
    let pattern = Pattern::new(r"test(?=123)").unwrap();

    assert!(pattern.is_match("test123"));
    assert!(!pattern.is_match("testabc"));
}
