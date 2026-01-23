/// Sequence matching - combine multiple pattern elements
///
/// Supports patterns like:
/// - ab+c* (char followed by quantified char followed by quantified char)
/// - \d+\w* (escape sequence followed by quantified escape)
/// - hello\d+ (literal followed by quantified escape)
use crate::parser::charclass::CharClass;
use crate::parser::quantifier::Quantifier;

/// A single element in a sequence
#[derive(Debug, Clone, PartialEq)]
pub enum SequenceElement {
    /// A literal character (e.g., 'a' in "abc")
    Char(char),
    /// A quantified character (e.g., 'a+' in "a+bc")
    QuantifiedChar(char, Quantifier),
    /// A character class (e.g., [a-z])
    CharClass(CharClass),
    /// A quantified character class (e.g., [0-9]+)
    QuantifiedCharClass(CharClass, Quantifier),
    /// A literal string (e.g., "hello" in "hello\d+")
    Literal(String),
}

impl SequenceElement {
    /// Try to match this element at a specific position in text
    /// Returns number of bytes consumed if successful, None otherwise
    pub fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        let remaining = &text[pos..];
        if remaining.is_empty() {
            return None;
        }

        match self {
            SequenceElement::Char(ch) => {
                if remaining.starts_with(*ch) {
                    Some(ch.len_utf8())
                } else {
                    None
                }
            }
            SequenceElement::QuantifiedChar(ch, quantifier) => {
                match_quantified_char(*ch, quantifier, remaining)
            }
            SequenceElement::CharClass(cc) => {
                let first_char = remaining.chars().next()?;
                if cc.matches(first_char) {
                    Some(first_char.len_utf8())
                } else {
                    None
                }
            }
            SequenceElement::QuantifiedCharClass(cc, quantifier) => {
                match_quantified_charclass(cc, quantifier, remaining)
            }
            SequenceElement::Literal(lit) => {
                if remaining.starts_with(lit) {
                    Some(lit.len())
                } else {
                    None
                }
            }
        }
    }
}

/// Match a quantified character
fn match_quantified_char(ch: char, quantifier: &Quantifier, text: &str) -> Option<usize> {
    let (min, max) = quantifier_bounds(quantifier);

    // Count matching characters WITHOUT collecting into Vec
    let mut count = 0;
    for c in text.chars() {
        if c == ch {
            count += 1;
        } else {
            break;
        }
    }

    if count < min {
        return None; // Not enough matches
    }

    // Greedy: take as many as possible (up to max)
    let actual_count = count.min(max);
    Some(text.chars().take(actual_count).map(|c| c.len_utf8()).sum())
}

/// Match a quantified character class
fn match_quantified_charclass(
    cc: &CharClass,
    quantifier: &Quantifier,
    text: &str,
) -> Option<usize> {
    let (min, max) = quantifier_bounds(quantifier);

    // OPTIMIZATION: For negated single-char class like [^"], use memchr to find terminator
    if cc.negated && cc.chars.len() == 1 && cc.ranges.is_empty() {
        let forbidden_char = *cc.chars.first().unwrap();
        if (forbidden_char as u32) < 128 {
            // Use memchr to find the forbidden character (terminator)
            use memchr::memchr;
            let terminator_pos =
                memchr(forbidden_char as u8, text.as_bytes()).unwrap_or(text.len());

            // Count chars up to terminator
            let matched_text = &text[..terminator_pos];
            let char_count = matched_text.chars().count();

            if char_count < min {
                return None;
            }

            let actual_count = char_count.min(max);
            return Some(
                matched_text
                    .chars()
                    .take(actual_count)
                    .map(|c| c.len_utf8())
                    .sum(),
            );
        }
    }

    // Count matching characters WITHOUT collecting into Vec
    let mut count = 0;
    for c in text.chars() {
        if cc.matches(c) {
            count += 1;
        } else {
            break;
        }
    }

    if count < min {
        return None; // Not enough matches
    }

    // Greedy: take as many as possible (up to max)
    let actual_count = count.min(max);
    Some(text.chars().take(actual_count).map(|c| c.len_utf8()).sum())
}

/// Get min/max bounds for a quantifier
fn quantifier_bounds(q: &Quantifier) -> (usize, usize) {
    match q {
        Quantifier::ZeroOrMore => (0, usize::MAX),
        Quantifier::OneOrMore => (1, usize::MAX),
        Quantifier::ZeroOrOne => (0, 1),
        Quantifier::Exactly(n) => (*n, *n),
        Quantifier::AtLeast(n) => (*n, usize::MAX),
        Quantifier::Between(n, m) => (*n, *m),
    }
}

/// A sequence of pattern elements
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub elements: Vec<SequenceElement>,
}

impl Sequence {
    /// Create a new sequence
    pub fn new(elements: Vec<SequenceElement>) -> Self {
        Sequence { elements }
    }

