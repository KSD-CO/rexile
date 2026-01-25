fn main() {
    let pattern = "(?i)hello";
    let text = "HELLO";

    println!("Pattern: {:?}", pattern);
    println!("Text: {:?}", text);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("Pattern compiled successfully");
            println!("Is match: {}", pat.is_match(text));
            if let Some((s, e)) = pat.find(text) {
                println!("Found at [{}, {}]: {:?}", s, e, &text[s..e]);
            } else {
                println!("No match found");
            }
        }
        Err(e) => println!("ERROR: {}", e),
    }

    // Test lowercase
    println!("\nTest with lowercase:");
    let text2 = "hello";
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("Is match: {}", pat.is_match(text2));
            if let Some((s, e)) = pat.find(text2) {
                println!("Found at [{}, {}]: {:?}", s, e, &text2[s..e]);
            } else {
                println!("No match found");
            }
        }
        Err(e) => println!("ERROR: {}", e),
    }
}
