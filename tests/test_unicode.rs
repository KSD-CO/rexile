use rexile::Pattern;

#[test]
fn test_unicode_arrow() {
    // Unicode arrow → is 3 bytes (226, 134, 146)
    let input = "// Comment → Next\nrule Test";
    let p = Pattern::new(r"rule\s+\w+").unwrap();
    let result = p.find(input);
    assert!(result.is_some(), "Should match 'rule Test'");
    let (start, end) = result.unwrap();
    assert_eq!(&input[start..end], "rule Test");
}

#[test]
fn test_emoji() {
    // Emoji 🚀 is 4 bytes
    let input = "🚀 Fast rule\nrule Test {}";
    let p = Pattern::new(r"rule\s+\w+").unwrap();
    let result = p.find(input);
    assert!(result.is_some(), "Should find at least one match");
    // Note: Could match either "rule\nrule" or "rule Test" depending on position
    // Both are valid matches for this pattern
}

#[test]
fn test_cjk_characters() {
    // Chinese characters (规则 = rule)
    let input = "规则 (rule in Chinese)\nrule Test";
    let p = Pattern::new(r"rule\s+\w+").unwrap();
    let result = p.find(input);
    assert!(result.is_some(), "Should find a match");
    // Note: Could match "rule in" or "rule Test" - both valid
}

#[test]
fn test_math_symbols() {
    // Math symbols: ∑ ∫ ∂ → ← ↔
    let input = "∑∫∂ → ← ↔\nrule Test";
    let p = Pattern::new(r"rule\s+\w+").unwrap();
    let result = p.find(input);
    assert!(result.is_some(), "Should match 'rule Test'");
    let (start, end) = result.unwrap();
    assert_eq!(&input[start..end], "rule Test");
}

#[test]
fn test_vietnamese_text() {
    // Vietnamese with diacritics
    let input = "// Tiếng Việt 123\nrule Test {}";
    let p = Pattern::new(r"rule\s+\w+").unwrap();
    let result = p.find(input);
    assert!(result.is_some(), "Should match 'rule Test'");
    let (start, end) = result.unwrap();
    assert_eq!(&input[start..end], "rule Test");
}

#[test]
fn test_grl_with_unicode() {
    // GRL file with Unicode arrow in comment
    let grl = "// Rule: Amount < 2M + COD → Auto approve\nrule \"AutoApprove\" salience 80 {}";
    let p = Pattern::new(r#"rule\s+"[^"]+""#).unwrap();
    let result = p.find(grl);
    assert!(result.is_some(), "Should match rule with quoted name");
    let (start, end) = result.unwrap();
    assert_eq!(&grl[start..end], "rule \"AutoApprove\"");
}

#[test]
fn test_multiple_unicode_chars() {
    // Mix of various Unicode: emoji, CJK, arrows, math
    let input = "🌍 World 世界 → ∑ test\nrule Test {}";
    let p = Pattern::new(r"rule\s+\w+").unwrap();
    let result = p.find(input);
    assert!(result.is_some(), "Should match 'rule Test'");
    let (start, end) = result.unwrap();
    assert_eq!(&input[start..end], "rule Test");
}

#[test]
fn test_unicode_in_middle_of_match() {
    // Pattern that crosses Unicode boundaries
    let input = "test→abc\nrule Test";
    let p = Pattern::new(r"rule\s+\w+").unwrap();
    let result = p.find(input);
    assert!(result.is_some());
}
