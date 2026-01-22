use rexile::Pattern;

#[test]
fn test_positive_lookahead_standalone() {
    // Standalone lookahead: checks if "bar" exists ahead at some position
    let pattern = Pattern::new(r"(?=bar)").unwrap();
    
    assert!(pattern.is_match("bar"));
    assert!(pattern.is_match("barfoo"));
    assert!(pattern.is_match("xbar")); // "bar" at position 1
    assert!(!pattern.is_match("foo"));
}

#[test]
fn test_negative_lookahead_standalone() {
    // Standalone negative lookahead: checks if there exists a position where "bar" does NOT match ahead
    let pattern = Pattern::new(r"(?!bar)").unwrap();
    
    // These should match because there's at least one position where bar doesn't match ahead
    assert!(pattern.is_match("foo"));
    assert!(pattern.is_match("baz"));
    assert!(pattern.is_match("bar")); // At position 1, "ar" != "bar", so negative lookahead succeeds
}

#[test]
fn test_positive_lookbehind_standalone() {
    // Standalone lookbehind: checks if "foo" exists behind at some position
    let pattern = Pattern::new(r"(?<=foo)").unwrap();
    
    assert!(pattern.is_match("foo")); // At end position, "foo" is behind
    assert!(pattern.is_match("foobar"));
    assert!(!pattern.is_match("bar")); // "foo" never behind any position
}

#[test]
fn test_negative_lookbehind_standalone() {
    // Standalone negative lookbehind
    let pattern = Pattern::new(r"(?<!foo)").unwrap();
    
    assert!(pattern.is_match("bar"));
    assert!(pattern.is_match("baz"));
    assert!(pattern.is_match("foo")); // At position 0, nothing behind, so negative lookbehind succeeds
}

#[test]
fn test_lookahead_with_find() {
    let pattern = Pattern::new(r"(?=bar)").unwrap();
    
    // Should find at position 0 where "bar" matches ahead
    let result = pattern.find("bar world");
    assert_eq!(result, Some((0, 0))); // Zero-width match
    
    // Find "bar" in "foobar" - should be at position 3
    let result2 = pattern.find("foobar");
    assert_eq!(result2, Some((3, 3))); // Zero-width match before "bar"
}

#[test]
fn test_lookbehind_with_find() {
    let pattern = Pattern::new(r"(?<=foo)").unwrap();
    
    // Should find at position 3 (after "foo")
    let result = pattern.find("foobar");
    assert_eq!(result, Some((3, 3))); // Zero-width match at end of "foo"
}

#[test]
// Phase 7.2 - NOW WORKING! - Combined patterns
fn test_lookahead_combined() {
    // Match "foo" only if followed by "bar"
    let pattern = Pattern::new(r"foo(?=bar)").unwrap();
    
    assert!(pattern.is_match("foobar"));
    assert!(!pattern.is_match("foobaz"));
}

#[test]
// Phase 7.2 - NOW WORKING! - Complex patterns  
fn test_lookahead_with_quantifier() {
    let pattern = Pattern::new(r"foo(?=\d+)").unwrap();
    
    assert!(pattern.is_match("foo123"));
    assert!(!pattern.is_match("foobar"));
}
