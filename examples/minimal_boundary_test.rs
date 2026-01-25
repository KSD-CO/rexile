use rexile::Pattern;

fn main() {
    // Test the simplest case
    let pattern_str = r"\bhello";
    let text = "hello";

    println!("Pattern: {:?}", pattern_str);
    println!("Text: {:?}", text);

    match Pattern::new(pattern_str) {
        Ok(pattern) => {
            println!("✓ Pattern compiled");

            let is_match = pattern.is_match(text);
            println!("is_match: {}", is_match);

            if let Some((start, end)) = pattern.find(text) {
                println!("find: Some(({}, {})) = {:?}", start, end, &text[start..end]);
            } else {
                println!("find: None");
            }
        }
        Err(e) => {
            println!("✗ Failed: {:?}", e);
        }
    }
}
