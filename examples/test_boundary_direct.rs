use rexile::Pattern;

fn test_pattern(pattern_str: &str, text: &str, description: &str) {
    println!("\n=== {} ===", description);
    println!("Pattern: {:?}", pattern_str);
    println!("Text: {:?}", text);

    match Pattern::new(pattern_str) {
        Ok(pattern) => {
            println!("✓ Pattern compiled");

            // Test is_match
            let is_match = pattern.is_match(text);
            println!("is_match: {}", is_match);

            // Test find
            match pattern.find(text) {
                Some((start, end)) => {
                    println!("find: Some(({}, {})) = {:?}", start, end, &text[start..end]);
                }
                None => {
                    println!("find: None");
                }
            }

            // Test find_all
            let all_matches = pattern.find_all(text);
            println!("find_all: {} matches", all_matches.len());
            for (start, end) in all_matches {
                println!("  [{}, {}) = {:?}", start, end, &text[start..end]);
            }
        }
        Err(e) => {
            println!("✗ Failed to compile: {:?}", e);
        }
    }
}

fn main() {
    println!("Testing word boundary patterns\n");

    // Test individual boundaries first
    test_pattern(r"\b", "hello world", "Standalone \\b");
    test_pattern(r"\B", "hello world", "Standalone \\B");

    // Test boundary + literal
    test_pattern(r"\bhello", "hello world", "\\bhello in 'hello world'");
    test_pattern(r"\bhello", "xhello world", "\\bhello in 'xhello world' (should NOT match)");
    test_pattern(r"\bhello", " hello world", "\\bhello in ' hello world'");

    test_pattern(r"hello\b", "hello world", "hello\\b in 'hello world'");
    test_pattern(r"hello\b", "hellox world", "hello\\b in 'hellox world' (should NOT match)");

    test_pattern(r"\bworld", "hello world", "\\bworld in 'hello world'");
    test_pattern(r"\bworld", "helloworld", "\\bworld in 'helloworld' (should NOT match)");

    // Test quantified + boundary (this works according to tests)
    test_pattern(r"\w+\b", "hello world", "\\w+\\b in 'hello world'");

    // Test boundary + quantified
    test_pattern(r"\b\w+", "hello world", "\\b\\w+ in 'hello world'");

    // Test more complex patterns
    test_pattern(r"\b\w{5}\b", "hello world", "\\b\\w{5}\\b in 'hello world'");
}
