#![forbid(unsafe_code)]

//! Strict evaluation-manifest and baseline-recording boundary for code intelligence.

mod error;
mod manifest;
mod recording;
mod result;
mod validation;

pub use error::{EvalError, EvalErrorKind};
pub use manifest::{ManifestRoot, load_manifest_root};
pub use recording::{BaselineArtifact, BaselineRequest, RevisionAncestry, record_baseline};

#[cfg(test)]
mod tests;
