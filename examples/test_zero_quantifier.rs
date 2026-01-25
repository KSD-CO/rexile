fn main() {
    println!("=== Testing zero-match quantifiers ===\n");

    // These should all match because quantifiers allow zero matches
    test("a*", ""); // Should match empty
    test("a*", "b"); // Should match empty at start
    test("a*b", "b"); // a* matches zero, then b matches
    test("\\s*", ""); // Should match empty
    test("\\s*a", "a"); // \s* matches zero, then a matches
    test("\\s*\\{", "{"); // \s* matches zero, then \{ matches

    println!("\n=== Testing with text ===");
    test("\\s*\\{", " {"); // \s* matches space, then \{ matches
    test("a*b", "aaab"); // a* matches 'aaa', then b matches
}

fn test(pattern: &str, text: &str) {
    print!(
        "Pattern: {:15} Text: {:10} ",
        format!("{:?}", pattern),
        format!("{:?}", text)
    );
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            if pat.is_match(text) {
                println!("✓ MATCH - find={:?}", pat.find(text));
            } else {
                println!("✗ NO MATCH");
            }
        }
        Err(e) => println!("✗ ERROR: {}", e),
    }
}
