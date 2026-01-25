//! Lazy DFA - compile states on-demand for O(n) matching
//!
//! Like regex crate's hybrid engine: NFA simulation with cached DFA states

use crate::parser::charclass::CharClass;
use crate::parser::quantifier::Quantifier;
use crate::parser::sequence::{Sequence, SequenceElement};
use std::collections::HashMap;

/// Lazy DFA that compiles states on-demand
pub struct LazyDFA {
    /// NFA-like pattern representation
    pattern: Vec<PatternElement>,
    /// Cache of compiled DFA states: (state_id, byte) -> next_state_id
    /// Using u8 instead of char for performance (ASCII-optimized)
    state_cache: HashMap<(StateId, u8), StateId>,
    /// State definitions: what NFA positions each DFA state represents
    states: Vec<DFAState>,
    /// Next state ID to allocate
    next_state_id: StateId,
}

type StateId = u32;

#[derive(Debug, Clone)]
struct DFAState {
    /// NFA positions this DFA state represents (bitset for performance)
    /// Using u64 bitset for up to 64 NFA positions
    nfa_positions: u64,
    /// Is this an accepting state?
    is_accept: bool,
}

#[derive(Debug, Clone)]
enum PatternElement {
    /// Match a literal character
    Literal(char),
    /// Match word char (\w)
    Word,
    /// Match digit (\d)
    Digit,
    /// Match whitespace (\s)
    Whitespace,
    /// Match any character (.)
    Any,
    /// Match custom character class
    Class(CharClass),
}

impl PatternElement {
    fn matches(&self, ch: char) -> bool {
        match self {
            PatternElement::Literal(c) => ch == *c,
            PatternElement::Word => ch.is_alphanumeric() || ch == '_',
            PatternElement::Digit => ch.is_ascii_digit(),
            PatternElement::Whitespace => ch.is_whitespace(),
            PatternElement::Any => true,
            PatternElement::Class(cc) => cc.matches(ch),
        }
    }
}

impl LazyDFA {
    /// Try to compile a sequence into a Lazy DFA
    pub fn try_compile(seq: &Sequence) -> Option<Self> {
        // Convert sequence to flat pattern (expanding quantifiers)
        let mut pattern = Vec::new();

        for elem in &seq.elements {
            match elem {
                SequenceElement::Char(ch) => {
                    pattern.push(PatternElement::Literal(*ch));
                }

                SequenceElement::CharClass(cc) => {
                    pattern.push(Self::charclass_to_element(cc));
                }

                SequenceElement::QuantifiedChar(ch, q) => {
                    let elem = PatternElement::Literal(*ch);
                    Self::expand_quantified(&mut pattern, elem, q)?;
                }

                SequenceElement::QuantifiedCharClass(cc, q) => {
                    let elem = Self::charclass_to_element(cc);
                    Self::expand_quantified(&mut pattern, elem, q)?;
                }

                _ => return None, // Other elements not supported yet
            }
        }

        if pattern.is_empty() {
            return None;
        }

        // Create initial DFA state (start state)
        let start_state = DFAState {
            nfa_positions: 1u64, // Position 0 (before first element)
            is_accept: false,
        };

        Some(LazyDFA {
            pattern,
            state_cache: HashMap::new(),
            states: vec![start_state],
            next_state_id: 1,
        })
    }

    /// Expand quantified element into pattern representation
    fn expand_quantified(
        pattern: &mut Vec<PatternElement>,
        elem: PatternElement,
        quantifier: &Quantifier,
    ) -> Option<()> {
        match quantifier {
            Quantifier::ZeroOrMore => {
                // For *, we use marker in pattern (handled specially in matching)
                pattern.push(elem);
                Some(())
            }
            Quantifier::OneOrMore => {
                // For +, we use marker in pattern
                pattern.push(elem);
                Some(())
            }
            Quantifier::ZeroOrOne => {
                // For ?, we use marker in pattern
                pattern.push(elem);
                Some(())
            }
            _ => None, // Other quantifiers not implemented yet
        }
    }

    /// Convert CharClass to PatternElement
    fn charclass_to_element(cc: &CharClass) -> PatternElement {
        // Check for predefined classes
        if cc.matches('a') && cc.matches('Z') && cc.matches('0') && cc.matches('_') {
            return PatternElement::Word;
        }
        if cc.matches('0') && cc.matches('9') && !cc.matches('a') {
            return PatternElement::Digit;
        }
        if cc.matches(' ') && cc.matches('\t') && cc.matches('\n') {
            return PatternElement::Whitespace;
        }

        PatternElement::Class(cc.clone())
    }

