fn main() {
    println!("Testing that lookahead returns error...");

    match rexile::Pattern::new("(?=b)") {
        Ok(_) => {
            println!("✗ UNEXPECTED: Pattern compiled successfully!");
            println!("The code change didn't take effect!");
        }
        Err(e) => {
            println!("✓ Got expected error: {}", e);
        }
    }
}
