fn main() {
    // Using inline flags and Regex options via patterns
    let patterns = vec![
        ("(?i)rust", "Rust"), // case-insensitive
        ("(?m)^begin", "begin\nnext"), // multiline ^ matches line start
        ("(?s)a.*b", "a\n\nb"), // dot matches newline
    ];

    for (pat, txt) in patterns {
        println!("Pattern: {} | Text: {}", pat, txt);
        println!("is_match: {:?}", rexile::is_match(pat, txt));
        println!("find: {:?}", rexile::find(pat, txt));
        println!("---");
    }
}
