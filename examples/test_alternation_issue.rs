use rexile::Pattern;

fn main() {
    println!("=== Testing alternation with optional groups ===\n");

    // Test simpler patterns to isolate the issue
    let patterns = vec![
        (r#"rule\s+"([^"]+)"\s*\{"#, r#"rule "Test" {"#),
        (r#"rule\s+([a-zA-Z_]\w*)\s*\{"#, r#"rule Test {"#),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*\{"#, r#"rule "Test" {"#),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*\{"#, r#"rule Test {"#),
        // With optional [^{]* group
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{"#, r#"rule "Test" {"#),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{"#, r#"rule Test {"#),
        (r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{"#, r#"rule Test salience 10 {"#),
    ];

    for (pattern_str, input) in &patterns {
        println!("Pattern: {}", pattern_str);
        println!("Input:   {}", input);
        
        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let is_match = pattern.is_match(input);
                let find = pattern.find(input);
                println!("  is_match: {}", is_match);
                println!("  find:     {:?}", find);
                
                if let Some(caps) = pattern.captures(input) {
                    println!("  captures: Some");
                    for i in 0..5 {
                        if let Some(c) = caps.get(i) {
                            println!("    [{}]: {:?}", i, c);
                        }
                    }
                } else {
                    println!("  captures: None");
                }
            }
            Err(e) => println!("  Error: {:?}", e),
        }
        println!();
    }
}
