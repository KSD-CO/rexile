//! Simple character class demo - showing what currently works

use rexile::Pattern;

fn main() {
    println!("=== ReXile Character Class Demo ===\n");

    // Test 1: Simple set
    let vowels = Pattern::new("[aeiou]").unwrap();
    println!("Pattern: [aeiou]");
    println!("  'apple': {}", vowels.is_match("apple"));
    println!("  'xyz': {}\n", vowels.is_match("xyz"));

    // Test 2: Range
    let lower = Pattern::new("[a-z]").unwrap();
    println!("Pattern: [a-z]");
    println!("  'hello': {}", lower.is_match("hello"));
    println!("  'HELLO': {}\n", lower.is_match("HELLO"));

    // Test 3: Multiple ranges
    let alphanum = Pattern::new("[a-zA-Z0-9]").unwrap();
    println!("Pattern: [a-zA-Z0-9]");
    println!("  'Test123': {}", alphanum.is_match("Test123"));
    println!("  '!!!': {}\n", alphanum.is_match("!!!"));

    // Test 4: Negation
    let non_digit = Pattern::new("[^0-9]").unwrap();
    println!("Pattern: [^0-9] (NOT digits)");
    println!("  'abc': {}", non_digit.is_match("abc"));
    println!("  '123': {}\n", non_digit.is_match("123"));

    // Test 5: Find
    let digit = Pattern::new("[0-9]").unwrap();
    println!("Pattern: [0-9]");
    println!("  find('abc123'): {:?}", digit.find("abc123"));
    println!("  find_all('a1b2c3'): {:?}\n", digit.find_all("a1b2c3"));

    println!("✅ Character classes working!");
    println!("   Next: Add quantifiers (+, *, ?) support");
}
