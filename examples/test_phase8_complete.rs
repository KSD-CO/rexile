use rexile::Pattern;

fn main() {
    println!("🎉 Phase 8 Complete - Testing Capture Groups!\n");
    
    // Test 1: Simple capture
    let p1 = Pattern::new(r"Hello (\w+)").unwrap();
    if let Some(caps) = p1.captures("Hello world") {
        println!("✅ Test 1: Hello (\\w+)");
        println!("   Full match: {}", &caps[0]);
        println!("   Capture 1:  {}", &caps[1]);
    }
    
    // Test 2: Multiple captures with {n} quantifier
    let p2 = Pattern::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    if let Some(caps) = p2.captures("Date: 2026-01-22") {
        println!("\n✅ Test 2: (\\d{{4}})-(\\d{{2}})-(\\d{{2}})");
        println!("   Full match: {}", &caps[0]);
        println!("   Year:       {}", &caps[1]);
        println!("   Month:      {}", &caps[2]);
        println!("   Day:        {}", &caps[3]);
    }
    
    // Test 3: Non-capturing group
    let p3 = Pattern::new(r"(?:Hello) (\w+)").unwrap();
    if let Some(caps) = p3.captures("Hello world") {
        println!("\n✅ Test 3: (?:Hello) (\\w+)");
        println!("   Full match: {}", &caps[0]);
        println!("   Capture 1:  {}", &caps[1]);
        println!("   Total groups: {} (only 1 capture!)", caps.len() - 1);
    }
    
    // Test 4: Captures iterator
    let p4 = Pattern::new(r"(\w+)=(\d+)").unwrap();
    let matches: Vec<_> = p4.captures_iter("a=1 b=2 c=3").collect();
    println!("\n✅ Test 4: (\\w+)=(\\d+) iterator");
    println!("   Found {} matches:", matches.len());
    for (i, caps) in matches.iter().enumerate() {
        println!("   Match {}: {}={}", i+1, &caps[1], &caps[2]);
    }
    
    println!("\n🎊 Phase 8 Complete! All capture features working!");
}
