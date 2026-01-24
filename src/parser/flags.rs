//! Regex flags support: (?s), (?i), (?m)
//!
//! Implements inline flag parsing and flag-aware matching.
//!
//! Supported flags:
//! - `(?i)` - Case-insensitive matching
//! - `(?m)` - Multi-line mode: ^ and $ match line boundaries
//! - `(?s)` - Single-line/DOTALL mode: . matches newlines
//!
//! Flags can be combined: `(?ims)` enables all three flags.

/// Regex flags that modify matching behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    /// Case-insensitive matching (`(?i)`)
    pub case_insensitive: bool,
    /// Multi-line mode (`(?m)`): ^ and $ match at line boundaries
    pub multiline: bool,
    /// Single-line/DOTALL mode (`(?s)`): . matches newlines
    pub dot_matches_newline: bool,
}

impl Flags {
    /// Create new empty flags
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any flag is set
    pub fn any_set(&self) -> bool {
        self.case_insensitive || self.multiline || self.dot_matches_newline
    }

    /// Parse flags from a pattern string like `(?ims)`
    /// Returns (Flags, remaining_pattern) or None if no flags found
    pub fn parse_from_pattern(pattern: &str) -> Option<(Self, &str)> {
        // Check for inline flags at start: (?...)
        if !pattern.starts_with("(?") {
            return None;
        }

        // Find the closing parenthesis
        let close_idx = pattern.find(')')?;
        let flags_str = &pattern[2..close_idx];

        // Check if this is actually a flags group (not lookahead, etc.)
        // Flags can only contain i, m, s, x, or - (for turning off)
        if flags_str.is_empty() {
            return None;
        }

        // Check for special groups that aren't flags
        let first_char = flags_str.chars().next()?;
        match first_char {
            '=' | '!' | '<' | ':' | '#' | '>' | 'P' => {
                // These are special groups, not flags
                // (?=...) positive lookahead
                // (?!...) negative lookahead
                // (?<=...) positive lookbehind
                // (?<!...) negative lookbehind
                // (?:...) non-capturing group
                // (?#...) comment
                // (?>...) atomic group
                // (?P<name>...) named capture
                return None;
            }
            _ => {}
        }

        // Parse flags
        let mut flags = Flags::new();
        let mut has_flags = false;

        for ch in flags_str.chars() {
            match ch {
                'i' => {
                    flags.case_insensitive = true;
                    has_flags = true;
                }
                'm' => {
                    flags.multiline = true;
                    has_flags = true;
                }
                's' => {
                    flags.dot_matches_newline = true;
                    has_flags = true;
                }
                // Ignore other valid flag modifiers for now
                'x' | 'U' | '-' => {
                    has_flags = true;
                }
                _ => {
                    // Invalid flag character - this might not be a flags group
                    return None;
                }
            }
        }

        if !has_flags {
            return None;
        }

        let remaining = &pattern[close_idx + 1..];
        Some((flags, remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_flag() {
        let (flags, rest) = Flags::parse_from_pattern("(?i)hello").unwrap();
        assert!(flags.case_insensitive);
        assert!(!flags.multiline);
        assert!(!flags.dot_matches_newline);
        assert_eq!(rest, "hello");
    }

    #[test]
    fn test_parse_multiple_flags() {
        let (flags, rest) = Flags::parse_from_pattern("(?ims)pattern").unwrap();
        assert!(flags.case_insensitive);
        assert!(flags.multiline);
        assert!(flags.dot_matches_newline);
        assert_eq!(rest, "pattern");
    }

    #[test]
    fn test_parse_dotall_flag() {
        let (flags, rest) = Flags::parse_from_pattern("(?s)a.*b").unwrap();
        assert!(!flags.case_insensitive);
        assert!(!flags.multiline);
        assert!(flags.dot_matches_newline);
        assert_eq!(rest, "a.*b");
    }

    #[test]
    fn test_no_flags() {
        assert!(Flags::parse_from_pattern("hello").is_none());
        assert!(Flags::parse_from_pattern("(?:hello)").is_none());
        assert!(Flags::parse_from_pattern("(?=lookahead)").is_none());
    }
}
