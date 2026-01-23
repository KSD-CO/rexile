//! Lookaround assertions (lookahead and lookbehind)
//!
//! Lookaround assertions are zero-width assertions that check if a pattern
//! matches ahead or behind the current position without consuming characters.
//!
//! # Lookahead
//! - `(?=pattern)` - Positive lookahead: succeeds if pattern matches ahead
//! - `(?!pattern)` - Negative lookahead: succeeds if pattern does NOT match ahead
//!
//! # Lookbehind
//! - `(?<=pattern)` - Positive lookbehind: succeeds if pattern matches behind
//! - `(?<!pattern)` - Negative lookbehind: succeeds if pattern does NOT match behind
//!
//! # Examples
//! ```
//! use rexile::Pattern;
//!
//! // Positive lookahead: match 'foo' only if followed by 'bar'
//! let pattern = Pattern::new(r"foo(?=bar)").unwrap();
//! assert!(pattern.is_match("foobar"));
//! assert!(!pattern.is_match("foobaz"));
//!
//! // Negative lookahead: match 'foo' only if NOT followed by 'bar'
//! let pattern = Pattern::new(r"foo(?!bar)").unwrap();
//! assert!(pattern.is_match("foobaz"));
//! assert!(!pattern.is_match("foobar"));
//! ```

use crate::{Ast, Matcher};

/// Type of lookaround assertion
#[derive(Debug, Clone, PartialEq)]
pub enum LookaroundType {
    /// Positive lookahead (?=...)
    PositiveLookahead,
    /// Negative lookahead (?!...)
    NegativeLookahead,
    /// Positive lookbehind (?<=...)
    PositiveLookbehind,
    /// Negative lookbehind (?<!...)
    NegativeLookbehind,
}

/// A lookaround assertion with its type and inner pattern
#[derive(Debug, Clone, PartialEq)]
pub struct Lookaround {
    pub lookaround_type: LookaroundType,
    pub pattern: Box<Ast>,
}

impl Lookaround {
    /// Create a new lookaround assertion
    pub fn new(lookaround_type: LookaroundType, pattern: Ast) -> Self {
        Self {
            lookaround_type,
            pattern: Box::new(pattern),
        }
    }

    /// Check if the lookaround matches at the given position
    ///
    /// For lookahead: checks if pattern matches starting at `pos`
    /// For lookbehind: checks if pattern matches ending at `pos`
    ///
    /// Returns true if the assertion succeeds (pattern matches for positive,
    /// or doesn't match for negative lookaround)
    pub fn matches_at(&self, text: &str, pos: usize, matcher: &Matcher) -> bool {
        match self.lookaround_type {
            LookaroundType::PositiveLookahead => {
                // Check if pattern matches starting at pos
                self.check_lookahead(text, pos, matcher, true)
            }
            LookaroundType::NegativeLookahead => {
                // Check if pattern does NOT match starting at pos
                self.check_lookahead(text, pos, matcher, false)
            }
            LookaroundType::PositiveLookbehind => {
                // Check if pattern matches ending at pos
                self.check_lookbehind(text, pos, matcher, true)
            }
            LookaroundType::NegativeLookbehind => {
                // Check if pattern does NOT match ending at pos
                self.check_lookbehind(text, pos, matcher, false)
            }
        }
    }

    /// Check lookahead assertion
    fn check_lookahead(&self, text: &str, pos: usize, matcher: &Matcher, positive: bool) -> bool {
        if pos > text.len() {
            return false;
        }

        // Try to match pattern starting at pos
        let matches = self.pattern_matches_at(text, pos, matcher);

        // For positive lookahead, return true if matches
        // For negative lookahead, return true if does NOT match
        if positive {
            matches
        } else {
            !matches
        }
    }

    /// Check lookbehind assertion
    fn check_lookbehind(&self, text: &str, pos: usize, matcher: &Matcher, positive: bool) -> bool {
        if pos > text.len() {
            return false;
        }

        // For lookbehind, we need to check all possible starting positions
        // that could end at `pos`
        let matches = self.find_match_ending_at(text, pos, matcher);

        // For positive lookbehind, return true if matches
        // For negative lookbehind, return true if does NOT match
        if positive {
            matches
        } else {
            !matches
        }
    }

    /// Check if pattern matches at the given position
    fn pattern_matches_at(&self, text: &str, pos: usize, matcher: &Matcher) -> bool {
        // Check if the pattern matches AT THE START of text[pos..]
        if pos > text.len() {
            return false;
        }

        let remaining = &text[pos..];
        // Use find() and check if it starts at position 0
        if let Some((start, _end)) = matcher.find(remaining) {
            start == 0
        } else {
            false
        }
    }

    /// Find if any match ends exactly at the given position
    fn find_match_ending_at(&self, text: &str, pos: usize, matcher: &Matcher) -> bool {
        // Try all possible starting positions that could end at `pos`
        for start in 0..=pos {
            if self.check_match_span(text, start, pos, matcher) {
                return true;
            }
        }
        false
    }

    /// Check if pattern matches exactly from start to end position
    fn check_match_span(&self, text: &str, start: usize, end: usize, matcher: &Matcher) -> bool {
        if start > text.len() || end > text.len() || start > end {
            return false;
        }

        let span = &text[start..end];

        // Check if the pattern matches exactly this span
        if let Some((match_start, match_end)) = matcher.find(span) {
            // Must match the entire span (start at 0, end at span.len())
            match_start == 0 && match_end == span.len()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Matcher;

    #[test]
    fn test_positive_lookahead() {
        let lookaround = Lookaround::new(
            LookaroundType::PositiveLookahead,
            Ast::Literal("bar".to_string()),
        );
        let matcher = Matcher::Literal("bar".to_string());

        // "foobar" at position 3 - lookahead should match "bar"
        assert!(lookaround.matches_at("foobar", 3, &matcher));

        // "foobaz" at position 3 - lookahead should not match
        assert!(!lookaround.matches_at("foobaz", 3, &matcher));
    }

    #[test]
    fn test_negative_lookahead() {
        let lookaround = Lookaround::new(
            LookaroundType::NegativeLookahead,
            Ast::Literal("bar".to_string()),
        );
        let matcher = Matcher::Literal("bar".to_string());

        // "foobaz" at position 3 - negative lookahead should succeed (bar not ahead)
        assert!(lookaround.matches_at("foobaz", 3, &matcher));

        // "foobar" at position 3 - negative lookahead should fail (bar is ahead)
        assert!(!lookaround.matches_at("foobar", 3, &matcher));
    }

    #[test]
    fn test_positive_lookbehind() {
        let lookaround = Lookaround::new(
            LookaroundType::PositiveLookbehind,
            Ast::Literal("foo".to_string()),
        );
        let matcher = Matcher::Literal("foo".to_string());

        // "foobar" at position 3 - lookbehind should match "foo"
        assert!(lookaround.matches_at("foobar", 3, &matcher));

        // "bazbar" at position 3 - lookbehind should not match
        assert!(!lookaround.matches_at("bazbar", 3, &matcher));
    }

    #[test]
    fn test_negative_lookbehind() {
        let lookaround = Lookaround::new(
            LookaroundType::NegativeLookbehind,
            Ast::Literal("foo".to_string()),
        );
        let matcher = Matcher::Literal("foo".to_string());

        // "bazbar" at position 3 - negative lookbehind should succeed (foo not behind)
        assert!(lookaround.matches_at("bazbar", 3, &matcher));

        // "foobar" at position 3 - negative lookbehind should fail (foo is behind)
        assert!(!lookaround.matches_at("foobar", 3, &matcher));
    }
}
