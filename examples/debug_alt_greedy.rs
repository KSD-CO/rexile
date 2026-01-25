fn main() {
    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;
    println!("Text: {}\n", text);

    let tests = vec![
        // Test alternation matching
        (r#""([^"]+)""#, "Quoted"),
        (r#"([a-zA-Z_]\w*)"#, "Identifier"),
        (r#"(?:"([^"]+)"|([a-zA-Z_]\w*))"#, "Alternation"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#, "rule + alt"),
        (r#"rule (?:"([^"]+)"|([a-zA-Z_]\w*))"#, "rule + alt (fixed space)"),

        // Test what actually gets matched
        (r#"(?:"([^"]+)"|([a-zA-Z_]\w*)) salience"#, "alt + ' salience'"),
    ];

    for (pattern, desc) in tests {
        print!("{:<40} => ", desc);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                if let Some((s, e)) = pat.find(text) {
                    let matched = &text[s..e];
                    println!("✓ [{}..{}] = {:?}", s, e, matched);
                } else {
                    println!("✗ NO MATCH");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
