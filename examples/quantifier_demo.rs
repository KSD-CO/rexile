use rexile::Pattern;

fn main() {
    println!("=== ReXile Quantifier Demo ===\n");

    // Zero or more (*)
    println!("--- Zero or More (*) ---");
    let pattern = Pattern::new("a*").unwrap();
    for text in &["", "a", "aaa", "b", "aaab"] {
        if let Some((start, end)) = pattern.find(text) {
            println!("  'a*' in '{}': found '{}' at [{}..{}]", 
                text, &text[start..end], start, end);
        } else {
            println!("  'a*' in '{}': no match", text);
        }
    }

    // One or more (+)
    println!("\n--- One or More (+) ---");
    let pattern = Pattern::new("b+").unwrap();
    for text in &["", "b", "bbb", "a", "abba"] {
        if let Some((start, end)) = pattern.find(text) {
            println!("  'b+' in '{}': found '{}' at [{}..{}]", 
                text, &text[start..end], start, end);
        } else {
            println!("  'b+' in '{}': no match", text);
        }
    }

    // Zero or one (?)
    println!("\n--- Zero or One (?) ---");
    let pattern = Pattern::new("x?").unwrap();
    for text in &["", "x", "xx", "y", "xyz"] {
        if let Some((start, end)) = pattern.find(text) {
            println!("  'x?' in '{}': found '{}' at [{}..{}]", 
                text, &text[start..end], start, end);
        } else {
            println!("  'x?' in '{}': no match", text);
        }
    }

    // Character classes with quantifiers
    println!("\n--- Character Classes + Quantifiers ---");
    
    // [0-9]+ for numbers
    let digits = Pattern::new("[0-9]+").unwrap();
    let text = "Order #12345 costs $67.89";
    println!("  Pattern: [0-9]+");
    println!("  Text: '{}'", text);
    for (start, end) in digits.find_all(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // [a-z]* for lowercase letters
    let letters = Pattern::new("[a-z]*").unwrap();
    let text = "Hello World";
    println!("\n  Pattern: [a-z]*");
    println!("  Text: '{}'", text);
    for (start, end) in letters.find_all(text).iter().take(5) {  // Show first 5
        println!("    Found: '{}' at [{}..{}]", &text[*start..*end], start, end);
    }

    // [A-Z]? for optional uppercase
    let upper = Pattern::new("[A-Z]?").unwrap();
    let text = "Read The Manual";
    println!("\n  Pattern: [A-Z]?");
    println!("  Text: '{}'", text);
    for (start, end) in upper.find_all(text).iter().take(6) {  // Show first 6
        let matched = &text[*start..*end];
        if matched.is_empty() {
            println!("    Found: (empty) at [{}..{}]", start, end);
        } else {
            println!("    Found: '{}' at [{}..{}]", matched, start, end);
        }
    }

    // Practical examples
    println!("\n--- Practical Examples ---");

    // Extract numbers from text
    let numbers = Pattern::new("[0-9]+").unwrap();
    let invoice = "Invoice #2024-1234: Items: 5, Total: $299";
    println!("  Extract numbers from: '{}'", invoice);
    for (start, end) in numbers.find_all(invoice) {
        println!("    Number: {}", &invoice[start..end]);
    }

    // Match variable names (letter followed by any letters/digits)
    let var_start = Pattern::new("[a-zA-Z]+").unwrap();
    let code = "let count = 42; let userName = 'John';";
    println!("\n  Extract words from: '{}'", code);
    for (start, end) in var_start.find_all(code) {
        println!("    Word: {}", &code[start..end]);
    }

    // Match optional punctuation
    let punct = Pattern::new("[.,!?]*").unwrap();
    let sentence = "Hello, World!";
    println!("\n  Find punctuation in: '{}'", sentence);
    for (start, end) in punct.find_all(sentence).iter().take(8) {
        if start != end {
            println!("    Punctuation: '{}' at [{}..{}]", &sentence[*start..*end], start, end);
        }
    }

    println!("\n=== Greedy Matching Behavior ===");
    let pattern = Pattern::new("a+").unwrap();
    let text = "aaabaaaa";
    println!("  Pattern 'a+' uses greedy matching");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Matches longest possible: '{}' at [{}..{}]", 
            &text[start..end], start, end);
    }

    println!("\n✓ Quantifier demo complete!");
}
