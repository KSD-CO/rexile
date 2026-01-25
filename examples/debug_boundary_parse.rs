use rexile::Pattern;

fn main() {
    let pattern = r"\bhello";
    println!("Testing pattern: {}", pattern);

    match Pattern::new(pattern) {
        Ok(p) => {
            println!("Pattern compiled successfully");
            let text = "hello world";
            if p.is_match(text) {
                println!("✓ MATCH in '{}'", text);
                if let Some(m) = p.find(text) {
                    println!("  Match: {:?}", m);
                }
            } else {
                println!("✗ NO MATCH in '{}'", text);
            }

            // Try different texts
            let tests = vec!["hello", " hello", "xhello", "hello world"];

            for t in tests {
                if let Some(m) = p.find(t) {
                    println!("  Found in '{}': {:?}", t, m);
                } else {
                    println!("  No match in '{}'", t);
                }
            }
        }
        Err(e) => {
            println!("✗ Pattern failed to compile: {:?}", e);
        }
    }

    // Test other boundary patterns
    println!("\n--- Testing other boundary patterns ---");
    let patterns = vec![r"\b", r"hello\b", r"\bworld"];
    for pat in patterns {
        match Pattern::new(pat) {
            Ok(p) => {
                if p.is_match("hello world") {
                    println!("Pattern '{}' matches 'hello world'", pat);
                } else {
                    println!("Pattern '{}' does not match 'hello world'", pat);
                }
            }
            Err(e) => println!("Pattern '{}' failed: {:?}", pat, e),
        }
    }
}
