//! NFA (Non-deterministic Finite Automaton) for efficient pattern matching
//!
//! Implements Thompson's NFA construction for regex patterns.
//! Compiles sequences like `\w+\s*>=\s*\d+` into state machines for O(n) scanning.

use crate::parser::charclass::CharClass;
use crate::parser::quantifier::Quantifier;
use crate::parser::sequence::{Sequence, SequenceElement};
use std::collections::HashSet;

/// NFA state machine for pattern matching
#[derive(Debug, Clone)]
pub struct NFA {
    states: Vec<State>,
    start_state: usize,
    accept_state: usize,
}

/// Individual NFA state
#[derive(Debug, Clone)]
struct State {
    transitions: Vec<Transition>,
}

/// Transition between states
#[derive(Debug, Clone)]
enum Transition {
    /// Consume a character matching the class
    Char(CharClassMatcher, usize), // (matcher, next_state)
    /// Epsilon transition (no character consumed)
    Epsilon(usize), // next_state
}

/// Character class matcher for transitions
#[derive(Debug, Clone)]
enum CharClassMatcher {
    /// Matches specific char
    Literal(char),
    /// Matches word characters (\w)
    Word,
    /// Matches digits (\d)
    Digit,
    /// Matches whitespace (\s)
    Whitespace,
    /// Matches custom character class
    Custom(CharClass),
}

impl CharClassMatcher {
    fn matches(&self, ch: char) -> bool {
        match self {
            CharClassMatcher::Literal(c) => ch == *c,
            CharClassMatcher::Word => ch.is_alphanumeric() || ch == '_',
            CharClassMatcher::Digit => ch.is_ascii_digit(),
            CharClassMatcher::Whitespace => ch.is_whitespace(),
            CharClassMatcher::Custom(cc) => cc.matches(ch),
        }
    }
}

impl NFA {
    /// Try to compile a sequence into an NFA
    pub fn try_compile(seq: &Sequence) -> Option<Self> {
        if seq.elements.is_empty() {
            return None;
        }

        let mut states = Vec::new();
        let start_state = 0;

        // Create start state
        states.push(State {
            transitions: vec![],
        });

        let mut current_state = start_state;

        // Build NFA from sequence elements
        for elem in &seq.elements {
            let next_state = Self::add_element(&mut states, current_state, elem)?;
            current_state = next_state;
        }

        // Current state is the accept state
        let accept_state = current_state;

        Some(NFA {
            states,
            start_state,
            accept_state,
        })
    }

    /// Add an element to the NFA, returns the final state after this element
    fn add_element(
        states: &mut Vec<State>,
        from_state: usize,
        elem: &SequenceElement,
    ) -> Option<usize> {
        match elem {
            SequenceElement::Char(ch) => {
                // Simple transition: from_state --ch--> new_state
                let new_state = states.len();
                states.push(State {
                    transitions: vec![],
                });
                states[from_state]
                    .transitions
                    .push(Transition::Char(CharClassMatcher::Literal(*ch), new_state));
                Some(new_state)
            }

            SequenceElement::CharClass(cc) => {
                let new_state = states.len();
                states.push(State {
                    transitions: vec![],
                });
                let matcher = Self::charclass_to_matcher(cc);
                states[from_state]
                    .transitions
                    .push(Transition::Char(matcher, new_state));
                Some(new_state)
            }

            SequenceElement::QuantifiedChar(ch, q) => {
                let matcher = CharClassMatcher::Literal(*ch);
                Self::add_quantified(states, from_state, matcher, q)
            }

            SequenceElement::QuantifiedCharClass(cc, q) => {
                let matcher = Self::charclass_to_matcher(cc);
                Self::add_quantified(states, from_state, matcher, q)
            }

            _ => None, // Other elements not supported yet
        }
    }

    /// Add a quantified element (e.g., \w+, \s*)
    fn add_quantified(
        states: &mut Vec<State>,
        from_state: usize,
        matcher: CharClassMatcher,
        quantifier: &Quantifier,
    ) -> Option<usize> {
        match quantifier {
            Quantifier::ZeroOrMore => {
                // from_state --epsilon--> loop_state --matcher--> loop_state
                //           \--epsilon--> exit_state
                let loop_state = states.len();
                let exit_state = loop_state + 1;

                states.push(State {
                    transitions: vec![
                        Transition::Char(matcher, loop_state), // Self loop
                        Transition::Epsilon(exit_state),       // Exit
                    ],
                });

                states.push(State {
                    transitions: vec![],
                });

                // from_state can skip or enter loop
                states[from_state]
                    .transitions
                    .push(Transition::Epsilon(loop_state));

                Some(exit_state)
            }

            Quantifier::OneOrMore => {
                // from_state --matcher--> loop_state --matcher--> loop_state
                //                                   \--epsilon--> exit_state
                let loop_state = states.len();
                let exit_state = loop_state + 1;

                states.push(State {
                    transitions: vec![
                        Transition::Char(matcher.clone(), loop_state), // Self loop
                        Transition::Epsilon(exit_state),               // Exit
                    ],
                });

                states.push(State {
                    transitions: vec![],
                });

                // Must match at least once
                states[from_state]
                    .transitions
                    .push(Transition::Char(matcher, loop_state));

                Some(exit_state)
            }

            Quantifier::ZeroOrOne => {
                // from_state --matcher--> new_state
                //           \--epsilon--> new_state
                let new_state = states.len();

                states.push(State {
                    transitions: vec![],
                });

                states[from_state]
                    .transitions
                    .push(Transition::Char(matcher, new_state));
                states[from_state]
                    .transitions
                    .push(Transition::Epsilon(new_state));

                Some(new_state)
            }

            _ => None, // Other quantifiers not implemented yet
        }
    }

