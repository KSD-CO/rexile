use rexile::Pattern;

/// Comprehensive test of production-ready regex features
/// This demonstrates all working features suitable for a rule engine
fn main() {
    println!("=== Rexile Production-Ready Feature Test ===");
    println!("Testing all features suitable for rule engine use\n");

    let tests = vec![
        // === LITERALS ===
        ("hello", "hello world", Some((0, 5)), "Literal match"),
        ("world", "hello world", Some((6, 11)), "Literal in middle"),

        // === CHARACTER CLASSES ===
        ("[a-z]+", "Hello123", Some((1, 5)), "Lowercase range"),
        ("[A-Z]+", "hello WORLD", Some((6, 11)), "Uppercase range"),
        ("[0-9]+", "abc123def", Some((3, 6)), "Digit range"),
        ("[a-zA-Z]+", "123abc", Some((3, 6)), "Multi-range"),
        ("[abc]", "xyz a", Some((4, 5)), "Simple class"),

        // === NEGATED CHARACTER CLASSES ===
        ("[^0-9]+", "123abc456", Some((3, 6)), "Negated digits"),
        ("[^a-z]+", "hello123", Some((5, 8)), "Negated lowercase"),
        (r"[^\s]+", " hello", Some((1, 6)), "Negated whitespace"),

        // === QUANTIFIERS ===
        ("a+", "aaa", Some((0, 3)), "One or more"),
        ("a*", "aaa", Some((0, 3)), "Zero or more"),
        ("a?b", "ab", Some((0, 2)), "Zero or one"),
        ("a{2,}", "aaaaa", Some((0, 5)), "At least N"),

        // === LAZY QUANTIFIERS ===
        (".*?b", "aaaaab", Some((0, 6)), "Lazy star"),
        (".+?b", "aaaaab", Some((0, 6)), "Lazy plus"),
        (".??b", "ab", Some((0, 2)), "Lazy optional"),

        // === ESCAPE SEQUENCES ===
        (r"\d", "abc1", Some((3, 4)), "Single digit"),
        (r"\d+", "abc123def", Some((3, 6)), "Digit sequence"),
        (r"\w+", "hello_world", Some((0, 11)), "Word characters"),
        (r"\s+", "a   b", Some((1, 4)), "Whitespace"),
        (r"\.", "a.b", Some((1, 2)), "Escaped dot"),

        // === WORD BOUNDARIES ===
        (r"\bhello", "hello world", Some((0, 5)), "Boundary at start"),
        (r"world\b", "hello world", Some((6, 11)), "Boundary at end"),
        (r"\bhello\b", "hello", Some((0, 5)), "Both boundaries"),
        (r"\btest\b", "testing", None, "No match - not whole word"),
        (r"\B", "hello", Some((1, 1)), "Non-word boundary"),

        // === ANCHORS ===
        ("^hello", "hello world", Some((0, 5)), "Start anchor"),
        ("world$", "hello world", Some((6, 11)), "End anchor"),
        ("^hello$", "hello", Some((0, 5)), "Full string match"),
        ("^hello", "say hello", None, "Start anchor - no match"),

        // === ALTERNATION ===
        ("cat|dog", "I have a dog", Some((9, 12)), "Simple alternation"),
        ("(http|https)://", "https://example.com", Some((0, 8)), "Complex alternation"),
        ("a|b|c", "xyz c", Some((4, 5)), "Multiple alternatives"),

        // === GROUPS ===
        ("(hello)", "hello world", Some((0, 5)), "Capture group"),
        ("(?:hello)", "hello world", Some((0, 5)), "Non-capturing group"),
        ("(\\d+)", "abc123", Some((3, 6)), "Capturing digits"),

        // === LOOKAHEAD ===
        (r"a(?=b)", "abc", Some((0, 1)), "Positive lookahead"),
        (r"a(?!b)", "acd", Some((0, 1)), "Negative lookahead"),
        (r"\w+(?=:)", "key:value", Some((0, 3)), "Word before colon"),

        // === CASE INSENSITIVE ===
        (r"(?i)hello", "HELLO", Some((0, 5)), "Case insensitive flag"),
        (r"(?i)test", "TeSt", Some((0, 4)), "Mixed case match"),

        // === DOT METACHARACTER ===
        ("h.llo", "hello", Some((0, 5)), "Dot matches char"),
        ("...", "abc", Some((0, 3)), "Multiple dots"),
        (".*", "hello", Some((0, 5)), "Dot star greedy"),

        // === PRACTICAL PATTERNS ===
        (r"\w+@\w+\.\w+", "user@example.com", Some((0, 16)), "Simple email"),
        (r"\d{1,3}\.\d{1,3}", "192.168.1.1", Some((0, 7)), "IP prefix"),
        (r"(?i)(GET|POST)", "GET /api", Some((0, 3)), "HTTP method"),
        (r"\b\d{4}\b", "Year: 2024!", Some((6, 10)), "Year extraction"),

        // === EDGE CASES ===
        ("", "", Some((0, 0)), "Empty pattern"),
        ("a", "", None, "No match in empty string"),
        (r"\b", "a", Some((0, 0)), "Boundary at position 0"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (pattern_str, text, expected, desc) in tests {
        print!("{:50} => ", desc);

        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let result = pattern.find(text);
                if result == expected {
                    println!("✓");
                    passed += 1;
                } else {
                    println!("✗ got {:?}, expected {:?}", result, expected);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("✗ ERROR: {:?}", e);
                failed += 1;
            }
        }
    }

    println!("\n=== Results ===");
    println!("Passed: {}/{}", passed, passed + failed);
    println!("Failed: {}", failed);
    println!("Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

    if failed == 0 {
        println!("\n✓ ALL TESTS PASSED");
        println!("✓ Library is production-ready for rule engine use!");
    } else {
        println!("\n⚠ Some tests failed - review above for details");
    }

    println!("\n=== Known Limitations ===");
    println!("- Range quantifiers {{n}} and {{n,m}} have bugs in find()");
    println!("- Standalone lookbehind patterns not supported (use combined patterns)");
    println!("- Some edge cases with empty strings may not behave as expected");
}
