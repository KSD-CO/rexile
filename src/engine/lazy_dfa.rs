//! Lazy DFA - compile states on-demand for O(n) matching
//!
//! Like regex crate's hybrid engine: NFA simulation with cached DFA states

use crate::parser::charclass::CharClass;
use crate::parser::quantifier::Quantifier;
use crate::parser::sequence::{Sequence, SequenceElement};
use std::collections::HashMap;

/// Helper to get quantifier bounds
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

/// Lazy DFA that compiles states on-demand
#[derive(Clone, Debug)]
pub struct LazyDFA {
    /// NFA instructions
    instructions: Vec<Instruction>,
    /// Cache of compiled DFA states: (nfa_state_set) -> dfa_state_id
    /// State set represented as sorted Vec for hashing
    state_cache: HashMap<Vec<usize>, StateId>,
    /// Transition cache: (dfa_state, byte) -> dfa_state
    transition_cache: HashMap<(StateId, u8), StateId>,
    /// Next state ID to allocate
    next_state_id: StateId,
    /// Accepting DFA states
    accept_states: HashMap<StateId, bool>,
}

type StateId = u32;

/// NFA instruction (like Thompson's NFA)
#[derive(Debug, Clone)]
enum Instruction {
    /// Match a character and advance
    Match(MatchType),
    /// Split into multiple paths (for alternation or quantifiers)
    Split { first: usize, second: usize },
    /// Jump to another instruction
    Jump(usize),
    /// Accept state
    Accept,
}

#[derive(Debug, Clone)]
enum MatchType {
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

impl MatchType {
    #[inline]
    fn matches(&self, ch: char) -> bool {
        match self {
            MatchType::Literal(c) => ch == *c,
            MatchType::Word => ch.is_alphanumeric() || ch == '_',
            MatchType::Digit => ch.is_ascii_digit(),
            MatchType::Whitespace => ch.is_whitespace(),
            MatchType::Any => ch != '\n',
            MatchType::Class(cc) => cc.matches(ch),
        }
    }

    #[inline]
    fn matches_byte(&self, byte: u8) -> bool {
        if !byte.is_ascii() {
            return false;
        }
        let ch = byte as char;
        match self {
            MatchType::Literal(c) => ch == *c,
            MatchType::Word => ch.is_ascii_alphanumeric() || ch == '_',
            MatchType::Digit => ch.is_ascii_digit(),
            MatchType::Whitespace => matches!(ch, ' ' | '\t' | '\n' | '\r'),
            MatchType::Any => ch != '\n',
            MatchType::Class(cc) => cc.matches(ch),
        }
    }
}

impl LazyDFA {
    /// Try to compile a sequence into a Lazy DFA
    pub fn try_compile(seq: &Sequence) -> Option<Self> {
        let mut compiler = NFACompiler::new();

        // Compile sequence to NFA instructions
        for elem in &seq.elements {
            compiler.compile_element(elem)?;
        }

        // Add accept instruction
        compiler.add_accept();

        Some(LazyDFA {
            instructions: compiler.instructions,
            state_cache: HashMap::new(),
            transition_cache: HashMap::new(),
            next_state_id: 1,
            accept_states: HashMap::new(),
        })
    }

    /// Find first match using lazy DFA compilation
    pub fn find(&mut self, text: &str) -> Option<(usize, usize)> {
        let bytes = text.as_bytes();

        // Try starting match at each position
        for start in 0..text.len() {
            if !text.is_char_boundary(start) {
                continue;
            }

            // Get initial NFA state set
            let mut nfa_states = vec![0];
            self.epsilon_closure(&mut nfa_states);

            // Convert to DFA state
            let mut dfa_state = self.get_or_create_dfa_state(&nfa_states);

            let mut pos = start;
            let mut last_match = None;

            // Check if initial state is accepting
            if self.is_accepting_state(dfa_state) {
                last_match = Some(pos);
            }

            // Process input
            while pos < text.len() {
                let byte = bytes[pos];

                // Try to get cached transition
                if let Some(&next_state) = self.transition_cache.get(&(dfa_state, byte)) {
                    dfa_state = next_state;
                } else {
                    // Compute new DFA state
                    let nfa_states = self.get_nfa_states(dfa_state);
                    let mut next_nfa_states = Vec::new();

                    // Simulate NFA step
                    for &nfa_state in &nfa_states {
                        if let Some(Instruction::Match(match_type)) =
                            self.instructions.get(nfa_state)
                        {
                            if match_type.matches_byte(byte) {
                                next_nfa_states.push(nfa_state + 1);
                            }
                        }
                    }

                    if next_nfa_states.is_empty() {
                        break;
                    }

                    self.epsilon_closure(&mut next_nfa_states);
                    let next_dfa_state = self.get_or_create_dfa_state(&next_nfa_states);

                    // Cache transition
                    self.transition_cache
                        .insert((dfa_state, byte), next_dfa_state);
                    dfa_state = next_dfa_state;
                }

                pos += 1;

                // Check if current state is accepting
                if self.is_accepting_state(dfa_state) {
                    last_match = Some(pos);
                }
            }

            // If we found a match from this start position, return it
            if let Some(end) = last_match {
                return Some((start, end));
            }
        }

        None
    }

    /// Get or create DFA state for a set of NFA states
    fn get_or_create_dfa_state(&mut self, nfa_states: &[usize]) -> StateId {
        let mut sorted_states = nfa_states.to_vec();
        sorted_states.sort_unstable();
        sorted_states.dedup();

        if let Some(&state_id) = self.state_cache.get(&sorted_states) {
            return state_id;
        }

        let state_id = self.next_state_id;
        self.next_state_id += 1;

        // Check if this is an accepting state
        let is_accept = sorted_states
            .iter()
            .any(|&s| matches!(self.instructions.get(s), Some(Instruction::Accept)));

        self.state_cache.insert(sorted_states.clone(), state_id);
        self.accept_states.insert(state_id, is_accept);

        state_id
    }

