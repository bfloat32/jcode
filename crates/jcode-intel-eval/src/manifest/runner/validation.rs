use crate::error::{EvalError, EvalErrorKind};

use super::{ResultIdentityContract, RunnerManifest};

impl RunnerManifest {
    pub(crate) fn validate(&self) -> Result<(), EvalError> {
        if self.schema_version != 1 {
            return Err(EvalError::new(
                EvalErrorKind::UnsupportedSchema,
                "unsupported release runner schema version",
            ));
        }
        if !self.fixed_contracts_valid() || !self.identity_contracts_valid() {
            return Err(EvalError::new(
                EvalErrorKind::InvalidManifest,
                "invalid release runner manifest contract",
            ));
        }
        Ok(())
    }

    fn fixed_contracts_valid(&self) -> bool {
        self.manifest_kind == "jcode-intel-eval-release-runner"
            && self.loader_policy.unknown_fields == "reject"
            && self.loader_policy.duplicate_ids == "reject"
            && self.loader_policy.unresolved_references == "reject"
            && self.loader_policy.mutable_identity_inputs == "reject"
            && self.loader_policy.planned_results == "reject"
            && self.loader_policy.partial_results == "reject"
            && self.loader_policy.stale_results == "reject"
            && self.loader_policy.post_hoc_baselines == "reject"
            && self.loader_policy.identity_mismatches == "reject"
            && self.loader_policy.missing_stage_evidence == "fail"
            && self.content_addressing.algorithm == "sha256"
            && self.content_addressing.encoding == "lowercase-hex"
            && self.content_addressing.canonicalization == "jcode-intel-eval-canonical-toml-v1"
            && self.content_addressing.artifact_extension == ".toml"
            && self.content_addressing.artifact_filename == "<result_id>.toml"
            && self.content_addressing.artifact_filename_format == "64-lowercase-hex-sha256"
            && self.content_addressing.content_key_must_equal_filename
            && self.content_addressing.canonical_payload_excludes == ["result_id"]
            && self.baseline_policy_valid()
            && self.execution_privacy_valid()
            && self.stages_valid()
    }

    fn identity_contracts_valid(&self) -> bool {
        self.stage_identity_contract.domain == "jcode-intel-eval-stage-v1"
            && self.stage_identity_contract.id_field == "stage_id"
            && self.stage_identity_contract.id_kind == "closed-versioned-stage-id"
            && self.stage_identity_contract.allowed_ids
                == ["f0", "f1", "f2", "f3", "f4", "f5", "f6", "f7"]
            && self.stage_identity_contract.required_fields
                == [
                    "stage_id",
                    "first_implementation_task",
                    "default_feature_state",
                    "required_corpus_spec_ids",
                    "required_gate_ids",
                    "baseline_required_before_implementation",
                    "baseline_required_before_enable",
                    "enable_only_after_passing_result",
                ]
            && self.stage_identity_contract.mutation_policy
                == "changed_stage_record_requires_manifest_schema_version_bump"
            && self.stage_identity_contract.duplicate_ids == "reject"
            && self.stage_identity_contract.references_target == "stage"
            && self.config_identity_contract.domain == "jcode-intel-eval-config-v1"
            && self.config_identity_contract.id_field == "config_id"
            && self.config_identity_contract.required_fields
                == [
                    "jcode_revision",
                    "jcode_binary_sha256",
                    "provider_identities",
                    "feature_set",
                    "execution_limits",
                    "render_limits",
                    "environment_policy_digest",
                    "sandbox_policy_digest",
                    "trust_mode",
                ]
            && self.config_identity_contract.ordered_fields
                == ["provider_identities", "feature_set"]
            && self.config_identity_contract.mutable_revisions_forbidden
            && self.hardware_identity_contract.domain == "jcode-intel-eval-hardware-v1"
            && self.hardware_identity_contract.id_field == "hardware_id"
            && self.hardware_identity_contract.required_fields
                == [
                    "cpu_model",
                    "physical_core_count",
                    "logical_core_count",
                    "memory_bytes",
                    "operating_system",
                    "architecture",
                    "kernel_version",
                    "storage_kind",
                    "runner_image_sha256",
                ]
            && self
                .hardware_identity_contract
                .current_hardware_records
                .is_empty()
            && self.exclusion_identity_contract.domain == "jcode-intel-eval-exclusion-v1"
            && self.exclusion_identity_contract.id_field == "exclusion_id"
            && self.exclusion_identity_contract.required_fields
                == [
                    "gate_id",
                    "corpus_id",
                    "scope",
                    "reason_code",
                    "rationale",
                    "approver",
                    "reviewed_at",
                    "expires_at",
                ]
            && self.exclusion_identity_contract.required_review_state == "approved"
            && self.exclusion_identity_contract.unreviewed_or_expired == "reject"
            && self
                .exclusion_identity_contract
                .semantic_effect_requires_reference
            && self.result_identity_contract.valid()
    }

