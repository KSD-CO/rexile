fn main() {
    println!("=== Testing the problematic component ===\n");

    // The component that starts failing
    let pattern = r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{"#;
    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;

    println!("Pattern: {}", pattern);
    println!("Text: {}\n", text);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("is_match: {}", pat.is_match(text));
            println!("find: {:?}", pat.find(text));
            println!(
                "captures: {:?}\n",
                pat.captures(text)
                    .map(|c| (c.get(0), c.get(1), c.get(2), c.get(3)))
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test without the { at the end
    println!("=== Without the opening brace ===\n");
    let pattern2 = r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)"#;
    println!("Pattern: {}", pattern2);

    match rexile::Pattern::new(pattern2) {
        Ok(pat) => {
            println!("is_match: {}", pat.is_match(text));
            println!("find: {:?}", pat.find(text));
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test just the brace matching part
    println!("\n=== Test escaped brace ===\n");
    let text3 = "rule test {";
    test_pattern(r"rule\s+\w+\s*\{", text3);
    test_pattern(r#"rule\s+\w+\s*\{"#, text3);
}

fn test_pattern(pattern: &str, text: &str) {
    println!("Pattern: {}", pattern);
    println!("Text: {}", text);
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("is_match: {}", pat.is_match(text));
            println!("find: {:?}\n", pat.find(text));
        }
        Err(e) => println!("Error: {}\n", e),
    }
}
