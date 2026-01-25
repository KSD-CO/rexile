/// Sequence matching - combine multiple pattern elements
///
/// Supports patterns like:
/// - ab+c* (char followed by quantified char followed by quantified char)
/// - \d+\w* (escape sequence followed by quantified escape)
/// - hello\d+ (literal followed by quantified escape)
use crate::parser::boundary::BoundaryType;
use crate::parser::charclass::CharClass;
use crate::parser::group::Group;
use crate::parser::quantifier::Quantifier;

/// A single element in a sequence
#[derive(Debug, Clone, PartialEq)]
pub enum SequenceElement {
    /// A literal character (e.g., 'a' in "abc")
    Char(char),
    /// Dot wildcard (matches any character except newline)
    Dot,
    /// A quantified character (e.g., 'a+' in "a+bc")
    QuantifiedChar(char, Quantifier),
    /// A character class (e.g., [a-z])
    CharClass(CharClass),
    /// A quantified character class (e.g., [0-9]+)
    QuantifiedCharClass(CharClass, Quantifier),
    /// A literal string (e.g., "hello" in "hello\d+")
    Literal(String),
    /// A group element (e.g., (?:foo|bar) or (abc) in a sequence)
    Group(Group),
    /// A quantified group (e.g., (?:foo|bar)+ in a sequence)
    QuantifiedGroup(Group, Quantifier),
    /// A word boundary (e.g., \b or \B)
    Boundary(BoundaryType),
}

impl SequenceElement {
    /// Try to match this element at a specific position in text
    /// Returns number of bytes consumed if successful, None otherwise
    pub fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        let remaining = &text[pos..];

        // Don't reject empty remaining for quantified elements (they can match zero times)
        // and boundaries (they are zero-width)
        if remaining.is_empty() {
            return match self {
                SequenceElement::QuantifiedChar(_, quantifier) => {
                    let (min, _) = quantifier_bounds(quantifier);
                    if min == 0 {
                        Some(0)
                    } else {
                        None
                    }
                }
                SequenceElement::QuantifiedCharClass(_, quantifier) => {
                    let (min, _) = quantifier_bounds(quantifier);
                    if min == 0 {
                        Some(0)
                    } else {
                        None
                    }
                }
                SequenceElement::QuantifiedGroup(_, quantifier) => {
                    let (min, _) = quantifier_bounds(quantifier);
                    if min == 0 {
                        Some(0)
                    } else {
                        None
                    }
                }
                SequenceElement::Boundary(boundary_type) => {
                    // Boundary is zero-width, check if it matches at end of text
                    if boundary_type.matches_at(text, pos) {
                        Some(0)
                    } else {
                        None
                    }
                }
                _ => None, // Other elements need at least one char
            };
        }

        match self {
            SequenceElement::Char(ch) => {
                if remaining.starts_with(*ch) {
                    Some(ch.len_utf8())
                } else {
                    None
                }
            }
            SequenceElement::Dot => {
                // Dot matches any character except newline
                let first_char = remaining.chars().next()?;
                if first_char != '\n' {
                    Some(first_char.len_utf8())
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
            SequenceElement::Group(group) => group.match_at(text, pos),
            SequenceElement::QuantifiedGroup(group, quantifier) => {
                match_quantified_group(group, quantifier, text, pos)
            }
            SequenceElement::Boundary(boundary_type) => {
                // Boundary is zero-width, so we return 0 if it matches, None otherwise
                if boundary_type.matches_at(text, pos) {
                    Some(0)
                } else {
                    None
                }
            }
        }
    }
}

/// Match a quantified character
/// For lazy quantifiers, this returns minimum match; backtracking will try more
fn match_quantified_char(ch: char, quantifier: &Quantifier, text: &str) -> Option<usize> {
    let (min, max) = quantifier_bounds(quantifier);
    let is_lazy = quantifier.is_lazy();

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

    // Lazy: return minimum match; Greedy: return maximum match
    let actual_count = if is_lazy { min } else { count.min(max) };
    Some(text.chars().take(actual_count).map(|c| c.len_utf8()).sum())
}

/// Match a quantified character class
/// For lazy quantifiers, this returns minimum match; backtracking will try more
fn match_quantified_charclass(
    cc: &CharClass,
    quantifier: &Quantifier,
    text: &str,
) -> Option<usize> {
    let (min, max) = quantifier_bounds(quantifier);
    let is_lazy = quantifier.is_lazy();

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

            // Lazy: return minimum match; Greedy: return maximum match
            let actual_count = if is_lazy { min } else { char_count.min(max) };
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

    // Lazy: return minimum match; Greedy: return maximum match
    let actual_count = if is_lazy { min } else { count.min(max) };
    Some(text.chars().take(actual_count).map(|c| c.len_utf8()).sum())
}

/// Match a quantified group (greedy: max match, lazy: min match)
fn match_quantified_group(
    group: &Group,
    quantifier: &Quantifier,
    text: &str,
    pos: usize,
) -> Option<usize> {
    let (min, max) = quantifier_bounds(quantifier);
    let is_lazy = quantifier.is_lazy();

    // Find all possible match lengths by repeatedly matching the group
    let mut byte_positions: Vec<usize> = Vec::new();
    byte_positions.push(0); // 0 repetitions = 0 bytes
    let mut current_pos = pos;
    let mut count = 0;

    while count < max && current_pos <= text.len() {
        if let Some(consumed) = group.match_at(text, current_pos) {
            if consumed == 0 {
                break; // Prevent infinite loop on zero-width matches
            }
            current_pos += consumed;
            count += 1;
            byte_positions.push(current_pos - pos);
        } else {
            break;
        }
    }

    if count < min {
        return None;
    }

    // Lazy: return minimum match; Greedy: return maximum match
    let actual_count = if is_lazy { min } else { count.min(max) };
    Some(byte_positions[actual_count])
}

/// Get min/max bounds for a quantifier
fn quantifier_bounds(q: &Quantifier) -> (usize, usize) {
    match q {
        Quantifier::ZeroOrMore | Quantifier::ZeroOrMoreLazy => (0, usize::MAX),
        Quantifier::OneOrMore | Quantifier::OneOrMoreLazy => (1, usize::MAX),
        Quantifier::ZeroOrOne | Quantifier::ZeroOrOneLazy => (0, 1),
        Quantifier::Exactly(n) => (*n, *n),
        Quantifier::AtLeast(n) => (*n, usize::MAX),
        Quantifier::Between(n, m) => (*n, *m),
    }
}

/// Pre-computed NFA transition table for fast is_match
#[derive(Debug, Clone, PartialEq)]
struct NfaTable {
    /// For each ASCII byte, bitmask of which elements match it
    byte_elem_mask: [u32; 128],
    /// Bitmask for the accept state (last element)
    accept_mask: u32,
    /// Bitmask of elements that can "stay" (quantified with +, *, etc.)
    /// Char elements (exact match) cannot stay - they advance immediately
    quantified_bits: u32,
    /// Bitmask of elements that are optional (min=0, can be skipped via epsilon transition)
    optional_bits: u32,
    /// Number of elements
    n: usize,
}

/// A sequence of pattern elements
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub elements: Vec<SequenceElement>,
    /// Cached NFA table for fast is_match (computed once at construction)
    nfa_table: Option<NfaTable>,
}

