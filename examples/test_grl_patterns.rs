/// Test rexile with real GRL (Grule Rule Language) patterns
/// These are actual patterns used in rust-rule-engine for parsing rule files
use rexile::Pattern;

fn main() {
    println!("Testing rexile with GRL patterns from rust-rule-engine\n");
    println!("{}", "=".repeat(70));
    
    // Test 1: Rule definition with Unicode comments
    test_rule_with_unicode();
    
    // Test 2: Rule name extraction
    test_rule_name_extraction();
    
    // Test 3: Salience extraction
    test_salience_extraction();
    
    // Test 4: When clause
    test_when_clause();
    
    // Test 5: Then clause
    test_then_clause();
    
    // Test 6: Complex multi-line rule
    test_complex_multiline_rule();
    
    // Test 7: Vietnamese comments (real-world case)
    test_vietnamese_comments();
    
    // Test 8: Mathematical symbols in comments
    test_math_symbols();
    
    println!("\n{}", "=".repeat(70));
    println!("✅ All GRL pattern tests completed successfully!");
}

fn test_rule_with_unicode() {
    println!("\n📝 Test 1: Rule with Unicode arrow in comment");
    
    let grl = r#"// Rule: Amount < 2M + COD → Auto approve
rule "AutoApproveSmallOrder" salience 80 {
    when
        Order.Amount < 2000000 &&
        Payment.Method == "COD"
    then
        Order.AutoApproved = true;
}"#;
    
    println!("Input GRL:\n{}", grl);
    
    // Pattern to match rule declaration
    let pattern = Pattern::new(r#"rule\s+"[^"]+"\s+salience\s+\d+"#).unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            let matched = &grl[start..end];
            println!("✓ Matched: {:?}", matched);
            assert_eq!(matched, r#"rule "AutoApproveSmallOrder" salience 80"#);
        }
        None => panic!("❌ Should match rule declaration"),
    }
}

fn test_rule_name_extraction() {
    println!("\n📝 Test 2: Extract rule name");
    
    let grl = r#"rule "CheckBalance" salience 50 {}"#;
    
    // Pattern to extract quoted rule name
    let pattern = Pattern::new(r#""[^"]+""#).unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            let matched = &grl[start..end];
            println!("✓ Extracted rule name: {}", matched);
            assert_eq!(matched, r#""CheckBalance""#);
        }
        None => panic!("❌ Should extract rule name"),
    }
}

fn test_salience_extraction() {
    println!("\n📝 Test 3: Extract salience value");
    
    let grl = r#"rule "Test" salience 100 {"#;
    
    // Pattern to extract salience number
    let pattern = Pattern::new(r"salience\s+\d+").unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            let matched = &grl[start..end];
            println!("✓ Extracted: {}", matched);
            assert_eq!(matched, "salience 100");
        }
        None => panic!("❌ Should extract salience"),
    }
}

fn test_when_clause() {
    println!("\n📝 Test 4: Match when clause");
    
    let grl = r#"rule "Test" {
    when
        Order.Amount > 1000
    then
        Log("High value order");
}"#;
    
    // Pattern to find when keyword
    let pattern = Pattern::new(r"\bwhen\b").unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            println!("✓ Found 'when' at position ({}, {})", start, end);
            assert_eq!(&grl[start..end], "when");
        }
        None => panic!("❌ Should find when clause"),
    }
}

fn test_then_clause() {
    println!("\n📝 Test 5: Match then clause");
    
    let grl = r#"when Order.Amount > 1000 then Order.Approved = true;"#;
    
    // Pattern to match then with assignment
    let pattern = Pattern::new(r"\bthen\b\s+\w+").unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            let matched = &grl[start..end];
            println!("✓ Matched: {:?}", matched);
            assert!(matched.starts_with("then"));
        }
        None => panic!("❌ Should match then clause"),
    }
}