    /// Check if the sequence matches at the start of text
    /// Returns bytes consumed if match, None otherwise
    pub fn match_at(&self, text: &str) -> Option<usize> {
        let mut pos = 0;

        for element in &self.elements {
            match element.match_at(text, pos) {
                Some(consumed) => pos += consumed,
                None => return None,
            }
        }

        Some(pos)
    }

    /// Check if the sequence matches anywhere in text (optimized)
    /// Returns immediately on first match without computing position
    pub fn is_match(&self, text: &str) -> bool {
        // Fast path: Try match at start first
        if self.match_at(text).is_some() {
            return true;
        }

        // Only scan forward if no match at start
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();

        for &start_pos in &byte_positions {
            if start_pos == 0 {
                continue; // Already tried
            }
            if self.match_at(&text[start_pos..]).is_some() {
                return true; // Early termination!
            }
        }

        false
    }

    /// Find the sequence anywhere in text
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        // OPTIMIZATION 1: Extract literal prefix for memchr acceleration
        if let Some((prefix_bytes, skip_count)) = self.extract_literal_prefix() {
            if prefix_bytes.len() >= 3 {
                // Multi-byte prefix: use memmem
                use memchr::memmem;
                let finder = memmem::Finder::new(&prefix_bytes);
                let mut pos = 0;

                while pos < text.len() {
                    if let Some(found) = finder.find(text[pos..].as_bytes()) {
                        let match_start = pos + found;
                        let after_prefix = match_start + prefix_bytes.len();

                        // Validate remaining elements
                        if let Some(consumed) =
                            self.match_at_skip(&text[after_prefix..], skip_count)
                        {
                            return Some((match_start, after_prefix + consumed));
                        }

                        pos = match_start + 1;
                    } else {
                        break;
                    }
                }
                return None;
            } else if prefix_bytes.len() == 1 {
                // Single byte prefix: use memchr
                use memchr::memchr;
                let byte = prefix_bytes[0];
                let mut pos = 0;

                while pos < text.len() {
                    if let Some(found) = memchr(byte, &text.as_bytes()[pos..]) {
                        let match_start = pos + found;
                        let after_prefix = match_start + 1;

                        // Validate remaining elements
                        if let Some(consumed) =
                            self.match_at_skip(&text[after_prefix..], skip_count)
                        {
                            return Some((match_start, after_prefix + consumed));
                        }

                        pos = match_start + 1;
                    } else {
                        break;
                    }
                }
                return None;
            }
        }

        // OPTIMIZATION 2: Inner literal with bidirectional matching
        // For patterns like \w+\s*>=\s*\d+ with anchor '>=' in middle
        // Only use for multi-byte literals (single chars too common)
        if let Some((anchor_literal, before_count, after_count)) = self.extract_inner_literal() {
            if anchor_literal.len() >= 2 {
                // At least 2 bytes
                use memchr::memmem;
                let finder = memmem::Finder::new(&anchor_literal);

                // For each anchor occurrence
                for anchor_pos in finder.find_iter(text.as_bytes()) {
                    // Try to match backwards from anchor (before_count elements)
                    // and forwards from anchor_end (after_count elements)
                    if let Some((match_start, match_end)) = self.match_around_anchor(
                        text,
                        anchor_pos,
                        anchor_literal.len(),
                        before_count,
                        after_count,
                    ) {
                        return Some((match_start, match_end));
                    }
                }
                return None;
            }
        }

        // Fallback: sequential search
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();

        for &start_pos in &byte_positions {
            if let Some(consumed) = self.match_at(&text[start_pos..]) {
                return Some((start_pos, start_pos + consumed));
            }
        }

