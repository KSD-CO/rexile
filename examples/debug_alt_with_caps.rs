fn main() {
    println!("=== Testing alternation with captures ===\n");

    // Test 1: Simple alternation with captures
    let patterns = vec![
        ("(?:(a)|(b))", vec!["a", "b", "c"]),
        ("(?:\"([^\"]+)\"|([a-z]+))", vec!["\"hello\"", "world", "\"test\"", "abc"]),
    ];

    for (pattern_str, test_cases) in patterns {
        println!("Pattern: {}", pattern_str);
        match rexile::Pattern::new(pattern_str) {
            Ok(pat) => {
                for text in test_cases {
                    let is_match = pat.is_match(text);
                    let find_result = pat.find(text);
                    let caps = pat.captures(text);
                    println!("  '{}': is_match={}, find={:?}, caps={:?}",
                             text, is_match, find_result,
                             caps.map(|c| (c.get(0), c.get(1), c.get(2))));
                }
            }
            Err(e) => {
                println!("  ✗ Compilation error: {}", e);
            }
        }
        println!();
    }
}
