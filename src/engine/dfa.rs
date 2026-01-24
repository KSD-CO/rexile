//! Simple DFA (Deterministic Finite Automaton) for optimizing sequence patterns
//!
//! Compiles simple sequence patterns like `\w+@\w+\.\w+` into a DFA for faster matching.

use crate::parser::charclass::CharClass;
use crate::parser::quantifier::Quantifier;
use crate::parser::sequence::{Sequence, SequenceElement};

/// A simple DFA state machine for sequence matching
#[derive(Debug, Clone)]
pub struct DFA {
    /// Transition table: (state, char_class) -> next_state
    states: Vec<DFAState>,
    /// Accept states (states where we have a match)
    accept_states: Vec<usize>,
}

#[derive(Debug, Clone)]
struct DFAState {
    /// Transitions: (char_class_id, next_state)
    transitions: Vec<(CharClassId, usize)>,
    /// Is this an accepting state?
    is_accept: bool,
}

/// Strategy for skipping non-matching positions
#[derive(Debug, Clone, Copy)]
enum SkipStrategy {
    /// Skip non-digit chars
    Digit,
    /// Skip non-word chars
    Word,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CharClassId {
    /// Matches a specific character
    Char(char),
    /// Matches \w (word characters)
    Word,
    /// Matches \d (digits)
    Digit,
    /// Matches \s (whitespace)
    Whitespace,
    /// Matches a custom character class
    Custom(CharClass),
}

impl CharClassId {
    #[inline]
    fn matches(&self, ch: char) -> bool {
        match self {
            CharClassId::Char(c) => ch == *c,
            CharClassId::Word => ch.is_alphanumeric() || ch == '_',
            CharClassId::Digit => ch.is_ascii_digit(),
            CharClassId::Whitespace => ch.is_whitespace(),
            CharClassId::Custom(cc) => cc.matches(ch),
        }
    }

    /// Fast byte-level matching for ASCII characters
    #[inline]
    fn matches_byte(&self, byte: u8, ch: char) -> bool {
        match self {
            CharClassId::Char(c) => byte == *c as u8,
            CharClassId::Digit => (b'0'..=b'9').contains(&byte),
            CharClassId::Word => {
                (b'a'..=b'z').contains(&byte)
                    || (b'A'..=b'Z').contains(&byte)
                    || (b'0'..=b'9').contains(&byte)
                    || byte == b'_'
            }
            CharClassId::Whitespace => {
                byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r'
            }
            CharClassId::Custom(cc) => cc.matches(ch),
        }
    }
}

impl DFA {
    /// Try to compile a sequence into a DFA
    /// Returns None if the sequence is too complex for DFA optimization
    pub fn try_compile(seq: &Sequence) -> Option<Self> {
        // Only handle simple patterns for now
        // Pattern: quantified_element literal quantified_element literal ...
        // Example: \w+ @ \w+ . \w+

        if seq.elements.is_empty() {
            return None;
        }

        // Check if pattern is simple enough
        if !Self::is_dfa_compilable(seq) {
            return None;
        }

        // Build DFA states
        // For pattern like \d+\.\d+:
        // State 0: Digit → 1
        // State 1: Digit → 1 (loop), '.' → 2
        // State 2: Digit → 3
        // State 3: Digit → 3 (loop, accept)

        let mut states = Vec::new();

        // First: count how many states we need
        // Each QuantifiedCharClass needs 2 states (entry + loop)
        let mut num_quantified = 0;
        for elem in seq.elements.iter() {
            if matches!(elem, SequenceElement::QuantifiedCharClass(_, _)) {
                num_quantified += 1;
            }
        }

        if num_quantified == 0 {
            return None;
        }

        // Create all states upfront
        for i in 0..num_quantified {
            let is_last = i == num_quantified - 1;
            // Entry state
            states.push(DFAState {
                transitions: vec![],
                is_accept: false,
            });
            // Loop state
            states.push(DFAState {
                transitions: vec![],
                is_accept: is_last,
            });
        }

        // Now fill in transitions
        let mut quantified_idx = 0; // Which quantified element we're at

        for (elem_idx, elem) in seq.elements.iter().enumerate() {
            match elem {
                SequenceElement::QuantifiedCharClass(cc, _) => {
                    let class_id = Self::char_class_to_id(cc);
                    let entry_state = quantified_idx * 2;
                    let loop_state = quantified_idx * 2 + 1;

                    // Entry → Loop
                    states[entry_state]
                        .transitions
                        .push((class_id.clone(), loop_state));

                    // Loop → Loop (self)
                    states[loop_state]
                        .transitions
                        .push((class_id.clone(), loop_state));

                    // Loop → Next (if not last quantified)
                    if quantified_idx < num_quantified - 1 {
                        // Find what separates this from next quantified
                        // Could be one or more Char elements
                        let mut next_elem_idx = elem_idx + 1;
                        let mut separator_chars = Vec::new();

                        while next_elem_idx < seq.elements.len() {
                            match &seq.elements[next_elem_idx] {
                                SequenceElement::Char(ch) => {
                                    separator_chars.push(*ch);
                                    next_elem_idx += 1;
                                }
                                SequenceElement::QuantifiedCharClass(_, _) => {
                                    break;
                                }
                                _ => return None,
                            }
                        }

                        // Add transition from loop state to next quantified's entry
                        let next_entry_state = (quantified_idx + 1) * 2;

                        if separator_chars.is_empty() {
                            // Direct quantified-to-quantified
                            // E.g., \w+\s+\d+
                            // The character triggering this transition already counts as
                            // the first match of the next element, so go directly to
                            // the loop state (entry + 1) instead of the entry state.
                            if let Some(SequenceElement::QuantifiedCharClass(next_cc, _)) =
                                seq.elements.get(next_elem_idx)
                            {
                                let next_class_id = Self::char_class_to_id(next_cc);
                                let next_loop_state = next_entry_state + 1;
                                states[loop_state]
                                    .transitions
                                    .push((next_class_id, next_loop_state));
                            }
                        } else {
                            // Has separator chars
                            // E.g., \d+.\d+ where separator is '.'
                            // Add transition with first separator char
                            states[loop_state]
                                .transitions
                                .push((CharClassId::Char(separator_chars[0]), next_entry_state));

                            // TODO: Handle multiple separator chars
                            // For now only support single char separator
                            if separator_chars.len() > 1 {
                                return None;
                            }
                        }
                    }

                    quantified_idx += 1;
                }
                SequenceElement::Char(_) => {
                    // Chars are handled as part of transitions, skip
                }
                _ => return None,
            }
        }

        let accept_states = states
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_accept)
            .map(|(i, _)| i)
            .collect();

