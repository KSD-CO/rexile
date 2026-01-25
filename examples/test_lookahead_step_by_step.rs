fn main() {
    println!("Step 1: Testing simple pattern");
    match rexile::Pattern::new("a") {
        Ok(_) => println!("✓ 'a' compiled"),
        Err(e) => println!("✗ 'a' failed: {:?}", e),
    }

    println!("\nStep 2: Testing lookahead inner");
    match rexile::Pattern::new("(?=b)") {
        Ok(_) => println!("✓ '(?=b)' compiled"),
        Err(e) => println!("✗ '(?=b)' failed: {:?}", e),
    }

    println!("\nStep 3: Testing combined - THIS WILL CRASH");
    println!("About to test 'a(?=b)'...");
    match rexile::Pattern::new("a(?=b)") {
        Ok(_) => println!("✓ 'a(?=b)' compiled"),
        Err(e) => println!("✗ 'a(?=b)' failed: {:?}", e),
    }

    println!("Done!");
}
