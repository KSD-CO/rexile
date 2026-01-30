use rexile::Pattern;

#[test]
fn test_exactly_n_basic() {
    // Test {n} - exactly n times
    let pattern = Pattern::new(r"\d{3}").unwrap();

    // Should match
    assert!(pattern.is_match("123"));
    assert!(pattern.is_match("abc123def"));

    // Should not match
    assert!(!pattern.is_match("12"));
    assert!(!pattern.is_match("ab"));
}

#[test]
fn test_exactly_n_find() {
    let pattern = Pattern::new(r"\d{3}").unwrap();

    // Find exactly 3 digits
    assert_eq!(pattern.find("abc123def"), Some((3, 6)));
    assert_eq!(pattern.find("12345"), Some((0, 3)));

    // Should not match less than 3
    assert_eq!(pattern.find("12"), None);
}

#[test]
fn test_between_n_m_basic() {
    // Test {n,m} - between n and m times
    let pattern = Pattern::new(r"\d{2,4}").unwrap();

    // Should match
    assert!(pattern.is_match("12"));
    assert!(pattern.is_match("123"));
    assert!(pattern.is_match("1234"));
    assert!(pattern.is_match("abc123def"));
}

#[test]
fn test_between_n_m_find() {
    let pattern = Pattern::new(r"\d{2,4}").unwrap();

    // Find 2-4 digits (greedy, should take max)
    assert_eq!(pattern.find("12"), Some((0, 2)));
    assert_eq!(pattern.find("1234"), Some((0, 4)));
    assert_eq!(pattern.find("12345"), Some((0, 4))); // Takes max 4

    assert_eq!(pattern.find("abc123def"), Some((3, 6))); // Takes 3
}

#[test]
fn test_ip_address_pattern() {
    // Classic IP pattern with range quantifiers
    let pattern = Pattern::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();

    assert!(pattern.is_match("192.168.1.1"));
    assert!(pattern.is_match("10.0.0.1"));

    // Find should return correct position
    let result = pattern.find("Server at 192.168.1.1 port 80");
    println!("IP find result: {:?}", result);
    assert_eq!(result, Some((10, 21))); // "192.168.1.1"

    if let Some((start, end)) = result {
        let matched = &"Server at 192.168.1.1 port 80"[start..end];
        println!("Matched: '{}'", matched);
        assert_eq!(
            matched, "192.168.1.1",
            "Should match IP only, not trailing space"
        );
    }
}

#[test]
fn test_date_pattern() {
    // Date pattern: YYYY-MM-DD
    let pattern = Pattern::new(r"\d{4}-\d{2}-\d{2}").unwrap();

    assert!(pattern.is_match("2024-01-30"));

    // Find should return correct position
    let result = pattern.find("Date: 2024-01-30 time");
    println!("Date find result: {:?}", result);
    assert_eq!(result, Some((6, 16))); // "2024-01-30"
}

#[test]
fn test_phone_pattern() {
    // Phone: 3 digits, dash, 3 digits, dash, 4 digits
    let pattern = Pattern::new(r"\d{3}-\d{3}-\d{4}").unwrap();

    assert!(pattern.is_match("555-123-4567"));

    let result = pattern.find("Call 555-123-4567 now");
    println!("Phone find result: {:?}", result);
    assert_eq!(result, Some((5, 17))); // "555-123-4567"
}

#[test]
fn test_hex_color() {
    // Hex color: # followed by exactly 6 hex digits
    let pattern = Pattern::new(r"#[0-9a-fA-F]{6}").unwrap();

    assert!(pattern.is_match("#FF5733"));
    assert!(pattern.is_match("color: #FF5733"));

    let result = pattern.find("color: #FF5733 background");
    println!("Color find result: {:?}", result);
    assert_eq!(result, Some((7, 14))); // "#FF5733"
}

#[test]
fn test_range_quantifier_find_all() {
    let pattern = Pattern::new(r"\d{2}").unwrap();

    let matches = pattern.find_all("11 22 33 44");
    println!("Find all result: {:?}", matches);

    // Should find all pairs
    assert_eq!(matches.len(), 4);
    assert_eq!(matches[0], (0, 2)); // "11"
    assert_eq!(matches[1], (3, 5)); // "22"
    assert_eq!(matches[2], (6, 8)); // "33"
    assert_eq!(matches[3], (9, 11)); // "44"
}

#[test]
fn test_mixed_quantifiers() {
    // Pattern with different quantifier types
    let pattern = Pattern::new(r"\w{3,5}@\w+\.com").unwrap();

    assert!(pattern.is_match("user@example.com"));
    assert!(pattern.is_match("admin@test.com"));

    let result = pattern.find("Email: user@example.com end");
    println!("Email find result: {:?}", result);
    // Should find "user@example.com"
}