        Some(DFA {
            states,
            accept_states,
        })
    }

    /// Check if sequence can be compiled to DFA
    fn is_dfa_compilable(seq: &Sequence) -> bool {
        // Prefer Sequence+NFA when all elements are QuantifiedCharClass(OneOrMore/ZeroOrMore) or Char
        // (the NFA with pre-computed transition table is faster than DFA for these patterns)
        let nfa_compatible = seq.elements.len() >= 2
            && seq.elements.iter().all(|e| {
                matches!(
                    e,
                    SequenceElement::QuantifiedCharClass(_, Quantifier::OneOrMore)
                        | SequenceElement::QuantifiedCharClass(_, Quantifier::ZeroOrMore)
                        | SequenceElement::Char(_)
                )
            });
        if nfa_compatible {
            return false;
        }

        // For now, only handle patterns like: quantified literal quantified literal
        // Example: \d+.\d+.\d+ (version numbers)
        // Avoid patterns with specific char anchors like '@', '://' - sequence matcher is faster with memchr

        // IMPORTANT: Don't compile if pattern has any literal strings or multiple char literals
        // Pattern like "rule\s+" should use Sequence matcher, not DFA
        // Because DFA would only contain \s+ and lose the literal prefix!

        // Check if there are any non-quantified elements at the start
        // These should stay as Sequence to preserve literal matching
        let has_literal_prefix = seq
            .elements
            .iter()
            .take_while(|e| {
                !matches!(
                    e,
                    SequenceElement::QuantifiedCharClass(_, _)
                        | SequenceElement::QuantifiedChar(_, _)
                )
            })
            .count()
            > 0;

        if has_literal_prefix {
            return false; // Use Sequence matcher to preserve literal prefix
        }

        // First check: must have at least one quantified element
        let has_quantified = seq
            .elements
            .iter()
            .any(|e| matches!(e, SequenceElement::QuantifiedCharClass(_, _)));

        if !has_quantified {
            return false;
        }

        // Second check: look for distinctive anchor chars that memchr can find quickly
        for elem in seq.elements.iter() {
            match elem {
                SequenceElement::Char(ch) => {
                    // Skip DFA if there's a distinctive anchor char
                    // '@', ':', '/', '#', etc are rare and memchr is VERY fast for them
                    // But '.', '-', '_' are common in version numbers, so DFA is better
                    if *ch == '@' || *ch == ':' || *ch == '/' || *ch == '#' || *ch == '!' {
                        return false; // Use sequence matcher with memchr anchor
                    }
                }
                SequenceElement::QuantifiedCharClass(_, Quantifier::OneOrMore) => {} // \w+, \d+ OK
                SequenceElement::QuantifiedCharClass(_, Quantifier::ZeroOrMore) => {} // \w*, \d* OK
                _ => return false, // Other patterns not supported yet
            }
        }

        // Third check: reject patterns with overlapping adjacent quantified charclasses
        // DFA can't backtrack, so overlapping classes (e.g., [a-z]+.+[0-9]+) would cause
        // the DFA to greedily consume everything in the first class and never transition
        let mut prev_cc: Option<&CharClass> = None;
        for elem in seq.elements.iter() {
            match elem {
                SequenceElement::QuantifiedCharClass(cc, _) => {
                    if let Some(prev) = prev_cc {
                        if prev.overlaps_with(cc) {
                            return false; // Use Sequence matcher with backtracking
                        }
                    }
                    prev_cc = Some(cc);
                }
                SequenceElement::Char(_) => {
                    // A separator char breaks the adjacency
                    prev_cc = None;
                }
                _ => {
                    prev_cc = None;
                }
            }
        }

        true
    }