impl Sequence {
    /// Create a new sequence, pre-computing NFA table if applicable
    pub fn new(elements: Vec<SequenceElement>) -> Self {
        let nfa_table = Self::build_nfa_table(&elements);
        Sequence {
            elements,
            nfa_table,
        }
    }

    /// Build NFA transition table for sequences of QuantifiedCharClass and Char elements
    fn build_nfa_table(elements: &[SequenceElement]) -> Option<NfaTable> {
        let n = elements.len();
        if !(2..=16).contains(&n) {
            return None;
        }

        // Validate elements and compute quantified_bits and optional_bits
        let mut quantified_bits: u32 = 0;
        let mut optional_bits: u32 = 0;
        let mut has_mid_optional = false; // Track if there are optional elements not at end
        for (i, elem) in elements.iter().enumerate() {
            match elem {
                SequenceElement::QuantifiedCharClass(_, quantifier) => {
                    let (min, _max) = quantifier_bounds(quantifier);

                    // Elements that are optional (min=0) can be skipped
                    if min == 0 {
                        optional_bits |= 1u32 << i;
                    }

                    // Elements with quantifiers that can match multiple times can "stay"
                    match quantifier {
                        Quantifier::OneOrMore
                        | Quantifier::OneOrMoreLazy
                        | Quantifier::ZeroOrMore
                        | Quantifier::ZeroOrMoreLazy => {
                            quantified_bits |= 1u32 << i; // Can stay
                        }
                        _ => {} // ZeroOrOne, Exactly(1), etc. don't stay
                    }
                }
                SequenceElement::Char(_) => {
                    // Char elements don't stay (match exactly 1 char)
                }
                _ => return None, // Not supported
            }
        }

        // Check for optional elements - NFA doesn't handle them correctly
        // The NFA model can't properly track "just finished matching element i" state
        // which is needed to epsilon-transition past optional elements
        if optional_bits != 0 {
            // There's at least one optional element
            // NFA doesn't handle skipping correctly, fall back to backtracking
            return None;
        }

        // Pre-compute byte→element match table
        let mut byte_elem_mask = [0u32; 128];
        for b in 0..128u8 {
            let idx = b as usize;
            let word_idx = idx / 64;
            let bit = 1u64 << (idx % 64);
            for (i, elem) in elements.iter().enumerate() {
                match elem {
                    SequenceElement::QuantifiedCharClass(cc, _) => {
                        if let Some(bm) = cc.get_ascii_bitmap() {
                            let hit = (bm[word_idx] & bit) != 0;
                            if hit != cc.negated {
                                byte_elem_mask[b as usize] |= 1u32 << i;
                            }
                        } else {
                            return None;
                        }
                    }
                    SequenceElement::Char(ch) => {
                        if (*ch as u32) < 128 && b == *ch as u8 {
                            byte_elem_mask[b as usize] |= 1u32 << i;
                        }
                    }
                    _ => return None,
                }
            }
        }

        Some(NfaTable {
            byte_elem_mask,
            accept_mask: 1u32 << (n - 1),
            quantified_bits,
            optional_bits,
            n,
        })
    }

