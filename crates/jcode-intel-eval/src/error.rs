use std::fmt;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalErrorKind {
    UnknownField,
    DuplicateIdentity,
    MutableRevision,
    MissingIdentity,
    PlannedResult,
    PartialResult,
    StaleResult,
    PostHocBaseline,
    StageMismatch,
    IdentityMismatch,
    OverwriteRefused,
    Io,
    MalformedToml,
    UnsupportedSchema,
    InvalidManifest,
    UnresolvedReference,
    MissingMetric,
    InvalidResult,
    MissingStageEvidence,
}

#[derive(Debug)]
pub struct EvalError {
    kind: EvalErrorKind,
    message: String,
}

impl EvalError {
    pub(crate) fn new(kind: EvalErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(crate) fn io(action: &str, path: &Path, error: &std::io::Error) -> Self {
        Self::new(
            EvalErrorKind::Io,
            format!("{action} {}: {error}", path.display()),
        )
    }

    #[must_use]
    pub const fn kind(&self) -> EvalErrorKind {
        self.kind
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EvalError {}

pub(crate) fn classify_toml(error: &toml::de::Error, result: bool) -> EvalError {
    let message = error.to_string();
    if message.contains("unknown field") {
        return EvalError::new(EvalErrorKind::UnknownField, message);
    }
    if result && message.contains("missing field `metrics`") {
        return EvalError::new(EvalErrorKind::MissingMetric, message);
    }
    if result && message.contains("missing field `coverage`") {
        return EvalError::new(EvalErrorKind::InvalidResult, message);
    }
    if result
        && [
            "corpus_ids",
            "config_id",
            "hardware_id",
            "gate_ids",
            "exclusion_ids",
        ]
        .iter()
        .any(|field| message.contains(&format!("missing field `{field}`")))
    {
        return EvalError::new(EvalErrorKind::MissingIdentity, message);
    }
    EvalError::new(EvalErrorKind::MalformedToml, message)
}
