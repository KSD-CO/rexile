fn main() {
    let tests = vec![
        ("test", false),
        ("xtest", false),
        ("testing", true),
        ("xtesting", true),
    ];

    let pattern = rexile::Pattern::new(".*test.*").unwrap();

    for (text, expected) in tests {
        let result = pattern.is_match(text);
        let status = if result == expected { "✅" } else { "❌" };
        println!("{} '{}': {} (expected {})", status, text, result, expected);
    }
}
