fn main() {
    // .*test.* should match ANY text containing "test"
    let tests = vec![
        ("test", true),           // .* matches "", test matches, .* matches ""
        ("xtest", true),          // .* matches "x", test matches, .* matches ""
        ("testing", true),        // .* matches "", test matches, .* matches "ing"
        ("xtesting", true),       // .* matches "x", test matches, .* matches "ing"
        ("this is a test", true), // .* matches "this is a ", test matches, .* matches ""
        ("no match", false),      // Doesn't contain "test"
    ];

    let pattern = rexile::Pattern::new(".*test.*").unwrap();

    for (text, expected) in tests {
        let result = pattern.is_match(text);
        let status = if result == expected { "✅" } else { "❌" };
        println!("{} '{}': {} (expected {})", status, text, result, expected);
    }
}
