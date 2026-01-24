fn main() {
    let text = "rule \"test\" {\n  content\n}";
    println!("Text: {:?}", text);

    println!("\n=== Component tests ===");

    // 1. rule literal
    let p = rexile::Pattern::new("rule").unwrap();
    println!("  'rule': {}", p.is_match(text));

    // 2. rule\s+
    let p = rexile::Pattern::new(r"rule\s+").unwrap();
    println!("  'rule\\s+': {}", p.is_match(text));

    // 3. rule\s+"[^"]+"
    let p = rexile::Pattern::new(r#"rule\s+"[^"]+""#).unwrap();
    println!("  'rule\\s+\"[^\"]+\"': {}", p.is_match(text));

    // 4. "test"
    let p = rexile::Pattern::new(r#""[^"]+""#).unwrap();
    println!("  '\"[^\"]+\"': {:?}", p.find(text));

    // 5. .*?\}  (non-greedy to closing brace)
    match rexile::Pattern::new(r".*?\}") {
        Ok(p) => println!("  '.*?\\}}': {}", p.is_match(text)),
        Err(e) => println!("  '.*?\\}}' ERROR: {}", e),
    }

    // 6. (?s).*?\}
    match rexile::Pattern::new(r"(?s).*?\}") {
        Ok(p) => println!("  '(?s).*?\\}}': {}", p.is_match(text)),
        Err(e) => println!("  '(?s).*?\\}}' ERROR: {}", e),
    }

    // 7. Non-capturing group
    match rexile::Pattern::new(r#"(?:"test"|foo)"#) {
        Ok(p) => println!("  '(?:\"test\"|foo)': {}", p.is_match(text)),
        Err(e) => println!("  non-capturing group ERROR: {}", e),
    }

    // 8. rule\s+"[^"]+".*\}  (greedy dot, single line - should FAIL on multi-line text)
    match rexile::Pattern::new(r#"rule\s+"[^"]+".*\}"#) {
        Ok(p) => println!(
            "  'rule\\s+\"[^\"]+\".*\\}}' (greedy, no dotall): {}",
            p.is_match(text)
        ),
        Err(e) => println!("  combined ERROR: {}", e),
    }

    // 9. (?s)rule\s+"[^"]+".*\}  (DOTALL mode - should match across newlines)
    match rexile::Pattern::new(r#"(?s)rule\s+"[^"]+".*\}"#) {
        Ok(p) => println!(
            "  '(?s)rule\\s+\"[^\"]+\".*\\}}' (dotall): {}",
            p.is_match(text)
        ),
        Err(e) => println!("  (?s) combined ERROR: {}", e),
    }

    // 10. (?s)rule\s+"[^"]+".*?\}  (DOTALL + non-greedy)
    match rexile::Pattern::new(r#"(?s)rule\s+"[^"]+".*?\}"#) {
        Ok(p) => println!(
            "  '(?s)rule\\s+\"[^\"]+\".*?\\}}' (dotall+lazy): {}",
            p.is_match(text)
        ),
        Err(e) => println!("  (?s) lazy ERROR: {}", e),
    }

    // 11. The full complex pattern with non-capturing group
    let pat = r#"(?s)rule\s+(?:"[^"]+"|[a-zA-Z_]\w*).*?\}"#;
    match rexile::Pattern::new(pat) {
        Ok(p) => println!("  full pattern (quoted name): {}", p.is_match(text)),
        Err(e) => println!("  full pattern ERROR: {}", e),
    }

    // 12. Same but with identifier name
    let text2 = "rule yara_example {\n  content\n}";
    println!("\nText2: {:?}", text2);
    match rexile::Pattern::new(pat) {
        Ok(p) => println!("  full pattern (ident name): {}", p.is_match(text2)),
        Err(e) => println!("  full pattern ERROR: {}", e),
    }

    // 13. Non-greedy .*? basic test
    let text3 = "start{abc}end{xyz}";
    println!("\nText3: {:?}", text3);
    match rexile::Pattern::new(r"start\{.*?\}") {
        Ok(p) => {
            println!("  'start\\{{.*?\\}}': match={}", p.is_match(text3));
            println!("  find: {:?}", p.find(text3));
            if let Some((s, e)) = p.find(text3) {
                println!("  matched: {:?}", &text3[s..e]);
            }
        }
        Err(e) => println!("  ERROR: {}", e),
    }

    // 14. Greedy .* for comparison
    match rexile::Pattern::new(r"start\{.*\}") {
        Ok(p) => {
            println!("  'start\\{{.*\\}}': match={}", p.is_match(text3));
            if let Some((s, e)) = p.find(text3) {
                println!("  matched: {:?}", &text3[s..e]);
            }
        }
        Err(e) => println!("  ERROR: {}", e),
    }
}
