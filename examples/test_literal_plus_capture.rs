fn main() {
    println!("=== Testing literal + capture combinations ===\n");

    // Test 1: Just the capturing group
    let p1 = r#"([^"]+)"#;
    let t1 = "CheckAge";
    test_pattern(p1, t1);

    // Test 2: Literal quote + capturing group + literal quote
    let p2 = r#""([^"]+)""#;
    let t2 = r#""CheckAge""#;
    test_pattern(p2, t2);

    // Test 3: Non-capturing group wrapping #2
    let p3 = r#"(?:"([^"]+)")"#;
    let t3 = r#""CheckAge""#;
    test_pattern(p3, t3);

    // Test 4: Alternation without captures
    let p4 = r#"(?:"foo"|bar)"#;
    let t4a = r#""foo""#;
    let t4b = "bar";
    println!("Pattern: {}", p4);
    println!("Text: {}", t4a);
    match rexile::Pattern::new(p4) {
        Ok(pat) => println!("Match: {}\n", pat.is_match(t4a)),
        Err(e) => println!("Error: {}\n", e),
    }
    println!("Text: {}", t4b);
    match rexile::Pattern::new(p4) {
        Ok(pat) => println!("Match: {}\n", pat.is_match(t4b)),
        Err(e) => println!("Error: {}\n", e),
    }
}

fn test_pattern(pattern: &str, text: &str) {
    println!("Pattern: {}", pattern);
    println!("Text: {}", text);
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            println!("Match: {}", pat.is_match(text));
            println!("Find: {:?}", pat.find(text));
            if let Some(caps) = pat.captures(text) {
                println!("Group 0: {:?}", caps.get(0));
                println!("Group 1: {:?}", caps.get(1));
            }
            println!();
        }
        Err(e) => println!("Error: {}\n", e),
    }
}
