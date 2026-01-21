/// Group support for regex patterns
/// 
/// Supports:
/// - Simple groups: (abc)
/// - Non-capturing groups: (?:abc)
/// - Alternation in groups: (a|b|c)
/// - Quantified groups: (abc)+

use crate::sequence::{Sequence, SequenceElement};
use crate::quantifier::Quantifier;

/// A group in a pattern
#[derive(Debug, Clone)]
pub struct Group {
    /// The content of the group (can be alternation or sequence)
    pub content: GroupContent,
    /// Whether this is a capturing group
    pub capturing: bool,
    /// Optional quantifier on the group
    pub quantifier: Option<Quantifier>,
}

/// Content inside a group
#[derive(Debug, Clone)]
pub enum GroupContent {
    /// Single pattern (like "abc" in (abc))
    Single(String),
    /// Alternation (like "a|b|c" in (a|b|c))
    Alternation(Vec<String>),
    /// Nested sequence
    Sequence(Sequence),
}

impl Group {
    /// Create a new capturing group
    pub fn new_capturing(content: GroupContent) -> Self {
        Group {
            content,
            capturing: true,
            quantifier: None,
        }
    }

    /// Create a new non-capturing group
    pub fn new_non_capturing(content: GroupContent) -> Self {
        Group {
            content,
            capturing: false,
            quantifier: None,
        }
    }

    /// Add a quantifier to this group
    pub fn with_quantifier(mut self, quantifier: Quantifier) -> Self {
        self.quantifier = Some(quantifier);
        self
    }

    /// Check if text matches this group at a given position
    /// Returns bytes consumed if match
    pub fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        let base_consumed = self.match_base_at(text, pos)?;

        // Apply quantifier if present
        if let Some(quantifier) = &self.quantifier {
            self.match_with_quantifier(text, pos, base_consumed, quantifier)
        } else {
            Some(base_consumed)
        }
    }

    /// Match group base pattern (without quantifier) at position
    fn match_base_at(&self, text: &str, pos: usize) -> Option<usize> {
        let remaining = &text[pos..];

        match &self.content {
            GroupContent::Single(pattern) => {
                if remaining.starts_with(pattern) {
                    Some(pattern.len())
                } else {
                    None
                }
            }
            GroupContent::Alternation(patterns) => {
                // Try each alternative
                for pattern in patterns {
                    if remaining.starts_with(pattern) {
                        return Some(pattern.len());
                    }
                }
                None
            }
            GroupContent::Sequence(seq) => {
                seq.match_at(remaining)
            }
        }
    }

    /// Match group with quantifier
    fn match_with_quantifier(
        &self,
        text: &str,
        start_pos: usize,
        base_match_size: usize,
        quantifier: &Quantifier,
    ) -> Option<usize> {
        let (min, max) = quantifier_bounds(quantifier);
        
        let mut total_consumed = 0;
        let mut count = 0;
        let mut pos = start_pos;

        // Greedy: match as many times as possible
        while count < max {
            match self.match_base_at(text, pos) {
                Some(consumed) if consumed > 0 => {
                    total_consumed += consumed;
                    pos += consumed;
                    count += 1;
                }
                _ => break,
            }
        }

        if count >= min {
            Some(total_consumed)
        } else {
            None
        }
    }

    /// Find the group pattern anywhere in text
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        
        for &start_pos in &byte_positions {
            if let Some(consumed) = self.match_at(text, start_pos) {
                return Some((start_pos, start_pos + consumed));
            }
        }
        
        None
    }

    /// Find all occurrences of the group in text
    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        
        let mut i = 0;
        while i < byte_positions.len() {
            let start_pos = byte_positions[i];
            
            if let Some(consumed) = self.match_at(text, start_pos) {
                let end_pos = start_pos + consumed;
                results.push((start_pos, end_pos));
                
                // Skip past this match
                while i < byte_positions.len() && byte_positions[i] < end_pos {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        results
    }
}

fn quantifier_bounds(q: &Quantifier) -> (usize, usize) {
    match q {
        Quantifier::ZeroOrMore => (0, usize::MAX),
        Quantifier::OneOrMore => (1, usize::MAX),
        Quantifier::ZeroOrOne => (0, 1),
        Quantifier::Exactly(n) => (*n, *n),
        Quantifier::AtLeast(n) => (*n, usize::MAX),
        Quantifier::Between(n, m) => (*n, *m),
    }
}

/// Parse a group from a pattern string
/// Returns (Group, bytes_consumed)
pub fn parse_group(pattern: &str) -> Result<(Group, usize), String> {
    if !pattern.starts_with('(') {
        return Err("Pattern must start with '('".to_string());
    }

    // Find matching closing paren
    let mut depth = 0;
    let mut close_idx = None;
    
    for (i, ch) in pattern.char_indices() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                close_idx = Some(i);
                break;
            }
        }
    }

    let close_idx = close_idx.ok_or("Unclosed group")?;
    
    // Extract group content
    let group_str = &pattern[1..close_idx];
    
    // Check if non-capturing group
    let (is_capturing, content_str) = if group_str.starts_with("?:") {
        (false, &group_str[2..])
    } else {
        (true, group_str)
    };

    // Parse group content
    let content = if content_str.contains('|') {
        // Alternation
        let parts: Vec<String> = content_str.split('|').map(|s| s.to_string()).collect();
        GroupContent::Alternation(parts)
    } else {
        // Single pattern
        GroupContent::Single(content_str.to_string())
    };

    let group = if is_capturing {
        Group::new_capturing(content)
    } else {
        Group::new_non_capturing(content)
    };

    let mut bytes_consumed = close_idx + 1;

    // Check for quantifier after group
    if bytes_consumed < pattern.len() {
        let remaining = &pattern[bytes_consumed..];
        if let Some(ch) = remaining.chars().next() {
            if let Some(quantifier) = parse_simple_quantifier(ch) {
                bytes_consumed += ch.len_utf8();
                return Ok((group.with_quantifier(quantifier), bytes_consumed));
            }
        }
    }

    Ok((group, bytes_consumed))
}

