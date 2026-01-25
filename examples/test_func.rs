use rexile::Pattern;

fn main() {
    let p = Pattern::new(r#"(\w+)\s*\(\s*(.+?)?\s*\)"#).unwrap();
    
    let tests = [
        "apply_discount(20000)",
        "  apply_discount(20000)",
        "apply_discount(20000);",
    ];
    
    for t in &tests {
        println!("Testing: {t:?}");
        if let Some(caps) = p.captures(t) {
            println!("  Matched!");
            for (i, cap) in caps.iter().enumerate() {
                println!("    cap[{i}] = {:?}", cap);
            }
        } else {
            println!("  No match");
        }
    }
}
