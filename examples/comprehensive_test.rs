use rexile::Pattern;

fn main() {
    println!("=== Rexile Comprehensive Feature Test ===\n");

    let tests = vec![
        // Basic patterns
        ("hello", "hello world", Some((0, 5)), "Literal match"),
        (
            "world",
            "hello world",
            Some((6, 11)),
            "Literal match mid-text",
        ),
        // Character classes
        ("[a-z]+", "Hello123", Some((1, 5)), "Lowercase letters"),
        ("[0-9]+", "abc123def", Some((3, 6)), "Digits"),
        ("[^0-9]+", "123abc456", Some((3, 6)), "Negated digits"),
        // Quantifiers
        ("a+", "aaa", Some((0, 3)), "One or more"),
        ("a*", "bbb", Some((0, 0)), "Zero or more (zero match)"),
        ("a?", "a", Some((0, 1)), "Zero or one"),
        ("a{2,4}", "aaaaa", Some((0, 4)), "Range quantifier"),
        // Lazy quantifiers
        (".*?b", "aaaaab", Some((0, 6)), "Lazy star"),
        (".+?b", "aaaaab", Some((0, 6)), "Lazy plus"),
        // Escapes
        (r"\d+", "abc123", Some((3, 6)), "Digit escape"),
        (r"\w+", "hello_world", Some((0, 11)), "Word escape"),
        (r"\s+", "a   b", Some((1, 4)), "Whitespace escape"),
        // Boundaries
        (
            r"\bhello",
            "hello world",
            Some((0, 5)),
            "Word boundary start",
        ),
        (
            r"world\b",
            "hello world",
            Some((6, 11)),
            "Word boundary end",
        ),
        (r"\bhello\b", "hello", Some((0, 5)), "Word boundary both"),
        (r"\B", "hello", Some((1, 1)), "Non-word boundary"),
        // Anchors
        ("^hello", "hello world", Some((0, 5)), "Start anchor"),
        ("world$", "hello world", Some((6, 11)), "End anchor"),
        ("^hello$", "hello", Some((0, 5)), "Both anchors"),
        // Alternation
        (
            "cat|dog",
            "I have a dog",
            Some((9, 12)),
            "Simple alternation",
        ),
        (
            "(http|https)://",
            "https://example.com",
            Some((0, 8)),
            "Protocol alternation",
        ),
        // Groups
        ("(hello)", "hello world", Some((0, 5)), "Capture group"),
        (
            "(?:hello)",
            "hello world",
            Some((0, 5)),
            "Non-capturing group",
        ),
        // Lookaround
        (r"a(?=b)", "abc", Some((0, 1)), "Positive lookahead"),
        (r"a(?!b)", "acd", Some((0, 1)), "Negative lookahead"),
        (r"(?<=a)b", "abc", Some((1, 2)), "Positive lookbehind"),
        (r"(?<!a)b", "cbc", Some((1, 2)), "Negative lookbehind"),
        // Complex patterns
        (
            r"\d{3}-\d{4}",
            "Call 555-1234 now",
            Some((5, 13)),
            "Phone number",
        ),
        (
            r"\b\w+@\w+\.\w+",
            "Email: user@example.com!",
            Some((7, 23)),
            "Email pattern",
        ),
        (r"(?i)hello", "HELLO", Some((0, 5)), "Case insensitive"),
        // Edge cases
        ("", "", Some((0, 0)), "Empty pattern"),
        (".*", "", Some((0, 0)), "Match empty with .*"),
        (r"\b", "a", Some((0, 0)), "Boundary at start"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (pattern_str, text, expected, desc) in tests {
        print!("{:50} => ", desc);

        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let result = pattern.find(text);
                if result == expected {
                    println!("✓ {:?}", result);
                    passed += 1;
                } else {
                    println!("✗ got {:?}, expected {:?}", result, expected);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("✗ compile error: {:?}", e);
                failed += 1;
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Passed: {}/{}", passed, passed + failed);
    println!("Failed: {}", failed);

    if failed == 0 {
        println!("\n✓ All features working! Ready for production use.");
    }
}
