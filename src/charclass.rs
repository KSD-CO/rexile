//! Character class matching implementation
//! 
//! Supports: [abc], [a-z], [0-9], [^abc] (negation), [A-Za-z0-9_]

/// Represents a character class pattern like [a-z] or [^0-9]
#[derive(Debug, Clone)]
pub struct CharClass {
    /// Individual characters to match (e.g., 'a', 'b', 'c' from [abc])
    chars: Vec<char>,
    /// Character ranges to match (e.g., ('a', 'z') from [a-z])
    ranges: Vec<(char, char)>,
    /// If true, matches anything NOT in chars/ranges
    negated: bool,
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
        
        let mut pattern_chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;
        
        while i < pattern_chars.len() {
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
    
    /// Check if a character matches this character class
    pub fn matches(&self, ch: char) -> bool {
        let ch_val = ch as u32;
        
        // Fast path: ASCII bitmap lookup
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
