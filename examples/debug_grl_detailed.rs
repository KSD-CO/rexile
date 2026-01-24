fn main() {
    println!("=== Debugging GRL pattern step by step ===\n");

    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;
    println!("Text: {}\n", text);

    let patterns = vec![
        r"rule",
        r"rule\s+",
        r#"rule\s+""#,
        r#"rule\s+"([^"]+)""#,
        r#"rule\s+(?:"([^"]+)"|([a-z]+))"#,
        r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#,
        r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*"#,
        r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)"#,
        r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{"#,
        r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)"#,
        r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)\}"#,
    ];

    for pattern in patterns {
        print!("{:<70} => ", pattern);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                if let Some((s, e)) = pat.find(text) {
                    println!("✓ [{}, {}] = {:?}", s, e, &text[s..e]);
                } else {
                    println!("✗ NO MATCH");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
