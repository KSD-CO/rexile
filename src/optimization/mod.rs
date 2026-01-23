//! Optimization module - fast paths and performance optimizations
//!
//! This module contains all optimization logic including:
//! - Fast path detection and execution (10 specialized patterns)
//! - Prefilter optimization for quick candidate detection
//! - Literal extraction for prefix/suffix optimization

pub mod fast_path;
pub mod literal;
pub mod prefilter;

// Re-export commonly used types
pub use fast_path::FastPath;
pub use literal::{Literal, LiteralKind};
pub use prefilter::Prefilter;
