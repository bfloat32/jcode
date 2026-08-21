use serde::Deserialize;

mod validation;

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunnerManifest {
    schema_version: u32,
    manifest_kind: String,
    loader_policy: LoaderPolicy,
    content_addressing: ContentAddressing,
    stage_identity_contract: StageIdentityContract,
    config_identity_contract: ConfigIdentityContract,
    hardware_identity_contract: HardwareIdentityContract,
    exclusion_identity_contract: ExclusionIdentityContract,
    result_identity_contract: ResultIdentityContract,
    baseline_policy: BaselinePolicy,
    execution_policy: ExecutionPolicy,
    privacy_policy: PrivacyPolicy,
    #[serde(rename = "stage")]
    pub(crate) stages: Vec<Stage>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct LoaderPolicy {
    unknown_fields: String,
    duplicate_ids: String,
    unresolved_references: String,
    mutable_identity_inputs: String,
    planned_results: String,
    partial_results: String,
    stale_results: String,
    post_hoc_baselines: String,
    identity_mismatches: String,
    missing_stage_evidence: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ContentAddressing {
    algorithm: String,
    encoding: String,
    canonicalization: String,
    artifact_extension: String,
    artifact_filename: String,
    artifact_filename_format: String,
    content_key_must_equal_filename: bool,
    canonical_payload_excludes: Vec<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct StageIdentityContract {
    domain: String,
    id_field: String,
    id_kind: String,
    allowed_ids: Vec<String>,
    required_fields: Vec<String>,
    mutation_policy: String,
    duplicate_ids: String,
    references_target: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ConfigIdentityContract {
    domain: String,
    id_field: String,
    required_fields: Vec<String>,
    ordered_fields: Vec<String>,
    mutable_revisions_forbidden: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct HardwareIdentityContract {
    domain: String,
    id_field: String,
    required_fields: Vec<String>,
    current_hardware_records: Vec<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ExclusionIdentityContract {
    domain: String,
    id_field: String,
    required_fields: Vec<String>,
    required_review_state: String,
    unreviewed_or_expired: String,
    semantic_effect_requires_reference: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ResultIdentityContract {
    domain: String,
    id_field: String,
    required_fields: Vec<String>,
    allowed_result_kinds: Vec<String>,
    passing_lifecycle_state: String,
    passing_disposition: String,
    required_freshness: String,
    baseline_ordering: String,
    all_gate_metrics_required: bool,
    all_references_must_resolve: bool,
    silent_drop_count_required: bool,
    post_hoc_baseline_can_pass: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct BaselinePolicy {
    pub(crate) directory: String,
    discovery_glob: String,
    missing_baseline_result: String,
    missing_baseline_cannot_pass: bool,
    baseline_command_accepts_only_complete_result: bool,
    baseline_command_recomputes_all_identities: bool,
    baseline_command_refuses_overwrite: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ExecutionPolicy {
    network_access: String,
    corpus_sources: Vec<String>,
    comparison_mode: String,
    clean_temporary_checkouts: bool,
    resource_capture_required: bool,
    ci_attestation_when_available: bool,
    raw_ci_upload: String,
    non_public_artifacts: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct PrivacyPolicy {
    independent_upload_transport: bool,
    analytics_transport: bool,
    telemetry_identity: bool,
    allowed_remote_model_evaluation: String,
    forbidden_remote_content: Vec<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stage {
    pub(crate) stage_id: String,
    first_implementation_task: String,
    default_feature_state: String,
    pub(crate) required_corpus_spec_ids: Vec<String>,
    pub(crate) required_gate_ids: Vec<String>,
    baseline_required_before_implementation: bool,
    baseline_required_before_enable: bool,
    enable_only_after_passing_result: bool,
    pub(crate) consecutive_full_passing_release_runs: Option<u64>,
}
