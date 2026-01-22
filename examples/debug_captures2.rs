use rexile::Pattern;

fn main() {
    // Test non-capture pattern first
    let p1 = Pattern::new(r"\d{4}").unwrap();
    let text = "Date: 2026-01-22";
    
    if let Some((start, end)) = p1.find(text) {
        println!("\\d{{4}} found: {}..{} = {:?}", start, end, &text[start..end]);
    } else {
        println!("\\d{{4}} NOT found");
    }
    
    // Test with a single capture group
    let p2 = Pattern::new(r"(\d{4})").unwrap();
    if let Some((start, end)) = p2.find(text) {
        println!("(\\d{{4}}) found: {}..{} = {:?}", start, end, &text[start..end]);
    } else {
        println!("(\\d{{4}}) NOT found");
    }
}
