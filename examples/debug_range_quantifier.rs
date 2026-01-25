use rexile::Pattern;

fn main() {
    let pattern = Pattern::new("a{2,4}").unwrap();
    let text = "aaaaa";

    println!("Pattern: a{{2,4}}");
    println!("Text: {:?}", text);
    println!();

    if let Some((start, end)) = pattern.find(text) {
        println!("Match: ({}, {}) = {:?}", start, end, &text[start..end]);
    } else {
        println!("No match");
    }

    let all = pattern.find_all(text);
    println!("\nAll matches:");
    for (start, end) in all {
        println!("  ({}, {}) = {:?}", start, end, &text[start..end]);
    }
}
