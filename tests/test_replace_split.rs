use rexile::Pattern;

#[test]
fn test_replace_simple() {
    let pattern = Pattern::new(r"\w+").unwrap();
    
    // Replace first word
    assert_eq!(
        pattern.replace("hello world", "goodbye"),
        "goodbye world"
    );
    
    // No match
    assert_eq!(
        pattern.replace("123 456", "xxx"),
        "xxx 456"
    );
}

#[test]
fn test_replace_with_captures() {
    let pattern = Pattern::new(r"(\w+)=(\d+)").unwrap();
    
    // Replace first match with captures
    assert_eq!(
        pattern.replace("a=1 b=2", "$1:[$2]"),
        "a:[1] b=2"
    );
    
    // Multiple groups
    assert_eq!(
        pattern.replace("foo=42 bar=99", "$2=$1"),
        "42=foo bar=99"
    );
}

#[test]
fn test_replace_no_match() {
    let pattern = Pattern::new(r"\d+").unwrap();
    
    // No match, return original
    assert_eq!(
        pattern.replace("no numbers here", "XXX"),
        "no numbers here"
    );
}

#[test]
fn test_replace_all_simple() {
    let pattern = Pattern::new(r"\d+").unwrap();
    
    // Replace all digits
    assert_eq!(
        pattern.replace_all("a1b2c3", "X"),
        "aXbXcX"
    );
}

#[test]
fn test_replace_all_with_captures() {
    let pattern = Pattern::new(r"(\w+)=(\d+)").unwrap();
    
    // Replace all with captures
    assert_eq!(
        pattern.replace_all("a=1 b=2 c=3", "$1:[$2]"),
        "a:[1] b:[2] c:[3]"
    );
}

#[test]
fn test_split_simple() {
    let pattern = Pattern::new(r"\s+").unwrap();
    
    // Split on whitespace
    let parts: Vec<_> = pattern.split("a  b   c").collect();
    assert_eq!(parts, vec!["a", "b", "c"]);
}

#[test]
fn test_split_single_delimiter() {
    let pattern = Pattern::new(r",").unwrap();
    
    let parts: Vec<_> = pattern.split("apple,banana,cherry").collect();
    assert_eq!(parts, vec!["apple", "banana", "cherry"]);
}

#[test]
fn test_split_no_match() {
    let pattern = Pattern::new(r",").unwrap();
    
    // No delimiter, return whole string
    let parts: Vec<_> = pattern.split("no delimiters").collect();
    assert_eq!(parts, vec!["no delimiters"]);
}

#[test]
fn test_split_empty_parts() {
    let pattern = Pattern::new(r",").unwrap();
    
    // Empty parts between delimiters
    let parts: Vec<_> = pattern.split("a,,b").collect();
    assert_eq!(parts, vec!["a", "", "b"]);
}

#[test]
fn test_replace_literal_dollar() {
    let pattern = Pattern::new(r"\w+").unwrap();
    
    // $ not followed by digit should be literal
    assert_eq!(
        pattern.replace("hello", "$price"),
        "$price"
    );
}

#[test]
fn test_replace_all_no_match() {
    let pattern = Pattern::new(r"\d+").unwrap();
    
    // No match, return original
    assert_eq!(
        pattern.replace_all("no numbers", "X"),
        "no numbers"
    );
}
