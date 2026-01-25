use rexile::Pattern;

fn main() {
    let tests = vec![
        (r"[^a]", "bac", Some((0, 1)), "Simple negated - should match 'b'"),
        (r"[^\s]", " a", Some((1, 2)), "Negated whitespace - should match 'a' at pos 1"),
        (r"[^\s]", "  a", Some((2, 3)), "Negated whitespace - should match 'a' at pos 2"),
        (r"[^\s]+", " hello", Some((1, 6)), "Negated whitespace+ - should match 'hello'"),
        (r"[^abc]", "abcd", Some((3, 4)), "Should match 'd' at pos 3"),
    ];

    for (pattern_str, text, expected, desc) in tests {
        print!("{:55} ", desc);

        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let result = pattern.find(text);
                if result == expected {
                    println!("✓ {:?}", result);
                } else {
                    println!("✗ got {:?}, expected {:?}", result, expected);
                    if let Some((start, end)) = result {
                        println!("       Matched: {:?} at pos {}", &text[start..end], start);
                    }
                }
            }
            Err(e) => {
                println!("✗ compile error: {:?}", e);
            }
        }
    }
}
