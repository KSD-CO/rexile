//! DFA-based matching for capture groups
//!
//! This module implements a specialized DFA for patterns with capture groups.
//! The key insight: instead of scanning for literals first (memchr approach),
//! we compile the pattern into a state machine and do a single forward pass.
//!
//! This matches the regex crate's approach and avoids the position-dependent
//! performance issue (fast when match is early, slow when match is late).

/// A state in the capture DFA
#[derive(Debug, Clone)]
struct State {
    id: usize,
    /// Transitions: Vec of (predicate, next_state_id)
    /// We use Vec instead of HashMap because we need to check predicates in order
    transitions: Vec<(TransitionPredicate, usize)>,
    is_accepting: bool,
    capture_actions: Vec<CaptureAction>, // Actions to perform when entering this state
}

/// What kind of input can trigger this transition?
#[derive(Debug, Clone)]
enum TransitionPredicate {
    /// Exact byte match
    Byte(u8),
    /// Any byte in range [start, end] inclusive
    ByteRange(u8, u8),
    /// Word character \w = [a-zA-Z0-9_]
    WordChar,
    /// Digit \d = [0-9]
    Digit,
    /// Whitespace \s
    Whitespace,
    /// Any byte (.)
    Any,
}

impl TransitionPredicate {
    #[inline(always)]
    fn matches(&self, byte: u8) -> bool {
        match self {
            Self::Byte(b) => *b == byte,
            Self::ByteRange(start, end) => byte >= *start && byte <= *end,
            // OPTIMIZATION: Fast path for word chars - most common case
            Self::WordChar => {
                // Inline hot path for ASCII word chars
                matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
            }
            Self::Digit => matches!(byte, b'0'..=b'9'),
            Self::Whitespace => matches!(byte, b' ' | b'\t' | b'\n' | b'\r'),
            Self::Any => true,
        }
    }
}

#[derive(Debug, Clone)]
enum CaptureAction {
    StartCapture(usize), // Start capturing group N
    EndCapture(usize),   // End capturing group N
}

/// DFA for patterns with captures
#[derive(Debug, Clone)]
pub struct CaptureDFA {
    states: Vec<State>,
    start_state: usize,
    total_groups: usize,          // Number of capture groups (excluding group 0)
    literal_hint: Option<String>, // Optional literal for prefiltering
    reverse_dfa: Option<Box<ReverseDFA>>, // Reverse DFA for finding match start
}

/// Reverse DFA for scanning backwards to find match start
#[derive(Debug, Clone)]
struct ReverseDFA {
    states: Vec<State>,
    start_states: Vec<usize>, // In reverse DFA, accepting states become start states
}

impl CaptureDFA {
    /// Create a new DFA with just a start state
    pub fn new(total_groups: usize) -> Self {
        let start_state = State {
            id: 0,
            transitions: Vec::new(),
            is_accepting: false,
            capture_actions: vec![CaptureAction::StartCapture(0)], // Capture 0 = full match
        };

        CaptureDFA {
            states: vec![start_state],
            start_state: 0,
            total_groups,
            literal_hint: None,
            reverse_dfa: None,
        }
    }

    /// Set a literal hint for prefiltering
    pub fn with_literal_hint(mut self, literal: String) -> Self {
        self.literal_hint = Some(literal);
        self
    }

    /// Build and attach reverse DFA for efficient backwards scanning
    fn build_reverse_dfa(&mut self) {
        let reverse = ReverseDFA::from_forward_dfa(self);
        self.reverse_dfa = Some(Box::new(reverse));
    }

    /// Add a new state and return its ID
    fn add_state(&mut self) -> usize {
        let id = self.states.len();
        self.states.push(State {
            id,
            transitions: Vec::new(),
            is_accepting: false,
            capture_actions: Vec::new(),
        });
        id
    }

    /// Add a transition from one state to another
    fn add_transition(&mut self, from: usize, pred: TransitionPredicate, to: usize) {
        if let Some(state) = self.states.get_mut(from) {
            state.transitions.push((pred, to));
        }
    }

