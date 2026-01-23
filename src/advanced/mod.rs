//! Advanced features module - capturing groups and lookaround assertions
//!
//! This module contains advanced regex features:
//! - Capture groups: Extract matched substrings
//! - Lookahead/Lookbehind: Zero-width assertions

pub mod captures;
pub mod lookaround;

// Re-export public types
pub use captures::{Captures, Group as CaptureGroup};
pub use lookaround::{Lookaround, LookaroundType};
