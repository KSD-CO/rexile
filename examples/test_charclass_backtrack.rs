fn main() {
    println!("=== Testing character class backtracking ===\n");

    let tests = vec![
        // Character class with * followed by literal
        ("a([^b]*)b", "axxxb"),
        ("\\{([^}]*)\\}", "{ abc }"),
        ("start([^e]*)end", "start123end"),

        // Compare with .+ (should work)
        ("a(.+)b", "axxxb"),
        ("\\{(.+)\\}", "{ abc }"),

        // The problematic GRL component
        ("salience([^{]*)\\{", "salience 10 {"),
        ("([^{]*)\\{", "salience 10 {"),
    ];

    for (pattern, text) in tests {
        print!("{:<30} with {:<20?} => ", pattern, text);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                if let Some((s, e)) = pat.find(text) {
                    let matched = &text[s..e];
                    println!("✓ {:?}", matched);
                } else {
                    println!("✗ NO MATCH");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
