use serde::Deserialize;

use crate::error::{EvalError, EvalErrorKind};

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorporaManifest {
    schema_version: u32,
    manifest_kind: String,
    loader_policy: LoaderPolicy,
    corpus_spec_identity_contract: SpecIdentityContract,
    materialized_corpus_identity_contract: MaterializedIdentityContract,
    materialized_corpus_contract: MaterializedContract,
    #[serde(rename = "corpus_spec")]
    pub(crate) specs: Vec<CorpusSpec>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct LoaderPolicy {
    unknown_fields: String,
    duplicate_ids: String,
    mutable_sources: String,
    network_access: String,
    missing_content: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct SpecIdentityContract {
    domain: String,
    id_field: String,
    id_kind: String,
    canonical_payload: String,
    mutation_policy: String,
    duplicate_ids: String,
    references_target: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct MaterializedIdentityContract {
    algorithm: String,
    encoding: String,
    canonicalization: String,
    domain: String,
    id_field: String,
    digest_field: String,
    required_identity_fields: Vec<String>,
    spec_reference_field: String,
    mutation_policy: String,
    references_target: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct MaterializedContract {
    allowed_source_kinds: Vec<String>,
    allowed_materialization_states: Vec<String>,
    immutable_revision_kinds: Vec<String>,
    required_case_fields: Vec<String>,
    forbidden_source_revisions: Vec<String>,
    case_ids_unique: bool,
    content_must_match_digest: bool,
    license_and_provenance_required: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorpusSpec {
    pub(crate) spec_id: String,
    stage: String,
    source_kinds: Vec<String>,
    generated_case_count: Option<u64>,
    curated_case_count: Option<u64>,
    total_case_count: Option<u64>,
    required_fixture_families: Option<Vec<String>>,
    required_differential_units: Option<Vec<String>>,
    intentional_differences_require_exclusion: Option<bool>,
    required_complete_empty_disposition: Option<String>,
    required_silent_drop_count: Option<u64>,
    oracle_contract: Option<OracleContract>,
    non_generated_rust_lines: Option<u64>,
    required_fixtures: Option<Vec<String>>,
    dense_search_match_count: Option<u64>,
    dense_search_default_rendered_bytes_limit_exclusive: Option<u64>,
    dense_search_execution_completion_distinct_from_render_truncation: Option<bool>,
    required_labels: Option<Vec<String>>,
    minimum_tasks_per_model_family: Option<u64>,
    mutation_case_count: Option<u64>,
    required_dimensions: Option<Vec<String>>,
    required_lanes: Option<Vec<String>>,
    required_measurement_contract_fields: Option<Vec<String>>,
    required_certificate_fields: Option<Vec<String>>,
    required_negative_controls: Option<Vec<String>>,
    required_oracle_faults: Option<Vec<String>>,
    required_exit_conditions: Option<Vec<String>>,
    minimum_task_count: Option<u64>,
    required_identity_fields: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct OracleContract {
    tool: String,
    required_arguments: Vec<String>,
    required_identity_fields: Vec<String>,
    binary_digest_algorithm: String,
    binary_digest_encoding: String,
    environment_policy: String,
    missing_or_mismatched_identity: String,
    comparison_scope: String,
}

include!("corpora/validation.rs");
