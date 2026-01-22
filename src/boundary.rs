//! Word boundary matching implementation
//! 
//! Supports:
//! - \b - word boundary (transition between \w and \W)
//! - \B - non-word boundary (NOT at word boundary)

/// Word boundary type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryType {
    /// \b - word boundary
    Word,
    /// \B - non-word boundary
    NonWord,
}

impl BoundaryType {
    /// Check if position is at a word boundary
    /// 
    /// Word boundary occurs at:
    /// - Start of text followed by word char
    /// - End of text preceded by word char
    /// - Between word char and non-word char
    /// - Between non-word char and word char
    #[inline]
    pub fn is_at_boundary(text: &str, pos: usize) -> bool {
        let bytes = text.as_bytes();
        
        // Check characters before and after position
        let before_is_word = if pos == 0 {
            false
        } else {
            Self::is_word_byte(bytes[pos - 1])
        };
        
        let after_is_word = if pos >= bytes.len() {
            false
        } else {
            Self::is_word_byte(bytes[pos])
        };
        
        // Boundary = transition between word/non-word
        before_is_word != after_is_word
    }
    
    /// Check if this boundary type matches at position
    #[inline]
    pub fn matches_at(&self, text: &str, pos: usize) -> bool {
        let is_boundary = Self::is_at_boundary(text, pos);
        match self {
            BoundaryType::Word => is_boundary,
            BoundaryType::NonWord => !is_boundary,
        }
    }
    
    /// Check if byte is a word character [a-zA-Z0-9_]
    #[inline(always)]
    fn is_word_byte(b: u8) -> bool {
        (b >= b'a' && b <= b'z') || 
        (b >= b'A' && b <= b'Z') || 
        (b >= b'0' && b <= b'9') || 
        b == b'_'
    }
    
    /// Find first position that matches this boundary in text
    pub fn find_first(&self, text: &str) -> Option<usize> {
        let bytes = text.as_bytes();
        
        // Empty text has no boundaries or non-boundaries
        if bytes.is_empty() {
            return None;
        }
        
        // Check position 0 (start of text)
        if self.matches_at(text, 0) {
            return Some(0);
        }
        
        // Check each position between chars
        for i in 1..bytes.len() {
            if self.matches_at(text, i) {
                return Some(i);
            }
        }
        
        // Check end of text
        if self.matches_at(text, bytes.len()) {
            return Some(bytes.len());
        }
        
        None
    }
    
    /// Find all positions that match this boundary in text
    pub fn find_all(&self, text: &str) -> Vec<usize> {
        let bytes = text.as_bytes();
        let mut positions = Vec::new();
        
        // Empty text has no boundaries or non-boundaries
        if bytes.is_empty() {
            return positions;
        }
        
        // Check position 0
        if self.matches_at(text, 0) {
            positions.push(0);
        }
        
        // Check each position between chars
        for i in 1..bytes.len() {
            if self.matches_at(text, i) {
                positions.push(i);
            }
        }
        
        // Check end of text
        if self.matches_at(text, bytes.len()) {
            positions.push(bytes.len());
        }
        
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_boundary() {
        let text = "hello world";
        
        // Word boundaries at: 0 (start), 5 (between o and space), 6 (between space and w), 11 (end)
        assert!(BoundaryType::Word.matches_at(text, 0));  // Start
        assert!(BoundaryType::Word.matches_at(text, 5));  // "hello|_world"
        assert!(BoundaryType::Word.matches_at(text, 6));  // "hello_|world"
        assert!(BoundaryType::Word.matches_at(text, 11)); // End
        
        // Not boundaries
        assert!(!BoundaryType::Word.matches_at(text, 1)); // "h|ello"
        assert!(!BoundaryType::Word.matches_at(text, 7)); // "w|orld"
    }
    
    #[test]
    fn test_non_word_boundary() {
        let text = "hello world";
        
        // Non-word boundaries (opposite of word boundaries)
        assert!(!BoundaryType::NonWord.matches_at(text, 0));  // Start
        assert!(!BoundaryType::NonWord.matches_at(text, 5));  
        assert!(!BoundaryType::NonWord.matches_at(text, 6));  
        assert!(!BoundaryType::NonWord.matches_at(text, 11)); // End
        
        // Inside words = non-word boundary
        assert!(BoundaryType::NonWord.matches_at(text, 1)); // "h|ello"
        assert!(BoundaryType::NonWord.matches_at(text, 7)); // "w|orld"
    }
    
    #[test]
    fn test_find_all_boundaries() {
        let text = "hello world";
        let boundaries = BoundaryType::Word.find_all(text);
        assert_eq!(boundaries, vec![0, 5, 6, 11]);
    }
    
    #[test]
    fn test_boundary_with_punctuation() {
        let text = "hello, world!";
        
        // Boundaries: 0 (start), 5 (o|,), 7 (,_|w), 12 (d|!)
        assert!(BoundaryType::Word.matches_at(text, 0));
        assert!(BoundaryType::Word.matches_at(text, 5));  // "hello|,"
        assert!(BoundaryType::Word.matches_at(text, 7));  // ", |world"
        assert!(BoundaryType::Word.matches_at(text, 12)); // "world|!"
        // Note: Position 13 (after !) is NOT a boundary (non-word followed by end)
    }
    
    #[test]
    fn test_boundary_at_start_end() {
        // Word at start
        assert!(BoundaryType::Word.matches_at("hello", 0));
        assert!(BoundaryType::Word.matches_at("hello", 5));
        
        // Non-word at start
        assert!(!BoundaryType::Word.matches_at(" hello", 0));
        assert!(BoundaryType::Word.matches_at(" hello", 1));
    }
}
