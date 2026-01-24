fn main() {
    let pattern = rexile::Pattern::new(".*test.*").unwrap();

    let tests = vec!["test", "testing", "this is a test", "test here", "thistest"];

    for text in tests {
        let result = pattern.is_match(text);
        println!("'.*test.*' matches '{}': {}", text, result);
    }
}
