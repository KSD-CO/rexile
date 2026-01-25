use rexile::Pattern;

fn main() {
    let tests = vec![
        (
            r"^hello",
            "hello world",
            Some((0, 5)),
            "Should match at start",
        ),
        (r"^hello", "say hello", None, "Should NOT match mid-text"),
        (r"^h", "hello", Some((0, 1)), "Single char at start"),
        (r"^h", "oh", None, "Single char not at start"),
        (r"^$", "", Some((0, 0)), "Empty string"),
        (
            r"^abc$",
            "abc",
            Some((0, 3)),
            "Exact match with both anchors",
        ),
    ];

    for (pattern_str, text, expected, desc) in tests {
        print!("{:50} ", desc);

        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let result = pattern.find(text);
                if result == expected {
                    println!("✓ {:?}", result);
                } else {
                    println!("✗ got {:?}, expected {:?}", result, expected);
                }
            }
            Err(e) => {
                println!("✗ compile error: {:?}", e);
            }
        }
    }
}
