fn main() {
    println!("=== Testing regex flags ===\n");

    let tests = vec![
        // DOTALL flag - . should match newlines
        ("(?s)a.b", "a\nb", "DOTALL: . matches newline"),
        ("a.b", "a\nb", "No flag: . doesn't match newline"),
        // Case insensitive
        ("(?i)hello", "HELLO", "Case insensitive flag"),
        ("hello", "HELLO", "No flag: case sensitive"),
        // Multiline - ^ and $ match line boundaries
        (
            "(?m)^hello",
            "world\nhello",
            "Multiline: ^ matches line start",
        ),
        ("^hello", "world\nhello", "No flag: ^ matches string start"),
    ];

    for (pattern, text, desc) in tests {
        print!("{:<50} => ", desc);
        match rexile::Pattern::new(pattern) {
            Ok(pat) => {
                if pat.is_match(text) {
                    println!("✓ MATCH");
                } else {
                    println!("✗ NO MATCH");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
