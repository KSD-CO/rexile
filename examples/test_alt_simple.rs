fn main() {
    println!("=== Testing simple alternation cases ===\n");

    let tests = vec![
        // Simple tests
        ("(?:a|b)", "a", Some((0, 1))),
        ("(?:a|b)", "b", Some((0, 1))),
        ("(?:a|b)", "ab", Some((0, 1))),  // Should match 'a', not 'ab'

        // With identifiers
        ("(?:rule|test)", "rule", Some((0, 4))),
        ("(?:rule|test)", "test", Some((0, 4))),
        ("(?:rule|test)", "ruletest", Some((0, 4))),  // Should match 'rule', not 'ruletest'

        // The problematic case
        (r#"(?:"x"|y)"#, "y", Some((0, 1))),
        (r#"(?:"x"|y)"#, "yz", Some((0, 1))),  // Should match 'y', not 'yz'

        // With quantified alternatives
        (r#"(?:[a-z]+|[0-9]+)"#, "abc", Some((0, 3))),
        (r#"(?:[a-z]+|[0-9]+)"#, "abc123", Some((0, 3))),  // Should match 'abc', not 'abc123'
    ];

    for (pattern, text, expected) in tests {
        print!("{:<30} with {:<15?} => ", pattern, text);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                let result = pat.find(text);
                if result == expected {
                    println!("✓ {:?}", result);
                } else {
                    println!("✗ got {:?}, expected {:?}", result, expected);
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
