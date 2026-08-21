mod sets;
mod time;

use crate::error::{EvalError, EvalErrorKind};
use crate::manifest::ManifestRoot;
use crate::recording::RevisionAncestry;
use crate::result::{CorpusIdentity, EvalResult, ExclusionIdentity};
use sets::{ensure_unique, same_set};

pub(crate) fn validate_result(
    manifests: &ManifestRoot,
    requested_stage: &str,
    result: &EvalResult,
    ancestry: &dyn RevisionAncestry,
    expected_key: Option<&str>,
) -> Result<(), EvalError> {
    validate_state(result)?;
    if result.stage_id != requested_stage {
        return Err(EvalError::new(
            EvalErrorKind::StageMismatch,
            format!(
                "requested stage {requested_stage} does not match result stage {}",
                result.stage_id
            ),
        ));
    }
    let stage = manifests.stage(requested_stage)?;
    if let Some(corpus) = result.identity_records.corpora.iter().find(|corpus| {
        !matches!(
            corpus.source_revision_kind.as_str(),
            "git_commit" | "sha256_archive" | "generated_from_pinned_inputs"
        )
    }) {
        return Err(EvalError::new(
            EvalErrorKind::MutableRevision,
            format!(
                "mutable source revision kind {}",
                corpus.source_revision_kind
            ),
        ));
    }
    validate_formats(result)?;
    validate_identity_shapes(result)?;
    validate_identity_bindings(result)?;
    let result_specs = result
        .identity_records
        .corpora
        .iter()
        .map(|corpus| corpus.spec_id.as_str())
        .collect::<Vec<_>>();
    ensure_unique(&result_specs, "result corpus specification")?;
    if !same_set(&result_specs, &stage.required_corpus_spec_ids) {
        return Err(EvalError::new(
            EvalErrorKind::IdentityMismatch,
            "result corpus specifications do not match stage requirements",
        ));
    }
    ensure_unique(&result.gate_ids, "result gate")?;
    if !same_set(&result.gate_ids, &stage.required_gate_ids) {
        return Err(EvalError::new(
            EvalErrorKind::IdentityMismatch,
            "result gates do not match stage requirements",
        ));
    }
    let required_checks = manifests.gate_checks(&stage.required_gate_ids);
    ensure_unique(&result.metrics.passed_checks, "result check")?;
    if !same_set(&result.metrics.passed_checks, &required_checks) {
        return Err(EvalError::new(
            EvalErrorKind::MissingMetric,
            "result metrics do not exactly match required gate checks",
        ));
    }
    validate_coverage(result, required_checks.len())?;
    validate_exclusions(manifests, result)?;
    if !ancestry.is_ancestor(
        &result.measurement_revision,
        &result.stage_first_implementation_revision,
    ) {
        return Err(EvalError::new(
            EvalErrorKind::PostHocBaseline,
            "measurement revision is not an ancestor of stage implementation revision",
        ));
    }
    match expected_key {
        Some(key) if result.result_id != key => Err(EvalError::new(
            EvalErrorKind::IdentityMismatch,
            "result_id does not match artifact content key",
        )),
        None if !result.result_id.is_empty() => Err(EvalError::new(
            EvalErrorKind::IdentityMismatch,
            "recording input result_id must be blank",
        )),
        Some(_) | None => Ok(()),
    }
}

fn validate_state(result: &EvalResult) -> Result<(), EvalError> {
    if result.lifecycle_state == "planned" {
        return Err(EvalError::new(
            EvalErrorKind::PlannedResult,
            "planned result cannot pass",
        ));
    }
    if result.lifecycle_state != "complete" || result.disposition != "complete_nonempty" {
        return Err(EvalError::new(
            EvalErrorKind::PartialResult,
            "result must be complete and complete_nonempty",
        ));
    }
    if result.freshness != "current" {
        return Err(EvalError::new(
            EvalErrorKind::StaleResult,
            "result freshness must be current",
        ));
    }
    if result.result_kind != "baseline" {
        return Err(EvalError::new(
            EvalErrorKind::InvalidResult,
            "stage evidence must be a baseline result",
        ));
    }
    if result.silent_drop_count != 0 {
        return Err(EvalError::new(
            EvalErrorKind::InvalidResult,
            "baseline result contains silent drops",
        ));
    }
    Ok(())
}

fn validate_coverage(result: &EvalResult, required_check_count: usize) -> Result<(), EvalError> {
    ensure_unique(&result.coverage.corpus_ids, "coverage corpus")?;
    ensure_unique(&result.coverage.gate_ids, "coverage gate")?;
    if result.coverage.state != "complete"
        || !same_set(&result.coverage.corpus_ids, &result.corpus_ids)
        || !same_set(&result.coverage.gate_ids, &result.gate_ids)
        || result.coverage.required_check_count != required_check_count
        || result.coverage.silent_drop_count != 0
    {
        return Err(EvalError::new(
            EvalErrorKind::InvalidResult,
            "result coverage is incomplete",
        ));
    }
    Ok(())
}

fn validate_exclusions(manifests: &ManifestRoot, result: &EvalResult) -> Result<(), EvalError> {
    let started = time::UtcTimestamp::parse(&result.measurement_started_at)?;
    let finished = time::UtcTimestamp::parse(&result.measurement_finished_at)?;
    for exclusion in &result.identity_records.exclusions {
        let gate = manifests.gate(&exclusion.gate_id)?;
        let corpus = result
            .identity_records
            .corpora
            .iter()
            .find(|corpus| corpus.corpus_id == exclusion.corpus_id)
            .ok_or_else(|| {
                EvalError::new(
                    EvalErrorKind::IdentityMismatch,
                    "exclusion corpus identity is unresolved",
                )
            })?;
        let reviewed = time::UtcTimestamp::parse(&exclusion.reviewed_at)?;
        let expires = time::UtcTimestamp::parse(&exclusion.expires_at)?;
        if gate.exclusion_policy == "none"
            || !gate.corpus_spec_ids.contains(&corpus.spec_id)
            || exclusion.review_state != "approved"
            || reviewed > started
            || finished > expires
            || !exclusion.fields_present()
        {
            return Err(EvalError::new(
                EvalErrorKind::InvalidResult,
                "result contains an invalid exclusion",
            ));
        }
    }
    Ok(())
}

include!("validation/identity.rs");
