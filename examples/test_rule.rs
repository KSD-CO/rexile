use rexile::Pattern;

fn main() {
    // Full original pattern
    let full = Pattern::new(r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)\}"#).unwrap();
    
    let tests = [
        r#"rule "Complex Business Rule" salience 10 { when x then y }"#,
        r#"rule MyRule { when x then y }"#,
        r#"rule MyRule salience 5 { when x then y }"#,
    ];
    
    for t in &tests {
        println!("Testing: {t:?}");
        if let Some(caps) = full.captures(t) {
            println!("  Matched!");
            for (i, cap) in caps.iter().enumerate() {
                println!("    cap[{i}] = {:?}", cap);
            }
        } else {
            println!("  No match");
        }
    }
}
