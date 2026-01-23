//! Basic usage examples for ReXile

use rexile::Pattern;

fn main() {
    println!("=== ReXile Basic Usage Examples ===\n");

    // Example 1: Simple literal matching
    println!("1. Literal Matching:");
    let pattern = Pattern::new("hello").unwrap();
    println!("   Pattern: 'hello'");
    println!("   Text: 'hello world'");
    println!("   Match: {}", pattern.is_match("hello world"));
    println!("   Find: {:?}\n", pattern.find("say hello there"));

    // Example 2: Multi-pattern alternation
    println!("2. Multi-Pattern Alternation (using aho-corasick):");
    let keywords = Pattern::new("import|export|function|class").unwrap();
    println!("   Pattern: 'import|export|function|class'");
    println!("   Text: 'export default function'");
    println!("   Match: {}", keywords.is_match("export default function"));
    println!("   Find: {:?}\n", keywords.find("export default function"));

    // Example 3: Start anchor
    println!("3. Start Anchor (^):");
    let starts = Pattern::new("^Hello").unwrap();
    println!("   Pattern: '^Hello'");
    println!("   'Hello World': {}", starts.is_match("Hello World"));
    println!("   'Say Hello': {}\n", starts.is_match("Say Hello"));

    // Example 4: End anchor
    println!("4. End Anchor ($):");
    let ends = Pattern::new("World$").unwrap();
    println!("   Pattern: 'World$'");
    println!("   'Hello World': {}", ends.is_match("Hello World"));
    println!("   'World Peace': {}\n", ends.is_match("World Peace"));

    // Example 5: Exact match
    println!("5. Exact Match (^...$):");
    let exact = Pattern::new("^exact$").unwrap();
    println!("   Pattern: '^exact$'");
    println!("   'exact': {}", exact.is_match("exact"));
    println!("   'not exact': {}\n", exact.is_match("not exact"));

    // Example 6: Find all occurrences
    println!("6. Find All Occurrences:");
    let needle = Pattern::new("needle").unwrap();
    let text = "needle in a haystack with another needle";
    println!("   Pattern: 'needle'");
    println!("   Text: '{}'", text);
    println!("   All matches: {:?}\n", needle.find_all(text));

    // Example 7: Cached API (recommended for repeated patterns)
    println!("7. Cached API (compile once, reuse many times):");
    println!("   Pattern: 'test'");
    println!(
        "   First call (compiles): {}",
        rexile::is_match("test", "this is a test").unwrap()
    );
    println!(
        "   Second call (cached): {}",
        rexile::is_match("test", "another test").unwrap()
    );
    println!(
        "   Find with cache: {:?}\n",
        rexile::find("test", "test 123").unwrap()
    );
}