    fn char_class_to_id(cc: &CharClass) -> CharClassId {
        // Check for common patterns
        if Self::is_word_class(cc) {
            CharClassId::Word
        } else if Self::is_digit_class(cc) {
            CharClassId::Digit
        } else if Self::is_whitespace_class(cc) {
            CharClassId::Whitespace
        } else {
            CharClassId::Custom(cc.clone())
        }
    }

    fn is_word_class(cc: &CharClass) -> bool {
        // Check if this matches \w: [a-zA-Z0-9_]
        !cc.negated
            && cc.ranges.len() == 3
            && cc.ranges.contains(&('a', 'z'))
            && cc.ranges.contains(&('A', 'Z'))
            && cc.ranges.contains(&('0', '9'))
            && cc.chars.contains(&'_')
    }

    fn is_digit_class(cc: &CharClass) -> bool {
        // Check if this matches \d: [0-9]
        !cc.negated && cc.ranges.len() == 1 && cc.ranges[0] == ('0', '9') && cc.chars.is_empty()
    }

    fn is_whitespace_class(cc: &CharClass) -> bool {
        // Check if this matches \s: [ \t\n\r]
        !cc.negated
            && cc.ranges.is_empty()
            && cc.chars.contains(&' ')
            && cc.chars.contains(&'\t')
            && cc.chars.contains(&'\n')
            && cc.chars.contains(&'\r')
    }

    /// Find first match using DFA with prefilter optimization
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        if text.is_empty() {
            return None;
        }

        // Optimization: Use memchr to find candidate positions for digit patterns
        // For pattern like \d+.\d+.\d+, use memchr to find digits quickly
        let first_chars = self.get_first_chars();

