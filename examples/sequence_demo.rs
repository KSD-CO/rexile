use rexile::Pattern;

fn main() {
    println!("=== ReXile Sequence Demo ===\n");

    // Simple character sequences
    println!("--- Simple Character Sequences ---");
    let pattern = Pattern::new("abc").unwrap();
    let text = "xyzabcdef";
    println!("  Pattern: abc");
    println!("  Text: '{}'", text);
    if let Some((start, end)) = pattern.find(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Quantified sequences
    println!("\n--- Quantified Sequences ---");
    let pattern = Pattern::new("a+b+c*").unwrap();
    for text in &["ab", "aaabbc", "aaaabbbbcccc", "aab", "abc"] {
        println!("  Pattern 'a+b+c*' in '{}':", text);
        if let Some((start, end)) = pattern.find(text) {
            println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
        } else {
            println!("    No match");
        }
    }

    // Character class sequences
    println!("\n--- Character Class Sequences ---");
    let pattern = Pattern::new("[0-9]+[a-z]+").unwrap();
    let text = "test123abc end";
    println!("  Pattern: [0-9]+[a-z]+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Escape sequence sequences
    println!("\n--- Escape Sequence Sequences ---");
    let pattern = Pattern::new("\\d+\\w+").unwrap();
    let text = "id123user name456data";
    println!("  Pattern: \\d+\\w+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Mixed sequences
    println!("\n--- Mixed Sequences ---");
    let pattern = Pattern::new("hello\\d+").unwrap();
    let text = "say hello123 and hello456";
    println!("  Pattern: hello\\d+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Whitespace sequences
    println!("\n--- Whitespace Sequences ---");
    let pattern = Pattern::new("\\w+\\s+\\w+").unwrap();
    let text = "hello world foo bar";
    println!("  Pattern: \\w+\\s+\\w+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Practical examples
    println!("\n=== Practical Examples ===");

    // Match version numbers
    println!("\n--- Version Numbers ---");
    let pattern = Pattern::new("\\d+\\.\\d+\\.\\d+").unwrap();
    let text = "Using version 1.2.3 and 4.5.6 today";
    println!("  Pattern: \\d+\\.\\d+\\.\\d+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Version: {}", &text[start..end]);
    }

    // Match email username pattern
    println!("\n--- Email Username Pattern ---");
    let pattern = Pattern::new("\\w+@").unwrap();
    let text = "Contact john@example.com or mary@test.org";
    println!("  Pattern: \\w+@");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Found: {}", &text[start..end]);
    }

    // Match file extensions
    println!("\n--- File Extensions ---");
    let pattern = Pattern::new("\\w+\\.\\w+").unwrap();
    let text = "Files: readme.txt, main.rs, config.json";
    println!("  Pattern: \\w+\\.\\w+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    File: {}", &text[start..end]);
    }

    // Match price patterns
    println!("\n--- Price Patterns ---");
    let pattern = Pattern::new("\\$\\d+\\.\\d+").unwrap();
    let text = "Prices: $12.99, $5.50, $100.00";
    println!("  Pattern: \\$\\d+\\.\\d+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Price: {}", &text[start..end]);
    }

    // Match phone-like patterns
    println!("\n--- Phone-Like Patterns ---");
    let pattern = Pattern::new("\\d+-\\d+-\\d+").unwrap();
    let text = "Call 555-123-4567 or 800-555-0000";
    println!("  Pattern: \\d+-\\d+-\\d+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Phone: {}", &text[start..end]);
    }

    // Match date-like patterns
    println!("\n--- Date-Like Patterns ---");
    let pattern = Pattern::new("\\d+/\\d+/\\d+").unwrap();
    let text = "Dates: 12/31/2023 and 01/15/2024";
    println!("  Pattern: \\d+/\\d+/\\d+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Date: {}", &text[start..end]);
    }

    // Match hexadecimal colors
    println!("\n--- Hexadecimal Colors ---");
    let pattern = Pattern::new("#[0-9a-fA-F]+").unwrap();
    let text = "Colors: #FF5733, #00FF00, #123ABC";
    println!("  Pattern: #[0-9a-fA-F]+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Color: {}", &text[start..end]);
    }

    // Match URL-like patterns (simplified)
    println!("\n--- URL-Like Patterns ---");
    let pattern = Pattern::new("https\\://\\w+\\.\\w+").unwrap();
    let text = "Visit https://example.com and https://test.org";
    println!("  Pattern: https\\://\\w+\\.\\w+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    URL: {}", &text[start..end]);
    }

    println!("\n✓ Sequence demo complete!");
    println!("\n💡 Sequences combine multiple pattern elements:");
    println!("   - Characters: abc");
    println!("   - Quantified: a+b*c?");
    println!("   - Character classes: [0-9]+[a-z]*");
    println!("   - Escape sequences: \\d+\\w*\\s?");
    println!("   - Mixed: hello\\d+world");
}
