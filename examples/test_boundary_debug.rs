fn main() {
    let patterns = vec![r"\bhello", r"hello\b", r"\bworld"];

    for pattern in patterns {
        println!("\nPattern: {}", pattern);
        match rexile::Pattern::new(pattern) {
            Ok(p) => {
                println!("  Compiled OK");
                let text = "hello world";
                if let Some((s, e)) = p.find(text) {
                    println!("  Found: [{}, {}] = {:?}", s, e, &text[s..e]);
                } else {
                    println!("  No match in: {}", text);
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
    }
}
