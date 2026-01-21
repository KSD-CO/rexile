fn main() {
    let pat = "\\d+";
    let text = "ids: 10, 20, 300";

    println!("pattern: {} | text: {}", pat, text);
    let r = rexile::get_regex(pat).unwrap();
    let matches: Vec<_> = r.find_iter(text).map(|m| (m.start(), m.end(), m.as_str())).collect();
    println!("matches: {:?}", matches);

    // Using rexile helpers for repeated is_match checks
    println!("is_match 10: {}", rexile::is_match(pat, "10").unwrap());
}
