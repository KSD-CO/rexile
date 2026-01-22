use rexile::Pattern;

fn main() {
    let pattern_str = r"(\d{4})-(\d{2})-(\d{2})";
    println!("Parsing: {}", pattern_str);
    
    match Pattern::new(pattern_str) {
        Ok(p) => {
            println!("Pattern created successfully");
            
            let text = "Date: 2026-01-22";
            println!("\nTesting against: {}", text);
            
            if let Some((start, end)) = p.find(text) {
                println!("find() found: {}..{} = {:?}", start, end, &text[start..end]);
            } else {
                println!("find() returned None");
            }
            
            if let Some(caps) = p.captures(text) {
                println!("captures() found: {}", &caps[0]);
            } else {
                println!("captures() returned None");
            }
            
            // Also test simpler pattern
            println!("\n--- Testing simpler pattern ---");
            let p2 = Pattern::new(r"(\d{4})").unwrap();
            if let Some(caps2) = p2.captures(text) {
                println!("Simple pattern captures(): {}", &caps2[0]);
            } else {
                println!("Simple pattern returned None");
            }
        },
        Err(e) => {
            println!("Error parsing pattern: {:?}", e);
        }
    }
}
