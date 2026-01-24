fn main() {
    let pattern_str = r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)\}"#;
    let text = r#"rule "CheckAge" salience 10 { when User.Age >= 18 then log("User is adult"); }"#;

    println!("Pattern: {}", pattern_str);
    println!("Text: {}", text);

    match rexile::Pattern::new(pattern_str) {
        Ok(p) => {
            println!("Pattern compiled successfully");
            println!("Is match: {}", p.is_match(text));
            println!("Find: {:?}", p.find(text));
        }
        Err(e) => {
            println!("Pattern compilation error: {}", e);
        }
    }

    // Try simpler patterns
    println!("\n=== Testing simpler patterns ===");

    let simple1 = r#"rule\s+"[^"]+""#;
    match rexile::Pattern::new(simple1) {
        Ok(p) => println!("Pattern '{}': match={}", simple1, p.is_match(text)),
        Err(e) => println!("Pattern '{}' error: {}", simple1, e),
    }

    let simple2 = r#"rule\s+(?:"test"|foo)"#;
    match rexile::Pattern::new(simple2) {
        Ok(p) => println!("Pattern '{}': match={}", simple2, p.is_match(text)),
        Err(e) => println!("Pattern '{}' error: {}", simple2, e),
    }

    let simple3 = r#"(?:"([^"]+)"|([a-zA-Z_]\w*))"#;
    let text3 = r#""CheckAge""#;
    match rexile::Pattern::new(simple3) {
        Ok(p) => {
            println!("Pattern '{}': match={}, find={:?}", simple3, p.is_match(text3), p.find(text3));
        }
        Err(e) => println!("Pattern '{}' error: {}", simple3, e),
    }
}
