//! Quantifier support: *, +, ?, {n,m}
//!
//! Implements a simple backtracking matcher for quantified patterns

use crate::charclass::CharClass;
use crate::escape::{parse_escape, starts_with_escape};

/// Represents a quantified pattern element
#[derive(Debug, Clone)]
pub enum QuantifiedElement {
    /// A literal character
    Char(char),
    /// A character class like [a-z]
    CharClass(CharClass),
}

impl QuantifiedElement {
    /// Check if a character matches this element
    pub fn matches(&self, ch: char) -> bool {
        match self {
            QuantifiedElement::Char(c) => *c == ch,
            QuantifiedElement::CharClass(cc) => cc.matches(ch),
        }
    }
}

/// Quantifier type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantifier {
    /// * - Zero or more (greedy)
    ZeroOrMore,
    /// + - One or more (greedy)
    OneOrMore,
    /// ? - Zero or one (greedy)
    ZeroOrOne,
    /// {n} - Exactly n times
    Exactly(usize),
    /// {n,} - At least n times
    AtLeast(usize),
    /// {n,m} - Between n and m times
    Between(usize, usize),
}

/// A quantified pattern: element + quantifier
#[derive(Debug, Clone)]
pub struct QuantifiedPattern {
    pub element: QuantifiedElement,
    pub quantifier: Quantifier,
}

impl QuantifiedPattern {
    /// Match this quantified pattern at the start of text
    /// Returns the number of characters consumed if matched
    pub fn match_at(&self, text: &str) -> Option<usize> {
        let mut chars: Vec<char> = text.chars().collect();
        let mut matches = Vec::new();
        
        // Greedy matching: try to match as many as possible
        let mut pos = 0;
        while pos < chars.len() && self.element.matches(chars[pos]) {
            matches.push(chars[pos]);
            pos += 1;
        }
        
        let match_count = matches.len();
        
        // Check if quantifier constraints are satisfied
        let valid = match self.quantifier {
            Quantifier::ZeroOrMore => true, // Any count is OK
            Quantifier::OneOrMore => match_count >= 1,
            Quantifier::ZeroOrOne => match_count <= 1,
            Quantifier::Exactly(n) => match_count == n,
            Quantifier::AtLeast(n) => match_count >= n,
            Quantifier::Between(min, max) => match_count >= min && match_count <= max,
        };
        
        if valid {
            // Calculate byte length consumed
            let byte_len = matches.iter().map(|c| c.len_utf8()).sum();
            Some(byte_len)
        } else {
            None
        }
    }
    
    /// Find first position in text where this pattern matches
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        
        for (start_byte, _) in &chars {
            if let Some(len) = self.match_at(&text[*start_byte..]) {
                return Some((*start_byte, *start_byte + len));
            }
        }
        
