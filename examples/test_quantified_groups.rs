fn main() {
    let tests = vec![
        // Zero-width matches
        (r"a(b)*c", "ac", "Capturing group * with 0 matches"),
        (r"a(b)*c", "abc", "Capturing group * with 1 match"),
        (r"a(b)*c", "abbc", "Capturing group * with 2 matches"),
        
        (r"a(?:b)*c", "ac", "Non-capturing group * with 0 matches"),
        (r"a(?:b)*c", "abc", "Non-capturing group * with 1 match"),
        (r"a(?:b)*c", "abbc", "Non-capturing group * with 2 matches"),
        
        // With + quantifier
        (r"a(b)+c", "abc", "Capturing group + with 1 match"),
        (r"a(b)+c", "abbc", "Capturing group + with 2 matches"),
        (r"a(?:b)+c", "abc", "Non-capturing group + with 1 match"),
        
        // More complex
        (r"x(?:ab)*y", "xy", "Non-capturing multi-char * with 0"),
        (r"x(?:ab)*y", "xaby", "Non-capturing multi-char * with 1"),
        (r"x(?:ab)*y", "xababy", "Non-capturing multi-char * with 2"),
    ];
    
    for (pattern, text, desc) in tests {
        print!("{:<50} => ", desc);
        match rexile::Pattern::new(pattern) {
            Ok(p) => {
                if p.is_match(text) {
                    println!("✓ OK");
                } else {
                    println!("✗ FAIL");
                }
            }
            Err(e) => println!("✗ ERROR: {}", e),
        }
    }
}