        if !first_chars.is_empty() {
            // Use memchr to find candidates
            match first_chars.len() {
                1 => {
                    // Single char - use memchr
                    let ch = first_chars[0];
                    let mut pos = 0;
                    while pos < text.len() {
                        if let Some(found) = memchr::memchr(ch, &text.as_bytes()[pos..]) {
                            let byte_start = pos + found;
                            if let Some(byte_end) = self.match_from_bytes(text, byte_start) {
                                return Some((byte_start, byte_end));
                            }
                            pos = byte_start + 1;
                        } else {
                            break;
                        }
                    }
                }
                2 => {
                    // Two chars - use memchr2
                    let mut pos = 0;
                    while pos < text.len() {
                        if let Some(found) =
                            memchr::memchr2(first_chars[0], first_chars[1], &text.as_bytes()[pos..])
                        {
                            let byte_start = pos + found;
                            if let Some(byte_end) = self.match_from_bytes(text, byte_start) {
                                return Some((byte_start, byte_end));
                            }
                            pos = byte_start + 1;
                        } else {
                            break;
                        }
                    }
                }
                3 => {
                    // Three chars - use memchr3
                    let mut pos = 0;
                    while pos < text.len() {
                        if let Some(found) = memchr::memchr3(
                            first_chars[0],
                            first_chars[1],
                            first_chars[2],
                            &text.as_bytes()[pos..],
                        ) {
                            let byte_start = pos + found;
                            if let Some(byte_end) = self.match_from_bytes(text, byte_start) {
                                return Some((byte_start, byte_end));
                            }
                            pos = byte_start + 1;
                        } else {
                            break;
                        }
                    }
                }
                _ => {
                    // Too many chars, fall back to position-by-position
                    return self.find_fallback(text);
                }
            }
            None
        } else {
            // No first chars optimization available, use fallback
            self.find_fallback(text)
        }
    }

    /// Fallback: scan position by position
    fn find_fallback(&self, text: &str) -> Option<(usize, usize)> {
        for (byte_start, _) in text.char_indices() {
            if let Some(byte_end) = self.match_from_bytes(text, byte_start) {
                return Some((byte_start, byte_end));
            }
        }
        None
    }

    /// Try to match from a byte position, return end byte position if match
    #[inline]
    fn match_from_bytes(&self, text: &str, start_byte: usize) -> Option<usize> {
        let bytes = text.as_bytes();
        let mut state = 0;
        let mut last_accept_end = None;
        let mut pos = start_byte;

        if !self.states.is_empty() && self.states[0].is_accept {
            last_accept_end = Some(start_byte);
        }

        while pos < bytes.len() {
            if state >= self.states.len() {
                break;
            }

            let byte = bytes[pos];
            // Fast path: ASCII bytes (most common case)
            let ch = if byte < 128 {
                byte as char
            } else {
                // Non-ASCII: decode UTF-8 char
                match text[pos..].chars().next() {
                    Some(c) => {
                        let current_state = &self.states[state];
                        let mut found = false;
                        for (class_id, next_state) in &current_state.transitions {
                            if class_id.matches(c) {
                                state = *next_state;
                                pos += c.len_utf8();
                                if self.states[state].is_accept {
                                    last_accept_end = Some(pos);
                                }
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            break;
                        }
                        continue;
                    }
                    None => break,
                }
            };

            let current_state = &self.states[state];
            let mut found_transition = false;
            for (class_id, next_state) in &current_state.transitions {
                if class_id.matches_byte(byte, ch) {
                    state = *next_state;
                    found_transition = true;
                    pos += 1;
                    if self.states[state].is_accept {
                        last_accept_end = Some(pos);
                    }
                    break;
                }
            }

            if !found_transition {
                break;
            }
        }

        last_accept_end
    }

    /// Check if pattern matches anywhere in text (faster than find)
    pub fn is_match(&self, text: &str) -> bool {
        let bytes = text.as_bytes();
        let len = bytes.len();

        // For longer texts, use memchr optimization if available
        if len >= 50 {
            let first_chars = self.get_first_chars();
            if !first_chars.is_empty() {
                return self.is_match_with_first_chars(text, &first_chars);
            }
        }

        // Byte-level scanning for ASCII text
        let mut pos = 0;
        while pos < len {
            if self.match_from_bytes(text, pos).is_some() {
                return true;
            }
            // Advance: skip multi-byte UTF-8 chars
            if bytes[pos] < 128 {
                pos += 1;
            } else {
                pos += text[pos..].chars().next().map_or(1, |c| c.len_utf8());
            }
        }
        false
    }

    /// Determine skip strategy based on first state transitions (DEPRECATED - use get_first_chars)
    fn get_skip_strategy(&self) -> Option<SkipStrategy> {
        if self.states.is_empty() {
            return None;
        }

        let first_state = &self.states[0];
        if first_state.transitions.len() != 1 {
            return None;
        }

        match &first_state.transitions[0].0 {
            CharClassId::Digit => Some(SkipStrategy::Digit),
            CharClassId::Word => Some(SkipStrategy::Word),
            _ => None,
        }
    }

    /// Match with skip strategy - skip non-matching character classes
    fn is_match_with_skip(&self, text: &str, strategy: SkipStrategy) -> bool {
        let chars = text.char_indices();

        for (byte_pos, ch) in chars {
            // Check if this char could start a match
            let could_start = match strategy {
                SkipStrategy::Digit => ch.is_ascii_digit(),
                SkipStrategy::Word => ch.is_alphanumeric() || ch == '_',
            };

            if could_start && self.match_from_bytes(text, byte_pos).is_some() {
                return true;
            }
            // Otherwise skip this position - it can't start a match
        }

        false
    }

    /// Get the set of chars that can start a match (from state 0 transitions)
    /// Returns Some(vec) if first state has specific chars (max 3 for memchr optimization)
    /// Returns None if first state is a char class (digit/word) or too many chars
    fn get_first_chars(&self) -> Vec<u8> {
        if self.states.is_empty() {
            return vec![];
        }

        let first_state = &self.states[0];
        let mut chars = Vec::new();

        for (class_id, _) in &first_state.transitions {
            match class_id {
                CharClassId::Char(ch) if ch.is_ascii() => {
                    chars.push(*ch as u8);
                }
                CharClassId::Digit => {
                    // For digit patterns, we'll use a different strategy
                    // Return digits 0-9 for memchr-based scanning
                    for d in b'0'..=b'9' {
                        chars.push(d);
                    }
                    return chars;
                }
                CharClassId::Word => {
                    // Word class has too many chars (a-z, A-Z, 0-9, _)
                    // Return empty to signal we need char-by-char scan
                    return vec![];
                }
                _ => {
                    // Other classes: return empty
                    return vec![];
                }
            }
        }

        // Limit to 3 chars for memchr optimization
        if chars.len() > 3 {
            return vec![];
        }

        chars
    }

    /// Match using memchr to find candidate positions (optimized for digit patterns)
    fn is_match_with_first_chars(&self, text: &str, first_chars: &[u8]) -> bool {
        let bytes = text.as_bytes();

        // Special handling for digit patterns (10 chars: 0-9)
        if first_chars.len() == 10
            && first_chars == [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9']
        {
            // Adaptive strategy based on text length:
            // - Short text (<100 bytes): Use simple DFA scan (faster due to low overhead)
            // - Long text (>=100 bytes): Use memchr to skip non-digit regions

            if bytes.len() < 100 {
                // Short text: simple position-by-position DFA matching is faster
                for (byte_pos, _) in text.char_indices() {
                    if self.match_from_bytes(text, byte_pos).is_some() {
                        return true;
                    }
                }
                return false;
            }

            // Long text: Use memchr to find ANY digit quickly
            // This is much faster for texts with few or no digits
            use memchr::memchr;
            let mut pos = 0;

            // Scan for each digit 0-9
            while pos < bytes.len() {
                // Find next digit using parallel search
                let mut next_digit_pos = None;
                for digit in b'0'..=b'9' {
                    if let Some(found) = memchr(digit, &bytes[pos..]) {
                        let abs_pos = pos + found;
                        next_digit_pos = Some(match next_digit_pos {
                            None => abs_pos,
                            Some(current_min) => abs_pos.min(current_min),
                        });
                    }
                }

                if let Some(digit_pos) = next_digit_pos {
                    if self.match_from_bytes(text, digit_pos).is_some() {
                        return true;
                    }
                    pos = digit_pos + 1;
                } else {
                    // No more digits found
                    break;
                }
            }
            return false;
        }

        if first_chars.len() == 1 {
            // Single char: use memchr
            use memchr::memchr;
            let mut pos = 0;
            while let Some(found) = memchr(first_chars[0], &bytes[pos..]) {
                let byte_pos = pos + found;
                if self.match_from_bytes(text, byte_pos).is_some() {
                    return true;
                }
                pos = byte_pos + 1;
            }
            false
        } else if first_chars.len() <= 3 {
            // Few chars: use memchr2/memchr3
            use memchr::{memchr2, memchr3};
            let mut pos = 0;

            loop {
                let search_result = if first_chars.len() == 2 {
                    memchr2(first_chars[0], first_chars[1], &bytes[pos..])
                } else {
                    memchr3(
                        first_chars[0],
                        first_chars[1],
                        first_chars[2],
                        &bytes[pos..],
                    )
                };

                match search_result {
                    Some(found) => {
                        let byte_pos = pos + found;
                        if self.match_from_bytes(text, byte_pos).is_some() {
                            return true;
                        }
                        pos = byte_pos + 1;
                    }
                    None => break,
                }
            }
            false
        } else {
            // Multiple chars: use aho-corasick
            let patterns: Vec<Vec<u8>> = first_chars.iter().map(|&c| vec![c]).collect();
            if let Ok(ac) = aho_corasick::AhoCorasick::new(&patterns) {
                for mat in ac.find_iter(bytes) {
                    if self.match_from_bytes(text, mat.start()).is_some() {
                        return true;
                    }
                }
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_dfa_simple() {
        // TODO: Add tests when integrated
    }
}
