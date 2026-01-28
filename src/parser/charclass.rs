//! Character class matching implementation
//!
//! Supports: [abc], [a-z], [0-9], [^abc] (negation), [A-Za-z0-9_]

/// Represents a character class pattern like [a-z] or [^0-9]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharClass {
    /// Individual characters to match (e.g., 'a', 'b', 'c' from [abc])
    pub(crate) chars: Vec<char>, // Made pub(crate) for optimization checks
    /// Character ranges to match (e.g., ('a', 'z') from [a-z])
    pub(crate) ranges: Vec<(char, char)>, // Made pub(crate) for optimization checks
    /// If true, matches anything NOT in chars/ranges
    pub(crate) negated: bool, // Made pub(crate) for optimization checks
    /// ASCII fast path: bitmap for ASCII characters (0-127)
    ascii_bitmap: Option<[u64; 2]>, // 128 bits = 2 x u64
}

impl CharClass {
    /// Create a new empty character class
    pub fn new() -> Self {
        CharClass {
            chars: Vec::new(),
            ranges: Vec::new(),
            negated: false,
            ascii_bitmap: None,
        }
    }

    /// Add a single character to the class
    pub fn add_char(&mut self, ch: char) {
        self.chars.push(ch);
        self.ascii_bitmap = None; // Invalidate bitmap
    }

    /// Add a character range to the class
    pub fn add_range(&mut self, start: char, end: char) {
        self.ranges.push((start, end));
        self.ascii_bitmap = None; // Invalidate bitmap
    }

    /// Negate this character class
    pub fn negate(&mut self) {
        self.negated = !self.negated;
    }

    /// Finalize the character class by building optimizations
    pub fn finalize(&mut self) {
        self.build_ascii_bitmap();
    }

    /// Parse character class from string like "a-z" or "^0-9"
    pub fn parse(pattern: &str) -> Result<Self, String> {
        if pattern.is_empty() {
            return Err("Empty character class".to_string());
        }

        let mut chars = Vec::new();
        let mut ranges = Vec::new();
        let negated = pattern.starts_with('^');
        let pattern = if negated { &pattern[1..] } else { pattern };

        let pattern_chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;

        while i < pattern_chars.len() {
            // Check for escape sequences like \s, \d, \w
            if pattern_chars[i] == '\\' && i + 1 < pattern_chars.len() {
                let escape_char = pattern_chars[i + 1];
                match escape_char {
                    's' => {
                        // Whitespace: space, tab, newline, carriage return, form feed, vertical tab
                        chars.push(' ');
                        chars.push('\t');
                        chars.push('\n');
                        chars.push('\r');
                        chars.push('\x0C'); // form feed
                        chars.push('\x0B'); // vertical tab
                        i += 2;
                    }
                    'd' => {
                        // Digits 0-9
                        ranges.push(('0', '9'));
                        i += 2;
                    }
                    'w' => {
                        // Word characters: a-z, A-Z, 0-9, _
                        ranges.push(('a', 'z'));
                        ranges.push(('A', 'Z'));
                        ranges.push(('0', '9'));
                        chars.push('_');
                        i += 2;
                    }
                    _ => {
                        // Other escapes like \., \-, etc. - treat as literal
                        chars.push(escape_char);
                        i += 2;
                    }
                }
                continue;
            }

            // Check for range (a-z)
            if i + 2 < pattern_chars.len() && pattern_chars[i + 1] == '-' {
                let start = pattern_chars[i];
                let end = pattern_chars[i + 2];

                if start > end {
                    return Err(format!("Invalid range: {}-{}", start, end));
                }

                ranges.push((start, end));
                i += 3;
            } else {
                // Single character
                chars.push(pattern_chars[i]);
                i += 1;
            }
        }

        let mut cc = CharClass {
            chars,
            ranges,
            negated,
            ascii_bitmap: None,
        };

        // Build ASCII bitmap for fast matching
        cc.build_ascii_bitmap();

        Ok(cc)
    }

