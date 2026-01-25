use rexile::Pattern;

fn main() {
    let tests = vec![
        (r"\d{1,3}\.\d{1,3}", "192.168.1.1", "IP prefix"),
        (r"(?i)(GET|POST)", "GET /api", "HTTP method"),
        (r"\b\d{4}\b", "Year: 2024!", "Year extraction"),
    ];

    for (pattern_str, text, desc) in tests {
        println!("Testing: {}", desc);
        println!("  Pattern: {}", pattern_str);
        println!("  Text: {:?}", text);

        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let result = pattern.find(text);
                println!("  Result: {:?}", result);
                if let Some((start, end)) = result {
                    println!("  Matched: {:?}", &text[start..end]);
                }
            }
            Err(e) => {
                println!("  Error: {:?}", e);
            }
        }
        println!();
    }
}
