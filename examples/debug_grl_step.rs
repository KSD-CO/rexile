fn main() {
    println!("=== Testing GRL pattern components ===\n");

    let tests = vec![
        // Test 1: Just the alternation
        ("(?:\"([^\"]+)\"|([a-z]+))", "\"hello\""),
        ("(?:\"([^\"]+)\"|([a-z]+))", "world"),

        // Test 2: rule + whitespace + alternation
        ("rule\\s+(?:\"([^\"]+)\"|([a-z]+))", "rule \"hello\""),
        ("rule\\s+(?:\"([^\"]+)\"|([a-z]+))", "rule world"),

        // Test 3: Add optional whitespace
        ("rule\\s+(?:\"([^\"]+)\"|([a-z]+))\\s*", "rule \"hello\" "),
        ("rule\\s+(?:\"([^\"]+)\"|([a-z]+))\\s*", "rule world"),

        // Test 4: Add simple capture group
        ("rule\\s+(?:\"([^\"]+)\"|([a-z]+))\\s*(x)", "rule \"hello\" x"),

        // Test 5: Add character class capture
        ("rule\\s+(?:\"([^\"]+)\"|([a-z]+))\\s*([a-z]*)", "rule \"hello\" attr"),
    ];

    for (pattern, text) in tests {
        print!("Pattern: {:<50} Text: {:<20} => ", pattern, text);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                let is_match = pat.is_match(text);
                if is_match {
                    if let Some((s, e)) = pat.find(text) {
                        println!("✓ [{}, {}]", s, e);
                    } else {
                        println!("✓");
                    }
                } else {
                    println!("✗ NO MATCH");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
