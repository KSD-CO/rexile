fn main() {
    let pattern = r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#;
    println!("Pattern: {}\n", pattern);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("Compiled structure:");
            println!("{:#?}\n", pat);
        }
        Err(e) => println!("ERROR: {}", e),
    }
}
