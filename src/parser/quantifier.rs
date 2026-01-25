//! Quantifier support: *, +, ?, {n,m}
//!
//! Implements a simple backtracking matcher for quantified patterns

use crate::parser::charclass::CharClass;
use crate::parser::escape::{parse_escape, starts_with_escape};

/// Represents a quantified pattern element
#[derive(Debug, Clone, PartialEq)]
pub enum QuantifiedElement {
    /// A literal character
    Char(char),
    /// A character class like [a-z]
    CharClass(CharClass),
}

impl QuantifiedElement {
    /// Check if a character matches this element (OPTIMIZED with fast paths)
    #[inline(always)]
    pub fn matches(&self, ch: char) -> bool {
        match self {
            QuantifiedElement::Char(c) => *c == ch,
            QuantifiedElement::CharClass(cc) => cc.matches(ch),
        }
    }

    /// Fast check for ASCII characters (inlined for performance)
    #[inline(always)]
    pub fn matches_byte(&self, byte: u8) -> bool {
        if byte >= 128 {
            return false; // Non-ASCII, use slow path
        }

        match self {
            QuantifiedElement::Char(c) => (*c as u32) == (byte as u32),
            QuantifiedElement::CharClass(cc) => cc.matches(byte as char),
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
    /// *? - Zero or more (non-greedy/lazy)
    ZeroOrMoreLazy,
    /// +? - One or more (non-greedy/lazy)
    OneOrMoreLazy,
    /// ?? - Zero or one (non-greedy/lazy)
    ZeroOrOneLazy,
}

impl Quantifier {
    /// Check if this quantifier is lazy (non-greedy)
    #[inline]
    pub fn is_lazy(&self) -> bool {
        matches!(
            self,
            Quantifier::ZeroOrMoreLazy | Quantifier::OneOrMoreLazy | Quantifier::ZeroOrOneLazy
        )
    }

    /// Get the minimum number of matches required
    #[inline]
    pub fn min_matches(&self) -> usize {
        match self {
            Quantifier::ZeroOrMore | Quantifier::ZeroOrMoreLazy => 0,
            Quantifier::OneOrMore | Quantifier::OneOrMoreLazy => 1,
            Quantifier::ZeroOrOne | Quantifier::ZeroOrOneLazy => 0,
            Quantifier::Exactly(n) => *n,
            Quantifier::AtLeast(n) => *n,
            Quantifier::Between(min, _) => *min,
        }
    }

    /// Get the maximum number of matches allowed
    #[inline]
    pub fn max_matches(&self) -> usize {
        match self {
            Quantifier::ZeroOrMore | Quantifier::ZeroOrMoreLazy => usize::MAX,
            Quantifier::OneOrMore | Quantifier::OneOrMoreLazy => usize::MAX,
            Quantifier::ZeroOrOne | Quantifier::ZeroOrOneLazy => 1,
            Quantifier::Exactly(n) => *n,
            Quantifier::AtLeast(_) => usize::MAX,
            Quantifier::Between(_, max) => *max,
        }
    }
}

/// A quantified pattern: element + quantifier
#[derive(Debug, Clone, PartialEq)]
pub struct QuantifiedPattern {
    pub element: QuantifiedElement,
    pub quantifier: Quantifier,
}

impl QuantifiedPattern {
    /// Match this quantified pattern at the start of text (OPTIMIZED)
    /// Returns the number of bytes consumed if matched
    pub fn match_at(&self, text: &str) -> Option<usize> {
        let bytes = text.as_bytes();

        // Fast path: ASCII-only text - use byte scanning (SIMD-friendly)
        if bytes.iter().all(|&b| b < 128) {
            let mut byte_len = 0;
            let mut match_count = 0;

            // OPTIMIZED: Direct byte scanning for ASCII
            for &byte in bytes {
                if self.element.matches_byte(byte) {
                    byte_len += 1;
                    match_count += 1;
                } else {
                    break;
                }
            }

            // Check quantifier constraints
            let valid = match self.quantifier {
                Quantifier::ZeroOrMore | Quantifier::ZeroOrMoreLazy => true,
                Quantifier::OneOrMore | Quantifier::OneOrMoreLazy => match_count >= 1,
                Quantifier::ZeroOrOne | Quantifier::ZeroOrOneLazy => match_count <= 1,
                Quantifier::Exactly(n) => match_count == n,
                Quantifier::AtLeast(n) => match_count >= n,
                Quantifier::Between(min, max) => match_count >= min && match_count <= max,
            };

            return if valid { Some(byte_len) } else { None };
        }

        // Slow path: UTF-8 text - scan char by char
        let mut byte_len = 0;
        let mut match_count = 0;

        for ch in text.chars() {
            if self.element.matches(ch) {
                byte_len += ch.len_utf8();
                match_count += 1;
            } else {
                break; // Stop at first non-match
            }
        }

        // Check if quantifier constraints are satisfied
        let valid = match self.quantifier {
            Quantifier::ZeroOrMore | Quantifier::ZeroOrMoreLazy => true, // Any count is OK
            Quantifier::OneOrMore | Quantifier::OneOrMoreLazy => match_count >= 1,
            Quantifier::ZeroOrOne | Quantifier::ZeroOrOneLazy => match_count <= 1,
            Quantifier::Exactly(n) => match_count == n,
            Quantifier::AtLeast(n) => match_count >= n,
            Quantifier::Between(min, max) => match_count >= min && match_count <= max,
        };

        if valid {
            Some(byte_len)
        } else {
            None
        }
    }

    /// Check if this pattern matches anywhere in text (optimized for speed)
    /// Returns immediately on first match without computing position
    pub fn is_match(&self, text: &str) -> bool {
        // Fast path: Try match at start first
        if self.match_at(text).is_some() {
            return true;
        }

        // Only scan forward if no match at start
        let chars: Vec<(usize, char)> = text.char_indices().collect();

        for (start_byte, _) in &chars {
            if *start_byte == 0 {
                continue; // Already tried
            }
            if self.match_at(&text[*start_byte..]).is_some() {
                return true; // Early termination!
            }
        }

        false
    }

    /// Find first position in text where this pattern matches
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        // Handle empty text first - zero-width quantifiers can match at position 0
        if text.is_empty() {
            if let Some(len) = self.match_at(text) {
                return Some((0, len));
            }
            return None;
        }

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

        // Check if it's a dot wildcard
        if ch == '.' {
            // Dot matches any character except newline - use CharClass
            use crate::parser::charclass::CharClass;
            // Create CharClass that excludes newline directly
            let mut dot_class = CharClass::new();
            dot_class.add_char('\n'); // Add newline character
            dot_class.negate(); // Negate to match anything EXCEPT newline
            dot_class.finalize(); // Finalize to build internal structures
            Ok(QuantifiedPattern {
                element: QuantifiedElement::CharClass(dot_class),
                quantifier,
            })
        } else {
            Ok(QuantifiedPattern {
                element: QuantifiedElement::Char(ch),
                quantifier,
            })
        }
    } else {
        Err("Invalid pattern format".to_string())
    }
}

fn parse_quantifier(s: &str) -> Result<Quantifier, String> {
    match s {
        // Greedy quantifiers
        "*" => Ok(Quantifier::ZeroOrMore),
        "+" => Ok(Quantifier::OneOrMore),
        "?" => Ok(Quantifier::ZeroOrOne),
        // Non-greedy (lazy) quantifiers
        "*?" => Ok(Quantifier::ZeroOrMoreLazy),
        "+?" => Ok(Quantifier::OneOrMoreLazy),
        "??" => Ok(Quantifier::ZeroOrOneLazy),
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
        // Handle {n}? and {n,m}? lazy quantifiers
        _ if s.ends_with("?") && s.len() > 1 => {
            // Strip the trailing ? and parse the base quantifier
            let _base = &s[..s.len() - 1];
            // For now, just parse without lazy support for bounded quantifiers
            // This will fall through to the error case
            Err(format!("Lazy bounded quantifiers not yet supported: {}", s))
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
        assert_eq!(
            parse_quantifier("{1,5}").unwrap(),
            Quantifier::Between(1, 5)
        );
    }

    #[test]
    fn test_parse_lazy_quantifiers() {
        assert_eq!(parse_quantifier("*?").unwrap(), Quantifier::ZeroOrMoreLazy);
        assert_eq!(parse_quantifier("+?").unwrap(), Quantifier::OneOrMoreLazy);
        assert_eq!(parse_quantifier("??").unwrap(), Quantifier::ZeroOrOneLazy);
    }

    #[test]
    fn test_quantifier_is_lazy() {
        assert!(!Quantifier::ZeroOrMore.is_lazy());
        assert!(!Quantifier::OneOrMore.is_lazy());
        assert!(!Quantifier::ZeroOrOne.is_lazy());
        assert!(Quantifier::ZeroOrMoreLazy.is_lazy());
        assert!(Quantifier::OneOrMoreLazy.is_lazy());
        assert!(Quantifier::ZeroOrOneLazy.is_lazy());
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
