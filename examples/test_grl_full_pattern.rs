fn main() {
    println!("=== Testing full GRL pattern ===\n");

    // The actual GRL pattern from rust-rule-engine
    let pattern = r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)\}"#;
    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;

    println!("Pattern: {}", pattern);
    println!("Text: {}\n", text);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("✓ Pattern compiled successfully");
            println!("Is match: {}", pat.is_match(text));
            println!("Find: {:?}", pat.find(text));

            if let Some(caps) = pat.captures(text) {
                println!("\n=== Captures ===");
                println!("Group 0 (full match): {:?}", caps.get(0));
                println!("Group 1 (quoted name): {:?}", caps.get(1));
                println!("Group 2 (unquoted name): {:?}", caps.get(2));
                println!("Group 3 (attributes): {:?}", caps.get(3));
                println!("Group 4 (body): {:?}", caps.get(4));
            } else {
                println!("\n✗ No captures found");
            }
        }
        Err(e) => {
            println!("✗ Pattern compilation error: {}", e);
        }
    }

    // Also test with unquoted name
    println!("\n\n=== Testing with unquoted name ===\n");
    let text2 = r#"rule CheckAge salience 10 { when User.Age >= 18 then log("User is adult"); }"#;
    println!("Text: {}\n", text2);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("Is match: {}", pat.is_match(text2));
            if let Some(caps) = pat.captures(text2) {
                println!("\n=== Captures ===");
                println!("Group 0 (full match): {:?}", caps.get(0));
                println!("Group 1 (quoted name): {:?}", caps.get(1));
                println!("Group 2 (unquoted name): {:?}", caps.get(2));
                println!("Group 3 (attributes): {:?}", caps.get(3));
                println!("Group 4 (body): {:?}", caps.get(4));
            }
        }
        Err(e) => {
            println!("✗ Error: {}", e);
        }
    }
}
