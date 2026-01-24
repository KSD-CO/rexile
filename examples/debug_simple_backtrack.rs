fn main() {
    // Simplest case
    let pattern = "a(.+)b";
    let text = "axxxb";

    println!("Pattern: {}", pattern);
    println!("Text: {}\n", text);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("Compiled structure:");
            println!("{:#?}\n", pat);

            println!("Testing is_match: {}", pat.is_match(text));
            println!("Testing find: {:?}", pat.find(text));
        }
        Err(e) => println!("ERROR: {}", e),
    }
}
