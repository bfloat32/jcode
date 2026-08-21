fn validate_formats(result: &EvalResult) -> Result<(), EvalError> {
    let raw_digests = [
        result.config_id.as_str(),
        result.hardware_id.as_str(),
        result.identity_records.config.jcode_binary_sha256.as_str(),
        result.identity_records.config.environment_policy_digest.as_str(),
        result.identity_records.config.sandbox_policy_digest.as_str(),
        result.identity_records.hardware.runner_image_sha256.as_str(),
    ];
    let valid = is_revision(&result.measurement_revision)
        && is_revision(&result.stage_first_implementation_revision)
        && result.corpus_ids.iter().all(|id| is_hex(id, 64))
        && result.gate_ids.iter().all(|id| !id.is_empty())
        && result.exclusion_ids.iter().all(|id| is_hex(id, 64))
        && raw_digests.iter().all(|digest| is_hex(digest, 64))
        && !result.artifact_digests.is_empty()
        && result.artifact_digests.iter().all(|digest| is_sha256(digest))
        && result.identity_records.corpora.iter().all(CorpusIdentity::formats_valid)
        && time::UtcTimestamp::ordered(
            &result.measurement_started_at,
            &result.measurement_finished_at,
        );
    if !valid {
        return Err(EvalError::new(
            EvalErrorKind::InvalidResult,
            "invalid result identity format",
        ));
    }
    Ok(())
}

fn validate_identity_bindings(result: &EvalResult) -> Result<(), EvalError> {
    let corpus_ids = result
        .identity_records
        .corpora
        .iter()
        .map(|corpus| corpus.corpus_id.as_str())
        .collect::<Vec<_>>();
    let exclusion_ids = result
        .identity_records
        .exclusions
        .iter()
        .map(|exclusion| exclusion.exclusion_id.as_str())
        .collect::<Vec<_>>();
    ensure_unique(&result.corpus_ids, "top-level corpus")?;
    ensure_unique(&corpus_ids, "embedded corpus")?;
    ensure_unique(&result.exclusion_ids, "top-level exclusion")?;
    ensure_unique(&exclusion_ids, "embedded exclusion")?;
    if !same_set(&result.corpus_ids, &corpus_ids)
        || result.config_id != result.identity_records.config.config_id
        || result.hardware_id != result.identity_records.hardware.hardware_id
        || !same_set(&result.exclusion_ids, &exclusion_ids)
    {
        return Err(EvalError::new(
            EvalErrorKind::IdentityMismatch,
            "top-level result references do not match embedded identities",
        ));
    }
    Ok(())
}

fn validate_identity_shapes(result: &EvalResult) -> Result<(), EvalError> {
    let config = &result.identity_records.config;
    let hardware = &result.identity_records.hardware;
    let config_valid = config.jcode_revision == result.measurement_revision
        && !config.provider_identities.is_empty()
        && config.provider_identities.iter().all(|value| !value.is_empty())
        && !config.feature_set.is_empty()
        && config.feature_set.iter().all(|value| !value.is_empty())
        && !config.execution_limits.is_empty()
        && !config.render_limits.is_empty()
        && matches!(config.trust_mode.as_str(), "baseline" | "staged");
    let hardware_valid = !hardware.cpu_model.is_empty()
        && hardware.physical_core_count > 0
        && hardware.logical_core_count >= hardware.physical_core_count
        && hardware.memory_bytes > 0
        && !hardware.operating_system.is_empty()
        && !hardware.architecture.is_empty()
        && !hardware.kernel_version.is_empty()
        && !hardware.storage_kind.is_empty();
    if !config_valid || !hardware_valid {
        return Err(EvalError::new(
            EvalErrorKind::InvalidResult,
            "invalid config or hardware identity",
        ));
    }
    Ok(())
}

impl CorpusIdentity {
    fn formats_valid(&self) -> bool {
        is_hex(&self.corpus_id, 64)
            && is_sha256(&self.content_digest)
            && matches!(self.source_kind.as_str(), "public" | "synthetic")
            && match self.source_revision_kind.as_str() {
                "git_commit" => is_revision(&self.source_revision),
                "sha256_archive" | "generated_from_pinned_inputs" => {
                    is_sha256(&self.source_revision)
                }
                _ => false,
            }
            && is_sha256(&self.generator_digest)
            && !self.license_id.is_empty()
            && !self.provenance_uri.is_empty()
            && self.case_count > 0
    }
}

impl ExclusionIdentity {
    fn fields_present(&self) -> bool {
        !self.scope.is_empty()
            && !self.reason_code.is_empty()
            && !self.rationale.is_empty()
            && !self.approver.is_empty()
    }
}

fn is_revision(value: &str) -> bool {
    is_hex(value, 40)
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| is_hex(digest, 64))
}

pub(crate) fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
