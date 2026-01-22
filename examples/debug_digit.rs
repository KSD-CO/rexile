use rexile::Pattern;

fn main() {
    let text = "Date: 2026-01-22";
    
    // Test various digit patterns
    let patterns = vec![
        r"\d",
        r"\d+",
        r"\d{4}",
        r"\d\d\d\d",
    ];
    
    for pat in patterns {
        let p = Pattern::new(pat).unwrap();
        if let Some((start, end)) = p.find(text) {
            println!("{:10} found: {}..{} = {:?}", pat, start, end, &text[start..end]);
        } else {
            println!("{:10} NOT found", pat);
        }
    }
}