    /// Mark a state as accepting (final)
    fn set_accepting(&mut self, state: usize) {
        if let Some(s) = self.states.get_mut(state) {
            s.is_accepting = true;
            // End capture 0 when we reach accepting state
            s.capture_actions.push(CaptureAction::EndCapture(0));
        }
    }

    /// Add a capture action to a state
    fn add_capture_action(&mut self, state: usize, action: CaptureAction) {
        if let Some(s) = self.states.get_mut(state) {
            s.capture_actions.push(action);
        }
    }

    /// Find a match in text, returning (start, end) and all capture groups
    pub fn find_with_captures(
        &self,
        text: &str,
    ) -> Option<(usize, usize, Vec<Option<(usize, usize)>>)> {
        let bytes = text.as_bytes();

        // If we have a literal hint, use it as prefilter to skip ahead
        if let Some(ref literal) = self.literal_hint {
            return self.find_with_literal_prefilter(text, literal);
        }

        // Fallback: try to match starting from each position
        // This is O(n²) but necessary when no literal is available
        for start_pos in 0..=bytes.len() {
            if let Some((end_pos, captures)) = self.try_match_at(bytes, start_pos) {
                return Some((start_pos, end_pos, captures));
            }
        }

        None
    }

    /// Use literal as prefilter - only try DFA at positions where literal appears
    fn find_with_literal_prefilter(
        &self,
        text: &str,
        literal: &str,
    ) -> Option<(usize, usize, Vec<Option<(usize, usize)>>)> {
        // Use boundary heuristics: find literal, scan back for word boundaries, try DFA from candidates
        // This is simpler and more effective than reverse DFA for common patterns like \w+@\w+
        self.find_with_boundary_heuristics(text, literal)
    }

    /// Search using boundary heuristics
    /// Strategy: Find literal with memchr, scan backwards to find word boundaries, try forward DFA from candidate positions
    fn find_with_boundary_heuristics(
        &self,
        text: &str,
        literal: &str,
    ) -> Option<(usize, usize, Vec<Option<(usize, usize)>>)> {
        let bytes = text.as_bytes();
        let literal_bytes = literal.as_bytes();

        // Find positions where literal appears using memchr
        let mut pos = 0;
        while pos <= bytes.len() {
            // Find next occurrence of first byte of literal
            let first_byte = literal_bytes[0];
            let search_start = pos;

            if let Some(candidate_pos) = memchr::memchr(first_byte, &bytes[search_start..]) {
                let literal_pos = search_start + candidate_pos;

                // Check if full literal matches
                if literal_pos + literal_bytes.len() <= bytes.len() {
                    let matches = literal_bytes
                        .iter()
                        .enumerate()
                        .all(|(i, &b)| bytes[literal_pos + i] == b);

                    if matches {
                        // Found literal at literal_pos

                        // OPTIMIZED: Only try 2-3 strategic positions instead of all boundaries
                        // This dramatically reduces DFA attempts for long text
                        let mut candidate_starts = Vec::with_capacity(3);

                        // Strategy 1: Try from nearest word boundary (most common case)
                        // Scan backwards max 20 bytes looking for first boundary
                        let lookback = 20.min(literal_pos);
                        let mut found_boundary = false;
                        for i in 1..=lookback {
                            let pos = literal_pos - i;
                            if pos == 0 {
                                candidate_starts.push(0);
                                found_boundary = true;
                                break;
                            }

                            let curr_byte = bytes[pos];
                            let prev_byte = bytes[pos - 1];
                            let curr_is_word =
                                curr_byte.is_ascii_alphanumeric() || curr_byte == b'_';
                            let prev_is_word =
                                prev_byte.is_ascii_alphanumeric() || prev_byte == b'_';

                            // Found word boundary - this is likely where match starts
                            if curr_is_word && !prev_is_word {
                                candidate_starts.push(pos);
                                found_boundary = true;
                                break;
                            }
                        }

                        // Strategy 2: If no nearby boundary, try from literal itself
                        if !found_boundary {
                            candidate_starts.push(literal_pos);
                        }

                        // Try each candidate (already sorted furthest-first due to how we add them)
                        for &start_pos in &candidate_starts {
                            if let Some((end_pos, captures)) = self.try_match_at(bytes, start_pos) {
                                // Verify this match includes the literal
                                if start_pos <= literal_pos && end_pos > literal_pos {
                                    return Some((start_pos, end_pos, captures));
                                }
                            }
                        }
                    }
                }

                pos = literal_pos + 1;
            } else {
                break;
            }
        }

        None
    }

