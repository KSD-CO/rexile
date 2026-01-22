use rexile::Pattern;

fn main() {
    // Test if \d{4} parses correctly
    match Pattern::new(r"\d{4}") {
        Ok(p) => {
            println!("✅ Pattern \\d{{4}} parsed successfully");
            println!("{:#?}", p);
        },
        Err(e) => {
            println!("❌ Failed to parse \\d{{4}}: {:?}", e);
        }
    }
}
