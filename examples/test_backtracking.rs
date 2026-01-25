fn main() {
    println!("=== Testing backtracking scenarios ===\n");

    let tests = vec![
        // Without captures - does backtracking work?
        ("a.+b", "axxxb"),
        ("a.+b", "axb"),
        ("\\{.+\\}", "{ abc }"),
        // With captures - the issue
        ("\\{(.+)\\}", "{ abc }"),
        ("a(.+)b", "axxxb"),
        ("start(.+)end", "start123end"),
        // Character class version (should work)
        ("\\{([^}]+)\\}", "{ abc }"),
        ("a([^b]+)b", "axxxb"),
        // More complex
        ("\"(.+)\"", "\"hello world\""),
        ("\\[(.+)\\]", "[test]"),
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