    /// Convert CharClass to matcher
    fn charclass_to_matcher(cc: &CharClass) -> CharClassMatcher {
        // Check if it's a predefined class
        if cc.matches('a') && cc.matches('Z') && cc.matches('0') && cc.matches('_') {
            return CharClassMatcher::Word;
        }
        if cc.matches('0') && cc.matches('9') && !cc.matches('a') {
            return CharClassMatcher::Digit;
        }
        if cc.matches(' ') && cc.matches('\t') && cc.matches('\n') {
            return CharClassMatcher::Whitespace;
        }

        CharClassMatcher::Custom(cc.clone())
    }

    /// Find first match in text using NFA simulation (single-pass, O(n))
    /// Optimized: no Vec allocation, direct char iteration
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        let mut current_states = HashSet::new();
        let mut match_start: Option<usize> = None;
        let mut match_start_byte: Option<usize> = None;

        let mut byte_pos = 0;
        let mut char_pos = 0;

        for (this_byte_pos, ch) in text.char_indices() {
            // If we have no active states, start a new potential match
            if current_states.is_empty() {
                match_start = Some(char_pos);
                match_start_byte = Some(this_byte_pos);
                self.add_epsilon_closure(&mut current_states, self.start_state);
            }

            // Check if we're in accept state before consuming character
            if current_states.contains(&self.accept_state) {
                if let (Some(_), Some(start_byte)) = (match_start, match_start_byte) {
                    return Some((start_byte, this_byte_pos));
                }
            }

            let mut next_states = HashSet::new();

            // Process all current states with this character
            for &state_id in &current_states {
                if state_id >= self.states.len() {
                    continue;
                }

                for transition in &self.states[state_id].transitions {
                    if let Transition::Char(matcher, next_state) = transition {
                        if matcher.matches(ch) {
                            self.add_epsilon_closure(&mut next_states, *next_state);
                        }
                    }
                }
            }

            // If no progress and not at start, try starting fresh from next position
            if next_states.is_empty() && match_start.is_some() {
                current_states.clear();
                match_start = Some(char_pos + 1);
                match_start_byte = Some(this_byte_pos + ch.len_utf8());
                self.add_epsilon_closure(&mut current_states, self.start_state);

                // Try to match this character with fresh start
                let mut fresh_next = HashSet::new();
                for &state_id in &current_states {
                    if state_id < self.states.len() {
                        for transition in &self.states[state_id].transitions {
                            if let Transition::Char(matcher, next_state) = transition {
                                if matcher.matches(ch) {
                                    self.add_epsilon_closure(&mut fresh_next, *next_state);
                                }
                            }
                        }
                    }
                }
                current_states = fresh_next;
            } else {
                current_states = next_states;
            }

            char_pos += 1;
            byte_pos = this_byte_pos + ch.len_utf8();
        }

        // Final check for accept state at end of text
        if current_states.contains(&self.accept_state) {
            if let (Some(_), Some(start_byte)) = (match_start, match_start_byte) {
                return Some((start_byte, text.len()));
            }
        }

        None
    }

    /// Try to match at a specific position, returns match length in chars if successful
    fn try_match_at(&self, chars: &[char], start_pos: usize) -> Option<usize> {
        let mut current_states = HashSet::new();

        // Start with epsilon closure of start state
        self.add_epsilon_closure(&mut current_states, self.start_state);

        let mut pos = start_pos;

        // Process each character
        while pos <= chars.len() {
            // Check if we're in accept state
            if current_states.contains(&self.accept_state) {
                return Some(pos - start_pos);
            }

            if pos >= chars.len() {
                break;
            }

            let ch = chars[pos];
            let mut next_states = HashSet::new();

            // Process all current states
            for &state_id in &current_states {
                if state_id >= self.states.len() {
                    continue;
                }

                for transition in &self.states[state_id].transitions {
                    match transition {
                        Transition::Char(matcher, next_state) => {
                            if matcher.matches(ch) {
                                self.add_epsilon_closure(&mut next_states, *next_state);
                            }
                        }
                        Transition::Epsilon(_) => {
                            // Already handled in epsilon closure
                        }
                    }
                }
            }

            if next_states.is_empty() {
                break;
            }

            current_states = next_states;
            pos += 1;
        }

        // Final check for accept state
        if current_states.contains(&self.accept_state) {
            Some(pos - start_pos)
        } else {
            None
        }
    }

    /// Add all states reachable via epsilon transitions
    fn add_epsilon_closure(&self, states: &mut HashSet<usize>, state_id: usize) {
        if states.contains(&state_id) {
            return;
        }

        states.insert(state_id);

        if state_id >= self.states.len() {
            return;
        }

        for transition in &self.states[state_id].transitions {
            if let Transition::Epsilon(next_state) = transition {
                self.add_epsilon_closure(states, *next_state);
            }
        }
    }
}
