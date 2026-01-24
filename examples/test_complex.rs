fn main() {
    println!("Testing complex pattern matching...");

    let pattern = rexile::Pattern::new("[a-z]+.+[0-9]+").unwrap();

    let test = "abc123";
    println!("Matching '{}' against '{}'...", "[a-z]+.+[0-9]+", test);

    let result = pattern.is_match(test);
    println!("Result: {}", result);

    println!("Done!");
}
