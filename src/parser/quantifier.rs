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
    /// {n}? - Exactly n times (non-greedy/lazy)
    ExactlyLazy(usize),
    /// {n,}? - At least n times (non-greedy/lazy)
    AtLeastLazy(usize),
    /// {n,m}? - Between n and m times (non-greedy/lazy)
    BetweenLazy(usize, usize),
}

impl Quantifier {
    /// Check if this quantifier is lazy (non-greedy)
    #[inline]
    pub fn is_lazy(&self) -> bool {
        matches!(
            self,
            Quantifier::ZeroOrMoreLazy
                | Quantifier::OneOrMoreLazy
                | Quantifier::ZeroOrOneLazy
                | Quantifier::ExactlyLazy(_)
                | Quantifier::AtLeastLazy(_)
                | Quantifier::BetweenLazy(_, _)
        )
    }

    /// Get the minimum number of matches required
    #[inline]
    pub fn min_matches(&self) -> usize {
        match self {
            Quantifier::ZeroOrMore | Quantifier::ZeroOrMoreLazy => 0,
            Quantifier::OneOrMore | Quantifier::OneOrMoreLazy => 1,
            Quantifier::ZeroOrOne | Quantifier::ZeroOrOneLazy => 0,
            Quantifier::Exactly(n) | Quantifier::ExactlyLazy(n) => *n,
            Quantifier::AtLeast(n) | Quantifier::AtLeastLazy(n) => *n,
            Quantifier::Between(min, _) | Quantifier::BetweenLazy(min, _) => *min,
        }
    }

    /// Get the maximum number of matches allowed
    #[inline]
    pub fn max_matches(&self) -> usize {
        match self {
            Quantifier::ZeroOrMore | Quantifier::ZeroOrMoreLazy => usize::MAX,
            Quantifier::OneOrMore | Quantifier::OneOrMoreLazy => usize::MAX,
            Quantifier::ZeroOrOne | Quantifier::ZeroOrOneLazy => 1,
            Quantifier::Exactly(n) | Quantifier::ExactlyLazy(n) => *n,
            Quantifier::AtLeast(_) | Quantifier::AtLeastLazy(_) => usize::MAX,
            Quantifier::Between(_, max) | Quantifier::BetweenLazy(_, max) => *max,
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
        let min = self.quantifier.min_matches();
        let max = self.quantifier.max_matches();

        // Try byte-level scanning first (works for ASCII chars)
        // No pre-scan of entire text - just scan until we hit non-ASCII or stop
        let mut byte_len = 0;
        let mut match_count = 0;

        for &byte in bytes {
            if match_count >= max {
                break;
            }
            if byte >= 128 {
                // Hit non-ASCII, fall through to char-based path for remainder
                let remaining = &text[byte_len..];
                for ch in remaining.chars() {
                    if match_count >= max {
                        break;
                    }
                    if self.element.matches(ch) {
                        byte_len += ch.len_utf8();
                        match_count += 1;
                    } else {
                        break;
                    }
                }
                return if match_count >= min {
                    if self.quantifier.is_lazy() {
                        Some(text.chars().take(min).map(char::len_utf8).sum())
                    } else {
                        Some(byte_len)
                    }
                } else {
                    None
                };
            }
            if self.element.matches_byte(byte) {
                byte_len += 1;
                match_count += 1;
            } else {
                break;
            }
        }

        if match_count >= min {
            if self.quantifier.is_lazy() {
                Some(text.chars().take(min).map(|ch| ch.len_utf8()).sum())
            } else {
                Some(byte_len)
            }
        } else {
            None
        }
    }

    /// Check if this pattern matches anywhere in text (optimized for speed)
    /// Returns immediately on first match without computing position
    pub fn is_match(&self, text: &str) -> bool {
        // OPTIMIZATION: Use the optimized find() method which has fast paths
        // This avoids scanning every position character-by-character
        self.find(text).is_some()
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

        if self.quantifier.min_matches() == 0 {
            return self.match_at(text).map(|len| (0, len));
        }

        // OPTIMIZATION: Fast path for digit patterns
        if matches!(&self.element, QuantifiedElement::CharClass(cc) if cc.is_digit_class()) {
            // Use memchr to find first digit
            let bytes = text.as_bytes();
            for (i, &b) in bytes.iter().enumerate() {
                if b.is_ascii_digit() {
                    // Found a digit, try to match from here
                    if let Some(len) = self.match_at(&text[i..]) {
                        return Some((i, i + len));
                    }
                }
            }
            return None;
        }

        // OPTIMIZATION: Fast path for word char patterns
        if matches!(&self.element, QuantifiedElement::CharClass(cc) if cc.is_word_class()) {
            // Scan for first word char
            let bytes = text.as_bytes();
            for (i, &b) in bytes.iter().enumerate() {
                if b.is_ascii_alphanumeric() || b == b'_' {
                    if let Some(len) = self.match_at(&text[i..]) {
                        return Some((i, i + len));
                    }
                }
            }
            return None;
        }

        // Generic path for other patterns
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
        let mut pos = 0;
        let mut last_match_was_non_empty = false;

        while pos <= text.len() {
            if let Some(len) = self.match_at(&text[pos..]) {
                let end = pos + len;

                if len == 0 && last_match_was_non_empty {
                    last_match_was_non_empty = false;
                    if let Some(next_pos) = next_char_boundary(text, pos) {
                        pos = next_pos;
                        continue;
                    }
                    break;
                }

                results.push((pos, end));
                last_match_was_non_empty = len > 0;

                if len > 0 {
                    pos = end;
                } else if let Some(next_pos) = next_char_boundary(text, pos) {
                    pos = next_pos;
                } else {
                    break;
                }
            } else if let Some(next_pos) = next_char_boundary(text, pos) {
                last_match_was_non_empty = false;
                pos = next_pos;
            } else {
                break;
            }
        }

        results
    }
}

fn next_char_boundary(text: &str, pos: usize) -> Option<usize> {
    if pos >= text.len() {
        return None;
    }

    text[pos..].chars().next().map(|ch| pos + ch.len_utf8())
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
        let close_idx = find_class_end(pattern).ok_or("Unclosed character class")?;
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

fn find_class_end(pattern: &str) -> Option<usize> {
    let mut escaped = false;

    for (idx, ch) in pattern.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            ']' => return Some(idx),
            _ => {}
        }
    }

