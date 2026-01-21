//! Parser/Lexer use case - matching keywords and tokens

use rexile::Pattern;

fn main() {
    println!("=== ReXile Parser/Lexer Example ===\n");

    // Simulating a simple lexer for a programming language
    let keywords = Pattern::new("let|const|var|function|return|if|else").unwrap();
    let identifier_start = Pattern::new("^[a-zA-Z_]").unwrap(); // Note: char classes not yet implemented
    
    let code = "function add(x, y) { return x + y; }";
    
    println!("Code: {}", code);
    println!("\nTokens found:");
    
    // Find all keywords
    let tokens = vec![
        "function", "add", "return"
    ];
    
    for token in tokens {
        if keywords.is_match(token) {
            println!("  KEYWORD: {}", token);
        } else {
            println!("  IDENTIFIER: {}", token);
        }
    }
    
    println!("\n=== Rule Engine Pattern Matching ===\n");
    
    // Rule engine use case
    let rule_start = Pattern::new("^rule").unwrap();
    let when_clause = Pattern::new("when").unwrap();
    let then_clause = Pattern::new("then").unwrap();
    
    let rule_text = "rule \"discount_rule\" when customer.age > 65 then apply_discount(0.2)";
    
    println!("Rule: {}", rule_text);
    println!("Starts with 'rule': {}", rule_start.is_match(rule_text));
    println!("Has 'when' clause: {}", when_clause.is_match(rule_text));
    println!("Has 'then' clause: {}", then_clause.is_match(rule_text));
    
    if let Some((start, end)) = when_clause.find(rule_text) {
        println!("'when' found at position: ({}, {})", start, end);
    }
}
