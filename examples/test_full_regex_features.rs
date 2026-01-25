fn main() {
    println!("=== Testing Full Regex Feature Parity ===\n");

    let tests = vec![
        // Lazy quantifiers (non-greedy)
        (r"a.*?b", "axxxbyyybzzz", Some((0, 5)), "Lazy .* (non-greedy)"),
        (r"a.+?b", "axxxbyyybzzz", Some((0, 5)), "Lazy .+ (non-greedy)"),
        (r"a.??b", "axb", Some((0, 3)), "Lazy .? (non-greedy)"),
        
        // Nested quantifiers
        (r"(a+)+", "aaaa", Some((0, 4)), "Nested quantifiers (a+)+"),
        (r"(a*)*", "aaaa", Some((0, 4)), "Nested quantifiers (a*)*"),
        
        // Character class ranges
        (r"[a-z]+", "hello123", Some((0, 5)), "Character class range [a-z]"),
        (r"[A-Z]+", "HELLO123", Some((0, 5)), "Character class range [A-Z]"),
        (r"[0-9]+", "abc123xyz", Some((3, 6)), "Character class range [0-9]"),
        
        // Negated character classes
        (r"[^a-z]+", "ABC123xyz", Some((0, 6)), "Negated class [^a-z]"),
        (r"[^\s]+", "hello world", Some((0, 5)), r"Negated whitespace [^\s]"),
        
        // Escapes
        (r"\d+", "abc123", Some((3, 6)), r"Digit escape \d+"),
        (r"\w+", "hello_world", Some((0, 11)), r"Word escape \w+"),
        (r"\s+", "a   b", Some((1, 4)), r"Space escape \s+"),
        
        // Word boundaries
        (r"\bhello\b", "hello world", Some((0, 5)), r"Word boundary \b"),
        (r"\Bhell", "shell", Some((1, 5)), r"Non-word boundary \B"),
        
        // Anchors
        (r"^hello", "hello world", Some((0, 5)), "Start anchor ^"),
        (r"world$", "hello world", Some((6, 11)), "End anchor $"),
        
        // Alternation with captures
        (r"(cat|dog)", "I have a cat", Some((9, 12)), "Alternation (cat|dog)"),
        (r"(http|https)://", "https://example.com", Some((0, 8)), "Alternation protocol"),
        
        // Complex nested patterns
        (r"a(b|c)*d", "abcbcd", Some((0, 6)), "Pattern with alternation inside"),
        (r"(a|b)+c", "ababc", Some((0, 5)), "Quantified alternation"),
        
        // Lookahead (if supported)
        (r"a(?=b)", "abc", Some((0, 1)), "Positive lookahead (?=)"),
        (r"a(?!b)", "acd", Some((0, 1)), "Negative lookahead (?!)"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (pattern, text, expected, desc) in tests {
        print!("{:<50} => ", desc);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                let result = pat.find(text);
                if result == expected {
                    println!("✓ {:?}", result);
                    passed += 1;
                } else {
                    println!("✗ got {:?}, expected {:?}", result, expected);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("✗ ERROR: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Passed: {}/{}", passed, passed + failed);
    println!("Failed: {}", failed);
}
