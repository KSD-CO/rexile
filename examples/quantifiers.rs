fn main() {
    let patterns = vec![
        ("a.*b", "axxxb"),
        ("a.*?b", "axxxb"),
        ("(ab){2,}", "ababab"),
    ];

    for (pat, txt) in patterns {
        println!("Pattern: {} | Text: {}", pat, txt);
        println!("is_match: {:?}", rexile::is_match(pat, txt));
        println!("find: {:?}", rexile::find(pat, txt));
        println!("---");
    }
}
