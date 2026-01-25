fn main() {
    println!("Testing lookahead parsing...");

    // Test simple patterns first
    let simple_patterns = vec!["a", "b", "ab"];

    for pattern in simple_patterns {
        print!("Pattern '{}': ", pattern);
        match rexile::Pattern::new(pattern) {
            Ok(_) => println!("✓ compiled"),
            Err(e) => println!("✗ error: {:?}", e),
        }
    }

    println!("\nTesting lookahead:");
    print!("Pattern '(?=b)': ");
    match rexile::Pattern::new("(?=b)") {
        Ok(_) => println!("✓ compiled"),
        Err(e) => println!("✗ error: {:?}", e),
    }

    println!("\nTesting combined:");
    print!("Pattern 'a(?=b)': ");
    match rexile::Pattern::new("a(?=b)") {
        Ok(_) => println!("✓ compiled"),
        Err(e) => println!("✗ error: {:?}", e),
    }
}
