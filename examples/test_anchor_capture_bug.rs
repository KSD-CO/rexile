use rexile::Pattern;

fn main() {
    println!("=== Testing Anchor + Capture Bug ===\n");

    // Bug case: anchor + complex capturing groups
    let test_cases = vec![
        (r"^test\(([a-zA-Z_]\w*)\(([^)]*)\)\)$", "test(is_valid_email(User.email))"),
        (r"^(\w+)=(\d+)$", "foo=123"),
        (r"^([a-z]+)@([a-z]+)\.com$", "test@example.com"),
        (r"^(\d+)-(\d+)-(\d+)$", "2024-01-15"),
    ];

    for (pattern_str, input) in &test_cases {
        println!("Pattern: {}", pattern_str);
        println!("Input:   {}", input);
        
        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                // Test is_match
                let is_match = pattern.is_match(input);
                println!("  is_match: {}", is_match);
                
                // Test find
                let find_result = pattern.find(input);
                println!("  find:     {:?}", find_result);
                
                // Test captures
                let captures_result = pattern.captures(input);
                if let Some(caps) = &captures_result {
                    println!("  captures: Some");
                    println!("    [0]: {:?}", caps.get(0));
                    println!("    [1]: {:?}", caps.get(1));
                    println!("    [2]: {:?}", caps.get(2));
                } else {
                    println!("  captures: None");
                }
                
                // Expected: All should succeed when anchored pattern fully matches
                let full_match = find_result == Some((0, input.len()));
                println!("  Expected full match: {}", full_match);
            }
            Err(e) => {
                println!("  Error: {:?}", e);
            }
        }
        println!();
    }

    // Test without anchors (should work)
    println!("=== Same patterns WITHOUT anchors ===\n");
    
    let test_cases_no_anchor = vec![
        (r"test\(([a-zA-Z_]\w*)\(([^)]*)\)\)", "test(is_valid_email(User.email))"),
        (r"(\w+)=(\d+)", "foo=123"),
        (r"([a-z]+)@([a-z]+)\.com", "test@example.com"),
        (r"(\d+)-(\d+)-(\d+)", "2024-01-15"),
    ];

    for (pattern_str, input) in &test_cases_no_anchor {
        println!("Pattern: {}", pattern_str);
        println!("Input:   {}", input);
        
        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                let is_match = pattern.is_match(input);
                let find_result = pattern.find(input);
                let captures_result = pattern.captures(input);
                
                println!("  is_match: {}", is_match);
                println!("  find:     {:?}", find_result);
                
                if let Some(caps) = &captures_result {
                    println!("  captures: Some");
                    println!("    [0]: {:?}", caps.get(0));
                    println!("    [1]: {:?}", caps.get(1));
                    println!("    [2]: {:?}", caps.get(2));
                } else {
                    println!("  captures: None");
                }
            }
            Err(e) => {
                println!("  Error: {:?}", e);
            }
        }
        println!();
    }
}