    /// Get NFA states represented by a DFA state
    fn get_nfa_states(&self, dfa_state: StateId) -> Vec<usize> {
        for (nfa_states, &state_id) in &self.state_cache {
            if state_id == dfa_state {
                return nfa_states.clone();
            }
        }
        vec![]
    }

    /// Compute epsilon closure (follow all epsilon transitions)
    fn epsilon_closure(&self, states: &mut Vec<usize>) {
        let mut i = 0;
        while i < states.len() {
            let state = states[i];
            if let Some(instr) = self.instructions.get(state) {
                match instr {
                    Instruction::Split { first, second } => {
                        if !states.contains(first) {
                            states.push(*first);
                        }
                        if !states.contains(second) {
                            states.push(*second);
                        }
                    }
                    Instruction::Jump(target) => {
                        if !states.contains(target) {
                            states.push(*target);
                        }
                    }
                    _ => {}
                }
            }
            i += 1;
        }
    }

    /// Check if DFA state is accepting
    fn is_accepting_state(&self, state: StateId) -> bool {
        self.accept_states.get(&state).copied().unwrap_or(false)
    }
}

/// NFA compiler helper
struct NFACompiler {
    instructions: Vec<Instruction>,
}

impl NFACompiler {
    fn new() -> Self {
        NFACompiler {
            instructions: Vec::new(),
        }
    }

    fn compile_element(&mut self, elem: &SequenceElement) -> Option<()> {
        match elem {
            SequenceElement::Char(ch) => {
                self.instructions
                    .push(Instruction::Match(MatchType::Literal(*ch)));
            }

            SequenceElement::Dot => {
                self.instructions.push(Instruction::Match(MatchType::Any));
            }

            SequenceElement::CharClass(cc) => {
                let match_type = Self::charclass_to_match_type(cc);
                self.instructions.push(Instruction::Match(match_type));
            }

            SequenceElement::QuantifiedChar(ch, q) => {
                self.compile_quantified(MatchType::Literal(*ch), q)?;
            }

            SequenceElement::QuantifiedCharClass(cc, q) => {
                let match_type = Self::charclass_to_match_type(cc);
                self.compile_quantified(match_type, q)?;
            }

            SequenceElement::Literal(s) => {
                for ch in s.chars() {
                    self.instructions
                        .push(Instruction::Match(MatchType::Literal(ch)));
                }
            }

            _ => return None, // Other elements not supported yet
        }
        Some(())
    }

    fn compile_quantified(&mut self, match_type: MatchType, quantifier: &Quantifier) -> Option<()> {
        let (min, max) = quantifier_bounds(quantifier);

        match (min, max) {
            (0, usize::MAX) => {
                // * quantifier: split(match, skip), match -> jump back
                let split_pos = self.instructions.len();
                let match_pos = split_pos + 1;
                let after_pos = split_pos + 3;

                self.instructions.push(Instruction::Split {
                    first: match_pos,
                    second: after_pos,
                });
                self.instructions.push(Instruction::Match(match_type));
                self.instructions.push(Instruction::Jump(split_pos));
            }

            (1, usize::MAX) => {
                // + quantifier: match, split(match, skip), match -> jump back
                let match_pos = self.instructions.len();
                let split_pos = match_pos + 1;

                self.instructions
                    .push(Instruction::Match(match_type.clone()));
                self.instructions.push(Instruction::Split {
                    first: match_pos,
                    second: split_pos + 1,
                });
            }

            (0, 1) => {
                // ? quantifier: split(match, skip)
                let split_pos = self.instructions.len();
                let match_pos = split_pos + 1;
                let after_pos = match_pos + 1;

                self.instructions.push(Instruction::Split {
                    first: match_pos,
                    second: after_pos,
                });
                self.instructions.push(Instruction::Match(match_type));
            }

            _ => {
                // {n,m} quantifier: repeat min times, then optional max-min times
                for _ in 0..min {
                    self.instructions
                        .push(Instruction::Match(match_type.clone()));
                }

                if max > min && max != usize::MAX {
                    for _ in 0..(max - min) {
                        let split_pos = self.instructions.len();
                        let match_pos = split_pos + 1;
                        let after_pos = match_pos + 1;

                        self.instructions.push(Instruction::Split {
                            first: match_pos,
                            second: after_pos,
                        });
                        self.instructions
                            .push(Instruction::Match(match_type.clone()));
                    }
                }
            }
        }

        Some(())
    }

    fn add_accept(&mut self) {
        self.instructions.push(Instruction::Accept);
    }

    fn charclass_to_match_type(cc: &CharClass) -> MatchType {
        // Check for predefined classes
        if !cc.negated {
            if cc.chars.is_empty() && cc.ranges.len() == 1 && cc.ranges[0] == ('0', '9') {
                return MatchType::Digit;
            }

            // Check for \w pattern
            if cc.ranges.contains(&('a', 'z'))
                && cc.ranges.contains(&('A', 'Z'))
                && cc.ranges.contains(&('0', '9'))
                && cc.chars.contains(&'_')
            {
                return MatchType::Word;
            }
        } else if cc.chars.len() == 1 && cc.chars[0] == '\n' && cc.ranges.is_empty() {
            // Negated [^\n] is dot
            return MatchType::Any;
        }

        MatchType::Class(cc.clone())
    }
}
