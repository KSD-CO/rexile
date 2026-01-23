use rexile::Pattern;

#[test]
fn test_word_boundary_detection() {
    // \b matches at word boundaries
    let pattern = Pattern::new("\\b").unwrap();

    // Should find boundaries in text
    assert!(pattern.is_match("hello world"));
    assert!(pattern.is_match("test"));

    // Empty text has no boundaries
    assert!(!pattern.is_match(""));
}

#[test]
fn test_non_word_boundary() {
    // \B matches at non-word boundaries (inside words)
    let pattern = Pattern::new("\\B").unwrap();

    // Should find non-boundaries
    assert!(pattern.is_match("hello")); // Inside "hello"
    assert!(!pattern.is_match("")); // No non-boundaries in empty text
}

#[test]
fn test_boundary_find_all() {
    let pattern = Pattern::new("\\b").unwrap();
    let text = "hello world";

    // Word boundaries in "hello world" are at positions where
    // word/non-word character transition occurs
    let matches = pattern.find_all(text);

    // Should find at least 4 boundaries: start, between words, end
    assert!(
        matches.len() >= 4,
        "Expected at least 4 boundaries, found {}",
        matches.len()
    );

    // Verify boundary positions are zero-width
    for (start, end) in &matches {
        assert_eq!(start, end, "Boundaries should be zero-width");
    }
}

#[test]
fn test_boundary_with_punctuation() {
    let pattern = Pattern::new("\\b").unwrap();
    let text = "hello, world!";

    // Boundaries at: 0 (start), 5 (o|,), 7 (,_|w), 12 (d|!)
    let matches = pattern.find_all(text);
    assert!(matches.len() >= 4);
}

#[test]
fn test_non_boundary_find_all() {
    let pattern = Pattern::new("\\B").unwrap();
    let text = "hello";

    // Non-boundaries are positions NOT at word boundaries
    let matches = pattern.find_all(text);

    // Inside "hello" there should be non-boundaries between letters
    assert!(!matches.is_empty(), "Expected non-boundaries inside word");

    // Verify non-boundary positions are zero-width
    for (start, end) in &matches {
        assert_eq!(start, end, "Non-boundaries should be zero-width");
    }
}

#[test]
fn test_boundary_empty_text() {
    let pattern = Pattern::new("\\b").unwrap();
    assert!(!pattern.is_match(""));
}

// NOTE: Complex patterns like \bhello\b require sequence parsing
// which will be implemented in future phases. For now, \b and \B
// work as standalone boundary matchers.
