use rexile::Pattern;

fn main() {
    println!("=== Testing Unicode handling ===\n");

    // Test with emojis
    let inputs = vec![
        "Hello 🎉 World",
        "Price: $100 💰",
        "User: John 👤 Doe",
        "Status: ✅ Complete",
        "Rating: ⭐⭐⭐⭐⭐",
    ];

    let patterns = vec![
        r"\w+",
        r"\d+",
        r"[a-zA-Z]+",
        r"\s+",
    ];

    for pattern_str in &patterns {
        println!("Pattern: {}", pattern_str);
        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                for input in &inputs {
                    print!("  Input: {:30} => ", format!("{:?}", input));
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        pattern.find_all(input)
                    })) {
                        Ok(matches) => println!("OK ({} matches)", matches.len()),
                        Err(_) => println!("PANIC!"),
                    }
                }
            }
            Err(e) => println!("  Pattern Error: {:?}", e),
        }
        println!();
    }
}