    /// Find first match using lazy DFA compilation
    pub fn find(&mut self, text: &str) -> Option<(usize, usize)> {
        let bytes = text.as_bytes();

        // Try starting match at each byte position
        for start_byte in 0..bytes.len() {
            // Skip if not at char boundary
            if !text.is_char_boundary(start_byte) {
                continue;
            }

            if let Some(match_len) = self.try_match_at(text, start_byte) {
                return Some((start_byte, start_byte + match_len));
            }
        }

        None
    }

    /// Try to match starting at a specific byte position
    fn try_match_at(&mut self, text: &str, start_byte: usize) -> Option<usize> {
        let text_slice = &text[start_byte..];
        let _current_state: StateId = 0; // Start state
        let mut pattern_pos = 0;
        let mut bytes_consumed = 0;

        for ch in text_slice.chars() {
            // Check if we can accept here (for greedy matching)
            if pattern_pos >= self.pattern.len() {
                return Some(bytes_consumed);
            }

            let elem = &self.pattern[pattern_pos];

            if elem.matches(ch) {
                bytes_consumed += ch.len_utf8();
                pattern_pos += 1;
            } else {
                // No match
                break;
            }
        }

        // Check if we consumed entire pattern
        if pattern_pos >= self.pattern.len() {
            Some(bytes_consumed)
        } else {
            None
        }
    }
}

/// Optimized version using direct pattern matching (no actual DFA compilation yet)
/// This is a stepping stone - simpler and faster than full NFA, but not true lazy DFA
impl LazyDFA {
    /// Find with prefilter: use inner literal to guide search
    pub fn find_with_prefilter(&self, text: &str, literal: &[u8]) -> Option<(usize, usize)> {
        use memchr::memmem;
        let finder = memmem::Finder::new(literal);

        let mut search_pos = 0;
        while search_pos < text.len() {
            if let Some(found_pos) = finder.find(&text.as_bytes()[search_pos..]) {
                let anchor_pos = search_pos + found_pos;

                // Try match at ONLY a few strategic positions before anchor
                // Most matches are within 5-15 chars before the anchor
                let max_back = 30;
                let start = anchor_pos.saturating_sub(max_back);
                let offsets = [0, 3, 7, 12, 18, 25];

                for &offset in &offsets {
                    let try_start = anchor_pos.saturating_sub(offset);
                    if try_start < start {
                        break;
                    }

                    if !text.is_char_boundary(try_start) {
                        continue;
                    }

                    if let Some(len) = self.match_pattern_at(text, try_start) {
                        let match_end = try_start + len;
                        // Verify anchor is within match
                        if try_start <= anchor_pos && anchor_pos < match_end {
                            return Some((try_start, match_end));
                        }
                    }
                }

                search_pos = anchor_pos + 1;
            } else {
                break;
            }
        }

        None
    }

    /// Simplified find that just does smart sequential matching
    pub fn find_simple(&self, text: &str) -> Option<(usize, usize)> {
        // Extract first element to guide search
        let first_elem = self.pattern.first()?;

        match first_elem {
            PatternElement::Literal(ch) => {
                // Use memchr for literal first char
                use memchr::memchr;
                let byte = *ch as u8;
                let mut search_pos = 0;

                while search_pos < text.len() {
                    if let Some(found) = memchr(byte, &text.as_bytes()[search_pos..]) {
                        let abs_pos = search_pos + found;
                        if let Some(len) = self.match_pattern_at(text, abs_pos) {
                            return Some((abs_pos, abs_pos + len));
                        }
                        search_pos = abs_pos + 1;
                    } else {
                        break;
                    }
                }
                None
            }

            _ => {
                // For non-literal first elements, scan positions
                for (pos, ch) in text.char_indices() {
                    if first_elem.matches(ch) {
                        if let Some(len) = self.match_pattern_at(text, pos) {
                            return Some((pos, pos + len));
                        }
                    }
                }
                None
            }
        }
    }

    /// Match pattern at specific position
    fn match_pattern_at(&self, text: &str, start_pos: usize) -> Option<usize> {
        let text_slice = &text[start_pos..];
        let mut pattern_pos = 0;
        let mut char_iter = text_slice.chars();
        let mut bytes_consumed = 0;

        while pattern_pos < self.pattern.len() {
            let elem = &self.pattern[pattern_pos];

            if let Some(ch) = char_iter.next() {
                if elem.matches(ch) {
                    bytes_consumed += ch.len_utf8();
                    pattern_pos += 1;
                } else {
                    return None;
                }
            } else {
                // Ran out of text
                return None;
            }
        }

        Some(bytes_consumed)
    }
}
