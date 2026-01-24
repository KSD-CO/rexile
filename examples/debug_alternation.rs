fn main() {
    let pattern_str = "(?:a|b)";
    println!("Pattern: {:?}\n", pattern_str);

    match rexile::Pattern::new(pattern_str) {
        Ok(pat) => {
            println!("✓ Compiled successfully");
            println!("Debug format: {:#?}\n", pat);

            println!("Testing matches:");
            for test_text in &["a", "b", "c", "ab", "ba", "xa", "bx"] {
                let is_match = pat.is_match(test_text);
                let find_result = pat.find(test_text);
                println!("  '{}': is_match={}, find={:?}", test_text, is_match, find_result);
            }
        }
        Err(e) => {
            println!("✗ Compilation error: {}", e);
        }
    }
}
