fn main() {
    println!("=== Testing escaped braces ===\n");

    // Test 1: Just a literal brace
    test_pattern(r"\{", "{");
    test_pattern(r"\}", "}");

    // Test 2: Braces in a larger pattern
    test_pattern(r"test\{", "test{");
    test_pattern(r"test\}", "test}");

    // Test 3: Braces with whitespace
    test_pattern(r"\s*\{", " {");
    test_pattern(r"\s*\{", "{");

    // Test 4: The failing pattern
    test_pattern(r"abc\{def\}", "abc{def}");
}

fn test_pattern(pattern: &str, text: &str) {
    print!("Pattern: {:20} Text: {:15} ", pattern, text);
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            let is_match = pat.is_match(text);
            let find = pat.find(text);
            if is_match {
                println!("✓ MATCH - find={:?}", find);
            } else {
                println!("✗ NO MATCH");
            }
        }
        Err(e) => println!("✗ ERROR: {}", e),
    }
}
