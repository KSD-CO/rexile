//! Capture groups for extracting matched substrings
//!
//! Capture groups allow you to extract parts of a matched string for later use.
//! They are defined using parentheses in the pattern.
//!
//! # Types of Groups
//! - `(pattern)` - Capturing group: captures the matched substring
//! - `(?:pattern)` - Non-capturing group: groups pattern without capturing
//!
//! # Backreferences
//! - `\1`, `\2`, etc. - Reference to previously captured group
//!
//! # Examples
//! ```
//! use rexile::Pattern;
//!
//! // Extract date components
//! let pattern = Pattern::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
//! let text = "Date: 2026-01-22";
//!
//! if let Some(caps) = pattern.captures(text) {
//!     println!("Year: {}", &caps[1]);  // 2026
//!     println!("Month: {}", &caps[2]); // 01
//!     println!("Day: {}", &caps[3]);   // 22
//! }
//! ```

use std::ops::Index;

/// A capture group in the pattern
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// Index of the capture group (0 = full match, 1+ = capture groups)
    pub index: usize,
    /// Whether this is a capturing group (false for non-capturing (?:...))
    pub is_capturing: bool,
    /// Name of the group (for named captures like (?P`<name>`...))
    pub name: Option<String>,
}

impl Group {
    /// Create a new capturing group
    pub fn new(index: usize) -> Self {
        Self {
            index,
            is_capturing: true,
            name: None,
        }
    }

    /// Create a new non-capturing group
    pub fn non_capturing() -> Self {
        Self {
            index: 0,
            is_capturing: false,
            name: None,
        }
    }

    /// Create a new named capturing group
    pub fn named(index: usize, name: String) -> Self {
        Self {
            index,
            is_capturing: true,
            name: Some(name),
        }
    }
}

/// A set of captured substrings from a single match
#[derive(Debug, Clone)]
pub struct Captures<'t> {
    /// The original text that was matched against
    text: &'t str,
    /// Vector of captured substring positions (start, end)
    /// Index 0 is always the full match
    /// Indices 1+ are the capture groups
    positions: Vec<Option<(usize, usize)>>,
}

impl<'t> Captures<'t> {
    /// Create a new Captures with the full match
    pub fn new(text: &'t str, full_match: (usize, usize), num_groups: usize) -> Self {
        let mut positions = vec![None; num_groups + 1];
        positions[0] = Some(full_match);
        Self { text, positions }
    }

    /// Get the matched substring for a capture group
    ///
    /// Index 0 returns the full match, indices 1+ return capture groups
    pub fn get(&self, index: usize) -> Option<&'t str> {
        self.positions
            .get(index)?
            .map(|(start, end)| &self.text[start..end])
    }

    /// Get the position (start, end) of a capture group
    pub fn pos(&self, index: usize) -> Option<(usize, usize)> {
        self.positions.get(index).and_then(|&pos| pos)
    }

    /// Get the full matched text (equivalent to get(0))
    pub fn as_str(&self) -> &'t str {
        self.get(0).unwrap_or("")
    }

    /// Set a capture group position
    pub(crate) fn set(&mut self, index: usize, start: usize, end: usize) {
        if let Some(slot) = self.positions.get_mut(index) {
            *slot = Some((start, end));
        }
    }

    /// Number of capture groups (including the full match at index 0)
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if there are no capture groups
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Iterate over all captured substrings
    pub fn iter(&self) -> CapturesIter<'_, 't> {
        CapturesIter {
            captures: self,
            index: 0,
        }
    }
}

/// Allow indexing Captures by group number
impl<'t> Index<usize> for Captures<'t> {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("no capture group at index {}", index))
    }
}

/// Iterator over captured substrings
pub struct CapturesIter<'c, 't> {
    captures: &'c Captures<'t>,
    index: usize,
}

impl<'c, 't> Iterator for CapturesIter<'c, 't> {
    type Item = Option<&'t str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.captures.len() {
            return None;
        }
        let result = self.captures.get(self.index);
        self.index += 1;
        Some(result)
    }
}

/// Iterator that yields Captures for each match in a text
pub struct CapturesMatches<'r, 't> {
    text: &'t str,
    last_end: usize,
    num_groups: usize,
    // This would hold a reference to the compiled pattern
    // For now, we'll keep it simple
    _phantom: std::marker::PhantomData<&'r ()>,
}

impl<'r, 't> CapturesMatches<'r, 't> {
    /// Create a new captures iterator
    pub fn new(text: &'t str, num_groups: usize) -> Self {
        Self {
            text,
            last_end: 0,
            num_groups,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'r, 't> Iterator for CapturesMatches<'r, 't> {
    type Item = Captures<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        // This is a placeholder - actual implementation would use the pattern
        // to find the next match starting from last_end
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captures_basic() {
        let text = "Hello, world!";
        let caps = Captures::new(text, (0, 5), 0);

        assert_eq!(caps.get(0), Some("Hello"));
        assert_eq!(caps.as_str(), "Hello");
        assert_eq!(caps.len(), 1);
    }

    #[test]
    fn test_captures_groups() {
        let text = "2026-01-22";
        let mut caps = Captures::new(text, (0, 10), 3);
        caps.set(1, 0, 4); // Year
        caps.set(2, 5, 7); // Month
        caps.set(3, 8, 10); // Day

        assert_eq!(caps.get(0), Some("2026-01-22"));
        assert_eq!(caps.get(1), Some("2026"));
        assert_eq!(caps.get(2), Some("01"));
        assert_eq!(caps.get(3), Some("22"));
        assert_eq!(caps.len(), 4);
    }

    #[test]
    fn test_captures_indexing() {
        let text = "foo=123";
        let mut caps = Captures::new(text, (0, 7), 2);
        caps.set(1, 0, 3); // "foo"
        caps.set(2, 4, 7); // "123"

        assert_eq!(&caps[0], "foo=123");
        assert_eq!(&caps[1], "foo");
        assert_eq!(&caps[2], "123");
    }

    #[test]
    fn test_captures_pos() {
        let text = "abc123";
        let mut caps = Captures::new(text, (0, 6), 2);
        caps.set(1, 0, 3);
        caps.set(2, 3, 6);

        assert_eq!(caps.pos(0), Some((0, 6)));
        assert_eq!(caps.pos(1), Some((0, 3)));
        assert_eq!(caps.pos(2), Some((3, 6)));
        assert_eq!(caps.pos(3), None);
    }

    #[test]
    fn test_group_types() {
        let capturing = Group::new(1);
        assert!(capturing.is_capturing);
        assert_eq!(capturing.index, 1);

        let non_capturing = Group::non_capturing();
        assert!(!non_capturing.is_capturing);

        let named = Group::named(2, "year".to_string());
        assert!(named.is_capturing);
        assert_eq!(named.name, Some("year".to_string()));
    }
}