    /// Build bitmap for ASCII characters (0-127) for fast lookup
    fn build_ascii_bitmap(&mut self) {
        let mut bitmap = [0u64; 2]; // 128 bits

        // Set bits for individual chars
        for &ch in &self.chars {
            if (ch as u32) < 128 {
                let idx = ch as usize;
                bitmap[idx / 64] |= 1u64 << (idx % 64);
            }
        }

        // Set bits for ranges
        for &(start, end) in &self.ranges {
            let start_val = start as u32;
            let end_val = end as u32;

            if start_val < 128 {
                let end_ascii = end_val.min(127);
                for ch in start_val..=end_ascii {
                    let idx = ch as usize;
                    bitmap[idx / 64] |= 1u64 << (idx % 64);
                }
            }
        }

        self.ascii_bitmap = Some(bitmap);
    }

    /// Get the pre-computed ASCII bitmap for direct inline lookup
    #[inline(always)]
    pub fn get_ascii_bitmap(&self) -> Option<&[u64; 2]> {
        self.ascii_bitmap.as_ref()
    }

    /// Check if an ASCII byte matches using pre-extracted bitmap (no function call overhead)
    #[inline(always)]
    pub fn matches_byte_bitmap(bitmap: &[u64; 2], negated: bool, byte: u8) -> bool {
        let idx = byte as usize;
        let bit_set = (bitmap[idx / 64] & (1u64 << (idx % 64))) != 0;
        if negated {
            !bit_set
        } else {
            bit_set
        }
    }

    /// Check if a character matches this character class
    #[inline]
    pub fn matches(&self, ch: char) -> bool {
        let ch_val = ch as u32;

        // Fast path: ASCII bitmap lookup (uses bit operations, very fast)
        if ch_val < 128 {
            if let Some(bitmap) = &self.ascii_bitmap {
                let idx = ch_val as usize;
                let bit_set = (bitmap[idx / 64] & (1u64 << (idx % 64))) != 0;
                return if self.negated { !bit_set } else { bit_set };
            }
        }

        // Slow path: check ranges and individual chars
        let mut matched = false;

        // Check individual characters
        if self.chars.contains(&ch) {
            matched = true;
        }

        // Check ranges
        if !matched {
            for &(start, end) in &self.ranges {
                if ch >= start && ch <= end {
                    matched = true;
                    break;
                }
            }
        }

        // Apply negation
        if self.negated {
            !matched
        } else {
            matched
        }
    }

    /// Check if this is a digit-only character class [0-9]
    pub fn is_digit_class(&self) -> bool {
        !self.negated
            && self.chars.is_empty()
            && self.ranges.len() == 1
            && self.ranges[0] == ('0', '9')
    }

    /// Check if this is a word character class [a-zA-Z0-9_]
    pub fn is_word_class(&self) -> bool {
        if self.negated {
            return false;
        }
        // Check if it's exactly \w (word chars: a-z, A-Z, 0-9, _)
        let has_lowercase = self.ranges.contains(&('a', 'z'));
        let has_uppercase = self.ranges.contains(&('A', 'Z'));
        let has_digits = self.ranges.contains(&('0', '9'));
        let has_underscore = self.chars.contains(&'_');

        // Must have exactly these components
        has_lowercase
            && has_uppercase
            && has_digits
            && has_underscore
            && self.ranges.len() == 3
            && self.chars.len() == 1
    }

    /// Check if this is a whitespace-only character class \s
    pub fn is_whitespace_class(&self) -> bool {
        !self.negated
            && self.chars.contains(&' ')
            && self.chars.contains(&'\t')
            && self.chars.contains(&'\n')
            && self.chars.contains(&'\r')
            && self.ranges.is_empty()
    }

