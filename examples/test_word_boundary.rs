fn main() {
    let tests = vec![
        (r"\bhello", "hello world", "Word boundary at start"),
        (r"hello\b", "hello world", "Word boundary at end"),
        (r"\bworld", "hello world", "Word boundary mid"),
        (r"\w+\b", "hello world", "Word + boundary"),
    ];
    
    for (pattern, text, desc) in tests {
        print!("{:<30} => ", desc);
        match rexile::Pattern::new(pattern) {
            Ok(p) => {
                if let Some((s, e)) = p.find(text) {
                    println!("✓ [{}, {}] = {:?}", s, e, &text[s..e]);
                } else {
                    println!("✗ NO MATCH");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
