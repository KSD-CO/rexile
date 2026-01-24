fn main() {
    println!("=== Debug sequence matching ===\n");

    // Test simple quantified + literal
    test_debug(r"\s*", "");
    test_debug(r"\s*", "a");
    test_debug(r"a", "a");
    test_debug(r"\sa", " a");
    test_debug(r"\s*a", "a");
    test_debug(r"\s*a", " a");
}

fn test_debug(pattern: &str, text: &str) {
    println!("\n=== Pattern: {:?}, Text: {:?} ===", pattern, text);
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("is_match: {}", pat.is_match(text));
            println!("find: {:?}", pat.find(text));
            if let Some((start, end)) = pat.find(text) {
                println!("Matched: {:?}", &text[start..end]);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
