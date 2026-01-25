use rexile::Pattern;

fn test_match(pattern_str: &str, text: &str, should_match: bool, description: &str) {
    print!("{:50} ", description);
    match Pattern::new(pattern_str) {
        Ok(pattern) => {
            let did_match = pattern.is_match(text);
            if did_match == should_match {
                println!(
                    "✓ PASS ({})",
                    if did_match { "matched" } else { "no match" }
                );
            } else {
                println!(
                    "✗ FAIL (expected {}, got {})",
                    if should_match { "match" } else { "no match" },
                    if did_match { "match" } else { "no match" }
                );

                // Show what was found
                if let Some((start, end)) = pattern.find(text) {
                    println!(
                        "     Found at [{}, {}) = {:?}",
                        start,
                        end,
                        &text[start..end]
                    );
                }
            }
        }
        Err(e) => {
            println!("✗ COMPILE ERROR: {:?}", e);
        }
    }
}

fn main() {
    println!("Testing word boundary in sequences:\n");

    // These should ALL work
    test_match(r"\bhello", "hello world", true, r"\bhello in 'hello world'");
    test_match(
        r"\bhello",
        "xhello",
        false,
        r"\bhello in 'xhello' (no boundary)",
    );
    test_match(r"\bhello", " hello", true, r"\bhello in ' hello'");

    test_match(r"hello\b", "hello world", true, r"hello\b in 'hello world'");
    test_match(
        r"hello\b",
        "hellox",
        false,
        r"hello\b in 'hellox' (no boundary)",
    );

    test_match(r"\bworld", "hello world", true, r"\bworld in 'hello world'");
    test_match(
        r"\bworld",
        "helloworld",
        false,
        r"\bworld in 'helloworld' (no boundary)",
    );

    test_match(r"\bhello\b", "hello", true, r"\bhello\b in 'hello'");
    test_match(
        r"\bhello\b",
        "hello world",
        true,
        r"\bhello\b in 'hello world'",
    );
    test_match(
        r"\bhello\b",
        "xhellox",
        false,
        r"\bhello\b in 'xhellox' (no boundaries)",
    );

    // These work (according to earlier tests)
    test_match(r"\w+\b", "hello world", true, r"\w+\b in 'hello world'");
    test_match(r"\b\w+", "hello world", true, r"\b\w+ in 'hello world'");
}
