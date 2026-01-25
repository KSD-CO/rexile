use rexile::Pattern;

fn main() {
    println!("Testing minimal lookahead...");

    let pattern_str = r"a(?=b)";
    let text = "abc";

    println!("Pattern: {:?}", pattern_str);
    println!("Text: {:?}", text);

    match Pattern::new(pattern_str) {
        Ok(pattern) => {
            println!("✓ Pattern compiled");

            println!("Attempting to match...");
            if let Some((start, end)) = pattern.find(text) {
                println!(
                    "✓ Match found: ({}, {}) = {:?}",
                    start,
                    end,
                    &text[start..end]
                );
            } else {
                println!("✗ No match");
            }
        }
        Err(e) => {
            println!("✗ Failed to compile: {:?}", e);
        }
    }
}
