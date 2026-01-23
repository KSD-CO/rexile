use rexile::Pattern;

fn main() {
    println!("=== ReXile Group Demo ===\n");

    // Simple groups
    println!("--- Simple Groups ---");
    let pattern = Pattern::new("(abc)").unwrap();
    let text = "xyz abc def";
    println!("  Pattern: (abc)");
    println!("  Text: '{}'", text);
    if let Some((start, end)) = pattern.find(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Non-capturing groups
    println!("\n--- Non-Capturing Groups (?:...) ---");
    let pattern = Pattern::new("(?:hello)").unwrap();
    let text = "say hello world";
    println!("  Pattern: (?:hello)");
    println!("  Text: '{}'", text);
    if let Some((start, end)) = pattern.find(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Alternation in groups
    println!("\n--- Alternation in Groups (a|b|c) ---");
    let pattern = Pattern::new("(foo|bar|baz)").unwrap();
    for text in &["I like foo", "I like bar", "I like baz", "I like qux"] {
        println!("  Pattern '(foo|bar|baz)' in '{}':", text);
        if let Some((start, end)) = pattern.find(text) {
            println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
        } else {
            println!("    No match");
        }
    }

    // Quantified groups
    println!("\n--- Quantified Groups ---");

    // Group with +
    let pattern = Pattern::new("(ab)+").unwrap();
    let text = "ababab xyz";
    println!("  Pattern: (ab)+");
    println!("  Text: '{}'", text);
    if let Some((start, end)) = pattern.find(text) {
        println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
    }

    // Group with *
    let pattern = Pattern::new("(xyz)*").unwrap();
    for text in &["xyzxyz", "xyz", "", "abc"] {
        println!("\n  Pattern '(xyz)*' in '{}':", text);
        if let Some((start, end)) = pattern.find(text) {
            println!("    Found: '{}' at [{}..{}]", &text[start..end], start, end);
        } else {
            println!("    No match");
        }
    }

    // Group with ?
    let pattern = Pattern::new("(test)?").unwrap();
    for text in &["test", "testing", "no", ""] {
        println!("\n  Pattern '(test)?' in '{}':", text);
        if let Some((start, end)) = pattern.find(text) {
            let matched = &text[start..end];
            if matched.is_empty() {
                println!("    Found: (empty) at [{}..{}]", start, end);
            } else {
                println!("    Found: '{}' at [{}..{}]", matched, start, end);
            }
        } else {
            println!("    No match");
        }
    }

    // Practical examples
    println!("\n=== Practical Examples ===");

    // Match protocol variants
    println!("\n--- Protocol Matching ---");
    let pattern = Pattern::new("(http|https|ftp)").unwrap();
    let text = "Visit http://example.com or https://secure.com or ftp://files.com";
    println!("  Pattern: (http|https|ftp)");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Protocol: {}", &text[start..end]);
    }

    // Match file types
    println!("\n--- File Types ---");
    let pattern = Pattern::new("(jpg|png|gif)").unwrap();
    let text = "Files: image.jpg, photo.png, anim.gif, doc.pdf";
    println!("  Pattern: (jpg|png|gif)");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Extension: {}", &text[start..end]);
    }

    // Match repeated patterns
    println!("\n--- Repeated Patterns ---");
    let pattern = Pattern::new("(ha)+").unwrap();
    let text = "haha hahaha ha hahahahaha";
    println!("  Pattern: (ha)+");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Laugh: {}", &text[start..end]);
    }

    // Match optional prefixes
    println!("\n--- Optional Prefixes ---");
    let pattern = Pattern::new("(Mr|Mrs|Ms)?").unwrap();
    for text in &["Mr Smith", "Mrs Johnson", "Ms Davis", "Anderson"] {
        println!("  Pattern '(Mr|Mrs|Ms)?' in '{}':", text);
        if let Some((start, end)) = pattern.find(text) {
            let matched = &text[start..end];
            if !matched.is_empty() {
                println!("    Title: {}", matched);
            } else {
                println!("    No title (matched empty)");
            }
        }
    }

    // Match repeated words
    println!("\n--- Repeated Words ---");
    let pattern = Pattern::new("(very)+").unwrap();
    for text in &["very good", "very very good", "very very very important"] {
        println!("  Pattern '(very)+' in '{}':", text);
        if let Some((start, end)) = pattern.find(text) {
            println!("    Found: '{}'", &text[start..end]);
        }
    }

    // Match currency symbols
    println!("\n--- Currency Symbols ---");
    let pattern = Pattern::new("(USD|EUR|GBP|JPY)").unwrap();
    let text = "Prices: 100 USD, 85 EUR, 70 GBP, 15000 JPY";
    println!("  Pattern: (USD|EUR|GBP|JPY)");
    println!("  Text: '{}'", text);
    for (start, end) in pattern.find_all(text) {
        println!("    Currency: {}", &text[start..end]);
    }

    println!("\n✓ Group demo complete!");
    println!("\n💡 Groups enable:");
    println!("   - Alternation: (a|b|c)");
    println!("   - Quantification: (abc)+");
    println!("   - Non-capturing: (?:...)");
    println!("   - Complex patterns: (foo|bar)+");
}
