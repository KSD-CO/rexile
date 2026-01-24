fn main() {
    let pattern = rexile::Pattern::new(".*test.*").unwrap();

    // Simpler: just "test"
    let text = "test";
    eprintln!("Testing '{}' against '{}'", ".*test.*", text);
    let result = pattern.is_match(text);
    eprintln!("Result: {}\n", result);

    // With prefix
    let text2 = "xtest";
    eprintln!("Testing '{}' against '{}'", ".*test.*", text2);
    let result2 = pattern.is_match(text2);
    eprintln!("Result: {}\n", result2);
}
