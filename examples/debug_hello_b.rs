use rexile::Pattern;

fn main() {
    let pattern = Pattern::new(r"hello\b").unwrap();
    let text = "hello world";

    println!("Pattern: hello\\b");
    println!("Text: {:?}", text);

    if let Some((start, end)) = pattern.find(text) {
        println!("Match: ({}, {}) = {:?}", start, end, &text[start..end]);
    } else {
        println!("No match");
    }

    let all = pattern.find_all(text);
    println!("All matches: {} total", all.len());
    for (start, end) in all {
        println!("  ({}, {}) = {:?}", start, end, &text[start..end]);
    }
}