fn parse_simple_quantifier(ch: char) -> Option<Quantifier> {
    match ch {
        '*' => Some(Quantifier::ZeroOrMore),
        '+' => Some(Quantifier::OneOrMore),
        '?' => Some(Quantifier::ZeroOrOne),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_group() {
        let (group, len) = parse_group("(abc)").unwrap();
        assert_eq!(len, 5);
        assert!(group.capturing);
        assert!(group.quantifier.is_none());
    }

    #[test]
    fn test_parse_non_capturing() {
        let (group, len) = parse_group("(?:abc)").unwrap();
        assert_eq!(len, 7);
        assert!(!group.capturing);
    }

    #[test]
    fn test_parse_alternation() {
        let (group, _) = parse_group("(a|b|c)").unwrap();
        match group.content {
            GroupContent::Alternation(parts) => {
                assert_eq!(parts.len(), 3);
                assert_eq!(parts, vec!["a", "b", "c"]);
            }
            _ => panic!("Expected alternation"),
        }
    }

    #[test]
    fn test_parse_quantified_group() {
        let (group, len) = parse_group("(abc)+").unwrap();
        assert_eq!(len, 6);
        assert!(group.quantifier.is_some());
    }

    #[test]
    fn test_match_simple() {
        let group = Group::new_capturing(GroupContent::Single("abc".to_string()));
        assert_eq!(group.match_at("abc", 0), Some(3));
        assert_eq!(group.match_at("xyzabc", 3), Some(3));
        assert_eq!(group.match_at("xyz", 0), None);
    }

    #[test]
    fn test_match_alternation() {
        let group = Group::new_capturing(GroupContent::Alternation(vec![
            "foo".to_string(),
            "bar".to_string(),
        ]));
        
        assert_eq!(group.match_at("foo", 0), Some(3));
        assert_eq!(group.match_at("bar", 0), Some(3));
        assert_eq!(group.match_at("baz", 0), None);
    }

    #[test]
    fn test_find() {
        let group = Group::new_capturing(GroupContent::Single("abc".to_string()));
        assert_eq!(group.find("xyzabcdef"), Some((3, 6)));
    }

    #[test]
    fn test_find_all() {
        let group = Group::new_capturing(GroupContent::Single("ab".to_string()));
        let matches = group.find_all("ab cd ab ef ab");
        assert_eq!(matches, vec![(0, 2), (6, 8), (12, 14)]);
    }
}
