/// Sequence matching - combine multiple pattern elements
/// 
/// Supports patterns like:
/// - ab+c* (char followed by quantified char followed by quantified char)
/// - \d+\w* (escape sequence followed by quantified escape)
/// - hello\d+ (literal followed by quantified escape)

use crate::charclass::CharClass;
use crate::quantifier::Quantifier;

/// A single element in a sequence
#[derive(Debug, Clone)]
pub enum SequenceElement {
    /// A literal character (e.g., 'a' in "abc")
    Char(char),
    /// A quantified character (e.g., 'a+' in "a+bc")
    QuantifiedChar(char, Quantifier),
    /// A character class (e.g., [a-z])
    CharClass(CharClass),
    /// A quantified character class (e.g., [0-9]+)
    QuantifiedCharClass(CharClass, Quantifier),
    /// A literal string (e.g., "hello" in "hello\d+")
    Literal(String),
}

impl SequenceElement {
    /// Try to match this element at a specific position in text
    /// Returns number of bytes consumed if successful, None otherwise
    pub fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        let remaining = &text[pos..];
        if remaining.is_empty() {
            return None;
        }

        match self {
            SequenceElement::Char(ch) => {
                if remaining.starts_with(*ch) {
                    Some(ch.len_utf8())
                } else {
                    None
                }
            }
            SequenceElement::QuantifiedChar(ch, quantifier) => {
                match_quantified_char(*ch, quantifier, remaining)
            }
            SequenceElement::CharClass(cc) => {
                let first_char = remaining.chars().next()?;
                if cc.matches(first_char) {
                    Some(first_char.len_utf8())
                } else {
                    None
                }
            }
            SequenceElement::QuantifiedCharClass(cc, quantifier) => {
                match_quantified_charclass(cc, quantifier, remaining)
            }
            SequenceElement::Literal(lit) => {
                if remaining.starts_with(lit) {
                    Some(lit.len())
                } else {
                    None
                }
            }
        }
    }
}

/// Match a quantified character
fn match_quantified_char(ch: char, quantifier: &Quantifier, text: &str) -> Option<usize> {
    let (min, max) = quantifier_bounds(quantifier);
    let chars: Vec<char> = text.chars().collect();
    
    // Count matching characters
    let mut count = 0;
    for &c in &chars {
        if c == ch {
            count += 1;
        } else {
            break;
        }
    }
    
    if count < min {
        return None; // Not enough matches
    }
    
    // Greedy: take as many as possible (up to max)
    let actual_count = count.min(max);
    Some(text.chars().take(actual_count).map(|c| c.len_utf8()).sum())
}

/// Match a quantified character class
fn match_quantified_charclass(cc: &CharClass, quantifier: &Quantifier, text: &str) -> Option<usize> {
    let (min, max) = quantifier_bounds(quantifier);
    let chars: Vec<char> = text.chars().collect();
    
    // Count matching characters
    let mut count = 0;
    for &c in &chars {
        if cc.matches(c) {
            count += 1;
        } else {
            break;
        }
    }
    
    if count < min {
        return None; // Not enough matches
    }
    
    // Greedy: take as many as possible (up to max)
    let actual_count = count.min(max);
    Some(text.chars().take(actual_count).map(|c| c.len_utf8()).sum())
}

/// Get min/max bounds for a quantifier
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

/// A sequence of pattern elements
#[derive(Debug, Clone)]
pub struct Sequence {
    pub elements: Vec<SequenceElement>,
}

impl Sequence {
    /// Create a new sequence
    pub fn new(elements: Vec<SequenceElement>) -> Self {
        Sequence { elements }
    }

    /// Check if the sequence matches at the start of text
    /// Returns bytes consumed if match, None otherwise
    pub fn match_at(&self, text: &str) -> Option<usize> {
        let mut pos = 0;
        
        for element in &self.elements {
            match element.match_at(text, pos) {
                Some(consumed) => pos += consumed,
                None => return None,
            }
        }
        
        Some(pos)
    }

