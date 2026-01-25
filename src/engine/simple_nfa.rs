//! Simple NFA (Non-deterministic Finite Automaton) for pattern matching
//! Implements Thompson's construction for basic regex patterns

use crate::parser::charclass::CharClass;
use crate::parser::quantifier::Quantifier;
use crate::parser::sequence::{Sequence, SequenceElement};

/// Simple NFA state
#[derive(Debug, Clone)]
enum State {
    /// Match a specific character
    Char(char),
    /// Match any character in a character class
    CharClass(CharClass),
    /// Epsilon transition (no input consumed)
    Epsilon,
    /// Split to multiple states (for quantifiers)
    Split,
    /// Accept state (match found)
    Accept,
}

/// NFA for pattern matching
pub struct SimpleNFA {
    states: Vec<State>,
    /// Transitions: state_id -> Vec<next_state_id>
    transitions: Vec<Vec<usize>>,
    start_state: usize,
    accept_state: usize,
}

impl SimpleNFA {
    /// Try to compile a sequence into NFA
    pub fn try_compile(seq: &Sequence) -> Option<Self> {
        let mut states = Vec::new();
        let mut transitions = Vec::new();

        // Start state
        let start_state = states.len();
        states.push(State::Epsilon);
        transitions.push(Vec::new());

        let mut current_state = start_state;

        // Build NFA from sequence elements
        for elem in &seq.elements {
            let next_state = Self::add_element(&mut states, &mut transitions, current_state, elem)?;
            current_state = next_state;
        }

        // Accept state
        let accept_state = states.len();
        states.push(State::Accept);
        transitions.push(Vec::new());
        transitions[current_state].push(accept_state);

        Some(SimpleNFA {
            states,
            transitions,
            start_state,
            accept_state,
        })
    }

    /// Add element to NFA
    fn add_element(
        states: &mut Vec<State>,
        transitions: &mut Vec<Vec<usize>>,
        from_state: usize,
        elem: &SequenceElement,
    ) -> Option<usize> {
        match elem {
            SequenceElement::Char(ch) => {
                let state_id = states.len();
                states.push(State::Char(*ch));
                transitions.push(Vec::new());
                transitions[from_state].push(state_id);
                Some(state_id)
            }
            SequenceElement::Dot => {
                // Dot wildcard - treat as CharClass matching anything except newline
                let state_id = states.len();
                use crate::parser::charclass::CharClass;
                let dot_class = CharClass::parse(
                    r"^
",
                )
                .ok()?;
                states.push(State::CharClass(dot_class));
                transitions.push(Vec::new());
                transitions[from_state].push(state_id);
                Some(state_id)
            }
            SequenceElement::CharClass(cc) => {
                let state_id = states.len();
                states.push(State::CharClass(cc.clone()));
                transitions.push(Vec::new());
                transitions[from_state].push(state_id);
                Some(state_id)
            }
            SequenceElement::QuantifiedChar(ch, q) => {
                Self::add_quantified_char(states, transitions, from_state, *ch, q)
            }
            SequenceElement::QuantifiedCharClass(cc, q) => {
                Self::add_quantified_charclass(states, transitions, from_state, cc, q)
            }
            SequenceElement::Literal(s) => {
                let mut current = from_state;
                for ch in s.chars() {
                    let state_id = states.len();
                    states.push(State::Char(ch));
                    transitions.push(Vec::new());
                    transitions[current].push(state_id);
                    current = state_id;
                }
                Some(current)
            }
            // Boundary is zero-width, so it doesn't add a state
            // SimpleNFA doesn't support boundaries - return None to fallback to other engines
            SequenceElement::Boundary(_) => None,
            // Groups not supported in simple NFA
            SequenceElement::Group(_) | SequenceElement::QuantifiedGroup(_, _) => None,
        }
    }

    /// Add quantified character (e.g., a+, a*, a?)
    fn add_quantified_char(
        states: &mut Vec<State>,
        transitions: &mut Vec<Vec<usize>>,
        from_state: usize,
        ch: char,
        quantifier: &Quantifier,
    ) -> Option<usize> {
        match quantifier {
            Quantifier::ZeroOrMore => {
                // a* : split -> char -> split (loop back)
                let split_state = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[from_state].push(split_state);

                let char_state = states.len();
                states.push(State::Char(ch));
                transitions.push(Vec::new());
                transitions[split_state].push(char_state);

                let end_split = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[char_state].push(end_split);

                // Loop back
                transitions[end_split].push(char_state);

                // Can skip
                transitions[split_state].push(end_split);

                Some(end_split)
            }
            Quantifier::OneOrMore => {
                // a+ : char -> split (loop back or continue)
                let char_state = states.len();
                states.push(State::Char(ch));
                transitions.push(Vec::new());
                transitions[from_state].push(char_state);

                let split_state = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[char_state].push(split_state);

                // Loop back
                transitions[split_state].push(char_state);

                Some(split_state)
            }
            Quantifier::ZeroOrOne => {
                // a? : split -> char or skip
                let split_state = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[from_state].push(split_state);

                let char_state = states.len();
                states.push(State::Char(ch));
                transitions.push(Vec::new());
                transitions[split_state].push(char_state);

                let end_state = states.len();
                states.push(State::Epsilon);
                transitions.push(Vec::new());
                transitions[char_state].push(end_state);

                // Can skip
                transitions[split_state].push(end_state);

                Some(end_state)
            }
            _ => None, // Other quantifiers not supported yet
        }
    }

