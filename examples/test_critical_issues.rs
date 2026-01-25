fn main() {
    println!("=== Testing Critical Issues ===\n");

    let tests = vec![
        // Word boundary issues
        (r"\bhello", "hello world", Some((0, 5)), "Word boundary at start"),
        (r"hello\b", "hello world", Some((0, 5)), "Word boundary at end"),
        (r"\bworld", "hello world", Some((6, 11)), "Word boundary mid-text"),
        
        // Start anchor issues  
        (r"^hello", "hello", Some((0, 5)), "Anchored start - exact"),
        (r"^h", "hello", Some((0, 1)), "Anchored start - single char"),
        
        // Negated character class
        (r"[^a]", "bac", Some((0, 1)), "Simple negated class"),
        (r"[^\s]", " a", Some((1, 2)), "Negated whitespace - single"),
    ];

    for (pattern, text, expected, desc) in tests {
        print!("{:<50} => ", desc);
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
