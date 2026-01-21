use rexile::Pattern;

fn main() {
    println!("=== ReXile Escape Sequence Demo ===\n");

    // \d - digits
    println!("--- \\d (Digits) ---");
    let digits = Pattern::new("\\d+").unwrap();
    let text = "Room 404, Floor 3, Building B-12";
    println!("  Pattern: \\d+");
    println!("  Text: '{}'", text);
    for (start, end) in digits.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // \w - word characters
    println!("\n--- \\w (Word Characters) ---");
    let words = Pattern::new("\\w+").unwrap();
    let text = "hello_world user123 test-case";
    println!("  Pattern: \\w+");
    println!("  Text: '{}'", text);
    for (start, end) in words.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // \s - whitespace
    println!("\n--- \\s (Whitespace) ---");
    let whitespace = Pattern::new("\\s+").unwrap();
    let text = "hello\tworld\ntest  spaces";
    println!("  Pattern: \\s+");
    println!("  Text: {:?}", text);
    for (start, end) in whitespace.find_all(text) {
        let matched = &text[start..end];
        println!("    Found: {:?} at [{}..{}]", matched, start, end);
    }

    // \D - non-digits
    println!("\n--- \\D (Non-Digits) ---");
    let non_digits = Pattern::new("\\D+").unwrap();
    let text = "id123name456";
    println!("  Pattern: \\D+");
    println!("  Text: '{}'", text);
    for (start, end) in non_digits.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // \W - non-word characters
    println!("\n--- \\W (Non-Word Characters) ---");
    let non_words = Pattern::new("\\W+").unwrap();
    let text = "hello, world! test-case";
    println!("  Pattern: \\W+");
    println!("  Text: '{}'", text);
    for (start, end) in non_words.find_all(text) {
        let matched = &text[start..end];
        println!("    Found: '{}' at [{}..{}]", matched, start, end);
    }

    // \S - non-whitespace
    println!("\n--- \\S (Non-Whitespace) ---");
    let non_space = Pattern::new("\\S+").unwrap();
    let text = "hello world  test";
    println!("  Pattern: \\S+");
    println!("  Text: '{}'", text);
    for (start, end) in non_space.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Special characters
    println!("\n--- Special Characters (\\n, \\t, \\r) ---");
    let newline = Pattern::new("\\n").unwrap();
    let text = "line1\nline2\nline3";
    println!("  Pattern: \\n");
    println!("  Text: {:?}", text);
    println!("    Matches: {}", newline.is_match(text));
    if let Some((start, end)) = newline.find(text) {
        println!("    First at: [{}..{}]", start, end);
    }

    // Literal escapes
    println!("\n--- Literal Escapes ---");
    let dot = Pattern::new("\\.").unwrap();
    let text = "example.com";
    println!("  Pattern: \\.");
    println!("  Text: '{}'", text);
    println!("    Matches: {}", dot.is_match(text));
    if let Some((start, end)) = dot.find(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    let star = Pattern::new("\\*").unwrap();
    let text = "multiply: 2 * 3 = 6";
    println!("\n  Pattern: \\*");
    println!("  Text: '{}'", text);
    if let Some((start, end)) = star.find(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Practical examples
    println!("\n=== Practical Examples ===");

    // Extract numbers from text
    println!("\n--- Extract All Numbers ---");
    let digits = Pattern::new("\\d+").unwrap();
    let invoice = "Invoice #2024-1234: Items: 5, Total: $299.99";
    println!("  Text: '{}'", invoice);
    println!("  Numbers found:");
    for (start, end) in digits.find_all(invoice) {
        println!("    - {}", &invoice[start..end]);
    }

    // Extract words (identifiers)
    println!("\n--- Extract Identifiers ---");
    let identifiers = Pattern::new("\\w+").unwrap();
    let code = "let count = 42; let userName = 'John';";
    println!("  Code: '{}'", code);
    println!("  Identifiers:");
    for (start, end) in identifiers.find_all(code) {
        println!("    - {}", &code[start..end]);
    }

    // Split on whitespace
    println!("\n--- Split on Whitespace ---");
    let whitespace = Pattern::new("\\s+").unwrap();
    let text = "hello   world\ttest\nline";
    println!("  Text: {:?}", text);
    println!("  Words:");
    let mut last_end = 0;
    for (start, end) in whitespace.find_all(text) {
        if start > last_end {
            println!("    - {}", &text[last_end..start]);
        }
        last_end = end;
    }
    if last_end < text.len() {
        println!("    - {}", &text[last_end..]);
    }

    // Find non-numeric parts
    println!("\n--- Find Non-Numeric Parts ---");
    let non_digits = Pattern::new("\\D+").unwrap();
    let data = "user123data456info789";
    println!("  Text: '{}'", data);
    println!("  Non-numeric parts:");
    for (start, end) in non_digits.find_all(data) {
        println!("    - {}", &data[start..end]);
    }

    // Match email-like patterns (simplified)
    println!("\n--- Match Simple Email Pattern ---");
    let user_part = Pattern::new("\\w+").unwrap();
    let email = "john.doe@example.com";
    println!("  Email: '{}'", email);
    if let Some((start, end)) = user_part.find(email) {
        println!("  Username part: '{}'", &email[start..end]);
    }

    println!("\n✓ Escape sequence demo complete!");
}
