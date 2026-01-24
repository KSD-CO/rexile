fn main() {
    let pattern = rexile::Pattern::new(".*test.*").unwrap();
    println!("Pattern structure:\n{:#?}\n", pattern);

    // Simple test
    println!("'.*test.*' matches 'test': {}", pattern.is_match("test"));
    println!(
        "'.*test.*' matches 'testing': {}",
        pattern.is_match("testing")
    );
    println!("'.*test.*' matches 'xtest': {}", pattern.is_match("xtest"));
    println!(
        "'.*test.*' matches 'xtestx': {}",
        pattern.is_match("xtestx")
    );
}