    fn baseline_policy_valid(&self) -> bool {
        self.baseline_policy.directory == "baselines"
            && self.baseline_policy.discovery_glob == "[0-9a-f][0-9a-f]*.toml"
            && self.baseline_policy.missing_baseline_result == "fail"
            && self.baseline_policy.missing_baseline_cannot_pass
            && self
                .baseline_policy
                .baseline_command_accepts_only_complete_result
            && self
                .baseline_policy
                .baseline_command_recomputes_all_identities
            && self.baseline_policy.baseline_command_refuses_overwrite
    }

    fn execution_privacy_valid(&self) -> bool {
        self.execution_policy.network_access == "deny"
            && self.execution_policy.corpus_sources == ["public", "synthetic"]
            && self.execution_policy.comparison_mode == "same_process_baseline_and_staged"
            && self.execution_policy.clean_temporary_checkouts
            && self.execution_policy.resource_capture_required
            && self.execution_policy.ci_attestation_when_available
            && self.execution_policy.raw_ci_upload == "public_or_synthetic_compatible_license_only"
            && self.execution_policy.non_public_artifacts == "local_unless_explicit_user_export"
            && !self.privacy_policy.independent_upload_transport
            && !self.privacy_policy.analytics_transport
            && !self.privacy_policy.telemetry_identity
            && self.privacy_policy.allowed_remote_model_evaluation
                == "explicit_configuration_public_or_synthetic_licensed_corpora_only"
            && self.privacy_policy.forbidden_remote_content.len() == 6
    }

    fn stages_valid(&self) -> bool {
        self.stages.len() == 8
            && self.stages.iter().enumerate().all(|(index, stage)| {
                stage.stage_id == format!("f{index}")
                    && !stage.first_implementation_task.is_empty()
                    && !stage.default_feature_state.is_empty()
                    && stage.baseline_required_before_enable
                    && stage.enable_only_after_passing_result
                    && (index == 0 || stage.baseline_required_before_implementation)
                    && match index {
                        7 => stage.consecutive_full_passing_release_runs == Some(2),
                        0..=6 => stage.consecutive_full_passing_release_runs.is_none(),
                        _ => false,
                    }
            })
    }
}

impl ResultIdentityContract {
    fn valid(&self) -> bool {
        self.domain == "jcode-intel-eval-result-v1"
            && self.id_field == "result_id"
            && self.required_fields
                == [
                    "result_id",
                    "stage_id",
                    "result_kind",
                    "lifecycle_state",
                    "disposition",
                    "corpus_ids",
                    "config_id",
                    "hardware_id",
                    "gate_ids",
                    "exclusion_ids",
                    "measurement_revision",
                    "stage_first_implementation_revision",
                    "measurement_started_at",
                    "measurement_finished_at",
                    "metrics",
                    "coverage",
                    "silent_drop_count",
                    "artifact_digests",
                ]
            && self.allowed_result_kinds == ["baseline", "staged"]
            && self.passing_lifecycle_state == "complete"
            && self.passing_disposition == "complete_nonempty"
            && self.required_freshness == "current"
            && self.baseline_ordering
                == "measurement_revision_is_ancestor_of_stage_first_implementation_revision"
            && self.all_gate_metrics_required
            && self.all_references_must_resolve
            && self.silent_drop_count_required
            && !self.post_hoc_baseline_can_pass
    }
}
