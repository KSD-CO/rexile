fn main() {
    let text = r#"rule "CheckAge" salience 10"#;
    println!("Text: {}\n", text);

    let tests = vec![
        // Build up the pattern step by step
        (r#""([^"]+)""#, "Match quoted"),
        (r#"([a-zA-Z_]\w*)"#, "Match identifier"),
        (r#"(?:"([^"]+)"|([a-zA-Z_]\w*))"#, "Alternation"),
        (r#"rule (?:"([^"]+)"|([a-zA-Z_]\w*))"#, "rule + alt (fixed space)"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#, "rule\\s+ + alt"),

        // Test what it should match
        (r#"rule "CheckAge""#, "Literal match"),
    ];

    for (pattern, desc) in tests {
        print!("{:<50} => ", desc);
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
