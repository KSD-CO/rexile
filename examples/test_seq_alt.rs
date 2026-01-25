fn main() {
    println!("=== Testing sequence + alternation ===\n");

    let tests = vec![
        // Sequence followed by alternation
        ("a(?:b|c)", "ab", Some((0, 2))),
        ("a(?:b|c)", "ac", Some((0, 2))),
        ("a(?:b|c)", "ad", None),

        // The actual problematic pattern
        ("rule (?:a|b)", "rule a", Some((0, 6))),
        ("rule (?:a|b)", "rule b", Some((0, 6))),
        ("rule (?:a|b)", "rule a salience", Some((0, 6))),  // Should match "rule a", not more

        // With \s+
        (r"rule\s+(?:a|b)", "rule a", Some((0, 6))),
        (r"rule\s+(?:a|b)", "rule  a", Some((0, 7))),
        (r"rule\s+(?:a|b)", "rule a salience", Some((0, 6))),  // Should match "rule a", not more

        // The identifier case
        (r"rule\s+(?:test|[a-z]+)", "rule test", Some((0, 9))),
        (r"rule\s+(?:test|[a-z]+)", "rule abc", Some((0, 8))),
        (r"rule\s+(?:test|[a-z]+)", "rule abc salience", Some((0, 8))),  // Should match "rule abc", not more
    ];

    for (pattern, text, expected) in tests {
        print!("{:<35} with {:<20?} => ", pattern, text);
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
