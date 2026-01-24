fn main() {
    println!("=== Testing nested capture patterns ===\n");

    // Test 1: Simple non-capturing group with alternation
    let p1 = r#"(?:"test"|foo)"#;
    let t1 = r#""test""#;
    match rexile::Pattern::new(p1) {
        Ok(pat) => {
            println!("Pattern: {}", p1);
            println!("Text: {}", t1);
            println!("Match: {}", pat.is_match(t1));
            println!("Find: {:?}\n", pat.find(t1));
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Test 2: Non-capturing group with capture inside
    let p2 = r#"(?:"([^"]+)")"#;
    let t2 = r#""CheckAge""#;
    match rexile::Pattern::new(p2) {
        Ok(pat) => {
            println!("Pattern: {}", p2);
            println!("Text: {}", t2);
            println!("Match: {}", pat.is_match(t2));
            println!("Find: {:?}", pat.find(t2));
            if let Some(caps) = pat.captures(t2) {
                println!("Captures: group 0={:?}, group 1={:?}\n", caps.get(0), caps.get(1));
            } else {
                println!("No captures\n");
            }
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Test 3: Non-capturing group with alternation and captures
    let p3 = r#"(?:"([^"]+)"|([a-z]+))"#;
    let t3a = r#""CheckAge""#;
    let t3b = "foo";
    match rexile::Pattern::new(p3) {
        Ok(pat) => {
            println!("Pattern: {}", p3);
            println!("Text 1: {}", t3a);
            println!("Match 1: {}", pat.is_match(t3a));
            println!("Find 1: {:?}", pat.find(t3a));

            println!("Text 2: {}", t3b);
            println!("Match 2: {}", pat.is_match(t3b));
            println!("Find 2: {:?}\n", pat.find(t3b));
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Test 4: Just the quoted string part
    let p4 = r#""([^"]+)""#;
    let t4 = r#""CheckAge""#;
    match rexile::Pattern::new(p4) {
        Ok(pat) => {
            println!("Pattern: {}", p4);
            println!("Text: {}", t4);
            println!("Match: {}", pat.is_match(t4));
            println!("Find: {:?}", pat.find(t4));
            if let Some(caps) = pat.captures(t4) {
                println!("Captures: group 0={:?}, group 1={:?}\n", caps.get(0), caps.get(1));
            } else {
                println!("No captures\n");
            }
        }
        Err(e) => println!("Error: {}\n", e),
    }
}