    /// Check if the sequence matches at the start of text
    /// Returns bytes consumed if match, None otherwise
    pub fn match_at(&self, text: &str) -> Option<usize> {
        // Use backtracking-enabled matching from start
        self.match_elements_backtracking(text, 0, 0)
    }

    /// Check if the sequence matches at a specific position in text
    /// Returns bytes consumed if match, None otherwise
    /// This preserves the full text context for boundary checks
    fn match_at_pos(&self, text: &str, pos: usize) -> Option<usize> {
        self.match_elements_backtracking(text, 0, pos)
    }

    /// Check if the sequence matches anywhere in text (optimized)
    /// Returns immediately on first match without computing position
    pub fn is_match(&self, text: &str) -> bool {
        // Try element-level NFA first (much faster for quantified charclass sequences)
        if let Some(result) = self.is_match_nfa(text) {
            return result;
        }
        self.find(text).is_some()
    }

    /// Check if the sequence matches with flags applied
    /// Flags can modify behavior (e.g., DOTALL makes . match newlines)
    pub fn is_match_with_flags(&self, text: &str, flags: &crate::parser::flags::Flags) -> bool {
        self.find_with_flags(text, flags).is_some()
    }

    /// Find match with flags applied
    pub fn find_with_flags(
        &self,
        text: &str,
        flags: &crate::parser::flags::Flags,
    ) -> Option<(usize, usize)> {
        // If DOTALL flag is set, replace Dot elements with "any char" matching
        if flags.dot_matches_newline {
            // For DOTALL mode, we need to match . against any character including \n
            // We do this by modifying the matching logic for Dot elements
            for start_pos in 0..text.len() {
                if let Some(end_pos) = self.match_at_with_dotall(text, start_pos) {
                    return Some((start_pos, end_pos));
                }
            }
            return None;
        }

        // No special flags, use normal find
        self.find(text)
    }

    /// Match at position with DOTALL flag (. matches newlines)
    fn match_at_with_dotall(&self, text: &str, start_pos: usize) -> Option<usize> {
        self.match_elements_backtracking_dotall(text, 0, start_pos)
    }

