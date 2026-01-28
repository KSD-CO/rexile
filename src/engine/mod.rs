//! Engine module - different matching engines for regex execution
//!
//! This module contains multiple regex matching engines:
//! - NFA: Non-deterministic finite automaton (general purpose)
//! - DFA: Deterministic finite automaton (faster, more memory)
//! - Lazy DFA: On-demand DFA compilation (hybrid approach)
//! - Simple NFA: Fallback for patterns without prefix optimization
//! - Capture DFA: Specialized DFA for capturing groups (single-pass, no memchr)

pub mod capture_dfa;
pub mod dfa;
pub mod lazy_dfa;
pub mod nfa;
pub mod simple_nfa;

// Re-export engine types
pub use dfa::DFA;
// CaptureDFA is only used internally via fast_path, no need to export