    /// Check if the sequence matches anywhere in text (optimized)
    /// Returns immediately on first match without computing position
    pub fn is_match(&self, text: &str) -> bool {
        // Fast path: Try match at start first
        if self.match_at(text).is_some() {
            return true;
        }
        
        // Only scan forward if no match at start
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        
        for &start_pos in &byte_positions {
            if start_pos == 0 {
                continue; // Already tried
            }
            if self.match_at(&text[start_pos..]).is_some() {
                return true; // Early termination!
            }
        }
        
        false
    }

    /// Find the sequence anywhere in text
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        
        for &start_pos in &byte_positions {
            if let Some(consumed) = self.match_at(&text[start_pos..]) {
                return Some((start_pos, start_pos + consumed));
            }
        }
        
        None
    }

    /// Find all occurrences of the sequence in text
    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        
        let mut i = 0;
        while i < byte_positions.len() {
            let start_pos = byte_positions[i];
            
            if let Some(consumed) = self.match_at(&text[start_pos..]) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence() {
        // "abc"
        let seq = Sequence::new(vec![
            SequenceElement::Char('a'),
            SequenceElement::Char('b'),
            SequenceElement::Char('c'),
        ]);
        
        assert_eq!(seq.match_at("abc"), Some(3));
        assert_eq!(seq.match_at("abcdef"), Some(3));
        assert_eq!(seq.match_at("ab"), None);
        assert_eq!(seq.match_at("xyz"), None);
    }

    #[test]
    fn test_quantified_sequence() {
        // "a+b"
        let seq = Sequence::new(vec![
            SequenceElement::QuantifiedChar('a', Quantifier::OneOrMore),
            SequenceElement::Char('b'),
        ]);
        
        assert_eq!(seq.match_at("ab"), Some(2));
        assert_eq!(seq.match_at("aaab"), Some(4));
        assert_eq!(seq.match_at("aaaabcd"), Some(5));
        assert_eq!(seq.match_at("b"), None); // Need at least one 'a'
    }

    #[test]
    fn test_charclass_sequence() {
        // "[0-9]+[a-z]"
        let mut digits = CharClass::new();
        digits.add_range('0', '9');
        digits.finalize();
        
        let mut letters = CharClass::new();
        letters.add_range('a', 'z');
        letters.finalize();
        
        let seq = Sequence::new(vec![
            SequenceElement::QuantifiedCharClass(digits, Quantifier::OneOrMore),
            SequenceElement::CharClass(letters),
        ]);
        
        assert_eq!(seq.match_at("123a"), Some(4));
        assert_eq!(seq.match_at("9z"), Some(2));
        assert_eq!(seq.match_at("abc"), None); // No digits
    }

    #[test]
    fn test_find() {
        // "ab+"
        let seq = Sequence::new(vec![
            SequenceElement::Char('a'),
            SequenceElement::QuantifiedChar('b', Quantifier::OneOrMore),
        ]);
        
        assert_eq!(seq.find("xyzabbc"), Some((3, 6)));
        assert_eq!(seq.find("nope"), None);
    }

    #[test]
    fn test_find_all() {
        // "a+b"
        let seq = Sequence::new(vec![
            SequenceElement::QuantifiedChar('a', Quantifier::OneOrMore),
            SequenceElement::Char('b'),
        ]);
        
        let matches = seq.find_all("ab aab aaab");
        assert_eq!(matches, vec![(0, 2), (3, 6), (7, 11)]);
    }

    #[test]
    fn test_literal_sequence() {
        // "hello[0-9]+"
        let mut digits = CharClass::new();
        digits.add_range('0', '9');
        digits.finalize();
        
        let seq = Sequence::new(vec![
            SequenceElement::Literal("hello".to_string()),
            SequenceElement::QuantifiedCharClass(digits, Quantifier::OneOrMore),
        ]);
        
        assert_eq!(seq.match_at("hello123"), Some(8));
        assert_eq!(seq.match_at("hello"), None); // Need digits
    }
}
