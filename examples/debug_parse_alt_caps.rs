fn main() {
    let pattern_str = "(?:(a)|(b))";
    println!("Pattern: {:?}\n", pattern_str);

    match rexile::Pattern::new(pattern_str) {
        Ok(pat) => {
            println!("✓ Compiled successfully");
            println!("Debug format: {:#?}\n", pat);
        }
        Err(e) => {
            println!("✗ Compilation error: {}", e);
        }
    }
}
