use rexile::Pattern;

fn main() {
    let tests = vec![
        (r"\bhello\b", "hello", true),
        (r"\bhello\b", "hello world", true),
        (r"\bhello\b", "xhello", false),
        (r"\bhello\b", "hellox", false),
        (r"\bhello\b", "xhellox", false),
    ];

    for (pattern_str, text, should_match) in tests {
        print!("{:20} in {:15} ", format!("'{}'", pattern_str), format!("'{}'", text));

        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let did_match = pattern.is_match(text);
                if did_match == should_match {
                    println!("✓ {}", if did_match { "match" } else { "no match" });
                } else {
                    println!("✗ expected {}, got {}",
                        if should_match { "match" } else { "no match" },
                        if did_match { "match" } else { "no match" });
                    if let Some((start, end)) = pattern.find(text) {
                        println!("      Found: [{}, {}) = {:?}", start, end, &text[start..end]);
                    }
                }
            }
            Err(e) => {
                println!("✗ compile error: {:?}", e);
            }
        }
    }
}
