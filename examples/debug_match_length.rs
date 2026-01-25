fn main() {
    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;
    println!("Text: {}\nLength: {}\n", text, text.len());

    // Mark positions
    for i in (0..text.len()).step_by(10) {
        println!("{:3}: {:?}", i, &text[i..i.min(text.len()).min(i+10)]);
    }
    println!();

    let pattern = r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#;
    println!("Pattern: {}\n", pattern);

    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            if let Some((s, e)) = pat.find(text) {
                println!("Matched: [{}..{}]", s, e);
                println!("Length: {}", e - s);
                println!("Content: {:?}", &text[s..e]);

                // Check what the alternation branches would match individually
                println!("\nChecking individual branches:");

                let quoted = rexile::Pattern::new(r#""([^"]+)""#).unwrap();
                if let Some((s2, e2)) = quoted.find(&text[5..]) {
                    println!("  Quoted branch would match: [{}..{}] = {:?}",
                             s2, e2, &text[5..][s2..e2]);
                }

                let ident = rexile::Pattern::new(r"[a-zA-Z_]\w*").unwrap();
                if let Some((s2, e2)) = ident.find(&text[5..]) {
                    println!("  Identifier branch would match: [{}..{}] = {:?}",
                             s2, e2, &text[5..][s2..e2]);
                }
            } else {
                println!("NO MATCH");
            }
        }
        Err(e) => println!("ERROR: {}", e),
    }
}
