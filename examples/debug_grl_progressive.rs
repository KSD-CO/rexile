fn main() {
    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;
    println!("Text: {}\n", text);

    // Build the pattern progressively
    let patterns = vec![
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#, "Just rule + name"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*"#, "+ optional whitespace"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))salience"#, "+ literal salience"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*)) salience"#, "+ space + salience"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s+salience"#, "+ \\s+ salience"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*salience"#, "+ \\s* salience"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*salience 10"#, "+ ' 10'"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*salience 10 "#, "+ trailing space"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([a-z ]+)\{"#, "+ charclass + {"),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]+)\{"#, "+ negated charclass + {"),
       (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{"#, "+ [^{]* + {"),
    ];

    for (pattern, desc) in patterns {
        print!("{:<60} => ", desc);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                if let Some((s, e)) = pat.find(text) {
                    println!("✓ matched {} chars", e - s);
                } else {
                    println!("✗ NO MATCH");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