        None
    }

    /// Extract literal prefix from sequence
    /// Returns (prefix_bytes, elements_to_skip_after_prefix)
    fn extract_literal_prefix(&self) -> Option<(Vec<u8>, usize)> {
        let mut prefix = Vec::new();
        let mut elements_scanned = 0;

        for elem in &self.elements {
            match elem {
                SequenceElement::Char(ch) => {
                    prefix.push(*ch as u8);
                    elements_scanned += 1;
                }
                SequenceElement::Literal(s) => {
                    prefix.extend_from_slice(s.as_bytes());
                    elements_scanned += 1;
                }
                _ => {
                    // Stop at first non-literal element
                    break;
                }
            }
        }

        if prefix.is_empty() {
            None
        } else {
            let skip_count = self.elements.len() - elements_scanned;
            Some((prefix, skip_count))
        }
    }

    /// Extract inner literal anchor from sequence
    /// Returns (literal_bytes, elements_before, elements_after)
    fn extract_inner_literal(&self) -> Option<(Vec<u8>, usize, usize)> {
        // Look for consecutive Char/Literal elements not at start
        for (start_idx, window) in self.elements.windows(2).enumerate() {
            if start_idx == 0 {
                // Skip if at start (that's a prefix, not inner)
                continue;
            }

            // Check if this position has literal elements
            let mut literal_bytes = Vec::new();
            let mut elements_consumed = 0;

            for elem in &self.elements[start_idx..] {
                match elem {
                    SequenceElement::Char(ch) => {
                        literal_bytes.push(*ch as u8);
                        elements_consumed += 1;
                    }
                    SequenceElement::Literal(s) => {
                        literal_bytes.extend_from_slice(s.as_bytes());
                        elements_consumed += 1;
                    }
                    _ => break,
                }
            }

            if literal_bytes.len() >= 2 {
                let before_count = start_idx;
                let after_count = self.elements.len() - start_idx - elements_consumed;
                return Some((literal_bytes, before_count, after_count));
            }
        }

        None
    }

    /// Match pattern around an anchor literal (bidirectional matching)
    /// Returns (match_start, match_end) if successful
    fn match_around_anchor(
        &self,
        text: &str,
        anchor_byte_pos: usize,
        anchor_len: usize,
        before_count: usize,
        after_count: usize,
    ) -> Option<(usize, usize)> {
        let anchor_end = anchor_byte_pos + anchor_len;

        // Match elements AFTER anchor (forward direction)
        let after_elements = &self.elements[self.elements.len() - after_count..];
        let mut pos = anchor_end;

        for elem in after_elements {
            if pos > text.len() {
                return None;
            }
            match elem.match_at(&text[pos..], 0) {
                Some(consumed) => pos += consumed,
                None => return None,
            }
        }
        let match_end = pos;

        // Match elements BEFORE anchor (backward direction - OPTIMIZED)
        // For greedy quantifiers, match backwards from anchor
        let before_elements = &self.elements[..before_count];

        if before_elements.is_empty() {
            return Some((anchor_byte_pos, match_end));
        }

        // Reverse-engineer: walk backwards through pattern
        let text_before = &text[..anchor_byte_pos];
        let mut match_start = anchor_byte_pos;

        // Process elements in REVERSE order
        for elem in before_elements.iter().rev() {
            match elem {
                SequenceElement::QuantifiedCharClass(cc, q) => {
                    // Match greedily backwards WITHOUT collecting into Vec
                    let (min, max) = quantifier_bounds(q);
                    let mut count = 0;
                    let mut byte_offset = 0;

                    // Walk backwards char by char
                    for ch in text_before[..match_start].chars().rev() {
                        if count >= max {
                            break;
                        }
                        if cc.matches(ch) {
                            count += 1;
                            byte_offset += ch.len_utf8();
                        } else {
                            break;
                        }
                    }

                    if count < min {
                        return None;
                    }

                    match_start -= byte_offset;
                }
                SequenceElement::QuantifiedChar(ch, q) => {
                    let (min, max) = quantifier_bounds(q);
                    let mut count = 0;
                    let mut byte_offset = 0;

                    for c in text_before[..match_start].chars().rev() {
                        if count >= max {
                            break;
                        }
                        if c == *ch {
                            count += 1;
                            byte_offset += c.len_utf8();
                        } else {
                            break;
                        }
                    }

                    if count < min {
                        return None;
                    }

                    match_start -= byte_offset;
                }
                SequenceElement::Char(target) => {
                    // Must match exactly one char before match_start
                    if match_start == 0 {
                        return None;
                    }

                    // Get last char before match_start
                    if let Some(ch) = text_before[..match_start].chars().next_back() {
                        if ch != *target {
                            return None;
                        }
                        match_start -= ch.len_utf8();
                    } else {
                        return None;
                    }
                }
                _ => {
                    // Other elements not supported in reverse yet
                    return None;
                }
            }
        }

        Some((match_start, match_end))
    }

    /// Match starting from position, skipping first N elements
    fn match_at_skip(&self, text: &str, skip_count: usize) -> Option<usize> {
        if skip_count == 0 {
            return Some(0); // No more elements to match
        }

        let start_idx = self.elements.len() - skip_count;
        let mut pos = 0;

        for elem in &self.elements[start_idx..] {
            match elem.match_at(text, pos) {
                Some(consumed) => pos += consumed,
                None => return None,
            }
        }

        Some(pos)
    }

    /// Find all occurrences of the sequence in text
    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();

        // OPTIMIZATION 1: Use literal prefix with memchr
        if let Some((prefix_bytes, skip_count)) = self.extract_literal_prefix() {
            if prefix_bytes.len() >= 3 {
                // Multi-byte prefix: use memmem
                use memchr::memmem;
                let finder = memmem::Finder::new(&prefix_bytes);

                for found_pos in finder.find_iter(text.as_bytes()) {
                    let after_prefix = found_pos + prefix_bytes.len();
                    if let Some(consumed) = self.match_at_skip(&text[after_prefix..], skip_count) {
                        results.push((found_pos, after_prefix + consumed));
                    }
                }
                return results;
            } else if prefix_bytes.len() == 1 {
                // Single byte prefix: use memchr_iter
                use memchr::memchr_iter;
                let byte = prefix_bytes[0];

                for found_pos in memchr_iter(byte, text.as_bytes()) {
                    let after_prefix = found_pos + 1;
                    if let Some(consumed) = self.match_at_skip(&text[after_prefix..], skip_count) {
                        results.push((found_pos, after_prefix + consumed));
                    }
                }
                return results;
            }
        }

        // OPTIMIZATION 2: Inner literal with bidirectional matching
        if let Some((anchor_literal, before_count, after_count)) = self.extract_inner_literal() {
            if anchor_literal.len() >= 2 {
                use memchr::memmem;
                let finder = memmem::Finder::new(&anchor_literal);

                for anchor_pos in finder.find_iter(text.as_bytes()) {
                    if let Some((match_start, match_end)) = self.match_around_anchor(
                        text,
                        anchor_pos,
                        anchor_literal.len(),
                        before_count,
                        after_count,
                    ) {
                        // Check if this match overlaps with previous
                        if results.is_empty() || match_start >= results.last().unwrap().1 {
                            results.push((match_start, match_end));
                        }
                    }
                }
                return results;
            }
        }

        // Fallback: sequential search
        let byte_positions: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();

        let mut i = 0;
        while i < byte_positions.len() {
            let start_pos = byte_positions[i];

            if let Some(consumed) = self.match_at(&text[start_pos..]) {
                let end_pos = start_pos + consumed;
                results.push((start_pos, end_pos));

                // Skip past this match
                while i < byte_positions.len() && byte_positions[i] < end_pos {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence() {
        // "abc"
        let seq = Sequence::new(vec![
            SequenceElement::Char('a'),
            SequenceElement::Char('b'),
            SequenceElement::Char('c'),
        ]);

        assert_eq!(seq.match_at("abc"), Some(3));
        assert_eq!(seq.match_at("abcdef"), Some(3));
        assert_eq!(seq.match_at("ab"), None);
        assert_eq!(seq.match_at("xyz"), None);
    }

    #[test]
    fn test_quantified_sequence() {
        // "a+b"
        let seq = Sequence::new(vec![
            SequenceElement::QuantifiedChar('a', Quantifier::OneOrMore),
            SequenceElement::Char('b'),
        ]);

        assert_eq!(seq.match_at("ab"), Some(2));
        assert_eq!(seq.match_at("aaab"), Some(4));
        assert_eq!(seq.match_at("aaaabcd"), Some(5));
        assert_eq!(seq.match_at("b"), None); // Need at least one 'a'
    }

    #[test]
    fn test_charclass_sequence() {
        // "[0-9]+[a-z]"
        let mut digits = CharClass::new();
        digits.add_range('0', '9');
        digits.finalize();

        let mut letters = CharClass::new();
        letters.add_range('a', 'z');
        letters.finalize();

        let seq = Sequence::new(vec![
            SequenceElement::QuantifiedCharClass(digits, Quantifier::OneOrMore),
            SequenceElement::CharClass(letters),
        ]);

        assert_eq!(seq.match_at("123a"), Some(4));
        assert_eq!(seq.match_at("9z"), Some(2));
        assert_eq!(seq.match_at("abc"), None); // No digits
    }

    #[test]
    fn test_find() {
        // "ab+"
        let seq = Sequence::new(vec![
            SequenceElement::Char('a'),
            SequenceElement::QuantifiedChar('b', Quantifier::OneOrMore),
        ]);

        assert_eq!(seq.find("xyzabbc"), Some((3, 6)));
        assert_eq!(seq.find("nope"), None);
    }

    #[test]
    fn test_find_all() {
        // "a+b"
        let seq = Sequence::new(vec![
            SequenceElement::QuantifiedChar('a', Quantifier::OneOrMore),
            SequenceElement::Char('b'),
        ]);

        let matches = seq.find_all("ab aab aaab");
        assert_eq!(matches, vec![(0, 2), (3, 6), (7, 11)]);
    }

    #[test]
    fn test_literal_sequence() {
        // "hello[0-9]+"
        let mut digits = CharClass::new();
        digits.add_range('0', '9');
        digits.finalize();

        let seq = Sequence::new(vec![
            SequenceElement::Literal("hello".to_string()),
            SequenceElement::QuantifiedCharClass(digits, Quantifier::OneOrMore),
        ]);

        assert_eq!(seq.match_at("hello123"), Some(8));
        assert_eq!(seq.match_at("hello"), None); // Need digits
    }
}
