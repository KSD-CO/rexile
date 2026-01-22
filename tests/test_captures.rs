use rexile::{Pattern, Captures};

#[test]
fn test_single_capture_group() {
    let pattern = Pattern::new(r"Hello (\w+)").unwrap();
    
    if let Some(caps) = pattern.captures("Hello world") {
        assert_eq!(&caps[0], "Hello world");  // Full match
        assert_eq!(&caps[1], "world");         // Capture group 1
    } else {
        panic!("Expected captures");
    }
}

#[test]
fn test_multiple_capture_groups() {
    let pattern = Pattern::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    
    if let Some(caps) = pattern.captures("Date: 2026-01-22") {
        assert_eq!(&caps[0], "2026-01-22");  // Full match
        assert_eq!(&caps[1], "2026");         // Year
        assert_eq!(&caps[2], "01");           // Month
        assert_eq!(&caps[3], "22");           // Day
    } else {
        panic!("Expected captures");
    }
}

#[test]
fn test_non_capturing_group() {
    // (?:...) should not create a capture
    let pattern = Pattern::new(r"(?:Hello) (\w+)").unwrap();
    
    if let Some(caps) = pattern.captures("Hello world") {
        assert_eq!(&caps[0], "Hello world");  // Full match
        assert_eq!(&caps[1], "world");         // Only one capture group
        assert_eq!(caps.len(), 2);             // Full match + 1 group
    } else {
        panic!("Expected captures");
    }
}

#[test]
fn test_captures_iter() {
    let pattern = Pattern::new(r"(\w+)=(\d+)").unwrap();
    let text = "a=1 b=2 c=3";
    
    let all_captures: Vec<_> = pattern.captures_iter(text).collect();
    
    assert_eq!(all_captures.len(), 3);
    
    // First match: a=1
    assert_eq!(&all_captures[0][1], "a");
    assert_eq!(&all_captures[0][2], "1");
    
    // Second match: b=2
    assert_eq!(&all_captures[1][1], "b");
    assert_eq!(&all_captures[1][2], "2");
    
    // Third match: c=3
    assert_eq!(&all_captures[2][1], "c");
    assert_eq!(&all_captures[2][2], "3");
}

#[test]
// DONE: Complex feature - implement in phase 8.2
fn test_nested_capture_groups() {
    let pattern = Pattern::new(r"(a(b(c)))").unwrap();
    
    if let Some(caps) = pattern.captures("abc") {
        assert_eq!(&caps[0], "abc");  // Full match
        assert_eq!(&caps[1], "abc");  // Group 1
        assert_eq!(&caps[2], "bc");   // Group 2
        assert_eq!(&caps[3], "c");    // Group 3
    } else {
        panic!("Expected captures");
    }
}

#[test]
#[ignore] // Complex feature - implement in phase 8.2
fn test_backreference() {
    // Match same word twice: (\w+)\s+\1
    let pattern = Pattern::new(r"(\w+)\s+\1").unwrap();
    
    assert!(pattern.is_match("hello hello"));
    assert!(!pattern.is_match("hello world"));
}

#[test]
fn test_replace_with_captures() {
    let pattern = Pattern::new(r"(\w+)=(\d+)").unwrap();
    let text = "a=1 b=2";
    
    let result = pattern.replace_all(text, "$1:[$2]");
    assert_eq!(result, "a:[1] b:[2]");
}

#[test]
fn test_split_with_captures() {
    let pattern = Pattern::new(r"(\d+)").unwrap();
    let text = "a1b2c3";
    
    let parts: Vec<_> = pattern.split(text).collect();
    assert_eq!(parts, vec!["a", "b", "c", ""]);
}

#[test]
fn test_capture_positions() {
    let pattern = Pattern::new(r"(\w+):(\d+)").unwrap();
    
    if let Some(caps) = pattern.captures("name:123 age:45") {
        // Check positions
        assert_eq!(caps.pos(0), Some((0, 8)));  // "name:123"
        assert_eq!(caps.pos(1), Some((0, 4)));  // "name"
        assert_eq!(caps.pos(2), Some((5, 8)));  // "123"
    } else {
        panic!("Expected captures");
    }
}
