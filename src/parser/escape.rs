use crate::parser::boundary::BoundaryType;
/// Escape sequence support for ReXile
///
/// Supports:
/// - Character classes: \d, \w, \s, \D, \W, \S
/// - Special chars: \n, \t, \r
/// - Word boundaries: \b, \B
/// - Literal escapes: \., \*, \\, \+, \?, \[, \], \(, \), \|, \^, \$
use crate::parser::charclass::CharClass;

#[derive(Debug, Clone, PartialEq)]
pub enum EscapeSequence {
    /// \d - digits [0-9]
    Digit,
    /// \D - non-digits [^0-9]
    NonDigit,
    /// \w - word characters [a-zA-Z0-9_]
    Word,
    /// \W - non-word characters [^a-zA-Z0-9_]
    NonWord,
    /// \s - whitespace [ \t\n\r]
    Whitespace,
    /// \S - non-whitespace [^ \t\n\r]
    NonWhitespace,
    /// \b - word boundary
    WordBoundary,
    /// \B - non-word boundary
    NonWordBoundary,
    /// \n - newline
    Newline,
    /// \t - tab
    Tab,
    /// \r - carriage return
    CarriageReturn,
    /// \. or \* or \\ etc - literal character
    Literal(char),
}

impl EscapeSequence {
    /// Convert escape sequence to CharClass
    pub fn to_char_class(&self) -> Option<CharClass> {
        match self {
            EscapeSequence::Digit => {
                // \d = [0-9]
                let mut cc = CharClass::new();
                cc.add_range('0', '9');
                cc.finalize();
                Some(cc)
            }
            EscapeSequence::NonDigit => {
                // \D = [^0-9]
                let mut cc = CharClass::new();
                cc.add_range('0', '9');
                cc.negate();
                cc.finalize();
                Some(cc)
            }
            EscapeSequence::Word => {
                // \w = [a-zA-Z0-9_]
                let mut cc = CharClass::new();
                cc.add_range('a', 'z');
                cc.add_range('A', 'Z');
                cc.add_range('0', '9');
                cc.add_char('_');
                cc.finalize();
                Some(cc)
            }
            EscapeSequence::NonWord => {
                // \W = [^a-zA-Z0-9_]
                let mut cc = CharClass::new();
                cc.add_range('a', 'z');
                cc.add_range('A', 'Z');
                cc.add_range('0', '9');
                cc.add_char('_');
                cc.negate();
                cc.finalize();
                Some(cc)
            }
            EscapeSequence::Whitespace => {
                // \s = [ \t\n\r]
                let mut cc = CharClass::new();
                cc.add_char(' ');
                cc.add_char('\t');
                cc.add_char('\n');
                cc.add_char('\r');
                cc.finalize();
                Some(cc)
            }
            EscapeSequence::NonWhitespace => {
                // \S = [^ \t\n\r]
                let mut cc = CharClass::new();
                cc.add_char(' ');
                cc.add_char('\t');
                cc.add_char('\n');
                cc.add_char('\r');
                cc.negate();
                cc.finalize();
                Some(cc)
            }
            _ => None, // Literals and boundaries don't convert to CharClass
        }
    }

    /// Convert escape sequence to BoundaryType
    pub fn to_boundary(&self) -> Option<BoundaryType> {
        match self {
            EscapeSequence::WordBoundary => Some(BoundaryType::Word),
            EscapeSequence::NonWordBoundary => Some(BoundaryType::NonWord),
            _ => None,
        }
    }

    /// Get the literal character for literal escapes
    pub fn to_char(&self) -> Option<char> {
        match self {
            EscapeSequence::Newline => Some('\n'),
            EscapeSequence::Tab => Some('\t'),
            EscapeSequence::CarriageReturn => Some('\r'),
            EscapeSequence::Literal(ch) => Some(*ch),
            _ => None,
        }
    }
}

/// Parse an escape sequence from a pattern string (OPTIMIZED)
/// Returns (EscapeSequence, bytes_consumed)
#[inline]
pub fn parse_escape(pattern: &str) -> Result<(EscapeSequence, usize), String> {
    let bytes = pattern.as_bytes();

    if bytes.is_empty() || bytes[0] != b'\\' {
        return Err("Pattern must start with backslash".to_string());
    }

    if bytes.len() < 2 {
        return Err("Incomplete escape sequence".to_string());
    }

    let escape_char = bytes[1] as char;
    let bytes_consumed = 2; // \d, \w, etc are always 2 bytes in ASCII

    let seq = match escape_char {
        'd' => EscapeSequence::Digit,
        'D' => EscapeSequence::NonDigit,
        'w' => EscapeSequence::Word,
        'W' => EscapeSequence::NonWord,
        's' => EscapeSequence::Whitespace,
        'S' => EscapeSequence::NonWhitespace,
        'b' => EscapeSequence::WordBoundary,    // NEW: \b
        'B' => EscapeSequence::NonWordBoundary, // NEW: \B
        'n' => EscapeSequence::Newline,
        't' => EscapeSequence::Tab,
        'r' => EscapeSequence::CarriageReturn,
        // Literal escapes for regex metacharacters
        '.' | '*' | '+' | '?' | '[' | ']' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '\\' => {
            EscapeSequence::Literal(escape_char)
        }
        _ => return Err(format!("Unknown escape sequence: \\{}", escape_char)),
    };

    Ok((seq, bytes_consumed))
}

