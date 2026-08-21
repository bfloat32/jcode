use std::collections::HashSet;

use crate::error::{EvalError, EvalErrorKind};

pub(crate) fn ensure_unique<T: AsRef<str>>(values: &[T], identity: &str) -> Result<(), EvalError> {
    let mut seen = HashSet::with_capacity(values.len());
    for value in values {
        if !seen.insert(value.as_ref()) {
            return Err(EvalError::new(
                EvalErrorKind::DuplicateIdentity,
                format!("duplicate {identity} identity {}", value.as_ref()),
            ));
        }
    }
    Ok(())
}

pub(crate) fn same_set<L: AsRef<str>, R: AsRef<str>>(left: &[L], right: &[R]) -> bool {
    left.len() == right.len()
        && left.iter().map(AsRef::as_ref).collect::<HashSet<_>>()
            == right.iter().map(AsRef::as_ref).collect::<HashSet<_>>()
}
