impl CorporaManifest {
    pub(crate) fn validate(&self) -> Result<(), EvalError> {
        let fixed = self.schema_version == 1
            && self.manifest_kind == "jcode-intel-eval-corpora"
            && self.loader_policy.unknown_fields == "reject"
            && self.loader_policy.duplicate_ids == "reject"
            && self.loader_policy.mutable_sources == "reject"
            && self.loader_policy.network_access == "reject"
            && self.loader_policy.missing_content == "reject"
            && self.corpus_spec_identity_contract.domain == "jcode-intel-eval-corpus-spec-v1"
            && self.corpus_spec_identity_contract.id_field == "spec_id"
            && self.corpus_spec_identity_contract.id_kind == "versioned-semantic-id"
            && self.corpus_spec_identity_contract.canonical_payload == "entire_corpus_spec_table"
            && self.corpus_spec_identity_contract.mutation_policy
                == "changed_specification_requires_new_spec_id"
            && self.corpus_spec_identity_contract.duplicate_ids == "reject"
            && self.corpus_spec_identity_contract.references_target == "corpus_spec"
            && self.materialized_corpus_identity_contract.algorithm == "sha256"
            && self.materialized_corpus_identity_contract.encoding == "lowercase-hex"
            && self.materialized_corpus_identity_contract.canonicalization
                == "jcode-intel-eval-canonical-toml-v1"
            && self.materialized_corpus_identity_contract.domain
                == "jcode-intel-eval-materialized-corpus-v1"
            && self.materialized_corpus_identity_contract.id_field == "corpus_id"
            && self.materialized_corpus_identity_contract.digest_field == "content_digest"
            && self.materialized_corpus_identity_contract.spec_reference_field == "spec_id"
            && self.materialized_corpus_identity_contract.mutation_policy
                == "changed_materialization_requires_new_corpus_id"
            && self.materialized_corpus_identity_contract.references_target
                == "materialized_corpus"
            && self.materialized_corpus_contract.allowed_source_kinds == ["public", "synthetic"]
            && self.materialized_corpus_contract.allowed_materialization_states == ["complete"]
            && self.materialized_corpus_contract.immutable_revision_kinds
                == ["git_commit", "sha256_archive", "generated_from_pinned_inputs"]
            && self.materialized_corpus_contract.forbidden_source_revisions
                == ["branch", "tag", "latest", "floating_url"]
            && self.materialized_corpus_contract.case_ids_unique
            && self.materialized_corpus_contract.content_must_match_digest
            && self.materialized_corpus_contract.license_and_provenance_required
            && self.specs.len() == 8;
        if !fixed {
            return Err(EvalError::new(
                if self.schema_version == 1 {
                    EvalErrorKind::InvalidManifest
                } else {
                    EvalErrorKind::UnsupportedSchema
                },
                "invalid corpora manifest contract",
            ));
        }
        self.validate_known_spec_fields()
    }

    fn validate_known_spec_fields(&self) -> Result<(), EvalError> {
        let all_specs_have_core = self.specs.iter().all(|spec| {
            !spec.stage.is_empty()
                && spec
                    .source_kinds
                    .iter()
                    .all(|kind| kind == "public" || kind == "synthetic")
        });
        if self.materialized_corpus_identity_contract.required_identity_fields
            != [
                "corpus_id",
                "spec_id",
                "content_digest",
                "license_id",
                "provenance_uri",
                "source_kind",
                "source_revision",
                "generator_digest",
                "case_count",
            ]
            || self.materialized_corpus_contract.required_case_fields
                != ["case_id", "fixture_family", "input_digest", "oracle_digest"]
            || !all_specs_have_core
            || !self.specs.iter().all(CorpusSpec::conditional_fields_valid)
        {
            return Err(EvalError::new(
                EvalErrorKind::InvalidManifest,
                "invalid corpus specification contract",
            ));
        }
        Ok(())
    }
}

impl CorpusSpec {
    fn conditional_fields_valid(&self) -> bool {
        match self.spec_id.as_str() {
            "search-supported-v1" => self.generated_case_count == Some(10_000)
                && self.curated_case_count == Some(2_000)
                && self.total_case_count == Some(12_000)
                && self.required_fixture_families.is_some()
                && self.required_differential_units.is_some()
                && self.intentional_differences_require_exclusion == Some(true)
                && self.required_complete_empty_disposition.as_deref() == Some("complete_empty")
                && self.required_silent_drop_count == Some(0)
                && self.oracle_contract.as_ref().is_some_and(OracleContract::valid),
            "intel-million-lines-v1" => self.non_generated_rust_lines == Some(1_000_000)
                && self.required_fixtures.is_some()
                && self.dense_search_match_count == Some(1_000_000)
                && self.dense_search_default_rendered_bytes_limit_exclusive == Some(1_048_576)
                && self.dense_search_execution_completion_distinct_from_render_truncation == Some(true),
            "rust-semantics-v1" => self.required_labels.is_some(),
            "agent-operability-v1" => self.minimum_tasks_per_model_family == Some(200) && self.required_labels.is_some(),
            "memory-mutation-v1" => self.mutation_case_count == Some(1_000) && self.required_dimensions.is_some(),
            "formal-negative-v1" => self.required_lanes.is_some(),
            "runtime-link-measurement-v1" => self.required_lanes.is_some()
                && self.required_measurement_contract_fields.is_some()
                && self.required_certificate_fields.is_some()
                && self.required_negative_controls.is_some()
                && self.required_oracle_faults.is_some()
                && self.required_exit_conditions.is_some(),
            "rust-change-history-v1" => self.minimum_task_count == Some(200)
                && self.required_identity_fields.is_some()
                && self.required_labels.is_some(),
            _ => false,
        }
    }
}

impl OracleContract {
    fn valid(&self) -> bool {
        self.tool == "rg"
            && self.required_arguments == ["--json", "--no-config"]
            && self.required_identity_fields.len() == 6
            && self.binary_digest_algorithm == "sha256"
            && self.binary_digest_encoding == "lowercase-hex"
            && self.environment_policy == "cleared_then_allowlisted"
            && self.missing_or_mismatched_identity == "fail"
            && self.comparison_scope == "supported_intersection"
    }
}
