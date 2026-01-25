use rexile::Pattern;

fn main() {
    println!("Testing IP pattern variations:");
    test(r"\d+\.\d+", "192.168.1.1", "Without range quantifier");
    test(r"\d{1,3}", "192", "Just range quantifier");
    test(r"\d{1,3}\.", "192.", "Range + dot");
    test(r"\d{1,3}\.\d{1,3}", "192.168", "Full IP pattern");

    println!("\nTesting HTTP method variations:");
    test(r"GET", "GET /api", "Just GET literal");
    test(r"(GET)", "GET /api", "GET in capture group");
    test(r"(GET|POST)", "GET /api", "Alternation in group");
    test(r"(?i)GET", "GET /api", "Case insensitive GET");
    test(
        r"(?i)(GET|POST)",
        "GET /api",
        "Case insensitive alternation",
    );

    println!("\nTesting Year pattern variations:");
    test(r"\d", "2024", "Single digit");
    test(r"\d+", "2024", "Multiple digits");
    test(r"\d{4}", "2024", "Exactly 4 digits");
    test(r"\b\d+\b", "Year: 2024!", "Digits with boundaries");
    test(r"\b\d{4}\b", "Year: 2024!", "4 digits with boundaries");
}

fn test(pattern_str: &str, text: &str, desc: &str) {
    print!("  {:40} => ", desc);
    match Pattern::new(pattern_str) {
        Ok(pattern) => {
            if let Some((start, end)) = pattern.find(text) {
                println!("✓ ({}, {}) = {:?}", start, end, &text[start..end]);
            } else {
                println!("✗ None");
            }
        }
        Err(e) => {
            println!("✗ ERROR: {:?}", e);
        }
    }
}