    /// Check if this character class overlaps with another
    /// (i.e., there exists at least one character matching both)
    pub fn overlaps_with(&self, other: &CharClass) -> bool {
        // If either is negated, they almost certainly overlap
        // (negated classes match the vast majority of characters)
        if self.negated || other.negated {
            return true;
        }

        // Check if any range in self overlaps with any range in other
        for &(s1, e1) in &self.ranges {
            for &(s2, e2) in &other.ranges {
                if s1 <= e2 && s2 <= e1 {
                    return true;
                }
            }
            // Check if any char in other falls within self's ranges
            for &ch in &other.chars {
                if ch >= s1 && ch <= e1 {
                    return true;
                }
            }
        }

        // Check if any char in self falls within other's ranges
        for &ch in &self.chars {
            for &(s2, e2) in &other.ranges {
                if ch >= s2 && ch <= e2 {
                    return true;
                }
            }
            // Check if any char matches directly
            if other.chars.contains(&ch) {
                return true;
            }
        }

        false
    }

    /// Find first character in text that matches this class (SIMD-optimized for ASCII)
    /// Returns byte position if found, None otherwise
    pub fn find_first(&self, text: &str) -> Option<usize> {
        let bytes = text.as_bytes();

        // Fast path for ASCII-only text with bitmap
        if let Some(bitmap) = &self.ascii_bitmap {
            // Check if text is ASCII-only by scanning in chunks
            if bytes.iter().all(|&b| b < 128) {
                // SIMD-friendly: Process bytes directly using bitmap
                for (idx, &byte) in bytes.iter().enumerate() {
                    let bit_set = (bitmap[byte as usize / 64] & (1u64 << (byte % 64))) != 0;
                    let matches = if self.negated { !bit_set } else { bit_set };
                    if matches {
                        return Some(idx);
                    }
                }
                return None;
            }
        }

        // Fallback: UTF-8 aware character-by-character scan
        for (idx, ch) in text.char_indices() {
            if self.matches(ch) {
                return Some(idx);
            }
        }
        None
    }

    /// Check if text starts with a character matching this class
    /// Returns Some(char_len) if matched, None otherwise
    pub fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        let ch = text[pos..].chars().next()?;
        if self.matches(ch) {
            Some(ch.len_utf8())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_chars() {
        let cc = CharClass::parse("abc").unwrap();
        assert!(cc.matches('a'));
        assert!(cc.matches('b'));
        assert!(cc.matches('c'));
        assert!(!cc.matches('d'));
        assert!(!cc.matches('z'));
    }

    #[test]
    fn test_range() {
        let cc = CharClass::parse("a-z").unwrap();
        assert!(cc.matches('a'));
        assert!(cc.matches('m'));
        assert!(cc.matches('z'));
        assert!(!cc.matches('A'));
        assert!(!cc.matches('0'));
    }

    #[test]
    fn test_multiple_ranges() {
        let cc = CharClass::parse("a-zA-Z0-9").unwrap();
        assert!(cc.matches('a'));
        assert!(cc.matches('Z'));
        assert!(cc.matches('5'));
        assert!(!cc.matches('!'));
        assert!(!cc.matches(' '));
    }

    #[test]
    fn test_negation() {
        let cc = CharClass::parse("^abc").unwrap();
        assert!(!cc.matches('a'));
        assert!(!cc.matches('b'));
        assert!(!cc.matches('c'));
        assert!(cc.matches('d'));
        assert!(cc.matches('z'));
        assert!(cc.matches('1'));
    }

    #[test]
    fn test_negated_range() {
        let cc = CharClass::parse("^0-9").unwrap();
        assert!(!cc.matches('0'));
        assert!(!cc.matches('5'));
        assert!(!cc.matches('9'));
        assert!(cc.matches('a'));
        assert!(cc.matches('Z'));
    }

    #[test]
    fn test_mixed() {
        let cc = CharClass::parse("a-z_0-9").unwrap();
        assert!(cc.matches('a'));
        assert!(cc.matches('_'));
        assert!(cc.matches('5'));
        assert!(!cc.matches('A'));
        assert!(!cc.matches('-'));
    }

    #[test]
    fn test_ascii_bitmap() {
        let cc = CharClass::parse("a-z").unwrap();
        // Should use ASCII bitmap for fast lookup
        assert!(cc.ascii_bitmap.is_some());
        assert!(cc.matches('a'));
        assert!(cc.matches('z'));
    }
}