    /// Add quantified character class
    fn add_quantified_charclass(
        states: &mut Vec<State>,
        transitions: &mut Vec<Vec<usize>>,
        from_state: usize,
        cc: &CharClass,
        quantifier: &Quantifier,
    ) -> Option<usize> {
        match quantifier {
            Quantifier::ZeroOrMore => {
                let split_state = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[from_state].push(split_state);

                let cc_state = states.len();
                states.push(State::CharClass(cc.clone()));
                transitions.push(Vec::new());
                transitions[split_state].push(cc_state);

                let end_split = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[cc_state].push(end_split);

                transitions[end_split].push(cc_state);
                transitions[split_state].push(end_split);

                Some(end_split)
            }
            Quantifier::OneOrMore => {
                let cc_state = states.len();
                states.push(State::CharClass(cc.clone()));
                transitions.push(Vec::new());
                transitions[from_state].push(cc_state);

                let split_state = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[cc_state].push(split_state);

                transitions[split_state].push(cc_state);

                Some(split_state)
            }
            Quantifier::ZeroOrOne => {
                let split_state = states.len();
                states.push(State::Split);
                transitions.push(Vec::new());
                transitions[from_state].push(split_state);

                let cc_state = states.len();
                states.push(State::CharClass(cc.clone()));
                transitions.push(Vec::new());
                transitions[split_state].push(cc_state);

                let end_state = states.len();
                states.push(State::Epsilon);
                transitions.push(Vec::new());
                transitions[cc_state].push(end_state);
                transitions[split_state].push(end_state);

                Some(end_state)
            }
            _ => None,
        }
    }

    /// Find first match in text
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        // Try matching from each position
        for (start_pos, _) in text.char_indices() {
            if let Some(len) = self.match_at(text, start_pos) {
                return Some((start_pos, start_pos + len));
            }
        }
        None
    }

    /// Find match starting from specific text position
    pub fn find_at(&self, text: &str, start_byte_pos: usize) -> Option<(usize, usize)> {
        if let Some(len) = self.match_at(text, start_byte_pos) {
            return Some((0, len)); // Return relative to start_byte_pos
        }
        None
    }

    /// Try to match at specific position
    fn match_at(&self, text: &str, start_pos: usize) -> Option<usize> {
        use std::collections::HashSet;

        let mut current_states = HashSet::new();
        current_states.insert(self.start_state);

        // Follow epsilon transitions
        self.epsilon_closure(&mut current_states);

        let mut pos = start_pos;
        let chars: Vec<char> = text[start_pos..].chars().collect();

        for ch in chars.iter() {
            let mut next_states = HashSet::new();

            for &state_id in &current_states {
                match &self.states[state_id] {
                    State::Char(expected) => {
                        if ch == expected {
                            for &next_id in &self.transitions[state_id] {
                                next_states.insert(next_id);
                            }
                        }
                    }
                    State::CharClass(cc) => {
                        if cc.matches(*ch) {
                            for &next_id in &self.transitions[state_id] {
                                next_states.insert(next_id);
                            }
                        }
                    }
                    _ => {}
                }
            }

            if next_states.is_empty() {
                break;
            }

            self.epsilon_closure(&mut next_states);
            current_states = next_states;
            pos += ch.len_utf8();

            // Check if we reached accept state
            if current_states.contains(&self.accept_state) {
                return Some(pos - start_pos);
            }
        }

        None
    }

    /// Add epsilon closure to state set
    fn epsilon_closure(&self, states: &mut std::collections::HashSet<usize>) {
        let mut to_process: Vec<usize> = states.iter().copied().collect();

        while let Some(state_id) = to_process.pop() {
            match &self.states[state_id] {
                State::Epsilon | State::Split => {
                    for &next_id in &self.transitions[state_id] {
                        if states.insert(next_id) {
                            to_process.push(next_id);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