        None
    }
    
    /// Find all matches in text
    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        
        let mut i = 0;
        while i < chars.len() {
            let (start_byte, _) = chars[i];
            
            if let Some(len) = self.match_at(&text[start_byte..]) {
                if len > 0 {
                    results.push((start_byte, start_byte + len));
                    // Skip past the match
                    let end_byte = start_byte + len;
                    while i < chars.len() && chars[i].0 < end_byte {
                        i += 1;
                    }
                } else {
                    // Zero-length match (e.g., a* matching empty)
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        results
    }
}

/// Parse a simple quantified pattern like "a+", "[0-9]*", "\d+", etc.
pub fn parse_quantified_pattern(pattern: &str) -> Result<QuantifiedPattern, String> {
    if pattern.is_empty() {
        return Err("Empty pattern".to_string());
    }
    
    // Check for escape sequence with quantifier: \d+, \w*, \s?, etc.
    if starts_with_escape(pattern) {
        let (seq, bytes_consumed) = parse_escape(pattern)?;
        let remaining = &pattern[bytes_consumed..];
        
        if !remaining.is_empty() {
            // We have a quantifier after the escape
            let quantifier = parse_quantifier(remaining)?;
            
            // Convert escape to CharClass if possible
            if let Some(cc) = seq.to_char_class() {
                return Ok(QuantifiedPattern {
                    element: QuantifiedElement::CharClass(cc),
                    quantifier,
                });
            }
            
            // Or to literal char
            if let Some(ch) = seq.to_char() {
                return Ok(QuantifiedPattern {
                    element: QuantifiedElement::Char(ch),
                    quantifier,
                });
            }
            
            return Err("Escape sequence cannot be quantified".to_string());
        }
        
        return Err("Escape without quantifier".to_string());
    }
    
    // Check for character class
    if pattern.starts_with('[') {
        let close_idx = pattern.find(']').ok_or("Unclosed character class")?;
        let class_content = &pattern[1..close_idx];
        let char_class = CharClass::parse(class_content)?;
        
        let remaining = &pattern[close_idx + 1..];
        let quantifier = parse_quantifier(remaining)?;
        
        Ok(QuantifiedPattern {
            element: QuantifiedElement::CharClass(char_class),
            quantifier,
        })
    } else if pattern.len() >= 2 {
        // Single character with quantifier
        let ch = pattern.chars().next().unwrap();
        let remaining = &pattern[ch.len_utf8()..];
        let quantifier = parse_quantifier(remaining)?;
        
        Ok(QuantifiedPattern {
            element: QuantifiedElement::Char(ch),
            quantifier,
        })
    } else {
        Err("Invalid pattern format".to_string())
    }
}

fn parse_quantifier(s: &str) -> Result<Quantifier, String> {
    match s {
        "*" => Ok(Quantifier::ZeroOrMore),
        "+" => Ok(Quantifier::OneOrMore),
        "?" => Ok(Quantifier::ZeroOrOne),
        "" => Ok(Quantifier::Exactly(1)), // No quantifier = exactly once
        _ if s.starts_with('{') && s.ends_with('}') => {
            let inner = &s[1..s.len() - 1];
            if let Ok(n) = inner.parse::<usize>() {
                Ok(Quantifier::Exactly(n))
            } else if inner.contains(',') {
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 2 {
                    if parts[1].is_empty() {
                        // {n,}
                        let min = parts[0].parse().map_err(|_| "Invalid number")?;
                        Ok(Quantifier::AtLeast(min))
                    } else {
                        // {n,m}
                        let min = parts[0].parse().map_err(|_| "Invalid min")?;
                        let max = parts[1].parse().map_err(|_| "Invalid max")?;
                        Ok(Quantifier::Between(min, max))
                    }
                } else {
                    Err("Invalid quantifier format".to_string())
                }
            } else {
                Err("Invalid quantifier".to_string())
            }
        }
        _ => Err(format!("Unknown quantifier: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_quantifiers() {
        assert_eq!(parse_quantifier("*").unwrap(), Quantifier::ZeroOrMore);
        assert_eq!(parse_quantifier("+").unwrap(), Quantifier::OneOrMore);
        assert_eq!(parse_quantifier("?").unwrap(), Quantifier::ZeroOrOne);
        assert_eq!(parse_quantifier("{3}").unwrap(), Quantifier::Exactly(3));
        assert_eq!(parse_quantifier("{2,}").unwrap(), Quantifier::AtLeast(2));
        assert_eq!(parse_quantifier("{1,5}").unwrap(), Quantifier::Between(1, 5));
    }
    
    #[test]
    fn test_char_star() {
        let pattern = parse_quantified_pattern("a*").unwrap();
        assert_eq!(pattern.match_at("aaab"), Some(3));
        assert_eq!(pattern.match_at("bbb"), Some(0)); // Zero is valid for *
    }
    
    #[test]
    fn test_char_plus() {
        let pattern = parse_quantified_pattern("a+").unwrap();
        assert_eq!(pattern.match_at("aaab"), Some(3));
        assert!(pattern.match_at("bbb").is_none()); // Need at least one
    }
    
    #[test]
    fn test_char_question() {
        let pattern = parse_quantified_pattern("a?").unwrap();
        assert_eq!(pattern.match_at("ab"), Some(1));
        assert_eq!(pattern.match_at("b"), Some(0)); // Zero is valid for ?
    }
    
    #[test]
    fn test_charclass_star() {
        let pattern = parse_quantified_pattern("[0-9]*").unwrap();
        assert_eq!(pattern.match_at("123abc"), Some(3));
        assert_eq!(pattern.match_at("abc"), Some(0));
    }
    
    #[test]
    fn test_charclass_plus() {
        let pattern = parse_quantified_pattern("[a-z]+").unwrap();
        assert_eq!(pattern.match_at("hello123"), Some(5));
        assert!(pattern.match_at("123").is_none());
    }
    
    #[test]
    fn test_find() {
        let pattern = parse_quantified_pattern("[0-9]+").unwrap();
        assert_eq!(pattern.find("abc123def"), Some((3, 6)));
        assert_eq!(pattern.find("no digits"), None);
    }
    
    #[test]
    fn test_find_all() {
        let pattern = parse_quantified_pattern("[0-9]+").unwrap();
        let matches = pattern.find_all("a1b22c333");
        assert_eq!(matches, vec![(1, 2), (3, 5), (6, 9)]);
    }
}