/// Check if a pattern starts with an escape sequence (OPTIMIZED)
#[inline(always)]
pub fn starts_with_escape(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    bytes.len() >= 2 && bytes[0] == b'\\'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_digit() {
        let (seq, len) = parse_escape("\\d").unwrap();
        assert_eq!(seq, EscapeSequence::Digit);
        assert_eq!(len, 2);

        let cc = seq.to_char_class().unwrap();
        assert!(cc.matches('0'));
        assert!(cc.matches('9'));
        assert!(!cc.matches('a'));
    }

    #[test]
    fn test_parse_non_digit() {
        let (seq, len) = parse_escape("\\D").unwrap();
        assert_eq!(seq, EscapeSequence::NonDigit);
        assert_eq!(len, 2);

        let cc = seq.to_char_class().unwrap();
        assert!(!cc.matches('0'));
        assert!(cc.matches('a'));
        assert!(cc.matches('Z'));
    }

    #[test]
    fn test_parse_word() {
        let (seq, _) = parse_escape("\\w").unwrap();
        let cc = seq.to_char_class().unwrap();

        assert!(cc.matches('a'));
        assert!(cc.matches('Z'));
        assert!(cc.matches('0'));
        assert!(cc.matches('_'));
        assert!(!cc.matches(' '));
        assert!(!cc.matches('.'));
    }

    #[test]
    fn test_parse_whitespace() {
        let (seq, _) = parse_escape("\\s").unwrap();
        let cc = seq.to_char_class().unwrap();

        assert!(cc.matches(' '));
        assert!(cc.matches('\t'));
        assert!(cc.matches('\n'));
        assert!(cc.matches('\r'));
        assert!(!cc.matches('a'));
    }

    #[test]
    fn test_parse_non_whitespace() {
        let (seq, _) = parse_escape("\\S").unwrap();
        let cc = seq.to_char_class().unwrap();

        assert!(!cc.matches(' '));
        assert!(!cc.matches('\t'));
        assert!(cc.matches('a'));
        assert!(cc.matches('0'));
    }

    #[test]
    fn test_parse_boundaries() {
        let (seq, len) = parse_escape("\\b").unwrap();
        assert_eq!(seq, EscapeSequence::WordBoundary);
        assert_eq!(len, 2);
        assert!(seq.to_boundary().is_some());

        let (seq, len) = parse_escape("\\B").unwrap();
        assert_eq!(seq, EscapeSequence::NonWordBoundary);
        assert_eq!(len, 2);
        assert!(seq.to_boundary().is_some());
    }

    #[test]
    fn test_parse_special_chars() {
        let (seq, _) = parse_escape("\\n").unwrap();
        assert_eq!(seq.to_char(), Some('\n'));

        let (seq, _) = parse_escape("\\t").unwrap();
        assert_eq!(seq.to_char(), Some('\t'));

        let (seq, _) = parse_escape("\\r").unwrap();
        assert_eq!(seq.to_char(), Some('\r'));
    }

    #[test]
    fn test_parse_literal_escapes() {
        let (seq, _) = parse_escape("\\.").unwrap();
        assert_eq!(seq, EscapeSequence::Literal('.'));
        assert_eq!(seq.to_char(), Some('.'));

        let (seq, _) = parse_escape("\\*").unwrap();
        assert_eq!(seq, EscapeSequence::Literal('*'));

        let (seq, _) = parse_escape("\\\\").unwrap();
        assert_eq!(seq, EscapeSequence::Literal('\\'));

        let (seq, _) = parse_escape("\\+").unwrap();
        assert_eq!(seq, EscapeSequence::Literal('+'));
    }

    #[test]
    fn test_unknown_escape() {
        let result = parse_escape("\\x");
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_escape() {
        let result = parse_escape("\\");
        assert!(result.is_err());
    }

    #[test]
    fn test_starts_with_escape() {
        assert!(starts_with_escape("\\d"));
        assert!(starts_with_escape("\\w+"));
        assert!(!starts_with_escape("hello"));
        assert!(!starts_with_escape("\\"));
    }
}
