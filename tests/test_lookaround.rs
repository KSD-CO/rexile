use rexile::Pattern;

#[test]
fn test_positive_lookahead_basic() {
    // Match "foo" only if followed by "bar"
    let pattern = Pattern::new(r"foo(?=bar)").unwrap();
    
    assert!(pattern.is_match("foobar"));
    assert!(!pattern.is_match("foobaz"));
    assert!(!pattern.is_match("foo"));
}

#[test]
fn test_negative_lookahead_basic() {
    // Match "foo" only if NOT followed by "bar"
    let pattern = Pattern::new(r"foo(?!bar)").unwrap();
    
    assert!(pattern.is_match("foobaz"));
    assert!(pattern.is_match("foo"));
    assert!(!pattern.is_match("foobar"));
}

#[test]
fn test_positive_lookbehind_basic() {
    // Match "bar" only if preceded by "foo"
    let pattern = Pattern::new(r"(?<=foo)bar").unwrap();
    
    assert!(pattern.is_match("foobar"));
    assert!(!pattern.is_match("bazbar"));
    assert!(!pattern.is_match("bar"));
}

#[test]
fn test_negative_lookbehind_basic() {
    // Match "bar" only if NOT preceded by "foo"
    let pattern = Pattern::new(r"(?<!foo)bar").unwrap();
    
    assert!(pattern.is_match("bazbar"));
    assert!(pattern.is_match("bar"));
    assert!(!pattern.is_match("foobar"));
}

#[test]
fn test_lookahead_with_find() {
    let pattern = Pattern::new(r"foo(?=bar)").unwrap();
    
    // Should find "foo" at position 0
    let result = pattern.find("foobar world");
    assert_eq!(result, Some((0, 3)));
    
    // Should not find in "foobaz"
    assert_eq!(pattern.find("foobaz world"), None);
}

#[test]
fn test_lookbehind_with_find() {
    let pattern = Pattern::new(r"(?<=foo)bar").unwrap();
    
    // Should find "bar" at position 3
    let result = pattern.find("foobar world");
    assert_eq!(result, Some((3, 6)));
    
    // Should not find in "bazbar"
    assert_eq!(pattern.find("bazbar world"), None);
}

#[test]
#[ignore] // Complex feature - implement in phase 7.2
fn test_multiple_lookarounds() {
    // Match word that has "a" ahead and "b" behind
    let pattern = Pattern::new(r"(?<=b)o(?=a)").unwrap();
    
    assert!(pattern.is_match("boa"));
    assert!(!pattern.is_match("aob"));
}

#[test]
#[ignore] // Complex feature - implement in phase 7.2
fn test_lookahead_with_quantifier() {
    // Match "foo" followed by one or more digits
    let pattern = Pattern::new(r"foo(?=\d+)").unwrap();
    
    assert!(pattern.is_match("foo123"));
    assert!(!pattern.is_match("foobar"));
}
