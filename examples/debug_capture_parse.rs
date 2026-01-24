fn main() {
    let pattern = "a(.+)b";
    println!("Pattern: {}\n", pattern);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("Compiled successfully:");
            println!("{:#?}", pat);
        }
        Err(e) => println!("ERROR: {}", e),
    }
}
