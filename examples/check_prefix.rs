use rexile::Pattern;

fn main() {
    let patterns = vec![r#""[^"]+""#, r#"rule\s+"[^"]+""#, r"\d+"];

    for pat_str in patterns {
        let pat = Pattern::new(pat_str).unwrap();
        println!("Pattern: {}", pat_str);
        // Can't access internal sequence, so just run it
        let test = r#"test "hello" world"#;
        if let Some(m) = pat.find(test) {
            println!("  Match found: {:?}", &test[m.0..m.1]);
        }
        println!();
    }
}
