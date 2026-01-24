fn main() {
    println!("Testing pattern parsing...");

    match rexile::Pattern::new("[a-z]+.+[0-9]+") {
        Ok(p) => println!("✅ Pattern compiled successfully"),
        Err(e) => println!("❌ Pattern failed: {}", e),
    }

    println!("Done!");
}
