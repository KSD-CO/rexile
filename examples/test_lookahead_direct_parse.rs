// Test by directly calling parsing functions to see where it breaks

fn main() {
    eprintln!("Testing simple literal first...");
    test_pattern("b");

    eprintln!("\nTesting lookahead...");
    test_pattern("(?=b)");
}

fn test_pattern(pattern: &str) {
    eprintln!("Pattern: {:?}", pattern);
    eprintln!("Creating Pattern::new...");
    match rexile::Pattern::new(pattern) {
        Ok(_) => eprintln!("✓ Success"),
        Err(e) => eprintln!("✗ Error: {:?}", e),
    }
}
