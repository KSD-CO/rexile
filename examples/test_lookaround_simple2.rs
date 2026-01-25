fn main() {
    println!("Testing simple lookaround compile...");

    match rexile::Pattern::new("(?=a)") {
        Ok(pat) => {
            println!("✓ Pattern compiled successfully");
            println!("Testing match on 'abc'...");
            match pat.find("abc") {
                Some((start, end)) => println!("  Match: ({}, {})", start, end),
                None => println!("  No match"),
            }
        }
        Err(e) => println!("✗ Error: {:?}", e),
    }
}
