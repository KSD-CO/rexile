use rexile::Pattern;

fn main() {
    let patterns = vec![
        (r"rule\s+", "Test 1: rule keywords"),
        (r#"rule\s+"[^"]+""#, "Test 2: extract names"),
        (r"salience\s+\d+", "Test 3: salience"),
        (r"\w+\s*>=\s*\d+", "Test 4: conditions"),
        (r#""[^"]+""#, "Test 5: strings"),
        (r"\d+", "Test 6: numbers"),
    ];

    for (pat_str, desc) in patterns {
        let pattern = Pattern::new(pat_str).unwrap();
        println!("\n{}: {:?}", desc, pat_str);
        println!("  Pattern type: {:?}", pattern);
    }
}
