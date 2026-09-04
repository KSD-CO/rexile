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

    /// Parse one leading global flag group such as `(?ims)`.
    ///
    /// Only `i`, `m`, and `s` are supported. Other inline flag syntax must
    /// fail instead of being accepted and silently ignored.
    pub fn parse_from_pattern(pattern: &str) -> Result<Option<(Self, &str)>, String> {
        // Check for inline flags at start: (?...)
        if !pattern.starts_with("(?") {
            return Ok(None);
        }

        // Find the closing parenthesis
        let Some(close_idx) = pattern.find(')') else {
            return Ok(None);
        };
        let flags_str = &pattern[2..close_idx];

        // Check if this is actually a flags group rather than a lookaround or
        // another special group.
        if flags_str.is_empty() {
            return Ok(None);
        }

        // Check for special groups that aren't flags
        let Some(first_char) = flags_str.chars().next() else {
            return Ok(None);
        };
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
                return Ok(None);
            }
            _ => {}
        }

        if flags_str.contains(':') {
            return Err("scoped inline flags are not supported".to_string());
        }

        // Parse flags.
        let mut flags = Flags::new();

        for ch in flags_str.chars() {
            match ch {
                'i' => {
                    flags.case_insensitive = true;
                }
                'm' => {
                    flags.multiline = true;
                }
                's' => {
                    flags.dot_matches_newline = true;
                }
                'x' | 'U' | 'u' | 'R' => {
                    return Err(format!("inline flag `{}` is not supported", ch));
                }
                '-' => return Err("inline flag disabling is not supported".to_string()),
                _ => {
                    return Err(format!("unsupported inline flag syntax `(?{})`", flags_str));
                }
            }
        }

        let remaining = &pattern[close_idx + 1..];
        Ok(Some((flags, remaining)))
    }

    /// Merge another global flag group into this one.
    pub fn merge(&mut self, other: Self) {
        self.case_insensitive |= other.case_insensitive;
        self.multiline |= other.multiline;
        self.dot_matches_newline |= other.dot_matches_newline;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_flag() {
        let (flags, rest) = Flags::parse_from_pattern("(?i)hello").unwrap().unwrap();
        assert!(flags.case_insensitive);
        assert!(!flags.multiline);
        assert!(!flags.dot_matches_newline);
        assert_eq!(rest, "hello");
    }

    #[test]
    fn test_parse_multiple_flags() {
        let (flags, rest) = Flags::parse_from_pattern("(?ims)pattern").unwrap().unwrap();
        assert!(flags.case_insensitive);
        assert!(flags.multiline);
        assert!(flags.dot_matches_newline);
        assert_eq!(rest, "pattern");
    }

    #[test]
    fn test_parse_dotall_flag() {
        let (flags, rest) = Flags::parse_from_pattern("(?s)a.*b").unwrap().unwrap();
        assert!(!flags.case_insensitive);
        assert!(!flags.multiline);
        assert!(flags.dot_matches_newline);
        assert_eq!(rest, "a.*b");
    }

    #[test]
    fn test_no_flags() {
        assert!(Flags::parse_from_pattern("hello").unwrap().is_none());
        assert!(Flags::parse_from_pattern("(?:hello)").unwrap().is_none());
        assert!(Flags::parse_from_pattern("(?=lookahead)")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_unsupported_flags_fail() {
        for pattern in ["(?x)hello", "(?U)hello", "(?-i)hello", "(?i:hello)"] {
            assert!(Flags::parse_from_pattern(pattern).is_err(), "{pattern}");
        }
    }
}