    None
}

fn parse_quantifier(s: &str) -> Result<Quantifier, String> {
    if s.is_empty() {
        return Ok(Quantifier::Exactly(1)); // No quantifier = exactly once
    }
    match parse_quantifier_at(s)? {
        Some((q, len)) if len == s.len() => Ok(q),
        Some(_) => Err(format!("Unexpected characters after quantifier: {}", s)),
        None => Err(format!("Unknown quantifier: {}", s)),
    }
}

/// Parse a quantifier at the start of `s`, returning the quantifier and number of bytes consumed.
///
/// Returns `Ok(None)` if `s` does not begin with a quantifier token (`*`, `+`, `?`, `{`).
/// Returns an error if the quantifier syntax is malformed (e.g., `{2,1}` or unclosed `{`).
pub(crate) fn parse_quantifier_at(s: &str) -> Result<Option<(Quantifier, usize)>, String> {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return Ok(None);
    };

    match first {
        '*' => {
            if chars.next() == Some('?') {
                Ok(Some((Quantifier::ZeroOrMoreLazy, 2)))
            } else {
                Ok(Some((Quantifier::ZeroOrMore, 1)))
            }
        }
        '+' => {
            if chars.next() == Some('?') {
                Ok(Some((Quantifier::OneOrMoreLazy, 2)))
            } else {
                Ok(Some((Quantifier::OneOrMore, 1)))
            }
        }
        '?' => {
            if chars.next() == Some('?') {
                Ok(Some((Quantifier::ZeroOrOneLazy, 2)))
            } else {
                Ok(Some((Quantifier::ZeroOrOne, 1)))
            }
        }
        '{' => {
            let close_idx = s
                .find('}')
                .ok_or_else(|| "Unclosed quantifier".to_string())?;
            let inner = &s[1..close_idx];
            let has_lazy = s[close_idx + 1..].starts_with('?');
            let end_idx = if has_lazy {
                close_idx + 2
            } else {
                close_idx + 1
            };

            let quantifier = if let Ok(n) = inner.parse::<usize>() {
                if has_lazy {
                    Quantifier::ExactlyLazy(n)
                } else {
                    Quantifier::Exactly(n)
                }
            } else if let Some((min_str, max_str)) = inner.split_once(',') {
                if max_str.is_empty() {
                    let min = min_str.parse().map_err(|_| "Invalid number")?;
                    if has_lazy {
                        Quantifier::AtLeastLazy(min)
                    } else {
                        Quantifier::AtLeast(min)
                    }
                } else {
                    let min = min_str.parse().map_err(|_| "Invalid min")?;
                    let max = max_str.parse().map_err(|_| "Invalid max")?;
                    if min > max {
                        return Err("Quantifier minimum exceeds maximum".to_string());
                    }
                    if has_lazy {
                        Quantifier::BetweenLazy(min, max)
                    } else {
                        Quantifier::Between(min, max)
                    }
                }
            } else {
                return Err("Invalid quantifier".to_string());
            };

            Ok(Some((quantifier, end_idx)))
        }
        _ => Ok(None),
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
        assert!(parse_quantifier("{2,1}").is_err());
    }

    #[test]
    fn test_parse_lazy_quantifiers() {
        assert_eq!(parse_quantifier("*?").unwrap(), Quantifier::ZeroOrMoreLazy);
        assert_eq!(parse_quantifier("+?").unwrap(), Quantifier::OneOrMoreLazy);
        assert_eq!(parse_quantifier("??").unwrap(), Quantifier::ZeroOrOneLazy);
        assert_eq!(
            parse_quantifier("{3}?").unwrap(),
            Quantifier::ExactlyLazy(3)
        );
        assert_eq!(
            parse_quantifier("{2,}?").unwrap(),
            Quantifier::AtLeastLazy(2)
        );
        assert_eq!(
            parse_quantifier("{1,5}?").unwrap(),
            Quantifier::BetweenLazy(1, 5)
        );
    }

    #[test]
    fn test_quantifier_is_lazy() {
        assert!(!Quantifier::ZeroOrMore.is_lazy());
        assert!(!Quantifier::OneOrMore.is_lazy());
        assert!(!Quantifier::ZeroOrOne.is_lazy());
        assert!(!Quantifier::Exactly(3).is_lazy());
        assert!(!Quantifier::AtLeast(2).is_lazy());
        assert!(!Quantifier::Between(1, 5).is_lazy());
        assert!(Quantifier::ZeroOrMoreLazy.is_lazy());
        assert!(Quantifier::OneOrMoreLazy.is_lazy());
        assert!(Quantifier::ZeroOrOneLazy.is_lazy());
        assert!(Quantifier::ExactlyLazy(3).is_lazy());
        assert!(Quantifier::AtLeastLazy(2).is_lazy());
        assert!(Quantifier::BetweenLazy(1, 5).is_lazy());
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

    #[test]
    fn test_parse_quantifier_at() {
        assert_eq!(parse_quantifier_at("").unwrap(), None);
        assert_eq!(parse_quantifier_at("abc").unwrap(), None);
        assert_eq!(
            parse_quantifier_at("*rest").unwrap(),
            Some((Quantifier::ZeroOrMore, 1))
        );
        assert_eq!(
            parse_quantifier_at("*?rest").unwrap(),
            Some((Quantifier::ZeroOrMoreLazy, 2))
        );
        assert_eq!(
            parse_quantifier_at("+rest").unwrap(),
            Some((Quantifier::OneOrMore, 1))
        );
        assert_eq!(
            parse_quantifier_at("+?rest").unwrap(),
            Some((Quantifier::OneOrMoreLazy, 2))
        );
        assert_eq!(
            parse_quantifier_at("?rest").unwrap(),
            Some((Quantifier::ZeroOrOne, 1))
        );
        assert_eq!(
            parse_quantifier_at("??rest").unwrap(),
            Some((Quantifier::ZeroOrOneLazy, 2))
        );
        assert_eq!(
            parse_quantifier_at("{3}rest").unwrap(),
            Some((Quantifier::Exactly(3), 3))
        );
        assert_eq!(
            parse_quantifier_at("{2,}rest").unwrap(),
            Some((Quantifier::AtLeast(2), 4))
        );
        assert_eq!(
            parse_quantifier_at("{1,5}rest").unwrap(),
            Some((Quantifier::Between(1, 5), 5))
        );
        assert_eq!(
            parse_quantifier_at("{3}?rest").unwrap(),
            Some((Quantifier::ExactlyLazy(3), 4))
        );
        assert_eq!(
            parse_quantifier_at("{2,}?rest").unwrap(),
            Some((Quantifier::AtLeastLazy(2), 5))
        );
        assert_eq!(
            parse_quantifier_at("{1,5}?rest").unwrap(),
            Some((Quantifier::BetweenLazy(1, 5), 6))
        );
        assert!(parse_quantifier_at("{2,1}rest").is_err());
        assert!(parse_quantifier_at("{2,1}?rest").is_err());
        assert!(parse_quantifier_at("{invalid}rest").is_err());
        assert!(parse_quantifier_at("{unclosed").is_err());
    }
}
