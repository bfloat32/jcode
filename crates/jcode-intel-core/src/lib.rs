#![forbid(unsafe_code)]

//! Boundary for the code-intelligence crate graph.
//!
//! This skeleton only exposes its five confirmed dependency boundaries.

pub use jcode_intel_provider as provider;
pub use jcode_intel_rust as rust;
pub use jcode_intel_search as search;
pub use jcode_intel_store as store;
pub use jcode_intel_types as types;
