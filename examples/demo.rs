fn main() {
    // Demonstrate core ReXile helpers and parity with regex::Regex
    let examples = vec![
        ("^rule\\s+\\d+", "rule 123"),
        ("(?P<n>user)\\s*=\\s*(?P<i>\\d+)", "data user =42 other"),
        ("gr(a|e)y|挨个", "gray"),
        ("gr(a|e)y|挨个", "挨个"),
    ];

    for (pat, txt) in examples {
        println!("Pattern: {}\nText: {}", pat, txt);

        match rexile::is_match(pat, txt) {
            Ok(b) => println!("rexile::is_match => {}", b),
            Err(e) => println!("rexile error: {}", e),
        }

        match rexile::get_regex(pat) {
            Ok(r) => println!("rexile::get_regex compiled: {}", r.as_str()),
            Err(e) => println!("rexile compile error: {}", e),
        }

        match rexile::find(pat, txt) {
            Ok(Some((s, e))) => println!("rexile::find => offsets {}..{} => '{}'", s, e, &txt[s..e]),
            Ok(None) => println!("rexile::find => no match"),
            Err(e) => println!("rexile find error: {}", e),
        }

        match rexile::get_regex_static(pat) {
            Ok(r) => println!("rexile::get_regex_static OK: {}", r.as_str()),
            Err(e) => println!("rexile static error: {}", e),
        }

        println!("---");
    }
}