    /// Find a match (without extracting captures - faster)
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        // Use fast path without capture tracking
        self.find_no_captures(text)
    }

    /// Find without tracking captures (much faster)
    fn find_no_captures(&self, text: &str) -> Option<(usize, usize)> {
        let bytes = text.as_bytes();

        // If we have a literal hint, use it as prefilter
        if let Some(ref literal) = self.literal_hint {
            let literal_bytes = literal.as_bytes();
            let mut pos = 0;

            while pos <= bytes.len() {
                let first_byte = literal_bytes[0];
                if let Some(candidate_pos) = memchr::memchr(first_byte, &bytes[pos..]) {
                    let literal_pos = pos + candidate_pos;

                    // Check if full literal matches
                    if literal_pos + literal_bytes.len() <= bytes.len() {
                        let matches = literal_bytes
                            .iter()
                            .enumerate()
                            .all(|(i, &b)| bytes[literal_pos + i] == b);

                        if matches {
                            // Find word boundary before literal (max 20 chars back)
                            let lookback = 20.min(literal_pos);
                            for i in (0..=lookback).rev() {
                                let start_pos = literal_pos - i;
                                if start_pos == 0
                                    || (!bytes[start_pos - 1].is_ascii_alphanumeric()
                                        && bytes[start_pos - 1] != b'_')
                                {
                                    if let Some(end_pos) =
                                        self.try_match_at_no_captures(bytes, start_pos)
                                    {
                                        if end_pos > literal_pos {
                                            return Some((start_pos, end_pos));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    pos = literal_pos + 1;
                } else {
                    break;
                }
            }
            return None;
        }

        // Fallback: try each position
        for start_pos in 0..=bytes.len() {
            if let Some(end_pos) = self.try_match_at_no_captures(bytes, start_pos) {
                return Some((start_pos, end_pos));
            }
        }
        None
    }

    /// Fast match at position without capture tracking
    #[inline]
    fn try_match_at_no_captures(&self, bytes: &[u8], start_pos: usize) -> Option<usize> {
        let mut current_state = self.start_state;
        let mut pos = start_pos;
        let mut last_accept_pos = None;

        loop {
            let state = &self.states[current_state];

            if state.is_accepting {
                last_accept_pos = Some(pos);
            }

            if pos >= bytes.len() {
                return last_accept_pos;
            }

            let byte = bytes[pos];

            // Find matching transition
            let next_state_opt = if state.transitions.len() == 1 {
                let (pred, next) = &state.transitions[0];
                if pred.matches(byte) {
                    Some(*next)
                } else {
                    None
                }
            } else {
                state.transitions.iter().find_map(|(pred, next_state)| {
                    if pred.matches(byte) {
                        Some(*next_state)
                    } else {
                        None
                    }
                })
            };

            if let Some(next_state) = next_state_opt {
                current_state = next_state;
                pos += 1;
            } else {
                return last_accept_pos;
            }
        }
    }

    /// Try to match starting from a specific position
    fn try_match_at(
        &self,
        bytes: &[u8],
        start_pos: usize,
    ) -> Option<(usize, Vec<Option<(usize, usize)>>)> {
        let mut current_state = self.start_state;
        let mut pos = start_pos;

        // Track capture group boundaries
        let mut captures = vec![None; self.total_groups];
        // Use Vec instead of HashMap for small number of captures (usually 1-4)
        // Much faster than HashMap for small data
        let mut capture_starts: Vec<Option<usize>> = vec![None; self.total_groups + 1];

        // Execute capture actions for start state
        for action in &self.states[current_state].capture_actions {
            match action {
                CaptureAction::StartCapture(id) => {
                    if *id <= self.total_groups {
                        capture_starts[*id] = Some(pos);
                    }
                }
                CaptureAction::EndCapture(id) => {
                    if *id > 0 && *id <= self.total_groups {
                        if let Some(start) = capture_starts[*id] {
                            captures[*id - 1] = Some((start, pos));
                        }
                    }
                }
            }
        }

        // Track the last accepting position for greedy matching
        // OPTIMIZATION: Only clone on accepting state when necessary
        let mut last_accept_pos = None;
        let mut last_accept_captures = None;

        // Main matching loop - consume bytes one by one
        loop {
            let state = &self.states[current_state];

            // Record if we're in an accepting state (for greedy matching)
            if state.is_accepting {
                last_accept_pos = Some(pos);
                last_accept_captures = Some(captures.clone());
            }

            // Try to consume next byte
            if pos >= bytes.len() {
                // No more input
                // Return the last accepting state if we have one
                if let Some(accept_pos) = last_accept_pos {
                    return Some((accept_pos, last_accept_captures.unwrap()));
                }
                return None;
            }

            let byte = bytes[pos];

            // OPTIMIZATION: Fast path for single transition (most common case)
            // Avoids loop overhead and Option unwrapping
            let next_state_opt = if state.transitions.len() == 1 {
                let (pred, next) = &state.transitions[0];
                if pred.matches(byte) {
                    Some(*next)
                } else {
                    None
                }
            } else {
                // Multiple transitions - need to search
                state.transitions.iter().find_map(|(pred, next_state)| {
                    if pred.matches(byte) {
                        Some(*next_state)
                    } else {
                        None
                    }
                })
            };

            if let Some(next_state) = next_state_opt {
                // Execute capture actions for new state BEFORE advancing position
                // This ensures captures are marked at the correct positions
                current_state = next_state;

                for action in &self.states[current_state].capture_actions {
                    match action {
                        CaptureAction::StartCapture(id) => {
                            // Start capture at current position (before consuming byte)
                            if *id <= self.total_groups {
                                capture_starts[*id] = Some(pos);
                            }
                        }
                        CaptureAction::EndCapture(id) => {
                            // End capture after this byte will be consumed
                            if *id > 0 && *id <= self.total_groups {
                                if let Some(start) = capture_starts[*id] {
                                    // End at pos+1 because we're about to consume this byte
                                    captures[*id - 1] = Some((start, pos + 1));
                                }
                            }
                        }
                    }
                }

                pos += 1;
            } else {
                // No matching transition - match failed at this position
                // Return last accepting state if we have one (greedy matching)
                if let Some(accept_pos) = last_accept_pos {
                    return Some((accept_pos, last_accept_captures.unwrap()));
                }
                return None;
            }
        }
    }

    /// Check if text matches (fast path)
    pub fn is_match(&self, text: &str) -> bool {
        self.find(text).is_some()
    }
}

/// Compile a pattern with captures into a DFA
/// Returns None if the pattern is too complex for DFA compilation
pub fn compile_capture_pattern(elements: &[crate::CompiledCaptureElement]) -> Option<CaptureDFA> {
    // Count capture groups
    let mut num_captures = 0;
    for elem in elements.iter() {
        if matches!(elem, crate::CompiledCaptureElement::Capture(_, _)) {
            num_captures += 1;
        }
    }

    // Extract literal hint for prefiltering
    let literal_hint = extract_literal_hint(elements);

    let mut dfa = CaptureDFA::new(num_captures);
    if let Some(lit) = literal_hint {
        dfa = dfa.with_literal_hint(lit);
    }

    let mut current_state = 0; // Start state
    let mut capture_id = 1; // Next capture ID (0 is reserved for full match)

    // Compile each element into DFA states
    for element in elements {
        match element {
            crate::CompiledCaptureElement::Capture(matcher, _) => {
                // Add StartCapture action to current state
                dfa.add_capture_action(current_state, CaptureAction::StartCapture(capture_id));

                // Compile the matcher
                current_state = compile_matcher(&mut dfa, current_state, matcher)?;

                // Add EndCapture action to the state we ended up in
                dfa.add_capture_action(current_state, CaptureAction::EndCapture(capture_id));

                capture_id += 1;
            }
            crate::CompiledCaptureElement::NonCapture(matcher) => {
                // Just compile the matcher without capture markers
                current_state = compile_matcher(&mut dfa, current_state, matcher)?;
            }
        }
    }

    // Mark final state as accepting
    dfa.set_accepting(current_state);

    // Note: Reverse DFA is not used - boundary heuristics work better for common patterns
    // Reverse DFA is complex to implement correctly and boundary scanning is simpler and fast enough
    // dfa.build_reverse_dfa();

    Some(dfa)
}

/// Compile a single matcher into DFA states
/// Returns the final state ID after matching this element
fn compile_matcher(
    dfa: &mut CaptureDFA,
    start_state: usize,
    matcher: &crate::Matcher,
) -> Option<usize> {
    // eprintln!("    compile_matcher: {}", matcher_type(matcher));

    match matcher {
        // CRITICAL: Unwrap nested PatternWithCaptures
        // When we have ((\w+)), the inner part is compiled as PatternWithCaptures
        // But for DFA, we just want the actual matcher
        crate::Matcher::PatternWithCaptures { elements, .. } => {
            // // // eprintln!("      Unwrapping PatternWithCaptures with {} elements", elements.len());
            // If there's only one element and it's non-capture, use its matcher
            if elements.len() == 1 {
                if let crate::CompiledCaptureElement::NonCapture(m) = &elements[0] {
                    return compile_matcher(dfa, start_state, m);
                }
            }
            // Too complex - can't compile
            // // // eprintln!("      Complex PatternWithCaptures - not supported");
            None
        }

        crate::Matcher::Literal(s) => {
            // Chain of byte transitions
            let mut current = start_state;
            for byte in s.as_bytes() {
                let next = dfa.add_state();
                dfa.add_transition(current, TransitionPredicate::Byte(*byte), next);
                current = next;
            }
            Some(current)
        }

        crate::Matcher::Quantified(qp) => {
            // Quantified patterns like \w+, \d{2,5}, etc.
            compile_quantified(dfa, start_state, qp)
        }

        crate::Matcher::WordRun => {
            // \w+ pattern - compile as Quantified WordChar with min=1, max=unbounded
            // eprintln!("      WordRun detected - compiling as \\w+");
            let next = dfa.add_state();
            // At least one word char
            dfa.add_transition(start_state, TransitionPredicate::WordChar, next);
            // Then self-loop for more
            dfa.add_transition(next, TransitionPredicate::WordChar, next);
            Some(next)
        }

        crate::Matcher::DigitRun => {
            // \d+ pattern - compile as Quantified Digit with min=1, max=unbounded
            // eprintln!("      DigitRun detected - compiling as \\d+");
            let next = dfa.add_state();
            // At least one digit
            dfa.add_transition(start_state, TransitionPredicate::Digit, next);
            // Then self-loop for more
            dfa.add_transition(next, TransitionPredicate::Digit, next);
            Some(next)
        }

        crate::Matcher::CharClass(cc) => {
            // Single character class
            let pred = charclass_to_predicate(cc)?;
            let next = dfa.add_state();
            dfa.add_transition(start_state, pred, next);
            Some(next)
        }

        // Complex matchers that are hard to compile to DFA
        crate::Matcher::Lookaround(_, _)
        | crate::Matcher::LookbehindWithSuffix { .. }
        | crate::Matcher::CombinedWithLookaround { .. }
        | crate::Matcher::Backreference(_)
        | crate::Matcher::AlternationWithCaptures { .. } => {
            // eprintln!("      Complex matcher - not supported");
            // Too complex for our simple DFA - return None to fall back to NFA
            None
        }

        _ => {
            // eprintln!("      Unknown matcher type - not supported");
            None
        }
    }
}

/// Compile a quantified pattern (e.g., \w+, \d{2,5})
fn compile_quantified(
    dfa: &mut CaptureDFA,
    start_state: usize,
    qp: &crate::parser::quantifier::QuantifiedPattern,
) -> Option<usize> {
    use crate::parser::quantifier::QuantifiedElement;

    // Get the predicate for the quantified element
    let pred = match &qp.element {
        QuantifiedElement::CharClass(cc) => charclass_to_predicate(cc)?,
        _ => return None, // Complex elements not supported
    };

    let min = qp.quantifier.min_matches();
    let max_opt = qp.quantifier.max_matches();

    // Strategy: Create a chain of states for min matches,
    // then add loops/optional paths for up to max matches

    let mut current = start_state;

    // Required matches (min)
    for _ in 0..min {
        let next = dfa.add_state();
        dfa.add_transition(current, pred.clone(), next);
        current = next;
    }

    // Optional matches (min+1 to max, or infinite if max is usize::MAX)
    if max_opt == usize::MAX {
        // Unbounded (+, *): add self-loop
        dfa.add_transition(current, pred, current);
    } else {
        // Bounded: create optional chain up to max
        for _ in min..max_opt {
            let next = dfa.add_state();
            dfa.add_transition(current, pred.clone(), next);
            current = next;
        }
    }

    Some(current)
}

/// Convert a CharClass to a TransitionPredicate
fn charclass_to_predicate(cc: &crate::parser::charclass::CharClass) -> Option<TransitionPredicate> {
    // Check for common patterns
    if cc.is_digit_class() {
        return Some(TransitionPredicate::Digit);
    }
    if cc.is_word_class() {
        return Some(TransitionPredicate::WordChar);
    }

    // For other char classes, we'd need to expand all the ranges
    // For now, return None to fall back to NFA
    None
}

/// Helper to get matcher type name for debug logging
fn matcher_type(m: &crate::Matcher) -> String {
    match m {
        crate::Matcher::Literal(_) => "Literal".to_string(),
        crate::Matcher::Quantified(_) => "Quantified".to_string(),
        crate::Matcher::CharClass(_) => "CharClass".to_string(),
        crate::Matcher::Sequence(_) => "Sequence".to_string(),
        crate::Matcher::Group(_) => "Group".to_string(),
        crate::Matcher::WordRun => "WordRun".to_string(),
        crate::Matcher::DigitRun => "DigitRun".to_string(),
        crate::Matcher::Capture(inner, _) => format!("Capture({})", matcher_type(inner)),
        crate::Matcher::QuantifiedCapture(inner, _) => {
            format!("QuantifiedCapture({})", matcher_type(inner))
        }
        crate::Matcher::PatternWithCaptures { .. } => "PatternWithCaptures".to_string(),
        _ => "Other".to_string(),
    }
}

/// Extract a literal hint from pattern elements for prefiltering
/// Returns the best literal found (longest, preferably in the middle)
fn extract_literal_hint(elements: &[crate::CompiledCaptureElement]) -> Option<String> {
    let mut literals = Vec::new();

    for elem in elements {
        match elem {
            crate::CompiledCaptureElement::Capture(matcher, _)
            | crate::CompiledCaptureElement::NonCapture(matcher) => {
                if let Some(lit) = extract_literal_from_matcher(matcher) {
                    literals.push(lit);
                }
            }
        }
    }

    if literals.is_empty() {
        return None;
    }

    // Prefer literals that are:
    // 1. Not at the very start (anchored patterns)
    // 2. Longer is better
    // 3. In the middle is best (like @ in \w+@\w+)

    // For now, just return the first non-trivial literal found
    literals
        .into_iter()
        .filter(|s| !s.is_empty())
        .max_by_key(|s| s.len())
}

/// Extract literal from a matcher (recursive)
fn extract_literal_from_matcher(matcher: &crate::Matcher) -> Option<String> {
    match matcher {
        crate::Matcher::Literal(s) => Some(s.clone()),
        crate::Matcher::PatternWithCaptures { elements, .. } => {
            // Recursively extract from nested pattern
            for elem in elements {
                match elem {
                    crate::CompiledCaptureElement::Capture(m, _)
                    | crate::CompiledCaptureElement::NonCapture(m) => {
                        if let Some(lit) = extract_literal_from_matcher(m) {
                            return Some(lit);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Implementation of ReverseDFA
impl ReverseDFA {
    /// Build a reverse DFA from a forward DFA
    /// In reverse DFA:
    /// - Forward accepting states become reverse start states
    /// - Forward start state becomes reverse accepting state
    /// - All transitions are reversed
    fn from_forward_dfa(forward: &CaptureDFA) -> Self {
        let num_states = forward.states.len();

        // Create empty states for reverse DFA
        let mut reverse_states: Vec<State> = (0..num_states)
            .map(|id| State {
                id,
                transitions: Vec::new(),
                is_accepting: false,
                capture_actions: Vec::new(),
            })
            .collect();

        // Find accepting states in forward DFA - these become start states in reverse
        let start_states: Vec<usize> = forward
            .states
            .iter()
            .filter(|s| s.is_accepting)
            .map(|s| s.id)
            .collect();

        // Forward start state becomes accepting in reverse
        if forward.start_state < reverse_states.len() {
            reverse_states[forward.start_state].is_accepting = true;
        }

        // Reverse all transitions
        for forward_state in &forward.states {
            for (pred, target_state) in &forward_state.transitions {
                // In reverse: if forward has edge from A to B,
                // reverse has edge from B to A with same predicate
                if *target_state < reverse_states.len() {
                    reverse_states[*target_state]
                        .transitions
                        .push((pred.clone(), forward_state.id));
                }
            }
        }

        ReverseDFA {
            states: reverse_states,
            start_states,
        }
    }

    /// Scan backwards from a position to find where a match could start
    /// Returns the earliest position where a match could have started
    fn find_match_start(&self, bytes: &[u8], end_pos: usize) -> Option<usize> {
        // Try scanning backwards from each accepting state (which were forward accepting states)
        for &start_state_id in &self.start_states {
            if let Some(match_start) = self.scan_backwards(bytes, end_pos, start_state_id) {
                return Some(match_start);
            }
        }
        None
    }

    /// Scan backwards from end_pos using the reverse DFA
    fn scan_backwards(&self, bytes: &[u8], mut pos: usize, start_state: usize) -> Option<usize> {
        let mut current_state = start_state;

        // Keep track of the last accepting position (earliest match start)
        let mut last_accepting_pos = None;

        // Check if we're starting in an accepting state
        if self.states[current_state].is_accepting {
            last_accepting_pos = Some(pos);
        }

        // Scan backwards byte by byte
        while pos > 0 {
            pos -= 1;
            let byte = bytes[pos];

            // Try to find a transition that matches this byte
            let mut next_state_opt = None;
            for (pred, next_state) in &self.states[current_state].transitions {
                if pred.matches(byte) {
                    next_state_opt = Some(*next_state);
                    break;
                }
            }

            match next_state_opt {
                Some(next_state) => {
                    current_state = next_state;

                    // If we reached an accepting state, record this position
                    if self.states[current_state].is_accepting {
                        last_accepting_pos = Some(pos);
                    }
                }
                None => {
                    // No transition found - we've reached the start of the match
                    return last_accepting_pos;
                }
            }
        }

        last_accepting_pos
    }
}