    /// Backtracking match with DOTALL mode
    fn match_elements_backtracking_dotall(
        &self,
        text: &str,
        elem_idx: usize,
        text_pos: usize,
    ) -> Option<usize> {
        // Base case: matched all elements
        if elem_idx >= self.elements.len() {
            return Some(text_pos);
        }

        let element = &self.elements[elem_idx];
        let remaining = if text_pos < text.len() {
            &text[text_pos..]
        } else {
            ""
        };

        match element {
            // Handle Dot specially in DOTALL mode - match ANY character
            SequenceElement::Dot => {
                if let Some(ch) = remaining.chars().next() {
                    // In DOTALL mode, dot matches ANY character including newline
                    let consumed = ch.len_utf8();
                    self.match_elements_backtracking_dotall(text, elem_idx + 1, text_pos + consumed)
                } else {
                    None
                }
            }
            // QuantifiedChar with backtracking in DOTALL mode
            SequenceElement::QuantifiedChar(ch, quantifier) => {
                let (min, max) = (quantifier.min_matches(), quantifier.max_matches());
                let is_lazy = quantifier.is_lazy();
                let ch_len = ch.len_utf8();

                let mut max_count = 0;
                let mut byte_positions: Vec<usize> = Vec::new();
                byte_positions.push(0);
                let mut byte_offset = 0;
                for c in remaining.chars() {
                    if c == *ch {
                        max_count += 1;
                        byte_offset += ch_len;
                        byte_positions.push(byte_offset);
                        if max_count >= max {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if max_count < min {
                    return None;
                }
                let max_count = max_count.min(max);

                if is_lazy {
                    for try_count in min..=max_count {
                        let consumed = byte_positions[try_count];
                        if let Some(final_pos) = self.match_elements_backtracking_dotall(
                            text,
                            elem_idx + 1,
                            text_pos + consumed,
                        ) {
                            return Some(final_pos);
                        }
                    }
                } else {
                    for try_count in (min..=max_count).rev() {
                        let consumed = byte_positions[try_count];
                        if let Some(final_pos) = self.match_elements_backtracking_dotall(
                            text,
                            elem_idx + 1,
                            text_pos + consumed,
                        ) {
                            return Some(final_pos);
                        }
                    }
                }
                None
            }
            // QuantifiedCharClass - handle both dot class (matches newlines in DOTALL) and normal
            SequenceElement::QuantifiedCharClass(cc, quantifier) => {
                // Check if this is a "dot class" [^\n] - if so, match everything in DOTALL mode
                let is_dot_class = cc.negated
                    && cc.chars.len() == 1
                    && cc.chars[0] == '\n'
                    && cc.ranges.is_empty();
                let (min, max) = (quantifier.min_matches(), quantifier.max_matches());
                let is_lazy = quantifier.is_lazy();

                if is_dot_class {
                    // In DOTALL mode, dot class matches any character including newlines
                    let max_count = remaining.chars().count().min(max);

                    if max_count < min {
                        return None;
                    }

                    // Build byte positions
                    let mut byte_positions: Vec<usize> = Vec::new();
                    byte_positions.push(0);
                    let mut byte_offset = 0;
                    for (i, ch) in remaining.chars().enumerate() {
                        if i >= max_count {
                            break;
                        }
                        byte_offset += ch.len_utf8();
                        byte_positions.push(byte_offset);
                    }

                    if is_lazy {
                        for try_count in min..=max_count {
                            let consumed = byte_positions[try_count];
                            if let Some(final_pos) = self.match_elements_backtracking_dotall(
                                text,
                                elem_idx + 1,
                                text_pos + consumed,
                            ) {
                                return Some(final_pos);
                            }
                        }
                    } else {
                        for try_count in (min..=max_count).rev() {
                            let consumed = byte_positions[try_count];
                            if let Some(final_pos) = self.match_elements_backtracking_dotall(
                                text,
                                elem_idx + 1,
                                text_pos + consumed,
                            ) {
                                return Some(final_pos);
                            }
                        }
                    }
                    None
                } else {
                    // Normal quantified charclass - backtrack with DOTALL continuation
                    let mut max_count = 0;
                    let mut byte_positions: Vec<usize> = Vec::new();
                    byte_positions.push(0);
                    let mut byte_offset = 0;
                    for c in remaining.chars() {
                        if cc.matches(c) {
                            max_count += 1;
                            byte_offset += c.len_utf8();
                            byte_positions.push(byte_offset);
                            if max_count >= max {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if max_count < min {
                        return None;
                    }
                    let max_count = max_count.min(max);

                    if is_lazy {
                        for try_count in min..=max_count {
                            let consumed = byte_positions[try_count];
                            if let Some(final_pos) = self.match_elements_backtracking_dotall(
                                text,
                                elem_idx + 1,
                                text_pos + consumed,
                            ) {
                                return Some(final_pos);
                            }
                        }
                    } else {
                        for try_count in (min..=max_count).rev() {
                            let consumed = byte_positions[try_count];
                            if let Some(final_pos) = self.match_elements_backtracking_dotall(
                                text,
                                elem_idx + 1,
                                text_pos + consumed,
                            ) {
                                return Some(final_pos);
                            }
                        }
                    }
                    None
                }
            }
            // Quantified group with backtracking in DOTALL mode
            SequenceElement::QuantifiedGroup(group, quantifier) => {
                let (min, max) = (quantifier.min_matches(), quantifier.max_matches());
                let is_lazy = quantifier.is_lazy();

                // Find all possible match lengths
                let mut byte_positions: Vec<usize> = Vec::new();
                byte_positions.push(0);
                let mut current_pos = text_pos;
                let mut count = 0;

                while count < max && current_pos <= text.len() {
                    if let Some(consumed) = group.match_at(text, current_pos) {
                        if consumed == 0 {
                            break;
                        }
                        current_pos += consumed;
                        count += 1;
                        byte_positions.push(current_pos - text_pos);
                    } else {
                        break;
                    }
                }

                if count < min {
                    return None;
                }
                let max_count = count.min(max);

                if is_lazy {
                    for try_count in min..=max_count {
                        let consumed = byte_positions[try_count];
                        if let Some(final_pos) = self.match_elements_backtracking_dotall(
                            text,
                            elem_idx + 1,
                            text_pos + consumed,
                        ) {
                            return Some(final_pos);
                        }
                    }
                } else {
                    for try_count in (min..=max_count).rev() {
                        let consumed = byte_positions[try_count];
                        if let Some(final_pos) = self.match_elements_backtracking_dotall(
                            text,
                            elem_idx + 1,
                            text_pos + consumed,
                        ) {
                            return Some(final_pos);
                        }
                    }
                }
                None
            }
            // Other elements use standard matching
            _ => {
                if let Some(consumed) = element.match_at(text, text_pos) {
                    self.match_elements_backtracking_dotall(text, elem_idx + 1, text_pos + consumed)
                } else {
                    None
                }
            }
        }
    }

    /// Element-level Thompson NFA simulation using pre-computed transition table.
    /// Returns Some(bool) if NFA table is available, None otherwise.
    #[inline]
    fn is_match_nfa(&self, text: &str) -> Option<bool> {
        let table = self.nfa_table.as_ref()?;

        // OPTIMIZATION: If pattern has a single Char element (inner literal),
        // use memchr to find it first, then verify context.
        if let Some(result) = self.is_match_nfa_with_char_prefilter(table, text) {
            return Some(result);
        }

        // Full NFA scan
        Some(self.run_nfa(table, text.as_bytes()))
    }

    /// Try memchr-based prefilter for patterns with a Char element.
    /// Returns Some(bool) if applicable, None to fall through to full NFA.
    #[inline]
    fn is_match_nfa_with_char_prefilter(&self, table: &NfaTable, text: &str) -> Option<bool> {
        // Find the Char element (if exactly one exists and not at position 0 or n-1)
        let n = table.n;
        let mut char_idx = None;
        let mut char_byte = 0u8;
        for (i, elem) in self.elements.iter().enumerate() {
            if let SequenceElement::Char(ch) = elem {
                if i > 0 && i < n - 1 && (*ch as u32) < 128 {
                    char_idx = Some(i);
                    char_byte = *ch as u8;
                    break;
                }
            }
        }
        let char_pos = char_idx?;

        // Use memchr to find the literal char
        let bytes = text.as_bytes();
        let mut search_pos = 0;
        while search_pos < bytes.len() {
            let found = memchr::memchr(char_byte, &bytes[search_pos..])?;
            let abs_pos = search_pos + found;

            // Verify left side: elements[0..char_pos] must match ending at abs_pos
            let left_ok = if char_pos == 0 {
                true
            } else {
                // Need at least char_pos bytes before (min 1 char per element)
                abs_pos >= char_pos && self.verify_left(table, &bytes[..abs_pos], char_pos)
            };

            if left_ok {
                // Verify right side: elements[char_pos+1..n] must match starting after abs_pos
                let right_start = abs_pos + 1;
                let right_ok = if char_pos + 1 >= n {
                    true
                } else {
                    right_start < bytes.len()
                        && self.verify_right(table, &bytes[right_start..], char_pos + 1)
                };

                if right_ok {
                    return Some(true);
                }
            }

            search_pos = abs_pos + 1;
        }

        Some(false)
    }

    /// Verify left side of pattern matches (elements[0..end_idx] in text ending at text end)
    #[inline]
    fn verify_left(&self, table: &NfaTable, text: &[u8], end_idx: usize) -> bool {
        // Check that at least one char at the end matches elements[end_idx-1]
        // and recursively backward. For simplicity, just check min-length matching.
        let byte_elem_mask = &table.byte_elem_mask;
        // Run NFA on the left text, checking if we reach state end_idx-1 (min met)
        let target_mask = 1u32 << (end_idx - 1);
        let quantified_bits = table.quantified_bits;
        let mut active: u32 = 0;

        for &byte in text {
            let elem_mask = if byte < 128 {
                byte_elem_mask[byte as usize]
            } else {
                0
            };
            active = (active & elem_mask & quantified_bits)
                | ((active << 1) & elem_mask)
                | (elem_mask & 1u32);
            // Only keep bits for elements before the char
            active &= (1u32 << end_idx) - 1;
            if active & target_mask != 0 {
                return true;
            }
        }
        false
    }

    /// Verify right side of pattern matches (elements[start_idx..n] in text)
    #[inline]
    fn verify_right(&self, table: &NfaTable, text: &[u8], start_idx: usize) -> bool {
        let byte_elem_mask = &table.byte_elem_mask;
        let accept_mask = table.accept_mask;
        let quantified_bits = table.quantified_bits;
        let start_bit = 1u32 << start_idx;
        let mut active: u32 = 0;

        for &byte in text {
            let elem_mask = if byte < 128 {
                byte_elem_mask[byte as usize]
            } else {
                0
            };
            // Start at start_idx instead of 0
            let starts = if elem_mask & start_bit != 0 {
                start_bit
            } else {
                0
            };
            active = (active & elem_mask & quantified_bits) | ((active << 1) & elem_mask) | starts;
            // Only keep bits for elements at or after start_idx
            active &= !((1u32 << start_idx) - 1);
            if active & accept_mask != 0 {
                return true;
            }
        }
        false
    }

    /// Run the full NFA scan on byte slice
    #[inline]
    fn run_nfa(&self, table: &NfaTable, bytes: &[u8]) -> bool {
        let accept_mask = table.accept_mask;
        let quantified_bits = table.quantified_bits;
        let optional_bits = table.optional_bits;
        let byte_elem_mask = &table.byte_elem_mask;

        // Start at element 0, then compute initial epsilon closure
        let mut active: u32 = 1u32; // Initially at element 0
        active = self.compute_epsilon_closure(active, optional_bits, table.n);

        // Check if we can accept before processing any bytes (e.g., pattern "a*" matching "")
        if active & accept_mask != 0 {
            return true;
        }

        for &byte in bytes {
            let elem_mask = if byte < 128 {
                byte_elem_mask[byte as usize]
            } else {
                let mut mask = 0u32;
                for (i, elem) in self.elements.iter().enumerate() {
                    if let SequenceElement::QuantifiedCharClass(cc, _) = elem {
                        if cc.negated {
                            mask |= 1u32 << i;
                        }
                    }
                }
                mask
            };

            // Standard NFA transition
            let mut next_active = (active & elem_mask & quantified_bits)  // Stay in quantified element
                | ((active << 1) & elem_mask)                             // Advance to next element
                | (elem_mask & 1u32); // Start at element 0 (for ".*" patterns)

            // Compute epsilon closure after transition
            next_active = self.compute_epsilon_closure(next_active, optional_bits, table.n);

            active = next_active;

            if active & accept_mask != 0 {
                return true;
            }
        }

        false
    }

    /// Compute epsilon closure: for each active state, follow epsilon transitions
    /// An epsilon transition exists from element i to element i+1 if element i is optional (min=0)
    #[inline]
    fn compute_epsilon_closure(&self, mut active: u32, optional_bits: u32, n: usize) -> u32 {
        // Repeatedly follow epsilon transitions until no new states are added
        loop {
            let before = active;

            // For each active position i, if element i is optional, we can also be at i+1
            for i in 0..n {
                if (active & (1u32 << i)) != 0 && (optional_bits & (1u32 << i)) != 0 {
                    // Element i is active and optional, so we can skip it
                    if i + 1 < n {
                        active |= 1u32 << (i + 1);
                    }
                }
            }

            if active == before {
                break;
            }
        }
        active
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
                            self.match_at_skip_from(text, after_prefix, skip_count)
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
                            self.match_at_skip_from(text, after_prefix, skip_count)
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
                        return Some((match_start, match_end));
                    }
                }
                return None;
            } else if anchor_literal.len() == 1 {
                // Single char inner literal - use memchr for rare/distinctive chars
                let byte = anchor_literal[0];
                if !byte.is_ascii_alphanumeric() && byte != b' ' && byte != b'.' {
                    // Rare char (like @, #, :, !, etc.) - memchr is effective
                    let mut pos = 0;
                    while pos < text.len() {
                        if let Some(found) = memchr::memchr(byte, &text.as_bytes()[pos..]) {
                            let anchor_pos = pos + found;
                            if let Some((match_start, match_end)) = self.match_around_anchor(
                                text,
                                anchor_pos,
                                1,
                                before_count,
                                after_count,
                            ) {
                                return Some((match_start, match_end));
                            }
                            pos = anchor_pos + 1;
                        } else {
                            return None;
                        }
                    }
                    return None;
                }
            }
        }

        // OPTIMIZATION 3: Charclass prefilter - use most selective element
        if let (Some(first), Some(last)) = (self.elements.first(), self.elements.last()) {
            let first_cc = match first {
                SequenceElement::QuantifiedCharClass(cc, q) => {
                    // Only use as prefilter if NOT optional (min > 0)
                    let (min, _) = quantifier_bounds(q);
                    if min > 0 {
                        Some(cc)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            let last_cc = match last {
                SequenceElement::QuantifiedCharClass(cc, _) => Some(cc),
                _ => None,
            };

            // Choose the most selective element as prefilter
            let use_last = match (first_cc, last_cc) {
                (Some(fc), Some(lc)) => Self::selectivity(lc) < Self::selectivity(fc),
                _ => false,
            };

            if use_last {
                if let Some(lc) = last_cc {
                    let fc = first_cc.unwrap();
                    // Reverse prefilter: find last element, then search backward for first element
                    let mut pos = 0;
                    while pos < text.len() {
                        if let Some(offset) = lc.find_first(&text[pos..]) {
                            let suffix_pos = pos + offset;
                            // Find where first element matches, working backwards from suffix
                            // Only check positions where first element actually matches
                            let search_start = suffix_pos.saturating_sub(128);
                            let window = &text[search_start..=suffix_pos.min(text.len() - 1)];
                            let mut try_pos = 0;
                            let found = false;
                            while try_pos < window.len() {
                                if let Some(foffset) = fc.find_first(&window[try_pos..]) {
                                    let abs_start = search_start + try_pos + foffset;
                                    if let Some(consumed) = self.match_at_pos(text, abs_start) {
                                        return Some((abs_start, abs_start + consumed));
                                    }
                                    try_pos += foffset + 1;
                                } else {
                                    break;
                                }
                            }
                            pos = suffix_pos + 1;
                        } else {
                            break;
                        }
                    }
                    return None;
                }
            } else if let Some(fc) = first_cc {
                // Forward prefilter: find first element, then try full match
                let mut pos = 0;
                while pos < text.len() {
                    if let Some(offset) = fc.find_first(&text[pos..]) {
                        let start_pos = pos + offset;
                        if let Some(consumed) = self.match_at_pos(text, start_pos) {
                            return Some((start_pos, start_pos + consumed));
                        }
                        pos = start_pos + 1;
                    } else {
                        break;
                    }
                }
                return None;
            }
        }

        // Fallback: sequential search
        for (start_pos, _) in text.char_indices() {
            if let Some(consumed) = self.match_at_pos(text, start_pos) {
                return Some((start_pos, start_pos + consumed));
            }
        }

        None
    }

    /// Estimate how many characters a CharClass matches (lower = more selective)
    fn selectivity(cc: &CharClass) -> usize {
        if cc.negated {
            return 1000; // Negated classes match almost everything
        }
        let mut count = cc.chars.len();
        for &(start, end) in &cc.ranges {
            count += (end as usize) - (start as usize) + 1;
        }
        count
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

            if !literal_bytes.is_empty() {
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
            match elem.match_at(text, pos) {
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
                SequenceElement::Boundary(boundary_type) => {
                    // Boundary is zero-width, just check if it matches at current position
                    if !boundary_type.matches_at(text, match_start) {
                        return None;
                    }
                    // Don't move match_start since boundary is zero-width
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

        // Use backtracking-enabled matching
        self.match_elements_backtracking(text, start_idx, 0)
    }

    /// Match remaining elements starting from a specific position in the full text
    /// This preserves context needed for boundary checks
    /// Returns the number of bytes consumed (not absolute position)
    fn match_at_skip_from(&self, text: &str, text_pos: usize, skip_count: usize) -> Option<usize> {
        if skip_count == 0 {
            return Some(0); // No more elements to match
        }

        let start_idx = self.elements.len() - skip_count;

        // Use backtracking with the full text and actual position
        // Returns absolute end position, convert to bytes consumed
        self.match_elements_backtracking(text, start_idx, text_pos)
            .map(|end_pos| end_pos - text_pos)
    }

    /// Match elements with backtracking support for quantified elements
    fn match_elements_backtracking(
        &self,
        text: &str,
        elem_idx: usize,
        text_pos: usize,
    ) -> Option<usize> {
        // Base case: all elements matched
        if elem_idx >= self.elements.len() {
            return Some(text_pos);
        }

        let elem = &self.elements[elem_idx];

        match elem {
            // Quantified elements: try different match lengths (greedy first, then backtrack)
            SequenceElement::QuantifiedChar(ch, quantifier) => {
                self.backtrack_quantified_char(*ch, quantifier, text, text_pos, elem_idx)
            }
            SequenceElement::QuantifiedCharClass(cc, quantifier) => {
                self.backtrack_quantified_charclass(cc, quantifier, text, text_pos, elem_idx)
            }
            SequenceElement::QuantifiedGroup(group, quantifier) => {
                self.backtrack_quantified_group(group, quantifier, text, text_pos, elem_idx)
            }
            // Non-quantified elements: simple match
            _ => {
                if let Some(consumed) = elem.match_at(text, text_pos) {
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + consumed)
                } else {
                    None
                }
            }
        }
    }

    /// Backtracking for quantified char: try greedy first (or lazy first for lazy quantifiers)
    fn backtrack_quantified_char(
        &self,
        ch: char,
        quantifier: &Quantifier,
        text: &str,
        text_pos: usize,
        elem_idx: usize,
    ) -> Option<usize> {
        let (min, max) = quantifier_bounds(quantifier);
        let is_lazy = quantifier.is_lazy();
        let remaining = &text[text_pos..];
        let ch_len = ch.len_utf8();

        // Count maximum possible matches and build byte position table
        let mut max_count = 0;
        let mut byte_offset = 0;
        // Pre-compute cumulative byte positions to avoid O(n²)
        let mut byte_positions: Vec<usize> = Vec::new();
        byte_positions.push(0);
        for c in remaining.chars() {
            if c == ch {
                max_count += 1;
                byte_offset += ch_len;
                byte_positions.push(byte_offset);
                if max_count >= max {
                    break;
                }
            } else {
                break;
            }
        }

        if max_count < min {
            return None;
        }
        max_count = max_count.min(max);

        if is_lazy {
            // Lazy: try from min to max_count (prefer shorter matches first)
            for try_count in min..=max_count {
                let consumed = byte_positions[try_count];
                if let Some(final_pos) =
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + consumed)
                {
                    return Some(final_pos);
                }
            }
        } else {
            // Greedy: try from max_count down to min (prefer longer matches first)
            for try_count in (min..=max_count).rev() {
                let consumed = byte_positions[try_count];
                if let Some(final_pos) =
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + consumed)
                {
                    return Some(final_pos);
                }
            }
        }
        None
    }

    /// Backtracking for quantified charclass: try greedy first (or lazy first for lazy quantifiers)
    fn backtrack_quantified_charclass(
        &self,
        cc: &CharClass,
        quantifier: &Quantifier,
        text: &str,
        text_pos: usize,
        elem_idx: usize,
    ) -> Option<usize> {
        let (min, max) = quantifier_bounds(quantifier);
        let is_lazy = quantifier.is_lazy();
        let remaining = &text[text_pos..];
        let rem_bytes = remaining.as_bytes();

        // Fast path for ASCII-only remaining text
        let is_ascii = rem_bytes
            .iter()
            .take(rem_bytes.len().min(256))
            .all(|&b| b < 128);

        if is_ascii {
            // FAST: For negated single-char class (like [^\n] from .+), use memchr
            let max_count;
            if cc.negated && cc.chars.len() == 1 && cc.ranges.is_empty() {
                let forbidden = cc.chars[0] as u8;
                let term_pos = memchr::memchr(forbidden, rem_bytes).unwrap_or(rem_bytes.len());
                max_count = term_pos.min(max);
            } else {
                // Count matching bytes
                let mut count = 0;
                for &byte in rem_bytes {
                    if cc.matches(byte as char) {
                        count += 1;
                        if count >= max {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                max_count = count;
            }

            if max_count < min {
                return None;
            }

            // For lazy quantifiers, skip the optimization and just try from min to max
            if is_lazy {
                for try_count in min..=max_count {
                    if let Some(final_pos) =
                        self.match_elements_backtracking(text, elem_idx + 1, text_pos + try_count)
                    {
                        return Some(final_pos);
                    }
                }
                return None;
            }

            // OPTIMIZATION: If next element is a QuantifiedCharClass, find its first match
            // position from the right instead of trying every position
            if let Some(next_cc) = self.peek_next_charclass(elem_idx + 1) {
                // Scan from right to left within the matched region for next element
                let end = text_pos + max_count;
                let mut try_pos = end;
                while try_pos > text_pos + min {
                    let byte = text.as_bytes()[try_pos - 1];
                    if !cc.matches(byte as char) {
                        break;
                    }
                    if next_cc.matches(byte as char) {
                        // Found a position where next element can start
                        let consumed = try_pos - 1 - text_pos;
                        if consumed >= min {
                            if let Some(final_pos) = self.match_elements_backtracking(
                                text,
                                elem_idx + 1,
                                text_pos + consumed,
                            ) {
                                return Some(final_pos);
                            }
                        }
                    }
                    try_pos -= 1;
                }
                // Don't return None here - fall through to standard backtracking
                // This handles the case when max_count = 0 (quantifier matches 0 times)
            }

            // Standard greedy backtracking
            for try_count in (min..=max_count).rev() {
                if let Some(final_pos) =
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + try_count)
                {
                    return Some(final_pos);
                }
            }
            return None;
        }

        // UTF-8 path: pre-compute cumulative byte positions
        let mut max_count = 0;
        let mut byte_offset = 0;
        let mut byte_positions: Vec<usize> = Vec::new();
        byte_positions.push(0);
        for c in remaining.chars() {
            if cc.matches(c) {
                max_count += 1;
                byte_offset += c.len_utf8();
                byte_positions.push(byte_offset);
                if max_count >= max {
                    break;
                }
            } else {
                break;
            }
        }

        if max_count < min {
            return None;
        }
        max_count = max_count.min(max);

        if is_lazy {
            // Lazy: try from min to max_count (prefer shorter matches first)
            for try_count in min..=max_count {
                let consumed = byte_positions[try_count];
                if let Some(final_pos) =
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + consumed)
                {
                    return Some(final_pos);
                }
            }
        } else {
            // Greedy: try from max_count down to min (prefer longer matches first)
            for try_count in (min..=max_count).rev() {
                let consumed = byte_positions[try_count];
                if let Some(final_pos) =
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + consumed)
                {
                    return Some(final_pos);
                }
            }
        }
        None
    }

    /// Backtracking for quantified group: try greedy first (or lazy first for lazy quantifiers)
    fn backtrack_quantified_group(
        &self,
        group: &Group,
        quantifier: &Quantifier,
        text: &str,
        text_pos: usize,
        elem_idx: usize,
    ) -> Option<usize> {
        let (min, max) = quantifier_bounds(quantifier);
        let is_lazy = quantifier.is_lazy();

        // Find all possible match lengths by repeatedly matching the group
        let mut byte_positions: Vec<usize> = Vec::new();
        byte_positions.push(0); // 0 repetitions = 0 bytes consumed
        let mut current_pos = text_pos;
        let mut count = 0;

        while count < max && current_pos <= text.len() {
            if let Some(consumed) = group.match_at(text, current_pos) {
                if consumed == 0 {
                    break; // Prevent infinite loop on zero-width matches
                }
                current_pos += consumed;
                count += 1;
                byte_positions.push(current_pos - text_pos);
            } else {
                break;
            }
        }

        if count < min {
            return None;
        }
        let max_count = count.min(max);

        if is_lazy {
            // Lazy: try from min to max_count (prefer shorter matches first)
            for try_count in min..=max_count {
                let consumed = byte_positions[try_count];
                if let Some(final_pos) =
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + consumed)
                {
                    return Some(final_pos);
                }
            }
        } else {
            // Greedy: try from max_count down to min (prefer longer matches first)
            for try_count in (min..=max_count).rev() {
                let consumed = byte_positions[try_count];
                if let Some(final_pos) =
                    self.match_elements_backtracking(text, elem_idx + 1, text_pos + consumed)
                {
                    return Some(final_pos);
                }
            }
        }
        None
    }

    /// Peek at the next element's CharClass if it's a QuantifiedCharClass
    fn peek_next_charclass(&self, next_idx: usize) -> Option<&CharClass> {
        match self.elements.get(next_idx) {
            Some(SequenceElement::QuantifiedCharClass(cc, _)) => Some(cc),
            _ => None,
        }
    }

    /// Find all occurrences of the sequence in text
    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        let mut results: Vec<(usize, usize)> = Vec::new();

        // OPTIMIZATION 1: Use literal prefix with memchr
        if let Some((prefix_bytes, skip_count)) = self.extract_literal_prefix() {
            if prefix_bytes.len() >= 3 {
                // Multi-byte prefix: use memmem
                use memchr::memmem;
                let finder = memmem::Finder::new(&prefix_bytes);

                for found_pos in finder.find_iter(text.as_bytes()) {
                    let after_prefix = found_pos + prefix_bytes.len();
                    if let Some(consumed) = self.match_at_skip_from(text, after_prefix, skip_count)
                    {
                        let match_start = found_pos;
                        let match_end = after_prefix + consumed;
                        // Check if this match overlaps with previous
                        if results.is_empty() || match_start >= results.last().unwrap().1 {
                            results.push((match_start, match_end));
                        }
                    }
                }
                return results;
            } else if prefix_bytes.len() == 1 {
                // Single byte prefix: use memchr_iter
                use memchr::memchr_iter;
                let byte = prefix_bytes[0];

                for found_pos in memchr_iter(byte, text.as_bytes()) {
                    let after_prefix = found_pos + 1;
                    if let Some(consumed) = self.match_at_skip_from(text, after_prefix, skip_count)
                    {
                        let match_start = found_pos;
                        let match_end = after_prefix + consumed;
                        // Check if this match overlaps with previous
                        if results.is_empty() || match_start >= results.last().unwrap().1 {
                            results.push((match_start, match_end));
                        }
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
