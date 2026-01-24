//! Parser module - converts regex pattern strings to AST
//!
//! This module contains all parsing logic for regex patterns,
//! including escape sequences, character classes, quantifiers,
//! groups, sequences, boundaries, and flags.

pub mod boundary;
pub mod charclass;
pub mod escape;
pub mod flags;
pub mod group;
pub mod quantifier;
pub mod sequence;
pub mod sequence_parser;

// Re-export commonly used types
pub use boundary::BoundaryType;
pub use charclass::CharClass;
pub use escape::{parse_escape, starts_with_escape};
pub use flags::Flags;
pub use group::Group;
pub use quantifier::{parse_quantified_pattern, QuantifiedPattern};
pub use sequence::Sequence;
pub use sequence_parser::{is_sequence_pattern, parse_sequence};
