fn main() {
    let patterns = vec![
        ("[A-Za-z0-9]+", "abc123"),
        ("\\d+", "42"),
        ("\\w+", "word_123"),
        ("[[:alpha:]]+", "alpha"),
    ];

    for (pat, txt) in patterns {
        println!("Pattern: {} | Text: {}", pat, txt);
        println!("is_match: {:?}", rexile::is_match(pat, txt));
        println!("find: {:?}", rexile::find(pat, txt));
        println!("---");
    }
}