fn test_complex_multiline_rule() {
    println!("\n📝 Test 6: Complex multi-line rule with emoji 🚀");
    
    let grl = r#"// 🚀 Fast processing rule
rule "ProcessHighPriority" salience 100 {
    when
        Order.Priority == "HIGH" &&
        Order.Amount < 5000000
    then
        Order.FastTrack = true;
        Log("Fast track enabled");
}"#;
    
    println!("Input GRL with emoji:\n{}", grl);
    
    // Pattern to match complete rule structure
    let pattern = Pattern::new(r#"rule\s+"[^"]+"\s+salience\s+\d+"#).unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            let matched = &grl[start..end];
            println!("✓ Matched rule header: {:?}", matched);
            assert!(matched.contains("ProcessHighPriority"));
        }
        None => panic!("❌ Should match rule with emoji in comment"),
    }
    
    // Test finding all identifiers
    let ident_pattern = Pattern::new(r"\b[A-Z][a-zA-Z]+\b").unwrap();
    let identifiers: Vec<_> = ident_pattern.find_all(grl)
        .into_iter()
        .map(|(s, e)| &grl[s..e])
        .collect();
    
    println!("✓ Found identifiers: {:?}", identifiers);
    assert!(identifiers.contains(&"Order"));
    assert!(identifiers.contains(&"Priority"));
}

fn test_vietnamese_comments() {
    println!("\n📝 Test 7: Vietnamese comments (real-world)");
    
    let grl = r#"// Quy tắc: Đơn hàng nhỏ < 2 triệu + COD → Tự động duyệt
rule "AutoApproveSmallOrderVN" salience 80 {
    when
        Order.Amount < 2000000 &&
        Payment.Method == "COD"
    then
        Order.AutoApproved = true;
        // Ghi log: "Đã tự động duyệt đơn hàng"
        Log("Auto approved");
}"#;
    
    println!("Input GRL with Vietnamese:\n{}", grl);
    
    // Should correctly parse despite Vietnamese characters
    let pattern = Pattern::new(r#"rule\s+"[^"]+""#).unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            let matched = &grl[start..end];
            println!("✓ Matched: {:?}", matched);
            assert_eq!(matched, r#"rule "AutoApproveSmallOrderVN""#);
        }
        None => panic!("❌ Should match rule with Vietnamese comments"),
    }
    
    // Test finding COD
    let cod_pattern = Pattern::new(r#""COD""#).unwrap();
    match cod_pattern.find(grl) {
        Some((s, e)) => {
            println!("✓ Found COD at ({}, {})", s, e);
            assert_eq!(&grl[s..e], r#""COD""#);
        }
        None => panic!("❌ Should find COD"),
    }
}

fn test_math_symbols() {
    println!("\n📝 Test 8: Math symbols in comments");
    
    let grl = r#"// Formula: ∑(amounts) ≥ threshold → approve
// Operators: ∧ (AND), ∨ (OR), ¬ (NOT)
rule "MathFormula" salience 90 {
    when
        Order.Total >= 1000
    then
        Order.Approved = true;
}"#;
    
    println!("Input GRL with math symbols:\n{}", grl);
    
    // Pattern should work despite math symbols
    let pattern = Pattern::new(r#"rule\s+"\w+"\s+salience\s+\d+"#).unwrap();
    
    match pattern.find(grl) {
        Some((start, end)) => {
            let matched = &grl[start..end];
            println!("✓ Matched: {:?}", matched);
            assert_eq!(matched, r#"rule "MathFormula" salience 90"#);
        }
        None => panic!("❌ Should match rule with math symbols"),
    }
    
    // Test finding >= operator
    let op_pattern = Pattern::new(r">=").unwrap();
    match op_pattern.find(grl) {
        Some((s, e)) => {
            println!("✓ Found >= operator at ({}, {})", s, e);
            assert_eq!(&grl[s..e], ">=");
        }
        None => panic!("❌ Should find >= operator"),
    }
}
