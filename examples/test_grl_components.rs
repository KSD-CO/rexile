fn main() {
    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;
    println!("Text: {}\n", text);

    // Test each component
    test("Component 1: rule\\s+", r"rule\s+", text);
    test("Component 2a: quoted name", r#"rule\s+"([^"]+)""#, text);
    test("Component 2b: with non-capturing", r#"rule\s+(?:"([^"]+)")"#, text);
    test("Component 2c: full alternation", r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#, text);
    test("Component 3: with attributes", r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)"#, text);
    test("Component 4: with open brace", r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{"#, text);
    test("Component 5: with body", r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)"#, text);
    test("Full pattern", r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)\}"#, text);

    println!("\n=== Testing simpler alternation patterns ===\n");
    let text2 = r#""test""#;
    test("Simple quoted", r#""test""#, text2);
    test("Quoted with capture", r#""([^"]+)""#, text2);
    test("Non-capturing wrapper", r#"(?:"test")"#, text2);
    test("Non-capturing with capture", r#"(?:"([^"]+)")"#, text2);
}

fn test(label: &str, pattern: &str, text: &str) {
    print!("{}: ", label);
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            let is_match = pat.is_match(text);
            let find = pat.find(text);
            if is_match {
                println!("✓ MATCH - find={:?}", find);
            } else {
                println!("✗ NO MATCH");
            }
        }
        Err(e) => println!("✗ ERROR: {}", e),
    }
}
