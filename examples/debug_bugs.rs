fn main() {
    println!("=== Bug 1: Backtracking with capture + literal ===\n");
    
    // Simple case
    test("Simple: \\{a\\}", "{ a }");
    test("Capture: \\{(a)\\}", "{ a }");
    test("Plus: \\{a+\\}", "{ aaa }");
    test("Capture+Plus: \\{(a+)\\}", "{ aaa }");
    test("Dot: \\{.\\}", "{ x }");
    test("Capture+Dot: \\{(.)\\}", "{ x }");
    test("Dot+: \\{.+\\}", "{ abc }");
    test("Capture+Dot+: \\{(.+)\\}", "{ abc }");
    
    println!("\n=== Bug 2: Alternation patterns ===\n");
    
    // Without following literal
    test("Alt simple: (?:a|b)", "a");
    test("Alt simple: (?:a|b)", "b");
    test("Alt+cap: (?:(a)|(b))", "a");
    test("Alt+cap: (?:(a)|(b))", "b");
    
    // With following literal
    test("Alt+lit: (?:a|b)c", "ac");
    test("Alt+lit: (?:a|b)c", "bc");
    test("Alt+cap+lit: (?:(a)|(b))c", "ac");
    test("Alt+cap+lit: (?:(a)|(b))c", "bc");
    
    // More complex
    test("Quoted alt: (?:\"x\"|y)z", "\"x\"z");
    test("Quoted alt: (?:\"x\"|y)z", "yz");
}

fn test(label: &str, text: &str) {
    let parts: Vec<&str> = label.split(": ").collect();
    let pattern = parts[1];
    print!("  {} = ", label);
    match rexile::Pattern::new(pattern) {
        Ok(pat) => {
            if pat.is_match(text) {
                print!("✓");
                if let Some((s, e)) = pat.find(text) {
                    print!(" [{}, {}]", s, e);
                }
                println!();
            } else {
                println!("✗ NO MATCH");
            }
        }
        Err(e) => println!("✗ ERROR: {}", e),
    }
}
