fn main() {
    let patterns = vec![
        ("\\p{L}+", "áéíöß"),
        ("\\p{N}+", "12345"),
        ("\\p{Letter}+", "Hello"),
    ];

    for (pat, txt) in patterns {
        println!("Pattern: {} | Text: {}", pat, txt);
        println!("is_match: {:?}", rexile::is_match(pat, txt));
        println!("find: {:?}", rexile::find(pat, txt));
        println!("---");
    }
}
